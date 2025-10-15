use crate::resources::economy::common::Id;
use bevy::prelude::Component;
use std::collections::HashMap;

#[derive(Component)]
pub struct Stock {
    pub resources: HashMap<Id, f64>,
}

impl Default for Stock {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}
