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
            state.invalidate_backup_cache();
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
        let mut delete_backup: Option<String> = None;
        let backup_dir_str = state.get_backup_dir();
        let backup_dir = std::path::Path::new(&backup_dir_str);

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

                    let mut btn_rect: Option<egui::Rect> = None;

                    let response = egui::Frame::group(ui.style())
                        .inner_margin(6.0)
                        .fill(frame_color)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            let label = format_backup_label(&backup.filename);
                            ui.label(
                                egui::RichText::new(label)
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
                            btn_rect = Some(btn.rect);
                        })
                        .response;

                    // Detecta clique no botão via input bruto (evita competição com .interact no frame)
                    let btn_clicked = ui.input(|i| {
                        let pointer = &i.pointer;
                        if let Some(pos) = pointer.interact_pos() {
                            if pointer.primary_clicked() && btn_rect.is_some_and(|r| r.contains(pos)) {
                                return true;
                            }
                        }
                        false
                    });
                    if btn_clicked {
                        clicked_restore = Some(backup.filename.clone());
                    }

                    let frame_response = response.interact(egui::Sense::click());
                    frame_response.context_menu(|ui| {
                        if ui.button("✏ Renomear").clicked() {
                            let current_name = backup
                                .filename
                                .strip_suffix(".zip")
                                .unwrap_or(&backup.filename);
                            state.rename_old_filename = Some(backup.filename.clone());
                            state.rename_new_name = current_name.to_string();
                            state.rename_dialog_open = true;
                            ui.close();
                        }
                        if ui.button("🗑 Excluir").clicked() {
                            delete_backup = Some(backup.filename.clone());
                            ui.close();
                        }
                    });

                    // Detecta clique no frame (excluindo o botão) via input bruto
                    let frame_clicked = ui.input(|i| {
                        let pointer = &i.pointer;
                        if let Some(pos) = pointer.interact_pos() {
                            if pointer.primary_clicked() && response.rect.contains(pos) {
                                let on_button = btn_rect.is_some_and(|r| r.contains(pos));
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

        if let Some(filename) = delete_backup {
            state.delete_backup(&filename);
        }

        if state.rename_dialog_open {
            let mut do_rename = false;
            let mut do_cancel = false;
            egui::Window::new("Renomear Backup")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Novo nome:");
                    let response = ui.text_edit_singleline(&mut state.rename_new_name);
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_rename = true;
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        do_cancel = true;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Salvar").clicked() {
                            do_rename = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            do_cancel = true;
                        }
                    });
                });
            if do_rename {
                if let Some(old_name) = state.rename_old_filename.take() {
                    let new_name = state.rename_new_name.clone();
                    state.rename_backup(&old_name, &new_name);
                }
                state.rename_dialog_open = false;
                state.rename_new_name.clear();
            } else if do_cancel {
                state.rename_dialog_open = false;
                state.rename_new_name.clear();
            }
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

fn format_backup_label(filename: &str) -> String {
    if let Some(stem) = filename.strip_prefix("backup_").and_then(|s| s.strip_suffix(".zip")) {
        if let Some((date_part, time_part)) = stem.split_once('_') {
            let date = date_part.replace('-', "/");
            let time = time_part.replace('-', ":");
            return format!("backup {} {}", date, time);
        }
    }
    filename.strip_suffix(".zip").unwrap_or(filename).to_string()
}
