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

                        })
                        .response;

                    let frame_response = response.interact(egui::Sense::click());
                    frame_response.context_menu(|ui| {
                        if ui.button("↩ Restaurar").clicked() {
                            state.restore_target_filename = Some(backup.filename.clone());
                            state.restore_dialog_open = true;
                            state.restore_dialog_focus_cancel = false;
                            ui.close();
                        }
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

                    let frame_clicked = ui.input(|i| {
                        let pointer = &i.pointer;
                        if let Some(pos) = pointer.interact_pos() {
                            if pointer.primary_clicked() && response.rect.contains(pos) {
                                return true;
                            }
                        }
                        false
                    });
                    if frame_clicked {
                        state.selected_backup_filename = Some(backup.filename.clone());
                    }

                    let frame_double_clicked = ui.input(|i| {
                        let pointer = &i.pointer;
                        if let Some(pos) = pointer.interact_pos() {
                            if pointer.button_double_clicked(egui::PointerButton::Primary) && response.rect.contains(pos) {
                                return true;
                            }
                        }
                        false
                    });
                    if frame_double_clicked {
                        state.restore_target_filename = Some(backup.filename.clone());
                        state.restore_dialog_open = true;
                        state.restore_dialog_focus_cancel = false;
                    }

                    ui.add_space(3.0);
                }
            });

        if let Some(filename) = delete_backup {
            state.delete_backup(&filename);
        }

        if state.restore_dialog_open {
            let mut do_restore = false;
            let mut do_cancel = false;

            let screen_rect = ui.ctx().input(|i| i.raw.screen_rect).unwrap_or_else(|| ui.max_rect());
            egui::Area::new(egui::Id::new("restore_modal_overlay"))
                .fixed_pos(screen_rect.min)
                .show(ui.ctx(), |ui| {
                    ui.set_min_size(screen_rect.size());
                    let overlay_response = ui.allocate_rect(screen_rect, egui::Sense::click());
                    ui.painter().rect_filled(
                        screen_rect,
                        0.0,
                        egui::Color32::from_rgba_premultiplied(0, 0, 0, 160),
                    );
                    overlay_response
                });

            egui::Window::new("Confirmar Restauração")
                .collapsible(false)
                .resizable(false)
                .movable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ui.ctx(), |ui| {
                    ui.set_min_width(280.0);
                    ui.label("Deseja restaurar este backup?");
                    if let Some(ref filename) = state.restore_target_filename {
                        let label = format_backup_label(filename);
                        ui.label(egui::RichText::new(label).strong().size(13.0));
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let cancel_btn = ui.button("Cancelar");
                            let ok_btn = ui.button("Ok");

                            if cancel_btn.clicked() {
                                do_cancel = true;
                            }
                            if ok_btn.clicked() {
                                do_restore = true;
                            }

                            if state.restore_dialog_focus_cancel {
                                if cancel_btn.has_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    do_cancel = true;
                                }
                                if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                                    state.restore_dialog_focus_cancel = false;
                                }
                            } else {
                                if (ok_btn.has_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    do_restore = true;
                                }
                                if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                                    state.restore_dialog_focus_cancel = true;
                                }
                            }
                        });
                    });
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        do_cancel = true;
                    }
                });

            if do_restore {
                if let Some(filename) = state.restore_target_filename.take() {
                    state.restore_backup(&filename);
                }
                state.restore_dialog_open = false;
                state.restore_dialog_focus_cancel = false;
            } else if do_cancel {
                state.restore_dialog_open = false;
                state.restore_target_filename = None;
                state.restore_dialog_focus_cancel = false;
            }
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
