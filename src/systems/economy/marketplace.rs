use crate::components::economy::company::Company;
use crate::components::economy::currency::Currency;
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
        .filter(|(_, order)| order.resource == resource)
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

/*
pub fn place_offer(offer: Offer, market_data: &mut MarketData) -> Option<OfferHandle> {
    // Offer sanity checks
    if offer.amount <= 0.0 || offer.resource >= market_data.resource_count {
        return None;
    }
    if offer.company.is_some() {
        self.statistics.company_offers_placed += 1;
    }
    self.next_offer_id += 1;
    market_data.offers.insert(self.next_offer_id, offer);
    self.update_price_index(market_data);
    Some(self.next_offer_id)
}

pub fn place_order(order: Order, market_data: &mut MarketData) -> Option<OfferHandle> {
    // Order sanity checks
    if order.amount <= 0.0 || order.resource >= market_data.resource_count {
        return None;
    }
    if order.company.is_some() {
        self.statistics.company_orders_placed += 1;
    }
    self.next_order_id += 1;
    market_data.orders.insert(self.next_order_id, order);
    self.update_order_index(market_data);
    Some(self.next_order_id)
}
*/

fn execute_orders(
    mut orders: Query<&mut Order>,
    mut offers: Query<(Entity, &mut Offer)>,
    mut companies: Query<(&mut Stock, &mut Currency)>,
    resources: Res<Resources>,
    mut market_data: ResMut<Marketplace>,
) {
    // Check all orders
    for mut order in orders.iter_mut() {
        // We are trying to fulfill the whole order
        while order.amount > 0.0 {
            match get_cheapest_offer(order.resource, offers.as_readonly().iter_mut()) {
                Some((offer, value)) => {
                    let offer = offers.get_mut(offer).unwrap().1;
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
                                let (mut ordering_stock, mut odering_currency) =
                                    companies.get_mut(ordering_company).unwrap();
                                // Give delta currency from max price back
                                let price_delta = (order.max_price_per_unit - offer.price_per_unit)
                                    * offer.amount;
                                odering_currency.0 += price_delta;
                                // Give resources to ordering company
                                let amount = ordering_stock
                                    .resources
                                    .get(&order.resource)
                                    .unwrap_or(&0.0)
                                    + offer.amount;
                                ordering_stock.resources.insert(order.resource, amount);
                            }
                            None => {
                                // Order was created by a consumer
                                // No company to add resources to
                            }
                        }
                    }
                }
                None => {
                    break;
                }
            }
        }
    }
}

/*
                                // Pay out offering company if it exists
                                match offer.company {
                                    Some(offering_company) => {
                                        companies[offering_company]
                                            .add_currency(offer.price_per_unit * offer.amount);
                                        self.statistics.company_offers_fulfilled += 1;
                                    }
                                    None => {
                                        // Offer was created by a producer
                                        // No company to add currency to
                                    }
                                }

                                // We consumed the hole amount of the offer and must therefore remove it from the market
                                market_data.offers.remove(&offer_handle);
                            } else {
                                // Offer will be partly consumed
                                // Order will be finished
                                // Check if the order was created by a real company
                                match order.company {
                                    Some(ordering_company) => {
                                        // Give resources to ordering company
                                        companies[ordering_company]
                                            .stock
                                            .add_resource_to_stock(order.resource, order.amount);
                                        // Give delta currency from max price back
                                        let price_delta = (order.max_price_per_unit
                                            - offer.price_per_unit)
                                            * order.amount;
                                        companies[ordering_company].add_currency(price_delta);
                                        self.statistics.company_orders_fulfilled += 1;
                                    }
                                    None => {
                                        // Order was created by a consumer
                                        // No company to add resources to and remove currency from
                                    }
                                }
                                // Pay out offering company if it exists
                                match offer.company {
                                    Some(offering_company) => {
                                        companies[offering_company]
                                            .add_currency(offer.price_per_unit * order.amount);
                                        self.statistics.company_offers_partly_fulfilled += 1;
                                    }
                                    None => {
                                        // Offer was created by a producer
                                        // No company to add currency to
                                    }
                                }
                                // Reduce offer and order amount
                                offer.amount -= order.amount;
                                order.amount = 0.0;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }
                None => {
                    break;
                }
            }
        }
    }
}

*/
