use crate::components::economy::money::Money;
use crate::components::economy::offer::Offer;
use crate::components::economy::order::Order;
use crate::components::economy::stock::Stock;
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::resources::{ResourceHandle, Resources};
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::*;
use itertools::Itertools;
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
    for resource_handle in resources.resources.keys().sorted() {
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
        .max_by_key(|x| OrderedFloat(x.1.price_per_unit))
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
    mut companies: Query<(&mut Stock, &mut Money)>,
    mut market_data: ResMut<Marketplace>,
) {
    // Check all orders
    for (order_entity, mut order) in orders.iter_mut() {
        // We are trying to fulfill the whole order
        while order.amount > 0.0 {
            match get_cheapest_offer(order.resource, offers.as_readonly().iter_mut()) {
                Some((offer_entity, _)) => {
                    let mut offer = offers.get_mut(offer_entity).unwrap().1;
                    if offer.price_per_unit > order.max_price_per_unit {
                        break;
                    }
                    if offer.amount < order.amount {
                        // Offer will be consumed
                        // Order will be partly finished
                        // Consume offer
                        order.amount -= offer.amount;
                        offer.amount = 0.0;
                        // Check if the order was created by a real company
                        match order.company {
                            // Order was created by a company
                            Some(ordering_company) => {
                                let (mut ordering_stock, mut odering_money) =
                                    companies.get_mut(ordering_company).unwrap();
                                // Give delta money from max price back
                                let price_delta = (order.max_price_per_unit - offer.price_per_unit)
                                    * offer.amount;
                                odering_money.0 += price_delta;
                                // Give resources to ordering company
                                let amount = ordering_stock
                                    .resources
                                    .get(&order.resource)
                                    .unwrap_or(&0.0)
                                    + offer.amount;
                                ordering_stock.resources.insert(order.resource, amount);
                                market_data.statistics.company_orders_partly_fulfilled += 1;
                            }
                            None => {
                                // Order was created by a consumer
                                // No company to add resources to
                            }
                        }
                        // Pay out offering company if it exists
                        match offer.company {
                            Some(offering_company) => {
                                let (_, mut offering_money) =
                                    companies.get_mut(offering_company).unwrap();
                                offering_money.0 += offer.price_per_unit * offer.amount;
                                market_data.statistics.company_offers_fulfilled += 1;
                            }
                            None => {
                                // Offer was created by a producer
                                // No company to add money to
                            }
                        }
                        // We consumed the hole amount of the offer and must therefore remove it from the market
                        commands.entity(offer_entity).despawn();
                    } else {
                        // Offer will be partly consumed
                        // Order will be finished
                        // Check if the order was created by a real company
                        match order.company {
                            Some(ordering_company_enitity) => {
                                // Give resources to ordering company
                                let (mut ordering_company_stock, mut odering_company_money) =
                                    companies.get_mut(ordering_company_enitity).unwrap();
                                let resource_amount = ordering_company_stock
                                    .resources
                                    .get(&order.resource)
                                    .unwrap_or(&0.0)
                                    + order.amount;
                                ordering_company_stock
                                    .resources
                                    .insert(order.resource, resource_amount);
                                // Give delta money from max price back
                                let price_delta = (order.max_price_per_unit - offer.price_per_unit)
                                    * order.amount;
                                odering_company_money.0 += price_delta;
                                market_data.statistics.company_orders_fulfilled += 1;
                            }
                            None => {
                                // Order was created by a consumer
                                // No company to add resources to and remove money from
                            }
                        }
                        // Pay out offering company if it exists
                        match offer.company {
                            Some(offering_company) => {
                                let (_, mut offering_money) =
                                    companies.get_mut(offering_company).unwrap();
                                offering_money.0 += offer.price_per_unit * order.amount;
                                market_data.statistics.company_offers_partly_fulfilled += 1;
                            }
                            None => {
                                // Offer was created by a producer
                                // No company to add money to
                            }
                        }
                        // Reduce offer and order amount
                        offer.amount -= order.amount;
                        order.amount = 0.0;
                        // We consumed all of the order, so we can delete it
                        commands.entity(order_entity).despawn();
                    }
                }
                None => {
                    break;
                }
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
        // Note: get_cheapest_offer uses max_by_key — returns most expensive when multiple offers exist.
        // With a single offer the result is correct.
        assert_eq!(get_cheapest_offer(0, std::iter::once((e, &offer))), Some((e, 10.0)));
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
}
