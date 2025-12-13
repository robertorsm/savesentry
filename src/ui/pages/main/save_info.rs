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
    ui.vertical(|ui| {
        ui.heading("💾 Savegame Atual");
        ui.add_space(8.0);

        if state.active_profile.is_none() {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("Nenhum savegame configurado").weak());
                ui.label(egui::RichText::new("Selecione um template acima").weak());
            });
            return;
        }

        // Nome do arquivo
        let filename = state
            .current_save_path
            .split(&['/', '\\'][..])
            .next_back()
            .unwrap_or("(desconhecido)");

        ui.horizontal(|ui| {
            ui.label("Arquivo:");
            ui.label(egui::RichText::new(filename).strong().size(14.0))
                .on_hover_text(&state.current_save_path);
        });

        ui.add_space(4.0);

        // Última modificação
        if let Some(modified) = state.current_save_modified {
            let datetime = chrono::DateTime::<chrono::Local>::from(modified);
            ui.horizontal(|ui| {
                ui.label("Modificado em:");
                ui.label(
                    egui::RichText::new(datetime.format("%d/%m/%Y às %H:%M:%S").to_string()).weak(),
                );
            });
        }

        ui.add_space(8.0);

        // Informações adicionais (se monitorando)
        if let Some(ref profile) = state.active_profile {
            if profile.is_active {
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.label(
                        egui::RichText::new("🟢 MONITORANDO")
                            .color(egui::Color32::GREEN)
                            .strong(),
                    );
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Timeout:");
                    ui.label(
                        egui::RichText::new(format!("{} minutos", profile.timeout_minutes)).weak(),
                    );
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Backups em:");
                    ui.label(egui::RichText::new(&profile.backup_dir).weak())
                        .on_hover_text(&profile.backup_dir);
                });
            }
        }

        ui.add_space(8.0);

        // Progresso (se monitorando)
        if state.active_watcher.is_some() {
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new("Aguardando alterações no arquivo...")
                        .italics()
                        .weak(),
                );
            });
        }
    });
}
