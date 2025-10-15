use crate::components::common::ComponentId;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Processor {
    pub production_speed: f64,
    pub productive: bool,
    pub recipe: ComponentId,
}

#[derive(Component)]
pub struct Processors {
    pub processors: Vec<Processor>,
}
