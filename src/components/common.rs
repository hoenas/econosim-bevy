use bevy::prelude::Component;
use std::usize;

#[derive(Component)]
pub struct Id(pub usize);

#[derive(Component)]
pub struct Name(pub String);
