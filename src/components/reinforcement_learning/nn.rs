use bevy::prelude::Component;
use burn::Tensor;
use burn::module::Module;
use burn::nn::loss::HuberLossConfig;
use burn::nn::{Linear, LinearConfig, Relu};
use burn::prelude::Backend;
use burn::tensor::backend::AutodiffBackend;

// Source: https://dev.to/philip_yaw/burn-the-future-of-deep-learning-in-rust-5c5e

// Define a simple feedforward neural network
#[derive(Module, Debug, Component)]
pub struct NeuralNetwork<B: Backend> {
    linear1: Linear<B>,
    linear2: Linear<B>,
    activation: Relu,
}

impl<B: AutodiffBackend> NeuralNetwork<B> {
    pub fn new(
        device: &B::Device,
        state_space_dimensions: usize,
        action_space_dimensions: usize,
    ) -> Self {
        Self {
            linear1: LinearConfig::new(state_space_dimensions, state_space_dimensions).init(device),
            linear2: LinearConfig::new(state_space_dimensions, action_space_dimensions)
                .init(device),
            activation: Relu::new(),
        }
    }

    pub fn forward(&self, input: Tensor<B, 1>) -> Tensor<B, 1> {
        let x = self.linear1.forward(input);
        let x = self.activation.forward(x);
        let x = self.linear2.forward(x);
        self.activation.forward(x)
    }

    pub fn train(&mut self, input: Tensor<B, 1>, target: Tensor<B, 1>) {
        let outputs = self.forward(input.clone());
        let loss = HuberLossConfig::new(0.2).init().forward(
            outputs.clone(),
            target.clone(),
            burn::nn::loss::Reduction::Auto,
        );
        loss.backward();
    }
}
