use crate::components::common::TimeToLive;
use crate::components::economy::company::CompanyHandle;
use crate::resources::economy::common::Id;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Component;

pub type OrderHandle = usize;

#[derive(Component, Default)]
pub struct Order {
    pub company: Option<CompanyHandle>,
    pub resource: Id,
    pub amount: f64,
    pub max_price_per_unit: f64,
    pub handle: OrderHandle,
}

#[derive(Bundle)]
pub struct OrderBundle {
    pub order: Order,
    pub time_to_live: TimeToLive,
}
