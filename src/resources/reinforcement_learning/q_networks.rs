use std::collections::HashMap;
use bevy::prelude::Entity;
use burn::backend::wgpu::WgpuDevice;
use burn::optim::{Adam, AdamConfig};
use burn::optim::adaptor::OptimizerAdaptor;
use burn::Tensor;
use crate::components::reinforcement_learning::nn::NeuralNetwork;
use crate::resources::reinforcement_learning::backend::MyAutodiffBackend;

pub type QOptimizer = OptimizerAdaptor<
    Adam,
    NeuralNetwork<MyAutodiffBackend>,
    MyAutodiffBackend,
>;

pub struct CompanyQState {
    // Option because optimizer.step() takes the network by value; we use take/put to move it
    // in and out without requiring an additional heap allocation.
    pub network: Option<NeuralNetwork<MyAutodiffBackend>>,
    pub optimizer: QOptimizer,
}

impl CompanyQState {
    pub fn new(device: &WgpuDevice, state_size: usize, action_size: usize) -> Self {
        Self {
            network: Some(NeuralNetwork::new(device, state_size, action_size)),
            optimizer: AdamConfig::new().init(),
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
    }

    pub fn forward(&self, input: Tensor<MyAutodiffBackend, 1>) -> Tensor<MyAutodiffBackend, 1> {
        self.network.as_ref().unwrap().forward(input)
    }
}

// Stored as a NonSend resource so the GPU handles stay on the main thread
pub struct QNetworkStore(pub HashMap<Entity, CompanyQState>);

impl Default for QNetworkStore {
    fn default() -> Self {
        QNetworkStore(HashMap::new())
    }
}
