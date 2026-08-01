use crate::components::common::{Name, RenderColor};
use crate::components::economy::money::Money;
use crate::components::economy::processor::Processors;
use crate::components::economy::stock::Stock;
use crate::components::reinforcement_learning::action::CompanyAction;
use crate::resources::economy::common::Currency;
use crate::resources::economy::recipes::Recipes;
use crate::resources::economy::resources::Resources;
use crate::resources::reinforcement_learning::action_space::{ActionSpace, CompanyActionEnum};
use crate::resources::sim_history::{CompanyRecord, SimHistory};
use crate::resources::sim_state::SimState;
use bevy::color::Color;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::time::Duration;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use itertools::Itertools;

// Keeps the rolling action log from growing without bound.
const ACTION_HISTORY_CAP: usize = 200;

fn action_label(
    idx: usize,
    action_space: &ActionSpace,
    resources: &Resources,
    recipes: &Recipes,
) -> String {
    match action_space.actions.get(idx) {
        None => "?".to_string(),
        Some(CompanyActionEnum::Nothing) => "Idle".to_string(),
        Some(CompanyActionEnum::BuyProcessor(r)) => {
            let name = recipes.recipes.get(r).map(|r| r.name.as_str()).unwrap_or("?");
            format!("+Proc({})", name)
        }
        Some(CompanyActionEnum::SellProcessor(r)) => {
            let name = recipes.recipes.get(r).map(|r| r.name.as_str()).unwrap_or("?");
            format!("-Proc({})", name)
        }
        Some(CompanyActionEnum::BuyResource(r, a)) => {
            let name = resources.resources.get(r).map(|s| s.as_str()).unwrap_or("?");
            format!("Buy {}×{}", a, name)
        }
        Some(CompanyActionEnum::SellResource(r, a)) => {
            let name = resources.resources.get(r).map(|s| s.as_str()).unwrap_or("?");
            format!("Sell {}×{}", a, name)
        }
    }
}

fn bevy_to_egui(color: Color) -> egui::Color32 {
    let c = color.to_srgba();
    egui::Color32::from_rgb(
        (c.red.clamp(0.0, 1.0) * 255.0) as u8,
        (c.green.clamp(0.0, 1.0) * 255.0) as u8,
        (c.blue.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

/// Records each company's money balance and last action every tick.
pub fn update_sim_history(
    mut history: ResMut<SimHistory>,
    query: Query<(Entity, &Name, &Money, &CompanyAction, &RenderColor)>,
    action_space: Res<ActionSpace>,
    resources: Res<Resources>,
    recipes: Res<Recipes>,
) {
    for (entity, name, money, action, color) in query.iter() {
        let record = history.companies.entry(entity).or_insert_with(|| CompanyRecord {
            name: name.0.clone(),
            color: color.0,
            money_history: Vec::new(),
            action_history: Vec::new(),
        });
        record.money_history.push(money.0);
        let label = action_label(action.0, &action_space, &resources, &recipes);
        record.action_history.push(label);
        if record.action_history.len() > ACTION_HISTORY_CAP {
            record.action_history.remove(0);
        }
    }
}

/// Draws the Company Dashboard egui window.
pub fn draw_dashboard(
    mut contexts: EguiContexts,
    history: Res<SimHistory>,
    query: Query<(Entity, &Name, &Stock, &Money, &Processors, &RenderColor)>,
    resources: Res<Resources>,
    currency: Res<Currency>,
    mut time_fixed: ResMut<Time<Fixed>>,
    mut sim_state: ResMut<SimState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };

    egui::Window::new("Company Dashboard")
        .default_width(720.0)
        .default_height(600.0)
        .resizable(true)
        .show(ctx, |ui| {
            // ── Playback controls ──────────────────────────────────────────
            ui.horizontal(|ui| {
                let pause_label = if sim_state.paused { "▶ Resume" } else { "⏸ Pause" };
                if ui.button(pause_label).clicked() {
                    sim_state.paused = !sim_state.paused;
                }
                let step_btn = ui.add_enabled(sim_state.paused, egui::Button::new("⏭ Step"));
                if step_btn.clicked() {
                    sim_state.step_requested = true;
                }

                ui.separator();

                let mut hz = 1.0 / time_fixed.timestep().as_secs_f64();
                ui.label("Speed:");
                if ui
                    .add(
                        egui::Slider::new(&mut hz, 0.5..=20.0)
                            .logarithmic(true)
                            .text("ticks/s"),
                    )
                    .changed()
                {
                    time_fixed.set_timestep(Duration::from_secs_f64(1.0 / hz));
                }
            });
            ui.separator();

            // ── Money over time ────────────────────────────────────────────
            ui.heading("Money Over Time");
            Plot::new("money_plot")
                .height(200.0)
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    for (_, record) in history.companies.iter().sorted_by_key(|(_, r)| r.name.clone()) {
                        let points: PlotPoints = record
                            .money_history
                            .iter()
                            .enumerate()
                            .map(|(i, &m)| [i as f64, m])
                            .collect();
                        plot_ui.line(
                            Line::new(&record.name, points)
                                .color(bevy_to_egui(record.color))
                                .width(2.0_f32),
                        );
                    }
                });

            ui.separator();

            // ── Current state ──────────────────────────────────────────────
            ui.heading("Current State");
            let sorted_companies: Vec<_> = query
                .iter()
                .sorted_by_key(|(_, name, _, _, _, _)| name.0.clone())
                .collect();
            let sorted_resources: Vec<(usize, String)> = resources
                .resources
                .iter()
                .sorted_by_key(|&(&id, _)| id)
                .map(|(&id, name)| (id, name.clone()))
                .collect();

            egui::Grid::new("state_grid")
                .striped(true)
                .min_col_width(120.0)
                .show(ui, |ui| {
                    // Header: company names + money balance
                    ui.strong("Metric");
                    for (_, name, _, money, _, color) in &sorted_companies {
                        ui.colored_label(
                            bevy_to_egui(color.0),
                            format!("{} ({:.0}{})", name.0, money.0, currency.unit),
                        );
                    }
                    ui.end_row();

                    // Processor count per company
                    ui.label("Processors");
                    for (_, _, _, _, procs, _) in &sorted_companies {
                        ui.label(procs.processors.len().to_string());
                    }
                    ui.end_row();

                    // Stock per resource
                    for (resource_id, resource_name) in &sorted_resources {
                        ui.label(resource_name);
                        for (_, _, stock, _, _, _) in &sorted_companies {
                            let amt = stock.resources.get(resource_id).copied().unwrap_or(0.0);
                            ui.label(format!("{:.0}", amt));
                        }
                        ui.end_row();
                    }
                });

            ui.separator();

            // ── Recent actions (newest first) ──────────────────────────────
            ui.heading("Recent Actions");
            egui::Grid::new("action_grid")
                .striped(true)
                .min_col_width(120.0)
                .show(ui, |ui| {
                    // Header
                    for (_, record) in history.companies.iter().sorted_by_key(|(_, r)| r.name.clone()) {
                        ui.colored_label(bevy_to_egui(record.color), &record.name);
                    }
                    ui.end_row();

                    let max_len = history
                        .companies
                        .values()
                        .map(|r| r.action_history.len())
                        .max()
                        .unwrap_or(0);

                    // Last 15 entries, newest at top
                    for i in (max_len.saturating_sub(15)..max_len).rev() {
                        for (_, record) in history.companies.iter().sorted_by_key(|(_, r)| r.name.clone()) {
                            let label = record.action_history.get(i).map(|s| s.as_str()).unwrap_or("-");
                            ui.label(label);
                        }
                        ui.end_row();
                    }
                });
        });
}
