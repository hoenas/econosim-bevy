use std::collections::HashMap;
use bevy::prelude::Entity;
use burn::backend::wgpu::WgpuDevice;
use burn::optim::{Adam, AdamConfig};
use burn::optim::adaptor::OptimizerAdaptor;
use burn::Tensor;
use burn::module::AutodiffModule;
use crate::components::reinforcement_learning::nn::NeuralNetwork;
use crate::resources::reinforcement_learning::backend::{MyAutodiffBackend, MyBackend};

const TARGET_UPDATE_INTERVAL: u32 = 20;

pub type QOptimizer = OptimizerAdaptor<
    Adam,
    NeuralNetwork<MyAutodiffBackend>,
    MyAutodiffBackend,
>;

pub struct CompanyQState {
    // Option because optimizer.step() takes the network by value; we use take/put to move it
    // in and out without requiring an additional heap allocation.
    pub network: Option<NeuralNetwork<MyAutodiffBackend>>,
    // Frozen copy on the inner (non-autodiff) backend — no graph nodes allocated during inference.
    // Updated every TARGET_UPDATE_INTERVAL train steps.
    target_network: NeuralNetwork<MyBackend>,
    pub optimizer: QOptimizer,
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
            steps: 0,
        }
    }

    pub fn train(
        &mut self,
        input: Tensor<MyAutodiffBackend, 1>,
        target: Tensor<MyAutodiffBackend, 1>,
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

    // Inference on the frozen target network — never touches autodiff.
    pub fn target_forward(&self, input: Tensor<MyBackend, 1>) -> Tensor<MyBackend, 1> {
        self.target_network.forward(input)
    }
}

// Stored as a NonSend resource so the GPU handles stay on the main thread
pub struct QNetworkStore(pub HashMap<Entity, CompanyQState>);

impl Default for QNetworkStore {
    fn default() -> Self {
        QNetworkStore(HashMap::new())
    }
}
