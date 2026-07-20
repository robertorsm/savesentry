use crate::ui::state::AppState;
use eframe::egui;

pub fn render_save_info(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("Save Atual").heading().strong());
    ui.add_space(4.0);

    if state.active_profile.is_none() {
        ui.label(egui::RichText::new("Nenhum save configurado").weak());
        return;
    }

    let profile_data = state.active_profile.as_ref().map(|p| {
        (p.is_active, p.backup_delay_minutes, p.backup_max_count)
    });

    let has_file = !state.current_save_file.is_empty();

    ui.horizontal(|ui| {
        if has_file {
            let filename = state
                .current_save_file
                .split(&['/', '\\'][..])
                .next_back()
                .unwrap_or("(desconhecido)");
            ui.label("Arquivo:");
            ui.label(egui::RichText::new(filename).strong().size(13.0))
                .on_hover_text(&state.current_save_file);
        } else {
            ui.label(
                egui::RichText::new("(nenhum arquivo encontrado no padrão)")
                    .weak()
                    .size(11.0)
                    .color(egui::Color32::from_rgb(200, 150, 80)),
            );
        }

        if let Some(modified) = state.current_save_modified {
            ui.add_space(16.0);
            let datetime = chrono::DateTime::<chrono::Local>::from(modified);
            ui.label("Modificado:");
            ui.label(egui::RichText::new(datetime.format("%d/%m/%Y %H:%M").to_string()).weak());
        }
    });

    if let Some((is_active, delay_minutes, max_count)) = profile_data {
        if is_active {
            let remaining = state
                .active_watcher
                .as_ref()
                .and_then(|w| w.remaining_backup_seconds(delay_minutes));

            let backup_count = state.backup_history.len();
            let max_count = max_count as usize;
            let count_color = if backup_count >= max_count {
                egui::Color32::from_rgb(220, 80, 80)
            } else {
                egui::Color32::from_rgb(180, 180, 180)
            };

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if let Some(remaining) = remaining {
                    let mins = remaining / 60;
                    let secs = remaining % 60;
                    let label = if remaining == 0 {
                        "Pronto".to_string()
                    } else {
                        format!("{:02}:{:02}", mins, secs)
                    };
                    ui.label("Próximo backup:");
                    ui.label(
                        egui::RichText::new(label)
                            .strong()
                            .color(if remaining == 0 {
                                egui::Color32::from_rgb(100, 220, 100)
                            } else {
                                egui::Color32::from_rgb(220, 180, 80)
                            })
                            .size(14.0),
                    );
                }

                ui.add_space(16.0);
                ui.label("Backups:");
                ui.label(
                    egui::RichText::new(format!("{} / {}", backup_count, max_count))
                        .strong()
                        .color(count_color)
                        .size(14.0),
                );

                ui.add_space(16.0);
                ui.label("Intervalo:");
                ui.label(egui::RichText::new(format!("{} min", delay_minutes)).weak());
            });

            if state.active_watcher.is_some() {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Aguardando alterações...")
                            .italics()
                            .weak(),
                    );
                });
            }
        }
    }

    let target_backup = state
        .selected_backup_filename
        .clone()
        .or_else(|| state.backup_history.first().map(|b| b.filename.clone()));

    if let Some(filename) = target_backup {
        let backup_dir = state.get_backup_dir();
        let screenshot_path = std::path::Path::new(&backup_dir)
            .join(&filename)
            .with_extension("png");
        if screenshot_path.exists() {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            let label = if state.selected_backup_filename.is_some() {
                "Screenshot do backup selecionado"
            } else {
                "Screenshot do backup mais recente"
            };
            ui.label(egui::RichText::new(label).weak().size(11.0));
            ui.add_space(4.0);

            let max_width = ui.available_width();
            let max_height = ui.available_height().min(280.0);

            if let Some(texture) = state.load_screenshot_texture(ui.ctx(), &filename) {
                let [tex_w, tex_h] = texture.size();
                let aspect = tex_w as f32 / tex_h as f32;
                let width = max_width.min(max_height * aspect);
                let height = width / aspect;

                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new(&texture).fit_to_exact_size(egui::vec2(width, height)));
                });
            }
        }
    }
}
