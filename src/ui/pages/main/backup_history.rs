//! Histórico de backups - Sidebar da aba Principal

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza o histórico de backups na sidebar
pub fn render_backup_history(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(8.0);
    ui.heading("📦 Backups");
    ui.separator();
    ui.add_space(8.0);

    if state.backup_history.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.label(egui::RichText::new("Nenhum backup ainda").weak());
            ui.label(egui::RichText::new("Configure e inicie o").weak());
            ui.label(egui::RichText::new("monitoramento").weak());
            ui.add_space(32.0);
        });
    } else {
        // Clone backup history para evitar borrow checker
        let backups_snapshot = state.backup_history.clone();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for backup in &backups_snapshot {
                    // Card de backup
                    egui::Frame::group(ui.style())
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            // Timestamp
                            let datetime =
                                chrono::DateTime::<chrono::Local>::from(backup.created_at);
                            ui.label(
                                egui::RichText::new(
                                    datetime.format("%d/%m/%Y %H:%M:%S").to_string(),
                                )
                                .strong()
                                .size(13.0),
                            );

                            ui.add_space(4.0);

                            // Tamanho do arquivo
                            let size_mb = backup.size_bytes as f64 / 1024.0 / 1024.0;
                            ui.label(
                                egui::RichText::new(format!("{:.2} MB", size_mb))
                                    .weak()
                                    .size(11.0),
                            );

                            ui.add_space(8.0);

                            // Botão de restaurar
                            let restore_button = egui::Button::new("↺ Restaurar")
                                .fill(egui::Color32::from_rgb(60, 100, 140))
                                .min_size(egui::vec2(ui.available_width(), 24.0));

                            if ui
                                .add(restore_button)
                                .on_hover_text("Restaurar este backup")
                                .clicked()
                            {
                                state.restore_backup(&backup.filename);
                            }
                        });

                    ui.add_space(4.0);
                }
            });
    }

    ui.add_space(8.0);

    // Botão de recarregar
    if ui
        .button("🔄 Atualizar Lista")
        .on_hover_text("Recarregar lista de backups")
        .clicked()
    {
        state.reload_backup_history();
    }
}
