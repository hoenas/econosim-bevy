use crate::components::common::Name;
use crate::components::economy::stock::Stock;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Component;

/// A standing demand for one resource. Each tick the consumer draws `consumption_rate` from
/// its internal storage and tries to buy back up to `target_stock`, bidding more the emptier
/// it is: `base_price` when full, rising toward `max_price` as storage approaches zero.
pub struct Demand {
    pub resource: usize,
    pub consumption_rate: f64,
    pub target_stock: f64,
    pub base_price: f64,
    pub max_price: f64,
}

#[derive(Component)]
pub struct ConsumerConfig {
    pub demands: Vec<Demand>,
}

#[derive(Bundle)]
pub struct Consumer {
    pub name: Name,
    pub config: ConsumerConfig,
    /// Internal storage the consumer draws from and replenishes from the market.
    pub stock: Stock,
}
