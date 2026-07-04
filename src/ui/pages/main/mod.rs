//! Aba Principal - Monitoramento e Backup
//!
//! Componentes:
//! - Seleção de template
//! - Configurações de backup (diretório, backup_delay)
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
pub fn render(ui: &mut egui::Ui, state: &mut AppState) {
    // Clone state values needed for panel configuration BEFORE rendering to avoid borrow issues
    let is_watching = state.active_watcher.is_some();

    // Side panel - Lista de backups (histórico)
    egui::Panel::left("backup_history_panel")
        .resizable(true)
        .default_size(220.0)
        .min_size(180.0)
        .max_size(350.0)
        .show(ui, |ui| {
            render_backup_history(ui, state);
        });

    // Extract style before the show call to avoid potential borrow conflicts
    let config_frame = egui::Frame::group(ui.style()).fill(ui.style().visuals.faint_bg_color);

    // Painel superior: Configuração (altura natural)
    egui::Panel::top("config_area")
        .frame(config_frame)
        .show(ui, |ui| {
            render_config_panel(ui, state);
        });

    // Painel inferior: Save Atual (preenche todo espaço restante)
    egui::CentralPanel::default().show(ui, |ui| {
        let available = ui.available_height();
        egui::Frame::group(ui.style())
            .fill(if is_watching {
                egui::Color32::from_rgb(20, 60, 30)
            } else {
                ui.style().visuals.faint_bg_color
            })
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.set_min_height(available - 16.0);
                render_save_info(ui, state);
            });
    });
}
