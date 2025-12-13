use crate::ui::state::{ActiveTab, AppState};
use eframe::egui;

/// Renderiza a barra de abas
pub fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        // Aba Principal
        let main_button = if state.active_tab == ActiveTab::Main {
            egui::Button::new("🏠 Principal")
                .fill(egui::Color32::from_rgb(60, 100, 140))
                .min_size(egui::vec2(120.0, 32.0))
        } else {
            egui::Button::new("🏠 Principal")
                .fill(egui::Color32::from_rgb(40, 40, 40))
                .min_size(egui::vec2(120.0, 32.0))
        };

        if ui.add(main_button).clicked() {
            state.active_tab = ActiveTab::Main;
        }

        ui.add_space(4.0);

        // Aba Templates
        let templates_button = if state.active_tab == ActiveTab::Templates {
            egui::Button::new("📋 Templates")
                .fill(egui::Color32::from_rgb(60, 100, 140))
                .min_size(egui::vec2(120.0, 32.0))
        } else {
            egui::Button::new("📋 Templates")
                .fill(egui::Color32::from_rgb(40, 40, 40))
                .min_size(egui::vec2(120.0, 32.0))
        };

        if ui.add(templates_button).clicked() {
            state.active_tab = ActiveTab::Templates;
        }

        ui.add_space(4.0);

        // Aba Configurações
        let settings_button = if state.active_tab == ActiveTab::Settings {
            egui::Button::new("⚙️ Configurações")
                .fill(egui::Color32::from_rgb(60, 100, 140))
                .min_size(egui::vec2(120.0, 32.0))
        } else {
            egui::Button::new("⚙️ Configurações")
                .fill(egui::Color32::from_rgb(40, 40, 40))
                .min_size(egui::vec2(120.0, 32.0))
        };

        if ui.add(settings_button).clicked() {
            state.active_tab = ActiveTab::Settings;
        }

        ui.add_space(8.0);
    });
}
