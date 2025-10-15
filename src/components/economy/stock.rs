use bevy::prelude::Component;
use bevy::prelude::Entity;
use std::collections::HashMap;

#[derive(Component)]
pub struct Stock {
    pub resources: HashMap<Entity, f64>,
}

impl Default for Stock {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}
