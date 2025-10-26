use bevy::prelude::*;

use crate::components::common::TimeToLive;
use crate::components::economy::consumer::ConsumerConfig;
use crate::components::economy::order::Order;
use crate::components::economy::order::OrderBundle;

pub fn manage_consumers(mut commands: Commands, mut consumers: Query<&mut ConsumerConfig>) {
    for mut consumer in consumers.iter_mut() {
        if consumer.ticks_since_last_order % consumer.ticks_between_orders == 0 {
            // Create a new order
            commands.spawn(OrderBundle {
                order: Order {
                    company: None,
                    resource: consumer.resource,
                    amount: consumer.order_amount,
                    max_price_per_unit: consumer.order_max_price,
                },
                time_to_live: TimeToLive(consumer.ticks_between_orders),
            });
            consumer.ticks_since_last_order = 0;
        }
        consumer.ticks_since_last_order += 1;
    }
}
