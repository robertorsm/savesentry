use crate::ui::state::{ActiveTab, AppState};
use eframe::egui;

/// Renderiza a barra de abas
pub fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        let tabs = [
            ("🏠 Principal", ActiveTab::Main),
            ("📋 Templates", ActiveTab::Templates),
            ("⚙️ Configurações", ActiveTab::Settings),
        ];

        for (label, tab) in tabs {
            let is_active = state.active_tab == tab;
            let button = egui::Button::new(label)
                .fill(if is_active {
                    egui::Color32::from_rgb(60, 100, 140)
                } else {
                    egui::Color32::from_rgb(40, 40, 40)
                })
                .min_size(egui::vec2(120.0, 32.0));

            if ui.add(button).clicked() {
                state.active_tab = tab;
            }

            ui.add_space(4.0);
        }

        ui.add_space(4.0);
    });
}
