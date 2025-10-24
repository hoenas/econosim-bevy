use crate::components::common::TimeToLive;
use crate::resources::economy::common::Id;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::prelude::Component;

#[derive(Component, Default)]
pub struct Offer {
    pub resource: Id,
    pub amount: f64,
    pub price_per_unit: f64,
    pub company: Option<Entity>,
}

#[derive(Bundle)]
pub struct OfferBundle {
    pub offer: Offer,
    pub time_to_live: TimeToLive,
}
