use bevy::prelude::*;

use crate::components::economy::consumer::ConsumerConfig;
use crate::components::economy::order::Order;

pub fn manage_consumers(mut commands: Commands, mut consumers: Query<&mut ConsumerConfig>) {
    for mut consumer in consumers.iter_mut() {
        consumer.ticks_since_last_order += 1;
        if consumer.ticks_between_orders == consumer.ticks_between_orders {
            // Create a new order
            commands.spawn(Order {
                company: None,
                resource: consumer.resource,
                amount: consumer.order_amount,
                max_price_per_unit: consumer.order_max_price,
            });
        }
    }
}
