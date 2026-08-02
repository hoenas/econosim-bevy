use bevy::prelude::*;

use crate::components::common::TimeToLive;
use crate::components::economy::consumer::ConsumerConfig;
use crate::components::economy::order::Order;
use crate::components::economy::order::OrderBundle;
use crate::components::economy::stock::Stock;

pub fn manage_consumers(
    mut commands: Commands,
    mut consumers: Query<(Entity, &mut Stock, &ConsumerConfig)>,
) {
    for (entity, mut stock, config) in consumers.iter_mut() {
        for demand in &config.demands {
            // Draw the consumption rate down from internal storage (floored at empty).
            let level = {
                let held = stock.resources.entry(demand.resource).or_insert(0.0);
                *held = (*held - demand.consumption_rate).max(0.0);
                *held
            };

            // Try to refill up to the target buffer, bidding higher the emptier we are.
            let deficit = (demand.target_stock - level).max(0.0);
            if deficit <= 0.0 {
                continue;
            }
            let scarcity = if demand.target_stock > 0.0 {
                (1.0 - level / demand.target_stock).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let price = demand.base_price + (demand.max_price - demand.base_price) * scarcity;

            // company: Some(entity) routes the delivery into this consumer's Stock. TTL 1 so a
            // fresh, correctly priced order is placed every tick instead of stacking up.
            commands.spawn(OrderBundle {
                order: Order {
                    company: Some(entity),
                    resource: demand.resource,
                    amount: deficit,
                    max_price_per_unit: price,
                },
                time_to_live: TimeToLive(1),
            });
        }
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

    /// One demand: base 4, max 12, target 100, consuming 10/tick.
    fn demand() -> Demand {
        Demand {
            resource: 0,
            consumption_rate: 10.0,
            target_stock: 100.0,
            base_price: 4.0,
            max_price: 12.0,
        }
    }

    fn spawn(app: &mut App, start_stock: f64) -> Entity {
        let mut stock = Stock::default();
        stock.resources.insert(0, start_stock);
        app.world_mut()
            .spawn((ConsumerConfig { demands: vec![demand()] }, stock))
            .id()
    }

    #[test]
    fn consumption_draws_down_storage() {
        let mut app = app();
        let e = spawn(&mut app, 100.0);
        app.update();
        assert_eq!(app.world().get::<Stock>(e).unwrap().resources[&0], 90.0);
    }

    #[test]
    fn consumption_floors_at_zero() {
        let mut app = app();
        let e = spawn(&mut app, 5.0);
        app.update();
        assert_eq!(app.world().get::<Stock>(e).unwrap().resources[&0], 0.0);
    }

    #[test]
    fn orders_deficit_up_to_target_routed_to_consumer() {
        let mut app = app();
        let e = spawn(&mut app, 100.0); // → 90 after consumption, deficit 10
        app.update();
        let mut q = app.world_mut().query::<&Order>();
        let order = q.iter(app.world()).next().unwrap();
        assert_eq!(order.resource, 0);
        assert_eq!(order.amount, 10.0);
        assert_eq!(order.company, Some(e));
    }

    #[test]
    fn price_scales_linearly_with_scarcity() {
        let mut app = app();
        spawn(&mut app, 60.0); // → 50 after consumption, half of target 100 → scarcity 0.5
        app.update();
        let mut q = app.world_mut().query::<&Order>();
        let order = q.iter(app.world()).next().unwrap();
        // base 4 + (12 - 4) * 0.5 = 8.0
        assert_eq!(order.max_price_per_unit, 8.0);
    }

    #[test]
    fn price_rises_toward_max_when_short() {
        let mut app = app();
        spawn(&mut app, 10.0); // → 0 after consumption → fully scarce
        app.update();
        let mut q = app.world_mut().query::<&Order>();
        let order = q.iter(app.world()).next().unwrap();
        assert_eq!(order.max_price_per_unit, 12.0); // max price at empty
    }

    #[test]
    fn no_order_when_at_or_above_target() {
        let mut app = app();
        // 100 target, consumes 10 → 90, still a deficit; use a non-consuming demand at target.
        let mut stock = Stock::default();
        stock.resources.insert(0, 100.0);
        app.world_mut().spawn((
            ConsumerConfig {
                demands: vec![Demand {
                    resource: 0,
                    consumption_rate: 0.0,
                    target_stock: 100.0,
                    base_price: 4.0,
                    max_price: 12.0,
                }],
            },
            stock,
        ));
        app.update();
        let mut q = app.world_mut().query::<&Order>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }
}
