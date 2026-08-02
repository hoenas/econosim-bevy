use crate::components::common::{Name, RenderColor};
use crate::components::economy::company::CompanyMarker;
use crate::resources::reinforcement_learning::training_history::TrainingHistory;
use bevy::color::Color;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_plot::{Legend, Line, Plot, PlotPoints};

fn bevy_to_egui(color: Color) -> egui::Color32 {
    let c = color.to_srgba();
    egui::Color32::from_rgb(
        (c.red.clamp(0.0, 1.0) * 255.0) as u8,
        (c.green.clamp(0.0, 1.0) * 255.0) as u8,
        (c.blue.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

/// Simple trailing moving average to smooth the (noisy) per-episode return curve.
fn moving_average(data: &[f64], window: usize) -> Vec<[f64; 2]> {
    let mut out = Vec::with_capacity(data.len());
    let mut sum = 0.0;
    for i in 0..data.len() {
        sum += data[i];
        if i >= window {
            sum -= data[i - window];
        }
        let n = (i + 1).min(window) as f64;
        out.push([i as f64, sum / n]);
    }
    out
}

/// Draws the RL learning-progress panel: return-per-episode curve plus live status.
pub fn draw_training_dashboard(
    mut contexts: EguiContexts,
    training: Res<TrainingHistory>,
    query: Query<(Entity, &Name, &RenderColor), With<CompanyMarker>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };

    egui::Window::new("Learning Progress")
        .default_width(560.0)
        .default_height(420.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Exploration ε: {:.3}", training.epsilon));
                ui.separator();
                let episodes = training
                    .companies
                    .values()
                    .map(|c| c.episode_returns.len())
                    .max()
                    .unwrap_or(0);
                ui.label(format!("Episodes completed: {}", episodes));
            });
            ui.separator();

            // ── Return per episode (raw + smoothed) ────────────────────────
            ui.heading("Return per Episode");
            ui.label("Total profit/loss earned per episode. Rising = learning.");
            Plot::new("return_plot")
                .height(240.0)
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    let mut companies: Vec<_> = query.iter().collect();
                    companies.sort_by(|a, b| a.1 .0.cmp(&b.1 .0));
                    for (entity, name, color) in companies {
                        let Some(record) = training.companies.get(&entity) else { continue; };
                        let raw: Vec<f64> = record.episode_returns.iter().copied().collect();
                        if raw.is_empty() {
                            continue;
                        }
                        let col = bevy_to_egui(color.0);
                        // Faint raw series, bold smoothed series on top.
                        let raw_points: PlotPoints = raw
                            .iter()
                            .enumerate()
                            .map(|(i, &r)| [i as f64, r])
                            .collect();
                        plot_ui.line(
                            Line::new(format!("{} (raw)", name.0), raw_points)
                                .color(col.gamma_multiply(0.35))
                                .width(1.0_f32),
                        );
                        let smoothed: PlotPoints = moving_average(&raw, 20).into();
                        plot_ui.line(
                            Line::new(name.0.clone(), smoothed)
                                .color(col)
                                .width(2.5_f32),
                        );
                    }
                });

            ui.separator();

            // ── Current in-progress episode returns ────────────────────────
            ui.heading("Current Episode");
            egui::Grid::new("current_return_grid")
                .striped(true)
                .min_col_width(120.0)
                .show(ui, |ui| {
                    let mut companies: Vec<_> = query.iter().collect();
                    companies.sort_by(|a, b| a.1 .0.cmp(&b.1 .0));
                    for (entity, name, color) in companies {
                        let ret = training
                            .companies
                            .get(&entity)
                            .map(|c| c.current_return)
                            .unwrap_or(0.0);
                        ui.colored_label(bevy_to_egui(color.0), &name.0);
                        ui.label(format!("{:+.0}", ret));
                        ui.end_row();
                    }
                });
        });
}
