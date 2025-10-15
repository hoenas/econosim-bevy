use crate::components::common::ComponentId;
use crate::components::common::Id;
use crate::components::common::Name;
use crate::components::economy::production_speed::ProductionSpeed;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Ingredients(pub std::collections::HashMap<ComponentId, f64>);

#[derive(Component)]
pub struct Products(pub std::collections::HashMap<ComponentId, f64>);

#[derive(Bundle)]
pub struct Recipe {
    pub name: Name,
    pub ingredients: Ingredients,
    pub products: Products,
    pub production_speed: ProductionSpeed,
    pub id: Id,
}
