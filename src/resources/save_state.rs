use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct SaveLoadState {
    pub name: String,
    pub save_requested: bool,
    pub load_requested: bool,
}

impl Default for SaveLoadState {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            save_requested: false,
            load_requested: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SaveRecipe {
    pub id: usize,
    pub name: String,
    pub ingredients: Vec<(usize, f64)>,
    pub products: Vec<(usize, f64)>,
    pub production_speed: f64,
}

#[derive(Serialize, Deserialize)]
pub struct SaveMetadata {
    pub resources: Vec<(usize, String)>,
    pub recipes: Vec<SaveRecipe>,
    pub companies: Vec<String>,
    pub state_size: usize,
    pub action_size: usize,
}
