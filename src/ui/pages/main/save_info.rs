use crate::ui::state::AppState;
use eframe::egui;

pub fn render_save_info(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("Save Atual").heading().strong());
    ui.add_space(4.0);

    if state.active_profile.is_none() {
        ui.label(egui::RichText::new("Nenhum save configurado").weak());
        return;
    }

    let has_file = !state.current_save_file.is_empty();

    if has_file {
        let filename = state
            .current_save_file
            .split(&['/', '\\'][..])
            .next_back()
            .unwrap_or("(desconhecido)");

        ui.horizontal(|ui| {
            ui.label("Arquivo:");
            ui.label(egui::RichText::new(filename).strong().size(13.0))
                .on_hover_text(&state.current_save_file);
        });
    } else {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("(nenhum arquivo encontrado no padrão)")
                    .weak()
                    .size(11.0)
                    .color(egui::Color32::from_rgb(200, 150, 80)),
            );
        });
    }

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
                ui.label("Backup Delay:");
                ui.label(egui::RichText::new(format!("{} min", profile.backup_delay_minutes)).weak());
            });
            ui.horizontal(|ui| {
                ui.label("Backups em:");
                ui.label(egui::RichText::new(&profile.backup_dir).weak())
                    .on_hover_text(&profile.backup_dir);
            });
        }
    }

    if let Some(ref profile) = state.active_profile {
        if profile.is_active {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(2.0);

            if let Some(ref process) = profile.process_name {
                ui.horizontal(|ui| {
                    ui.label("Processo:");
                    ui.label(egui::RichText::new(process).strong().size(12.0))
                        .on_hover_text("Processo monitorado");
                });
            }

            ui.horizontal(|ui| {
                ui.label("Save em:");
                ui.label(egui::RichText::new(&profile.save_path).weak().size(11.0))
                    .on_hover_text("Diretório do save");
            });
        }
    }

    if let Some(ref watcher) = state.active_watcher {
        if let Some(ref profile) = state.active_profile {
            if profile.is_active {
                if let Some(remaining) = watcher.remaining_backup_seconds(profile.backup_delay_minutes) {
                    ui.add_space(4.0);
                    let mins = remaining / 60;
                    let secs = remaining % 60;
                    let label = if remaining == 0 {
                        "Pronto para backup".to_string()
                    } else {
                        format!("Próximo backup em: {:02}:{:02}", mins, secs)
                    };
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(label)
                                .color(if remaining == 0 {
                                    egui::Color32::from_rgb(100, 200, 100)
                                } else {
                                    egui::Color32::from_rgb(200, 150, 80)
                                })
                                .size(12.0),
                        );
                    });
                }
            }
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
