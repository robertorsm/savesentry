//! Aba Configurações - Configurações Gerais da Aplicação

pub mod panel;

pub use panel::render_settings_panel;

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza a aba de configurações
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_space(6.0);
        render_settings_panel(ui, state);
    });
}
