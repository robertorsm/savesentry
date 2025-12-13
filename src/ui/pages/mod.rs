//! Páginas - Conteúdo das 3 abas principais
//!
//! - main: Aba Principal (monitoramento e backups)
//! - templates: Aba Templates (gerenciamento de templates)
//! - settings: Aba Configurações (configurações gerais)

pub mod main;
pub mod settings;
pub mod templates;

use crate::ui::state::{ActiveTab, AppState};
use eframe::egui;

/// Renderiza a página ativa baseada na aba selecionada
pub fn render_active_page(ctx: &egui::Context, state: &mut AppState) {
    match state.active_tab {
        ActiveTab::Main => main::render(ctx, state),
        ActiveTab::Templates => templates::render(ctx, state),
        ActiveTab::Settings => settings::render(ctx, state),
    }
}
