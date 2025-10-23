use crate::components::economy::company::CompanyHandle;
use crate::resources::economy::common::Id;
use crate::resources::economy::resources::ResourceHandle;
use bevy::prelude::Component;

pub type OfferHandle = usize;

#[derive(Component, Default)]
pub struct Offer {
    pub resource: Id,
    pub amount: f64,
    pub price_per_unit: f64,
    pub company: Option<CompanyHandle>,
    pub time_to_live: usize,
    pub handle: OfferHandle,
}
