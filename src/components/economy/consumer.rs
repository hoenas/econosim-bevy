use crate::components::common::Name;
use bevy::prelude::Component;

#[derive(Component)]
pub struct ConsumerConfig {
    pub resource: usize,
    pub order_amount: f64,
    pub order_max_price: f64,
    pub ticks_between_orders: usize,
    pub ticks_since_last_order: usize,
}

#[derive(Component)]
pub struct Consumer {
    pub name: Name,
    pub config: ConsumerConfig,
}
