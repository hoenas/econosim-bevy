use crate::components::common::Id;
use bevy::prelude::Component;
use std::collections::HashMap;

#[derive(Component)]
pub struct Recipe {
    pub ingredients: HashMap<Id, f64>,
    pub products: HashMap<Id, f64>,
    pub production_speed: f64,
}
