use bevy::color::Color;
use bevy::prelude::{Entity, Resource};
use std::collections::{HashMap, VecDeque};

pub const MAX_SUPPLY_DEMAND_SNAPSHOTS: usize = 60;
pub const MAX_MONEY_HISTORY: usize = 1_000;
pub const MAX_PRICE_HISTORY: usize = 1_000;
pub const MAX_ACTION_HISTORY: usize = 200;

pub struct CompanyRecord {
    pub name: String,
    pub color: Color,
    /// Rolling window of money balances — one entry per simulation tick.
    pub money_history: VecDeque<f64>,
    /// Rolling window of (label, confidence) pairs — one entry per simulation tick.
    /// Confidence is `None` for exploratory (random) actions.
    pub action_history: VecDeque<(String, Option<f32>)>,
}

/// One tick's worth of raw offer/order data per resource.
pub struct MarketSnapshot {
    /// resource_id → [(price_per_unit, amount)] for open offers
    pub offers: HashMap<usize, Vec<(f64, f64)>>,
    /// resource_id → [(max_price_per_unit, amount)] for open orders
    pub orders: HashMap<usize, Vec<(f64, f64)>>,
}

/// Per-resource price histories and supply-demand snapshots captured each FixedUpdate tick.
#[derive(Default)]
pub struct MarketplaceRecord {
    /// Best offer price (supply side) per resource — rolling window.
    pub offer_price_history: HashMap<usize, VecDeque<f64>>,
    /// Best order price (demand side) per resource — rolling window.
    pub order_price_history: HashMap<usize, VecDeque<f64>>,
    /// Rolling window of raw offer/order snapshots (oldest first).
    pub supply_demand_history: VecDeque<MarketSnapshot>,
}

#[derive(Resource, Default)]
pub struct SimHistory {
    pub companies: HashMap<Entity, CompanyRecord>,
    pub marketplace: MarketplaceRecord,
}
