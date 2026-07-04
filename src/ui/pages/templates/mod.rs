//! Aba Templates - Gerenciamento de Templates de Jogos
//!
//! Funcionalidades:
//! - Listagem de templates (oficiais e customizados)
//! - Criar novo template personalizado
//! - Editar templates existentes
//! - Excluir templates customizados

pub mod manager;

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza a aba de templates
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    egui::Panel::left("templates_list_panel")
        .resizable(true)
        .default_size(280.0)
        .min_size(220.0)
        .max_size(400.0)
        .show(ui, |ui| {
            manager::render_templates_list(ui, state);
        });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_space(6.0);
        manager::render_template_form(ui, state);
    });
}
