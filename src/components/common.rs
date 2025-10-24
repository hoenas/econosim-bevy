use bevy::prelude::Component;

#[derive(Component)]
pub struct Name(pub String);

#[derive(Component)]
pub struct TimeToLive(pub usize);
