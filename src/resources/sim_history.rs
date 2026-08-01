use bevy::color::Color;
use bevy::prelude::{Entity, Resource};
use std::collections::HashMap;

pub struct CompanyRecord {
    pub name: String,
    pub color: Color,
    /// Full money balance history — one entry per simulation tick.
    pub money_history: Vec<f64>,
    /// Rolling buffer of (label, confidence) pairs — one entry per simulation tick.
    /// Confidence is `None` for exploratory (random) actions.
    pub action_history: Vec<(String, Option<f32>)>,
}

/// Per-resource price histories captured each FixedUpdate tick.
#[derive(Default)]
pub struct MarketplaceRecord {
    /// Best offer price (supply side) per resource, one entry per tick.
    pub offer_price_history: HashMap<usize, Vec<f64>>,
    /// Best order price (demand side) per resource, one entry per tick.
    pub order_price_history: HashMap<usize, Vec<f64>>,
}

#[derive(Resource, Default)]
pub struct SimHistory {
    pub companies: HashMap<Entity, CompanyRecord>,
    pub marketplace: MarketplaceRecord,
}
