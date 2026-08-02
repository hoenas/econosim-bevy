use bevy::prelude::*;

#[derive(Resource)]
pub struct SimState {
    pub paused: bool,
    pub step_requested: bool,
    pub reset_requested: bool,
    pub spawn_company_requested: bool,
    pub remove_company_requested: Option<bevy::prelude::Entity>,
    /// Automatically reset after this many ticks. 0 = disabled.
    pub auto_reset_interval: u32,
    pub tick_count: u32,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            paused: true,
            step_requested: false,
            reset_requested: false,
            spawn_company_requested: false,
            remove_company_requested: None,
            auto_reset_interval: 200,
            tick_count: 0,
        }
    }
}
