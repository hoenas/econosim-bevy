use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SimState {
    pub paused: bool,
    pub step_requested: bool,
}
