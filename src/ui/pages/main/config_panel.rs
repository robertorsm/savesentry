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
    ui.vertical(|ui| {
        ui.heading("⚙️ Configuração");
        ui.add_space(8.0);

        egui::Grid::new("config_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                // Seleção de template
                ui.label("Template do Jogo:");

                // Otimização: captura ID selecionado para evitar conflito de borrow
                let selected_name = state
                    .selected_template_id
                    .and_then(|id| state.templates.iter().find(|t| t.id == id))
                    .map(|t| t.name.as_str())
                    .unwrap_or("Selecione um jogo...");

                let mut clicked_template: Option<i64> = None;

                egui::ComboBox::from_id_salt("template_selector")
                    .selected_text(selected_name)
                    .show_ui(ui, |ui| {
                        // Itera diretamente sem clone (ComboBox só renderiza quando aberto)
                        for template in &state.templates {
                            let is_selected = state.selected_template_id == Some(template.id);
                            if ui.selectable_label(is_selected, &template.name).clicked() {
                                clicked_template = Some(template.id);
                            }
                        }
                    });

                // Processa ação APÓS ComboBox para evitar borrow checker issues
                if let Some(template_id) = clicked_template {
                    state.select_template(template_id);
                }

                ui.end_row();

                // Diretório de backup
                ui.label("Diretório de Backup:");
                ui.horizontal(|ui| {
                    let available_width = (ui.available_width() - 100.0).max(50.0);
                    ui.add_sized(
                        [available_width, 20.0],
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
                ui.label("Timeout (minutos):");
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
                    ui.label("min");
                });
                ui.end_row();
            });

        ui.add_space(12.0);

        // Controles de monitoramento
        ui.horizontal(|ui| {
            let can_start = state.selected_template_id.is_some()
                && !state.config_backup_dir.is_empty()
                && state.active_watcher.is_none();

            let can_stop = state.active_watcher.is_some();

            // Botão Iniciar
            let start_button = egui::Button::new("▶ Iniciar Monitoramento")
                .fill(egui::Color32::from_rgb(40, 120, 40))
                .min_size(egui::vec2(180.0, 32.0));

            if ui
                .add_enabled(can_start, start_button)
                .on_hover_text(if can_start {
                    "Iniciar backup automático"
                } else {
                    "Configure template e diretório primeiro"
                })
                .clicked()
            {
                state.start_monitoring();
            }

            ui.add_space(8.0);

            // Botão Parar
            let stop_button = egui::Button::new("⏸ Parar Monitoramento")
                .fill(egui::Color32::from_rgb(120, 40, 40))
                .min_size(egui::vec2(180.0, 32.0));

            if ui
                .add_enabled(can_stop, stop_button)
                .on_hover_text("Parar backup automático")
                .clicked()
            {
                state.stop_monitoring();
            }
        });
    });
}
