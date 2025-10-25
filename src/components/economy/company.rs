use crate::components::common::Name;
use crate::components::economy::currency::Currency;
use crate::components::economy::processor::Processors;
use crate::components::economy::stock::Stock;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;

#[derive(Component, Default)]
pub struct CompanyMarker();

#[derive(Bundle)]
pub struct Company {
    pub stock: Stock,
    pub currency: Currency,
    pub processors: Processors,
    pub name: Name,
    pub marker: CompanyMarker,
}
