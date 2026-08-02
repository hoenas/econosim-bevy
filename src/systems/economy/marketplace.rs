use crate::components::economy::money::Money;
use crate::components::economy::offer::Offer;
use crate::components::economy::order::Order;
use crate::components::economy::stock::Stock;
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::resources::{ResourceHandle, Resources};
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::*;
use ordered_float::OrderedFloat;

pub fn update_price_index(
    offers: Query<(Entity, &Offer)>,
    resources: Res<Resources>,
    mut market_data: ResMut<Marketplace>,
) {
    for resource_handle in resources.resources.keys() {
        let offer = get_cheapest_offer(*resource_handle, offers.iter());
        market_data.price_index.insert(*resource_handle, offer);
    }
}

pub fn update_order_index(
    orders: Query<(Entity, &Order)>,
    resources: Res<Resources>,
    mut market_data: ResMut<Marketplace>,
) {
    for resource_handle in resources.resources.keys() {
        let order = get_highest_order(*resource_handle, orders.iter());
        market_data.order_index.insert(*resource_handle, order);
    }
}

pub fn get_cheapest_offer<'a>(
    resource: ResourceHandle,
    offers: impl Iterator<Item = (Entity, &'a Offer)>,
) -> Option<(Entity, f64)> {
    offers
        .filter(|(_, o)| o.resource == resource && o.amount > 0.0)
        .min_by_key(|x| OrderedFloat(x.1.price_per_unit))
        .map(|x| (x.0, x.1.price_per_unit))
}

pub fn get_highest_order<'a>(
    resource: ResourceHandle,
    orders: impl Iterator<Item = (Entity, &'a Order)>,
) -> Option<(Entity, f64)> {
    orders
        .filter(|(_, order)| order.resource == resource)
        .max_by_key(|x| OrderedFloat(x.1.max_price_per_unit))
        .map(|x| (x.0, x.1.max_price_per_unit))
}

pub fn execute_orders(
    mut commands: Commands,
    mut orders: Query<(Entity, &mut Order)>,
    mut offers: Query<(Entity, &mut Offer)>,
    // Also matches consumers: they carry a Stock (their storage) but no Money, which we treat
    // as an unlimited budget — they are the economy's exogenous demand / money source.
    mut companies: Query<(&mut Stock, Option<&mut Money>)>,
    mut market_data: ResMut<Marketplace>,
) {
    for (order_entity, mut order) in orders.iter_mut() {
        while order.amount > 0.0 {
            let Some((offer_entity, _)) =
                get_cheapest_offer(order.resource, offers.as_readonly().iter_mut())
            else {
                break;
            };
            let mut offer = offers.get_mut(offer_entity).unwrap().1;

            if offer.price_per_unit > order.max_price_per_unit {
                break;
            }

            // Copy scalar fields before any mutable borrows below.
            let offer_price = offer.price_per_unit;
            let offer_resource = offer.resource;
            let seller = offer.company;
            let buyer = order.company;
            // Self-trade: buyer and seller are the same company. Money and stock net
            // to zero, so we skip the company mutations to avoid a double-borrow panic.
            let is_self_trade = buyer.is_some() && buyer == seller;
            // Consumers carry a Stock but no Money; only real companies count toward the
            // company-order statistics below.
            let buyer_is_company = match buyer {
                Some(e) => companies
                    .as_readonly()
                    .get(e)
                    .map(|(_, money)| money.is_some())
                    .unwrap_or(false),
                None => false,
            };

            // Cap the fill by what the buyer can actually afford.
            let max_affordable = if let Some(buyer_entity) = buyer {
                if is_self_trade {
                    f64::MAX
                } else {
                    let companies_ro = companies.as_readonly();
                    let (_, money) = companies_ro.get(buyer_entity).unwrap();
                    match money {
                        // A buyer with no Money component (a consumer) buys without limit.
                        Some(m) if offer_price > 0.0 => m.0 / offer_price,
                        _ => f64::MAX,
                    }
                }
            } else {
                f64::MAX // consumers have no tracked money
            };

            let filled = order.amount.min(offer.amount).min(max_affordable);

            if filled <= 0.0 {
                break; // buyer is broke
            }

            let offer_exhausted = offer.amount - filled <= 0.0;
            let order_satisfied = order.amount - filled <= 0.0;

            offer.amount -= filled;
            order.amount -= filled;

            // Apply money and stock changes. Skip self-trades entirely — a company
            // buying from itself has zero net effect on money or stock.
            if !is_self_trade {
                if let Some(buyer_entity) = buyer {
                    let (mut stock, money) = companies.get_mut(buyer_entity).unwrap();
                    // Consumers (no Money) receive the goods but pay from an unlimited budget.
                    if let Some(mut money) = money {
                        money.0 -= offer_price * filled;
                    }
                    *stock.resources.entry(order.resource).or_insert(0.0) += filled;
                }
                if let Some(seller_entity) = seller {
                    let (mut stock, money) = companies.get_mut(seller_entity).unwrap();
                    if let Some(mut money) = money {
                        money.0 += offer_price * filled;
                    }
                    // Deduct sold units from the seller's stock; floor at 0 to absorb rounding.
                    let held = stock.resources.entry(offer_resource).or_insert(0.0);
                    *held = (*held - filled).max(0.0);
                }
            }

            if buyer_is_company {
                if order_satisfied {
                    market_data.statistics.company_orders_fulfilled += 1;
                } else {
                    market_data.statistics.company_orders_partly_fulfilled += 1;
                }
            }
            if seller.is_some() {
                if offer_exhausted {
                    market_data.statistics.company_offers_fulfilled += 1;
                } else {
                    market_data.statistics.company_offers_partly_fulfilled += 1;
                }
            }

            if offer_exhausted {
                commands.entity(offer_entity).despawn();
            }
            if order_satisfied {
                commands.entity(order_entity).despawn();
            }

            // If neither supply nor demand was exhausted, the buyer's budget was
            // the binding constraint — no point trying the next offer.
            if !offer_exhausted && !order_satisfied {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::economy::money::Money;
    use crate::components::economy::offer::Offer;
    use crate::components::economy::order::Order;
    use crate::components::economy::stock::Stock;
    use crate::resources::economy::marketplace::Marketplace;

    fn app() -> App {
        let mut a = App::new();
        a.insert_resource(Marketplace::default());
        a.add_systems(Update, execute_orders);
        a
    }

    #[test]
    fn cheapest_offer_empty() {
        assert!(get_cheapest_offer(0, std::iter::empty::<(Entity, &Offer)>()).is_none());
    }

    #[test]
    fn cheapest_offer_filters_wrong_resource() {
        let offer = Offer { resource: 1, amount: 5.0, price_per_unit: 10.0, company: None };
        let e = World::new().spawn_empty().id();
        assert!(get_cheapest_offer(0, std::iter::once((e, &offer))).is_none());
    }

    #[test]
    fn cheapest_offer_filters_zero_amount() {
        let offer = Offer { resource: 0, amount: 0.0, price_per_unit: 10.0, company: None };
        let e = World::new().spawn_empty().id();
        assert!(get_cheapest_offer(0, std::iter::once((e, &offer))).is_none());
    }

    #[test]
    fn cheapest_offer_single() {
        let offer = Offer { resource: 0, amount: 5.0, price_per_unit: 10.0, company: None };
        let e = World::new().spawn_empty().id();
        assert_eq!(get_cheapest_offer(0, std::iter::once((e, &offer))), Some((e, 10.0)));
    }

    #[test]
    fn cheapest_offer_picks_min_price() {
        let mut world = World::new();
        let e_cheap = world.spawn_empty().id();
        let e_expensive = world.spawn_empty().id();
        let cheap = Offer { resource: 0, amount: 5.0, price_per_unit: 3.0, company: None };
        let expensive = Offer { resource: 0, amount: 5.0, price_per_unit: 9.0, company: None };
        let result = get_cheapest_offer(0, [(e_cheap, &cheap), (e_expensive, &expensive)].into_iter());
        assert_eq!(result, Some((e_cheap, 3.0)));
    }

    #[test]
    fn highest_order_empty() {
        assert!(get_highest_order(0, std::iter::empty::<(Entity, &Order)>()).is_none());
    }

    #[test]
    fn highest_order_picks_max_price() {
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        let o1 = Order { resource: 0, amount: 1.0, max_price_per_unit: 5.0, company: None };
        let o2 = Order { resource: 0, amount: 1.0, max_price_per_unit: 10.0, company: None };
        let result = get_highest_order(0, [(e1, &o1), (e2, &o2)].into_iter());
        assert_eq!(result, Some((e2, 10.0)));
    }

    #[test]
    fn order_fully_filled_by_offer() {
        let mut app = app();
        let offer_e = app
            .world_mut()
            .spawn(Offer { resource: 0, amount: 10.0, price_per_unit: 8.0, company: None })
            .id();
        let order_e = app
            .world_mut()
            .spawn(Order { resource: 0, amount: 5.0, max_price_per_unit: 10.0, company: None })
            .id();
        app.update();
        assert!(!app.world().entities().contains(order_e));
        assert_eq!(app.world().get::<Offer>(offer_e).unwrap().amount, 5.0);
    }

    #[test]
    fn order_not_filled_when_offer_too_expensive() {
        let mut app = app();
        app.world_mut()
            .spawn(Offer { resource: 0, amount: 10.0, price_per_unit: 15.0, company: None });
        let order_e = app
            .world_mut()
            .spawn(Order { resource: 0, amount: 5.0, max_price_per_unit: 10.0, company: None })
            .id();
        app.update();
        assert!(app.world().entities().contains(order_e));
    }

    #[test]
    fn offer_partially_consumed_by_small_order() {
        let mut app = app();
        let offer_e = app
            .world_mut()
            .spawn(Offer { resource: 0, amount: 10.0, price_per_unit: 5.0, company: None })
            .id();
        app.world_mut()
            .spawn(Order { resource: 0, amount: 3.0, max_price_per_unit: 10.0, company: None });
        app.update();
        assert_eq!(app.world().get::<Offer>(offer_e).unwrap().amount, 7.0);
    }

    #[test]
    fn consumer_order_no_company_lookup() {
        let mut app = app();
        let seller = app.world_mut().spawn((Stock::default(), Money(0.0))).id();
        app.world_mut().spawn(Offer {
            resource: 0,
            amount: 10.0,
            price_per_unit: 5.0,
            company: Some(seller),
        });
        let order_e = app
            .world_mut()
            .spawn(Order { resource: 0, amount: 5.0, max_price_per_unit: 10.0, company: None })
            .id();
        app.update();
        assert!(!app.world().entities().contains(order_e));
        assert_eq!(app.world().get::<Money>(seller).unwrap().0, 25.0);
    }

    // ── Money correctness tests ────────────────────────────────────────────

    #[test]
    fn buyer_pays_actual_price_on_full_fill() {
        // Order fully filled (offer.amount >= order.amount)
        let mut app = app();
        let buyer = app.world_mut().spawn((Stock::default(), Money(100.0))).id();
        app.world_mut().spawn(Offer { resource: 0, amount: 10.0, price_per_unit: 4.0, company: None });
        app.world_mut().spawn(Order {
            resource: 0,
            amount: 5.0,
            max_price_per_unit: 10.0,
            company: Some(buyer),
        });
        app.update();
        // buyer pays 4.0 * 5.0 = 20.0
        assert_eq!(app.world().get::<Money>(buyer).unwrap().0, 80.0);
        assert_eq!(*app.world().get::<Stock>(buyer).unwrap().resources.get(&0).unwrap(), 5.0);
    }

    #[test]
    fn buyer_pays_actual_price_on_partial_fill() {
        // Offer fully consumed (offer.amount < order.amount)
        let mut app = app();
        let buyer = app.world_mut().spawn((Stock::default(), Money(100.0))).id();
        app.world_mut().spawn(Offer { resource: 0, amount: 3.0, price_per_unit: 4.0, company: None });
        app.world_mut().spawn(Order {
            resource: 0,
            amount: 10.0,
            max_price_per_unit: 10.0,
            company: Some(buyer),
        });
        app.update();
        // buyer pays 4.0 * 3.0 = 12.0, receives 3 units
        assert_eq!(app.world().get::<Money>(buyer).unwrap().0, 88.0);
        assert_eq!(*app.world().get::<Stock>(buyer).unwrap().resources.get(&0).unwrap(), 3.0);
    }

    #[test]
    fn order_not_filled_when_buyer_is_broke() {
        let mut app = app();
        let buyer = app.world_mut().spawn((Stock::default(), Money(0.0))).id();
        app.world_mut()
            .spawn(Offer { resource: 0, amount: 10.0, price_per_unit: 5.0, company: None });
        let order_e = app
            .world_mut()
            .spawn(Order { resource: 0, amount: 5.0, max_price_per_unit: 10.0, company: Some(buyer) })
            .id();
        app.update();
        // buyer has no money — order must not be filled and money must stay at 0
        assert_eq!(app.world().get::<Money>(buyer).unwrap().0, 0.0);
        assert!(app.world().entities().contains(order_e));
    }

    #[test]
    fn order_capped_by_buyer_budget() {
        let mut app = app();
        // buyer can afford 5 units at price 4 (budget 20), but wants 8
        let buyer = app.world_mut().spawn((Stock::default(), Money(20.0))).id();
        app.world_mut()
            .spawn(Offer { resource: 0, amount: 10.0, price_per_unit: 4.0, company: None });
        let order_e = app
            .world_mut()
            .spawn(Order { resource: 0, amount: 8.0, max_price_per_unit: 10.0, company: Some(buyer) })
            .id();
        app.update();
        assert_eq!(app.world().get::<Money>(buyer).unwrap().0, 0.0);
        assert_eq!(*app.world().get::<Stock>(buyer).unwrap().resources.get(&0).unwrap(), 5.0);
        // order is still open (partially unfilled), money floor not breached
        assert!(app.world().entities().contains(order_e));
    }

    #[test]
    fn consumer_without_money_receives_goods_and_pays_nothing() {
        // A consumer has Stock but no Money: unlimited budget, goods delivered into storage,
        // and the selling company is still paid (money enters the economy from the consumer).
        let mut app = app();
        let seller = app.world_mut().spawn((Stock::default(), Money(0.0))).id();
        app.world_mut().spawn(Offer {
            resource: 0,
            amount: 10.0,
            price_per_unit: 5.0,
            company: Some(seller),
        });
        let consumer = app.world_mut().spawn(Stock::default()).id(); // no Money
        app.world_mut().spawn(Order {
            resource: 0,
            amount: 4.0,
            max_price_per_unit: 10.0,
            company: Some(consumer),
        });
        app.update();
        // consumer's storage grew by the filled amount, seller was paid 5.0 * 4.0
        assert_eq!(*app.world().get::<Stock>(consumer).unwrap().resources.get(&0).unwrap(), 4.0);
        assert_eq!(app.world().get::<Money>(seller).unwrap().0, 20.0);
        // ...and the consumer's purchase does not count toward company-order statistics
        let stats = &app.world().resource::<Marketplace>().statistics;
        assert_eq!(stats.company_orders_fulfilled, 0);
        assert_eq!(stats.company_orders_partly_fulfilled, 0);
    }

    #[test]
    fn seller_receives_payment_when_offer_fully_consumed() {
        // Offer fully consumed (offer.amount < order.amount)
        let mut app = app();
        let seller = app.world_mut().spawn((Stock::default(), Money(0.0))).id();
        app.world_mut().spawn(Offer {
            resource: 0,
            amount: 3.0,
            price_per_unit: 4.0,
            company: Some(seller),
        });
        app.world_mut().spawn(Order { resource: 0, amount: 10.0, max_price_per_unit: 10.0, company: None });
        app.update();
        // seller receives 4.0 * 3.0 = 12.0
        assert_eq!(app.world().get::<Money>(seller).unwrap().0, 12.0);
    }
}
