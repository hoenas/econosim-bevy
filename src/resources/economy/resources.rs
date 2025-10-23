use crate::resources::economy::common::Id;
use bevy::prelude::Resource;
use std::collections::HashMap;

pub type ResourceHandle = usize;

#[derive(Resource)]
pub struct Resources {
    pub resources: HashMap<Id, String>,
}

impl Default for Resources {
    fn default() -> Self {
        let mut resources: HashMap<Id, String> = HashMap::new();
        resources.insert(0, "Water".to_string());
        resources.insert(1, "Dirt".to_string());
        resources.insert(2, "Wood".to_string());
        resources.insert(3, "Coal".to_string());
        Resources {
            resources: resources,
        }
    }
}
