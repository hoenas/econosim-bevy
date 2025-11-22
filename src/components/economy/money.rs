use bevy::prelude::Component;

#[derive(Component)]
pub struct Money(pub f64);

#[derive(Component)]
pub struct LastTickMoney(pub f64);
