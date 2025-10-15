use crate::components::common::ComponentId;
use bevy::platform::collections::HashMap;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Recipe {
    pub ingredients: HashMap<ComponentId, f64>,
    pub products: HashMap<ComponentId, f64>,
    pub production_speed: f64,
}
