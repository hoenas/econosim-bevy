use std::collections::HashMap;
use std::collections::VecDeque;
use bevy::prelude::Entity;
use burn::backend::wgpu::WgpuDevice;
use burn::optim::{Adam, AdamConfig};
use burn::optim::adaptor::OptimizerAdaptor;
use burn::Tensor;
use burn::module::AutodiffModule;
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::components::reinforcement_learning::nn::NeuralNetwork;
use crate::resources::reinforcement_learning::backend::{MyAutodiffBackend, MyBackend};

const TARGET_UPDATE_INTERVAL: u32 = 20;
// How many past transitions to keep for experience replay.
const REPLAY_CAPACITY: usize = 10_000;

pub type QOptimizer = OptimizerAdaptor<
    Adam,
    NeuralNetwork<MyAutodiffBackend>,
    MyAutodiffBackend,
>;

/// One observed environment transition (s, a, r, s'). States are stored as the flat
/// scaled feature vectors the network consumes, so no GPU tensors are held in the buffer.
#[derive(Clone)]
pub struct Transition {
    pub state: Vec<f32>,
    pub action: usize,
    pub reward: f32,
    pub next_state: Vec<f32>,
}

pub struct CompanyQState {
    // Option because optimizer.step() takes the network by value; we use take/put to move it
    // in and out without requiring an additional heap allocation.
    pub network: Option<NeuralNetwork<MyAutodiffBackend>>,
    // Frozen copy on the inner (non-autodiff) backend — no graph nodes allocated during inference.
    // Updated every TARGET_UPDATE_INTERVAL train steps.
    target_network: NeuralNetwork<MyBackend>,
    pub optimizer: QOptimizer,
    // Experience-replay buffer: decorrelates the samples each gradient step sees.
    replay: VecDeque<Transition>,
    steps: u32,
}

impl CompanyQState {
    pub fn new(device: &WgpuDevice, state_size: usize, action_size: usize) -> Self {
        let network = NeuralNetwork::new(device, state_size, action_size);
        let target_network = NeuralNetwork::<MyBackend>::new(device, state_size, action_size);
        Self {
            network: Some(network),
            target_network,
            optimizer: AdamConfig::new().init(),
            replay: VecDeque::with_capacity(REPLAY_CAPACITY),
            steps: 0,
        }
    }

    /// Records a transition, evicting the oldest once the buffer is full.
    pub fn remember(&mut self, transition: Transition) {
        if self.replay.len() == REPLAY_CAPACITY {
            self.replay.pop_front();
        }
        self.replay.push_back(transition);
    }

    /// Uniformly samples a minibatch (with replacement), or None until `batch_size`
    /// transitions have been collected.
    pub fn sample_batch(&self, batch_size: usize, rng: &mut ThreadRng) -> Option<Vec<Transition>> {
        let len = self.replay.len();
        if len < batch_size {
            return None;
        }
        Some(
            (0..batch_size)
                .map(|_| self.replay[rng.random_range(0..len)].clone())
                .collect(),
        )
    }

    /// Number of completed gradient steps — used to schedule epsilon decay.
    pub fn train_steps(&self) -> u32 {
        self.steps
    }

    pub fn train(
        &mut self,
        input: Tensor<MyAutodiffBackend, 2>,
        target: Tensor<MyAutodiffBackend, 2>,
        lr: f64,
    ) {
        let network = self.network.take().unwrap();
        self.network = Some(network.train_step(&mut self.optimizer, input, target, lr));
        self.steps += 1;
        if self.steps % TARGET_UPDATE_INTERVAL == 0 {
            // valid() strips autodiff from every parameter tensor — no graph nodes kept.
            self.target_network = self.network.as_ref().unwrap().valid();
        }
    }

    // Inference on the live network without building an autodiff graph.
    pub fn infer(&self, input: Tensor<MyBackend, 1>) -> Tensor<MyBackend, 1> {
        self.network.as_ref().unwrap().valid().forward(input)
    }

    // Batched inference on the live network (state batch → Q batch), no autodiff.
    pub fn infer_batch(&self, input: Tensor<MyBackend, 2>) -> Tensor<MyBackend, 2> {
        self.network.as_ref().unwrap().valid().forward_batch(input)
    }

    // Batched inference on the frozen target network — never touches autodiff.
    pub fn target_forward_batch(&self, input: Tensor<MyBackend, 2>) -> Tensor<MyBackend, 2> {
        self.target_network.forward_batch(input)
    }
}

// Stored as a NonSend resource so the GPU handles stay on the main thread
pub struct QNetworkStore(pub HashMap<Entity, CompanyQState>);

impl Default for QNetworkStore {
    fn default() -> Self {
        QNetworkStore(HashMap::new())
    }
}
