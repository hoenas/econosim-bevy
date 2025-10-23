use crate::components::economy::offer::{Offer, OfferHandle};
use crate::components::economy::order::{Order, OrderHandle};
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::resources::{ResourceHandle, Resources};
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::*;
use itertools::Itertools;

pub fn update_price_index(
    offers: Query<&Offer>,
    resources: Res<Resources>,
    mut market_data: ResMut<Marketplace>,
) {
    for resource_handle in resources.resources.keys().sorted() {
        let offer = get_cheapest_offer(*resource_handle, offers);
        market_data.price_index.insert(*resource_handle, offer);
    }
}

pub fn update_order_index(
    orders: Query<&Order>,
    resources: Res<Resources>,
    mut market_data: ResMut<Marketplace>,
) {
    for resource_handle in resources.resources.keys().sorted() {
        let order = get_highest_order(*resource_handle, orders);
        market_data.order_index.insert(*resource_handle, order);
    }
}

pub fn get_cheapest_offer(
    resource: ResourceHandle,
    offers: Query<&Offer>,
) -> Option<(OfferHandle, f64)> {
    let mut cheapest_offer: Option<(OfferHandle, f64)> = None;
    for offer in offers.iter() {
        if offer.resource == resource {
            if cheapest_offer.is_none() {
                cheapest_offer = Some((offer.handle, offer.price_per_unit));
            } else if cheapest_offer.unwrap().1 > offer.price_per_unit {
                cheapest_offer = Some((offer.handle, offer.price_per_unit));
            }
        }
    }
    cheapest_offer
}

pub fn get_highest_order(
    resource: ResourceHandle,
    orders: Query<&Order>,
) -> Option<(OrderHandle, f64)> {
    let mut highest_order: Option<(OrderHandle, f64)> = None;
    for order in orders.iter() {
        if order.resource == resource {
            if highest_order.is_none() {
                highest_order = Some((order.handle, order.max_price_per_unit));
            } else if highest_order.unwrap().1 < order.max_price_per_unit {
                highest_order = Some((order.handle, order.max_price_per_unit));
            }
        }
    }
    highest_order
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

pub fn get_offer_by_handle(
    offer_handle: OfferHandle,
    market_data: &mut MarketData,
) -> Option<&Offer> {
    market_data.offers.get(&offer_handle)
}

pub fn get_order_by_handle(
    order_handle: OrderHandle,
    market_data: &mut MarketData,
) -> Option<&Order> {
    market_data.orders.get(&order_handle)
}

fn execute_orders(market_data: &mut MarketData, companies: &mut Vec<Company>) {
    // Check all orders
    for order in market_data.orders.values_mut() {
        // We are trying to fulfill the hole order
        while order.amount > 0.0 {
            match self.get_cheapest_offer(order.resource, &market_data.offers) {
                Some(value) => {
                    let offer_handle = value.0;
                    let offer_price = value.1;
                    if offer_price <= order.max_price_per_unit {
                        match market_data.offers.get_mut(&offer_handle) {
                            Some(offer) => {
                                if offer.amount < order.amount {
                                    // Offer will be consumed
                                    // Order will be partly finished
                                    // Consume offer
                                    order.amount -= offer.amount;
                                    // Check if the order was created by a real company
                                    match order.company {
                                        // Give resources to company
                                        Some(ordering_company) => {
                                            companies[ordering_company]
                                                .stock
                                                .add_resource_to_stock(
                                                    order.resource,
                                                    offer.amount,
                                                );
                                            // Give delta currency from max price back
                                            let price_delta = (order.max_price_per_unit
                                                - offer.price_per_unit)
                                                * offer.amount;
                                            companies[ordering_company].add_currency(price_delta);
                                            self.statistics.company_orders_partly_fulfilled += 1;
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
                                                .add_resource_to_stock(
                                                    order.resource,
                                                    order.amount,
                                                );
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
                    } else {
                        break;
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
