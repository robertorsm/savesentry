//! Aba Templates - Gerenciamento de Templates de Jogos
//!
//! Funcionalidades:
//! - Listagem de templates (oficiais e customizados)
//! - Criar novo template personalizado
//! - Editar templates existentes
//! - Excluir templates customizados

pub mod manager;

pub use manager::render_templates_manager;

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza a aba de templates
pub fn render(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        crate::ui::components::render_messages(ui, state);
        ui.add_space(8.0);
        render_templates_manager(ui, state);
    });
}
