use crate::components::economy::company::CompanyHandle;
use crate::resources::economy::common::Id;
use bevy::prelude::Component;

#[derive(Component, Default)]
pub struct Order {
    pub company: Option<CompanyHandle>,
    pub resource: Id,
    pub amount: f64,
    pub max_price_per_unit: f64,
    pub time_to_live: usize,
}
