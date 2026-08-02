use bevy::prelude::{Entity, Resource};
use std::collections::{HashMap, VecDeque};

/// How many completed-episode returns to keep per company (the learning curve).
pub const MAX_EPISODE_RETURNS: usize = 1_000;

#[derive(Default)]
pub struct CompanyTraining {
    /// Cumulative reward for the episode currently in progress (raw currency PnL).
    pub current_return: f64,
    /// One entry per completed episode — the learning curve, oldest first.
    pub episode_returns: VecDeque<f64>,
}

impl CompanyTraining {
    /// Closes the current episode: files its return and resets the accumulator.
    pub fn end_episode(&mut self) {
        self.episode_returns.push_back(self.current_return);
        if self.episode_returns.len() > MAX_EPISODE_RETURNS {
            self.episode_returns.pop_front();
        }
        self.current_return = 0.0;
    }
}

/// Learning-progress metrics, deliberately kept out of `SimHistory` so it is NOT wiped
/// on simulation reset — the whole point is to track improvement across episodes.
#[derive(Resource, Default)]
pub struct TrainingHistory {
    pub companies: HashMap<Entity, CompanyTraining>,
    /// Current exploration rate (epsilon), shared across companies since they train in lockstep.
    pub epsilon: f64,
}
