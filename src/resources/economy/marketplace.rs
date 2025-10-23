use crate::components::economy::offer::OfferHandle;
use crate::components::economy::order::OrderHandle;
use crate::resources::economy::resources::ResourceHandle;
use bevy::prelude::Resource;
use std::collections::HashMap;

#[derive(Default)]
pub struct MarketplaceStatistics {
    pub company_orders_placed: usize,
    pub company_offers_placed: usize,
    pub company_orders_partly_fulfilled: usize,
    pub company_offers_partly_fulfilled: usize,
    pub company_orders_fulfilled: usize,
    pub company_offers_fulfilled: usize,
}

#[derive(Resource, Default)]
pub struct Marketplace {
    pub statistics: MarketplaceStatistics,
    pub price_index: HashMap<ResourceHandle, Option<(OfferHandle, f64)>>,
    pub order_index: HashMap<ResourceHandle, Option<(OrderHandle, f64)>>,
    pub next_offer_id: usize,
    pub next_order_id: usize,
}
