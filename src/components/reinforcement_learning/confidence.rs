use bevy::prelude::Component;

/// Softmax probability of the last chosen action over all Q-values.
/// `None` when the action was chosen by random exploration (epsilon-greedy).
/// `Some(p)` when chosen greedily; `p` ranges from 1/N (no preference) to ~1.0 (certain).
#[derive(Component, Default)]
pub struct CompanyConfidence(pub Option<f32>);
