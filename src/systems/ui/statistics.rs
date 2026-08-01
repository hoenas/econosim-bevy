use crate::components::economy::offer::Offer;
use crate::components::economy::order::Order;
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::resources::Resources;
use crate::resources::sim_history::{SimHistory, MAX_SUPPLY_DEMAND_SNAPSHOTS};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui_plot::{Legend, Line, LineStyle, Plot, PlotPoints};
use itertools::Itertools;

fn resource_color(id: usize) -> egui::Color32 {
    const PALETTE: &[(u8, u8, u8)] = &[
        (100, 180, 255), // blue   – Water
        (160, 120, 60),  // brown  – Dirt
        (80, 180, 80),   // green  – Wood
        (160, 160, 160), // gray   – Coal
    ];
    let (r, g, b) = PALETTE[id % PALETTE.len()];
    egui::Color32::from_rgb(r, g, b)
}

/// Returns a color at reduced opacity. `age` 0 = newest (fully visible), `depth` = oldest shown.
fn faded(base: egui::Color32, age: usize, depth: usize) -> egui::Color32 {
    let t = if depth == 0 { 1.0_f32 } else { 1.0 - age as f32 / depth as f32 };
    let alpha = (t * 215.0 + 40.0) as u8;
    egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha)
}

fn supply_steps(pairs: &[(f64, f64)]) -> Vec<[f64; 2]> {
    let mut sorted = pairs.to_vec();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut pts = Vec::with_capacity(sorted.len() * 2);
    let mut cum = 0.0_f64;
    for (price, amount) in sorted {
        pts.push([cum, price]);
        cum += amount;
        pts.push([cum, price]);
    }
    pts
}

fn demand_steps(pairs: &[(f64, f64)]) -> Vec<[f64; 2]> {
    let mut sorted = pairs.to_vec();
    sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut pts = Vec::with_capacity(sorted.len() * 2);
    let mut cum = 0.0_f64;
    for (price, amount) in sorted {
        pts.push([cum, price]);
        cum += amount;
        pts.push([cum, price]);
    }
    pts
}

pub struct MarketUiState {
    selected_resource: usize,
    history_depth: usize, // number of past ticks to ghost behind the current curve
}

impl Default for MarketUiState {
    fn default() -> Self {
        Self { selected_resource: 0, history_depth: 5 }
    }
}

pub fn draw_marketplace_dashboard(
    mut contexts: EguiContexts,
    history: Res<SimHistory>,
    marketplace: Res<Marketplace>,
    offers: Query<&Offer>,
    orders: Query<&Order>,
    resources: Res<Resources>,
    mut state: Local<MarketUiState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };

    egui::Window::new("Marketplace Dashboard")
        .default_width(600.0)
        .default_height(700.0)
        .resizable(true)
        .show(ctx, |ui| {
            let sorted_resources: Vec<(usize, String)> = resources
                .resources
                .iter()
                .sorted_by_key(|&(&id, _)| id)
                .map(|(&id, name)| (id, name.clone()))
                .collect();

            // ── Trade statistics ───────────────────────────────────────────
            ui.heading("Trade Statistics");
            let s = &marketplace.statistics;
            egui::Grid::new("trade_stats_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Orders placed:");
                    ui.label(s.company_orders_placed.to_string());
                    ui.end_row();
                    ui.label("Offers placed:");
                    ui.label(s.company_offers_placed.to_string());
                    ui.end_row();
                    ui.label("Orders fulfilled:");
                    ui.label(s.company_orders_fulfilled.to_string());
                    ui.end_row();
                    ui.label("Orders partly fulfilled:");
                    ui.label(s.company_orders_partly_fulfilled.to_string());
                    ui.end_row();
                    ui.label("Offers fulfilled:");
                    ui.label(s.company_offers_fulfilled.to_string());
                    ui.end_row();
                    ui.label("Offers partly fulfilled:");
                    ui.label(s.company_offers_partly_fulfilled.to_string());
                    ui.end_row();
                    ui.label("Open orders:");
                    ui.label(orders.iter().count().to_string());
                    ui.end_row();
                    ui.label("Open offers:");
                    ui.label(offers.iter().count().to_string());
                    ui.end_row();
                });

            ui.separator();

            // ── Price history plot ─────────────────────────────────────────
            ui.heading("Price History");
            ui.small("Solid = best offer (supply)  ·  Dashed = best order (demand)");

            Plot::new("marketplace_price_plot")
                .height(180.0)
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    let mp = &history.marketplace;
                    for (id, name) in &sorted_resources {
                        let color = resource_color(*id);
                        if let Some(prices) = mp.offer_price_history.get(id) {
                            let points: PlotPoints = prices
                                .iter()
                                .enumerate()
                                .map(|(i, &p)| [i as f64, p])
                                .collect();
                            plot_ui.line(
                                Line::new(name.as_str(), points)
                                    .color(color)
                                    .width(2.0_f32),
                            );
                        }
                        if let Some(prices) = mp.order_price_history.get(id) {
                            let points: PlotPoints = prices
                                .iter()
                                .enumerate()
                                .map(|(i, &p)| [i as f64, p])
                                .collect();
                            plot_ui.line(
                                Line::new(format!("{} (demand)", name), points)
                                    .color(color)
                                    .width(1.5_f32)
                                    .style(LineStyle::Dashed { length: 8.0 }),
                            );
                        }
                    }
                });

            ui.separator();

            // ── Supply & Demand curve ──────────────────────────────────────
            ui.heading("Supply & Demand");

            if sorted_resources.is_empty() {
                return;
            }

            state.selected_resource = state.selected_resource.min(sorted_resources.len() - 1);
            let sel_idx = state.selected_resource;
            let (sel_id, sel_name) = &sorted_resources[sel_idx];

            // Controls row
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Resource")
                    .selected_text(sel_name.as_str())
                    .show_ui(ui, |ui| {
                        for (i, (_, name)) in sorted_resources.iter().enumerate() {
                            ui.selectable_value(&mut state.selected_resource, i, name.as_str());
                        }
                    });
            });

            let available = history.marketplace.supply_demand_history.len();
            let max_depth = available.saturating_sub(1).min(MAX_SUPPLY_DEMAND_SNAPSHOTS - 1);
            state.history_depth = state.history_depth.min(max_depth);
            ui.add(
                egui::Slider::new(&mut state.history_depth, 0..=max_depth)
                    .text("History depth (ticks)"),
            );
            ui.small("Green = supply (offers) · Red = demand (orders) · Faded = older ticks");

            // How many snapshots to render: current + history_depth past ones
            let show_count = (state.history_depth + 1).min(available);
            let start_idx = available.saturating_sub(show_count);
            let depth = state.history_depth;

            Plot::new("supply_demand_plot")
                .height(220.0)
                .auto_bounds(egui::Vec2b::TRUE)
                .show(ui, |plot_ui| {
                    let supply_base = egui::Color32::from_rgb(80, 200, 120);
                    let demand_base = egui::Color32::from_rgb(220, 80, 80);

                    let snaps: Vec<_> = history
                        .marketplace
                        .supply_demand_history
                        .iter()
                        .skip(start_idx)
                        .collect();

                    let n = snaps.len();
                    for (slot, snap) in snaps.into_iter().enumerate() {
                        // slot 0 = oldest visible, slot n-1 = newest
                        let age = n - 1 - slot;
                        let is_current = age == 0;
                        let s_color = faded(supply_base, age, depth);
                        let d_color = faded(demand_base, age, depth);
                        let width = if is_current { 2.0_f32 } else { 1.0_f32 };

                        if let Some(pairs) = snap.offers.get(sel_id) {
                            let pts = supply_steps(pairs);
                            if !pts.is_empty() {
                                let label = if is_current { "Supply".to_string() } else { String::new() };
                                plot_ui.line(
                                    Line::new(label, PlotPoints::new(pts))
                                        .color(s_color)
                                        .width(width),
                                );
                            }
                        }
                        if let Some(pairs) = snap.orders.get(sel_id) {
                            let pts = demand_steps(pairs);
                            if !pts.is_empty() {
                                let label = if is_current { "Demand".to_string() } else { String::new() };
                                plot_ui.line(
                                    Line::new(label, PlotPoints::new(pts))
                                        .color(d_color)
                                        .width(width),
                                );
                            }
                        }
                    }
                });

            ui.separator();

            // ── Current price index ────────────────────────────────────────
            ui.heading("Current Prices");
            egui::Grid::new("price_index_grid")
                .striped(true)
                .num_columns(3)
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.strong("Resource");
                    ui.strong("Best offer");
                    ui.strong("Best order");
                    ui.end_row();
                    for (id, name) in &sorted_resources {
                        let offer_str = marketplace
                            .price_index
                            .get(id)
                            .and_then(|opt| opt.as_ref())
                            .map(|(_, p)| format!("{:.2}", p))
                            .unwrap_or_else(|| "—".to_string());
                        let order_str = marketplace
                            .order_index
                            .get(id)
                            .and_then(|opt| opt.as_ref())
                            .map(|(_, p)| format!("{:.2}", p))
                            .unwrap_or_else(|| "—".to_string());
                        ui.colored_label(resource_color(*id), name);
                        ui.label(offer_str);
                        ui.label(order_str);
                        ui.end_row();
                    }
                });
        });
}
