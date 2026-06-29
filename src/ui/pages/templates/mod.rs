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
pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::SidePanel::left("templates_list_panel")
        .resizable(true)
        .default_width(280.0)
        .width_range(220.0..=400.0)
        .show(ctx, |ui| {
            manager::render_templates_list(ui, state);
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(6.0);
        manager::render_template_form(ui, state);
    });
}
