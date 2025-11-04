use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

pub fn graph_system(mut contexts: EguiContexts) -> Result {
    egui::Window::new("Graph").show(contexts.ctx_mut()?, |ui| ui.label("Graph"));
    Ok(())
}
