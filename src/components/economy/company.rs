use crate::components::common::Name;
use crate::components::economy::currency::Currency;
use crate::components::economy::processor::Processors;
use crate::components::economy::stock::Stock;
use bevy::ecs::bundle::Bundle;

#[derive(Bundle)]
pub struct Company {
    pub stock: Stock,
    pub currency: Currency,
    pub processors: Processors,
    pub name: Name,
}
