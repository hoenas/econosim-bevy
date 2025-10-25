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
) -> Option<Entity> {
    offers
        .filter(|(_, order)| order.resource == resource)
        .max_by_key(|x| OrderedFloat(x.1.price_per_unit))
        .map(|x| x.0)
}

pub fn get_highest_order<'a>(
    resource: ResourceHandle,
    orders: impl Iterator<Item = (Entity, &'a Order)>,
) -> Option<Entity> {
    orders
        .filter(|(_, order)| order.resource == resource)
        .max_by_key(|x| OrderedFloat(x.1.max_price_per_unit))
        .map(|x| x.0)
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
                Some(offer_entity) => {
                    let mut offer = offers.get_mut(offer_entity).unwrap().1;
                    if offer.price_per_unit > order.max_price_per_unit {
                        break;
                    }
                    if offer.amount < order.amount {
                        // Offer will be consumed
                        // Order will be partly finished
                        // Consume offer
                        order.amount -= offer.amount;
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
