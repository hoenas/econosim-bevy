use bevy::color::Color;
use bevy::prelude::{Entity, Resource};
use std::collections::HashMap;

pub struct CompanyRecord {
    pub name: String,
    pub color: Color,
    /// Full money balance history — one entry per simulation tick.
    pub money_history: Vec<f64>,
    /// Rolling buffer of human-readable action labels.
    pub action_history: Vec<String>,
}

#[derive(Resource, Default)]
pub struct SimHistory {
    pub companies: HashMap<Entity, CompanyRecord>,
}
