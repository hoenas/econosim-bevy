use crate::components::common::ComponentId;
use bevy::prelude::Component;
use std::collections::HashMap;

#[derive(Component)]
pub struct Stock {
    pub resources: HashMap<ComponentId, f64>,
}

impl Stock {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}
