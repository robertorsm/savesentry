//! Painel de configuração - Aba Principal
//!
//! Permite:
//! - Seleção de template
//! - Configuração de diretório de backup
//! - Ajuste de timeout
//! - Controles de monitoramento

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza o painel de configuração (template + settings)
pub fn render_config_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("Configuração").heading().strong());
    ui.add_space(4.0);

    egui::Grid::new("config_grid")
        .num_columns(2)
        .spacing([10.0, 6.0])
        .show(ui, |ui| {
            ui.label("Jogo:");

            let selected_name = state
                .selected_template_id
                .and_then(|id| state.templates.iter().find(|t| t.id == id))
                .map(|t| t.name.as_str())
                .unwrap_or("Selecione...");

            let mut clicked_template: Option<i64> = None;

            egui::ComboBox::from_id_salt("template_selector")
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    for template in &state.templates {
                        let is_selected = state.selected_template_id == Some(template.id);
                        if ui.selectable_label(is_selected, &template.name).clicked() {
                            clicked_template = Some(template.id);
                        }
                    }
                });

            if let Some(template_id) = clicked_template {
                state.select_template(template_id);
            }

            ui.end_row();

            ui.label("Backup em:");
            ui.horizontal(|ui| {
                let btn_width = 70.0;
                let available_width = (ui.available_width() - btn_width).max(80.0);
                ui.add_sized(
                    [available_width, 20.0],
                    egui::TextEdit::singleline(&mut state.config.backup_dir).hint_text("Diretório"),
                );
                if ui.button("Buscar").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Selecionar diretório")
                        .pick_folder()
                    {
                        state.set_backup_directory(path.display().to_string());
                    }
                }
            });
            ui.end_row();

            ui.label("Timeout:");
            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [70.0, 20.0],
                        egui::DragValue::new(&mut state.config.timeout_minutes)
                            .speed(0.5)
                            .range(1..=1440),
                    )
                    .changed()
                {
                    state.set_timeout(state.config.timeout_minutes);
                }
                ui.label("min");
            });
            ui.end_row();
        });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        let can_start = state.selected_template_id.is_some()
            && !state.config.backup_dir.is_empty()
            && state.active_watcher.is_none();

        let can_stop = state.active_watcher.is_some();

        let start_button =
            egui::Button::new(egui::RichText::new("Iniciar").color(egui::Color32::WHITE))
                .fill(egui::Color32::from_rgb(40, 120, 40))
                .min_size(egui::vec2(160.0, 30.0));

        if ui
            .add_enabled(can_start, start_button)
            .on_hover_text(if can_start {
                "Iniciar backup automático"
            } else {
                "Configure jogo e diretório primeiro"
            })
            .clicked()
        {
            state.start_monitoring();
        }

        ui.add_space(6.0);

        let stop_button =
            egui::Button::new(egui::RichText::new("Parar").color(egui::Color32::WHITE))
                .fill(egui::Color32::from_rgb(140, 40, 40))
                .min_size(egui::vec2(160.0, 30.0));

        if ui
            .add_enabled(can_stop, stop_button)
            .on_hover_text("Parar backup automático")
            .clicked()
        {
            state.stop_monitoring();
        }
    });
}
