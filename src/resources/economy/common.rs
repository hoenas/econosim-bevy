use bevy::prelude::Resource;

pub type Id = usize;

#[derive(Resource)]
pub struct Currency {
    pub name: String,
    pub unit: char,
}
