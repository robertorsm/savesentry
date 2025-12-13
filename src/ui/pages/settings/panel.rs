//! Painel de Configurações - Aba Configurações
//!
//! Seções:
//! - Configurações de backup (diretório, timeout)
//! - Interface (tema, intervalo)
//! - Informações (versão, descrição)

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza o painel de configurações gerais (aba Settings)
pub fn render_settings_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(16.0);
    ui.heading("⚙️ Configurações Gerais");
    ui.separator();
    ui.add_space(16.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // Seção: Backup
            ui.group(|ui| {
                ui.label(
                    egui::RichText::new("📦 Configurações de Backup")
                        .strong()
                        .size(16.0),
                );
                ui.add_space(8.0);

                egui::Grid::new("settings_backup_grid")
                    .num_columns(2)
                    .spacing([12.0, 12.0])
                    .show(ui, |ui| {
                        // Diretório de backup
                        ui.label("Diretório de Backup:");
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [ui.available_width() - 100.0, 20.0],
                                egui::TextEdit::singleline(&mut state.config_backup_dir)
                                    .hint_text("Onde salvar os backups"),
                            );
                            if ui.button("📁 Buscar").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Selecionar diretório de backups")
                                    .pick_folder()
                                {
                                    state.set_backup_directory(path.display().to_string());
                                }
                            }
                        });
                        ui.end_row();

                        // Timeout
                        ui.label("Intervalo de Backup:");
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::DragValue::new(&mut state.config_timeout)
                                        .speed(0.5)
                                        .range(1..=1440),
                                )
                                .changed()
                            {
                                state.set_timeout(state.config_timeout);
                            }
                            ui.label("minutos");
                        });
                        ui.end_row();
                    });
            });

            ui.add_space(16.0);

            // Seção: Interface
            ui.group(|ui| {
                ui.label(egui::RichText::new("🎨 Interface").strong().size(16.0));
                ui.add_space(8.0);

                ui.label("Tema: Escuro (padrão)");
                ui.label("Intervalo de atualização: 1 segundo");
            });

            ui.add_space(16.0);

            // Seção: Informações
            ui.group(|ui| {
                ui.label(egui::RichText::new("ℹ️ Informações").strong().size(16.0));
                ui.add_space(8.0);

                ui.label(format!("Versão: {}", env!("CARGO_PKG_VERSION")));
                ui.label("SaveGameWatcher - Monitor e Backup Automático de Saves");
                ui.label("Desenvolvido em Rust + egui");
            });
        });
}
