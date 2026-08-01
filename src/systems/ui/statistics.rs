use crate::components::economy::offer::Offer;
use crate::components::economy::order::Order;
use crate::resources::economy::marketplace::Marketplace;
use crate::resources::economy::resources::Resources;
use crate::resources::sim_history::SimHistory;
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

pub fn draw_marketplace_dashboard(
    mut contexts: EguiContexts,
    history: Res<SimHistory>,
    marketplace: Res<Marketplace>,
    offers: Query<&Offer>,
    orders: Query<&Order>,
    resources: Res<Resources>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };

    egui::Window::new("Marketplace Dashboard")
        .default_width(600.0)
        .default_height(550.0)
        .resizable(true)
        .show(ctx, |ui| {
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

            let sorted_resources: Vec<(usize, String)> = resources
                .resources
                .iter()
                .sorted_by_key(|&(&id, _)| id)
                .map(|(&id, name)| (id, name.clone()))
                .collect();

            Plot::new("marketplace_price_plot")
                .height(200.0)
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
