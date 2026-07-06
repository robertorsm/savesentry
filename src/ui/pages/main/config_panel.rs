//! Painel de configuração - Aba Principal

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
        });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        let can_start = state.selected_template_id.is_some() && state.active_watcher.is_none();

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
