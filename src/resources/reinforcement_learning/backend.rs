use bevy::prelude::Resource;
use burn::backend::Autodiff;
use burn::backend::Wgpu;

pub type MyBackend = Wgpu<f32, i32>;
pub type MyAutodiffBackend = Autodiff<MyBackend>;
