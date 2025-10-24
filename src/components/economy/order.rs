use crate::components::common::TimeToLive;
use crate::resources::economy::common::Id;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::prelude::Component;

#[derive(Component, Default)]
pub struct Order {
    pub company: Option<Entity>,
    pub resource: Id,
    pub amount: f64,
    pub max_price_per_unit: f64,
}

#[derive(Bundle)]
pub struct OrderBundle {
    pub order: Order,
    pub time_to_live: TimeToLive,
}
