use crate::resources::economy::common::Id;
use bevy::prelude::Component;
use std::collections::HashMap;

#[derive(Component)]
#[derive(Default)]
pub struct Stock {
    pub resources: HashMap<Id, f64>,
}

