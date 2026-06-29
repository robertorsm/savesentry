//! Informações do savegame atual - Aba Principal
//!
//! Exibe:
//! - Nome e caminho do arquivo
//! - Data da última modificação
//! - Status de monitoramento
//! - Timeout configurado
//! - Diretório de backups

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza informações do save atual (painel inferior)
pub fn render_save_info(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("Save Atual").heading().strong());
    ui.add_space(4.0);

    if state.active_profile.is_none() {
        ui.label(egui::RichText::new("Nenhum save configurado").weak());
        return;
    }

    let filename = state
        .current_save_path
        .split(&['/', '\\'][..])
        .next_back()
        .unwrap_or("(desconhecido)");

    ui.horizontal(|ui| {
        ui.label("Arquivo:");
        ui.label(egui::RichText::new(filename).strong().size(13.0))
            .on_hover_text(&state.current_save_path);
    });

    ui.add_space(2.0);

    if let Some(modified) = state.current_save_modified {
        let datetime = chrono::DateTime::<chrono::Local>::from(modified);
        ui.horizontal(|ui| {
            ui.label("Modificado:");
            ui.label(egui::RichText::new(datetime.format("%d/%m/%Y %H:%M").to_string()).weak());
        });
    }

    if let Some(ref profile) = state.active_profile {
        if profile.is_active {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Timeout:");
                ui.label(egui::RichText::new(format!("{} min", profile.timeout_minutes)).weak());
            });
            ui.horizontal(|ui| {
                ui.label("Backups em:");
                ui.label(egui::RichText::new(&profile.backup_dir).weak())
                    .on_hover_text(&profile.backup_dir);
            });
        }
    }

    if state.active_watcher.is_some() {
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(
                egui::RichText::new("Aguardando alterações...")
                    .italics()
                    .weak(),
            );
        });
    }
}
