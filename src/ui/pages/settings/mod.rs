//! Aba Configurações - Configurações Gerais da Aplicação

pub mod panel;

pub use panel::render_settings_panel;

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza a aba de configurações
pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        crate::ui::components::render_messages(ui, state);
        ui.add_space(8.0);
        render_settings_panel(ui, state);
    });
}
