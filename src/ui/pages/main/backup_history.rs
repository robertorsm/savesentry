//! Histórico de backups - Sidebar da aba Principal

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza o histórico de backups na sidebar
pub fn render_backup_history(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(4.0);
    ui.label(egui::RichText::new("Backups").heading().strong());
    ui.separator();
    ui.add_space(4.0);

    if state.backup_history.is_empty() {
        if state.active_profile.is_some() || !state.config.backup_dir.is_empty() {
            state.reload_backup_history();
        }

        if state.backup_history.is_empty() {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Nenhum backup").weak());
            ui.label(egui::RichText::new("Inicie o monitoramento").weak());
            ui.add_space(8.0);
        }
    }

    if !state.backup_history.is_empty() {
        let mut clicked_restore: Option<String> = None;
        let backup_dir = std::path::Path::new(&state.config.backup_dir);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for backup in &state.backup_history {
                    let is_selected =
                        state.selected_backup_filename.as_ref() == Some(&backup.filename);

                    let frame_color = if is_selected {
                        egui::Color32::from_rgb(40, 80, 120)
                    } else {
                        ui.style().visuals.widgets.inactive.weak_bg_fill
                    };

                    let mut button_rect: Option<egui::Rect> = None;

                    let response = egui::Frame::group(ui.style())
                        .inner_margin(6.0)
                        .fill(frame_color)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            let datetime =
                                chrono::DateTime::<chrono::Local>::from(backup.created_at);
                            ui.label(
                                egui::RichText::new(datetime.format("%d/%m %H:%M").to_string())
                                    .strong()
                                    .size(12.0),
                            );

                            ui.add_space(2.0);

                            let size_mb = backup.size_bytes as f64 / 1024.0 / 1024.0;
                            ui.label(
                                egui::RichText::new(format!("{:.1} MB", size_mb))
                                    .weak()
                                    .size(11.0),
                            );

                            let screenshot_path =
                                backup_dir.join(&backup.filename).with_extension("png");
                            if screenshot_path.exists() {
                                ui.add_space(2.0);
                                ui.label(egui::RichText::new("📷 Screenshot").weak().size(10.0));
                            }

                            ui.add_space(4.0);

                            let restore_button = egui::Button::new("Restaurar")
                                .fill(egui::Color32::from_rgb(60, 100, 140))
                                .min_size(egui::vec2(ui.available_width(), 22.0));

                            let btn = ui
                                .add(restore_button)
                                .on_hover_text("Restaurar este backup");
                            if btn.clicked() {
                                clicked_restore = Some(backup.filename.clone());
                            }
                            button_rect = Some(btn.rect);
                        })
                        .response;

                    let frame_clicked = ui.input(|i| {
                        let pointer = &i.pointer;
                        if let Some(pos) = pointer.interact_pos() {
                            if pointer.primary_clicked() && response.rect.contains(pos) {
                                let on_button = button_rect.is_some_and(|r| r.contains(pos));
                                return !on_button;
                            }
                        }
                        false
                    });

                    if frame_clicked && clicked_restore.is_none() {
                        state.selected_backup_filename = Some(backup.filename.clone());
                    }

                    ui.add_space(3.0);
                }
            });

        if let Some(filename) = clicked_restore {
            state.restore_backup(&filename);
        }
    }

    ui.add_space(4.0);

    if ui
        .button("Atualizar")
        .on_hover_text("Recarregar lista")
        .clicked()
    {
        state.reload_backup_history();
    }
}
