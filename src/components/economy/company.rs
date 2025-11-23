use crate::components::common::Name;
use crate::components::common::RenderColor;
use crate::components::economy::money::LastTickMoney;
use crate::components::economy::money::Money;
use crate::components::economy::processor::Processors;
use crate::components::economy::stock::Stock;
use crate::components::reinforcement_learning::action::CompanyAction;
use crate::components::reinforcement_learning::company_state::CompanyState;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;

#[derive(Component, Default)]
pub struct CompanyMarker();

#[derive(Bundle)]
pub struct Company {
    pub stock: Stock,
    pub money: Money,
    pub last_tick_money: LastTickMoney,
    pub last_state: CompanyState,
    pub last_action: CompanyAction,
    pub processors: Processors,
    pub name: Name,
    pub color: RenderColor,
    pub marker: CompanyMarker,
}
