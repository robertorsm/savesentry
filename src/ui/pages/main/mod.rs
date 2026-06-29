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
        .default_width(220.0)
        .width_range(180.0..=350.0)
        .show(ctx, |ui| {
            render_backup_history(ui, state);
        });

    // Painel superior: Configuração (altura natural)
    egui::TopBottomPanel::top("config_area")
        .frame(egui::Frame::group(&ctx.style()).fill(ctx.style().visuals.faint_bg_color))
        .show(ctx, |ui| {
            render_config_panel(ui, state);
        });

    // Painel inferior: Save Atual (preenche todo espaço restante)
    egui::CentralPanel::default().show(ctx, |ui| {
        let available = ui.available_height();
        egui::Frame::group(ui.style())
            .fill(if state.active_watcher.is_some() {
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
