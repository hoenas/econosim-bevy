use bevy::prelude::*;

use crate::components::common::TimeToLive;
use crate::components::economy::consumer::ConsumerConfig;
use crate::components::economy::order::Order;
use crate::components::economy::order::OrderBundle;

pub fn manage_consumers(mut commands: Commands, mut consumers: Query<&mut ConsumerConfig>) {
    for mut consumer in consumers.iter_mut() {
        if consumer.ticks_since_last_order % consumer.ticks_between_orders == 0 {
            // Emit one order per demand in the basket.
            for demand in &consumer.demands {
                commands.spawn(OrderBundle {
                    order: Order {
                        company: None,
                        resource: demand.resource,
                        amount: demand.amount,
                        max_price_per_unit: demand.max_price,
                    },
                    time_to_live: TimeToLive(consumer.ticks_between_orders),
                });
            }
            consumer.ticks_since_last_order = 0;
        }
        consumer.ticks_since_last_order += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::economy::consumer::{ConsumerConfig, Demand};
    use crate::components::economy::order::Order;

    fn app() -> App {
        let mut a = App::new();
        a.add_systems(Update, manage_consumers);
        a
    }

    fn config(ticks_since: usize, ticks_between: usize) -> ConsumerConfig {
        ConsumerConfig {
            demands: vec![Demand { resource: 0, amount: 5.0, max_price: 10.0 }],
            ticks_between_orders: ticks_between,
            ticks_since_last_order: ticks_since,
        }
    }

    #[test]
    fn spawns_order_on_interval_tick() {
        let mut app = app();
        app.world_mut().spawn(config(0, 4));
        app.update();
        let mut q = app.world_mut().query::<&Order>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn no_order_between_intervals() {
        let mut app = app();
        app.world_mut().spawn(config(2, 4));
        app.update();
        let mut q = app.world_mut().query::<&Order>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    #[test]
    fn order_has_correct_resource_and_price() {
        let mut app = app();
        app.world_mut().spawn(config(0, 4));
        app.update();
        let mut q = app.world_mut().query::<&Order>();
        let order = q.iter(app.world()).next().unwrap();
        assert_eq!(order.resource, 0);
        assert_eq!(order.amount, 5.0);
        assert_eq!(order.max_price_per_unit, 10.0);
        assert!(order.company.is_none());
    }

    #[test]
    fn basket_emits_one_order_per_demand() {
        let mut app = app();
        app.world_mut().spawn(ConsumerConfig {
            demands: vec![
                Demand { resource: 0, amount: 5.0, max_price: 10.0 },
                Demand { resource: 1, amount: 2.0, max_price: 3.0 },
                Demand { resource: 2, amount: 9.0, max_price: 7.0 },
            ],
            ticks_between_orders: 4,
            ticks_since_last_order: 0,
        });
        app.update();
        let mut q = app.world_mut().query::<&Order>();
        let mut resources: Vec<usize> = q.iter(app.world()).map(|o| o.resource).collect();
        resources.sort();
        assert_eq!(resources, vec![0, 1, 2]);
    }
}
