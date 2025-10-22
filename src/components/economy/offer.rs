use crate::components::economy::company::CompanyHandle;
use crate::resources::economy::common::Id;
use bevy::prelude::Component;

#[derive(Component, Default)]
pub struct Offer {
    pub resource: Id,
    pub amount: f64,
    pub price_per_unit: f64,
    pub company: Option<CompanyHandle>,
    pub time_to_live: usize,
}
