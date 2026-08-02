use bevy::prelude::Component;
use burn::Tensor;
use burn::module::Module;
use burn::nn::loss::HuberLossConfig;
use burn::nn::{Linear, LinearConfig, Relu};
use burn::optim::{GradientsParams, Optimizer};
use burn::prelude::Backend;
use burn::tensor::backend::AutodiffBackend;

#[derive(Module, Debug, Component)]
pub struct NeuralNetwork<B: Backend> {
    linear1: Linear<B>,
    linear2: Linear<B>,
    activation: Relu,
}

// forward and new work on any Backend — no autodiff required.
impl<B: Backend> NeuralNetwork<B> {
    pub fn new(
        device: &B::Device,
        state_space_dimensions: usize,
        action_space_dimensions: usize,
    ) -> Self {
        Self {
            linear1: LinearConfig::new(state_space_dimensions, state_space_dimensions)
                .init(device),
            linear2: LinearConfig::new(state_space_dimensions, action_space_dimensions)
                .init(device),
            activation: Relu::new(),
        }
    }

    pub fn forward(&self, input: Tensor<B, 1>) -> Tensor<B, 1> {
        // burn's Linear requires 2D input (batch × features); add a dummy batch dim
        let x = input.unsqueeze::<2>();
        let x = self.linear1.forward(x);
        let x = self.activation.forward(x);
        let x = self.linear2.forward(x);
        // No ReLU on the output: Q-values can be negative, clamping them would bias learning
        x.squeeze::<1>()
    }
}

// train_step requires AutodiffBackend — only used during the gradient update.
impl<B: AutodiffBackend> NeuralNetwork<B> {
    // Takes self by value because burn's optimizer.step() consumes the module to return
    // an updated copy with new weights — there is no in-place mutation API.
    pub fn train_step<O: Optimizer<NeuralNetwork<B>, B>>(
        self,
        optimizer: &mut O,
        input: Tensor<B, 1>,
        target: Tensor<B, 1>,
        lr: f64,
    ) -> Self {
        let outputs = self.forward(input);
        // Huber loss is more robust than MSE when rewards have occasional large spikes
        let loss = HuberLossConfig::new(1.0).init().forward(
            outputs,
            target,
            burn::nn::loss::Reduction::Auto,
        );
        let grads = loss.backward();
        let grads = GradientsParams::from_grads(grads, &self);
        optimizer.step(lr, self, grads)
    }
}
