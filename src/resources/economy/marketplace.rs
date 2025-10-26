use crate::resources::economy::resources::ResourceHandle;
use bevy::{ecs::entity::Entity, prelude::Resource};
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
    pub price_index: HashMap<ResourceHandle, Option<(Entity, f64)>>,
    pub order_index: HashMap<ResourceHandle, Option<(Entity, f64)>>,
}
