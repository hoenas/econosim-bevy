use crate::components::common::Name;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Component;

/// A single standing demand: buy `amount` of `resource` at up to `max_price` per unit.
pub struct Demand {
    pub resource: usize,
    pub amount: f64,
    pub max_price: f64,
}

#[derive(Component)]
pub struct ConsumerConfig {
    /// The basket this consumer buys each cycle — one order is emitted per demand.
    pub demands: Vec<Demand>,
    pub ticks_between_orders: usize,
    pub ticks_since_last_order: usize,
}

#[derive(Bundle)]
pub struct Consumer {
    pub name: Name,
    pub config: ConsumerConfig,
}
