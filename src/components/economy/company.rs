use crate::components::common::Name;
use crate::components::common::RenderColor;
use crate::components::economy::money::LastTickMoney;
use crate::components::economy::money::Money;
use crate::components::economy::processor::Processors;
use crate::components::economy::stock::Stock;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;

#[derive(Component, Default)]
pub struct CompanyMarker();

#[derive(Bundle)]
pub struct Company {
    pub stock: Stock,
    pub money: Money,
    pub last_tick_money: LastTickMoney,
    pub processors: Processors,
    pub name: Name,
    pub color: RenderColor,
    pub marker: CompanyMarker,
}
