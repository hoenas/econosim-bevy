use bevy::prelude::Component;
use std::usize;

pub type ComponentId = usize;

#[derive(Component)]
pub struct Id(pub ComponentId);

#[derive(Component)]
pub struct Name(pub String);
