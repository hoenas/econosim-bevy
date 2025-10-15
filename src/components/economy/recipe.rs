use crate::resources::economy::common::Id;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Recipe(pub Id);
