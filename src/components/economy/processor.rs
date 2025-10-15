use crate::components::economy::production_speed::ProductionSpeed;
use crate::components::economy::recipe::Recipe;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Productive(pub bool);

#[derive(Bundle)]
pub struct Processor {
    pub production_speed: ProductionSpeed,
    pub productive: Productive,
    pub recipe: Recipe,
}

#[derive(Component)]
pub struct Processors {
    pub processors: Vec<Processor>,
}
