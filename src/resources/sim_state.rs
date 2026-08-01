use bevy::prelude::*;

#[derive(Resource)]
pub struct SimState {
    pub paused: bool,
    pub step_requested: bool,
    pub reset_requested: bool,
    pub spawn_company_requested: bool,
    pub remove_company_requested: Option<bevy::prelude::Entity>,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            paused: true,
            step_requested: false,
            reset_requested: false,
            spawn_company_requested: false,
            remove_company_requested: None,
        }
    }
}
