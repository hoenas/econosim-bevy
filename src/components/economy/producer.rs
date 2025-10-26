use crate::components::common::Name;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Component;

#[derive(Component)]
pub struct ProducerConfig {
    pub resource: usize,
    pub offer_amount: f64,
    pub offer_price: f64,
    pub ticks_between_offers: usize,
    pub ticks_since_last_offer: usize,
}

#[derive(Bundle)]
pub struct Producer {
    pub name: Name,
    pub config: ProducerConfig,
}
