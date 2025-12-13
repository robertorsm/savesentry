//! Aba Principal - Monitoramento e Backup
//!
//! Componentes:
//! - Seleção de template
//! - Configurações de backup (diretório, timeout)
//! - Controles de monitoramento (iniciar/parar)
//! - Histórico de backups (sidebar)
//! - Informações do save atual

pub mod backup_history;
pub mod config_panel;
pub mod save_info;

pub use backup_history::render_backup_history;
pub use config_panel::render_config_panel;
pub use save_info::render_save_info;

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza a aba principal
pub fn render(ctx: &egui::Context, state: &mut AppState) {
    // Side panel - Lista de backups (histórico)
    egui::SidePanel::left("backup_history_panel")
        .resizable(true)
        .default_width(250.0)
        .width_range(200.0..=400.0)
        .show(ctx, |ui| {
            render_backup_history(ui, state);
        });

    // Central panel - Área principal
    egui::CentralPanel::default().show(ctx, |ui| {
        // Mensagens de erro/sucesso
        crate::ui::components::render_messages(ui, state);

        ui.add_space(8.0);

        // Painel superior: Template selection + configurações
        egui::Frame::group(ui.style())
            .fill(ui.style().visuals.faint_bg_color)
            .inner_margin(12.0)
            .show(ui, |ui| {
                render_config_panel(ui, state);
            });

        ui.add_space(12.0);

        // Painel inferior: Informações do save atual
        egui::Frame::group(ui.style())
            .fill(if state.active_watcher.is_some() {
                egui::Color32::from_rgb(20, 60, 30)
            } else {
                ui.style().visuals.faint_bg_color
            })
            .inner_margin(12.0)
            .show(ui, |ui| {
                render_save_info(ui, state);
            });
    });
}
