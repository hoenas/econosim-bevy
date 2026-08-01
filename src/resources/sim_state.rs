use bevy::prelude::*;

#[derive(Resource)]
pub struct SimState {
    pub paused: bool,
    pub step_requested: bool,
    pub reset_requested: bool,
}

impl Default for SimState {
    fn default() -> Self {
        Self { paused: true, step_requested: false, reset_requested: false }
    }
}
