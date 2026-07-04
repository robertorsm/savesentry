use crate::ui::state::{ActiveTab, AppState};
use eframe::egui;

/// Renderiza a barra de abas (integrada no header)
pub fn render_tab_bar(ui: &mut egui::Ui, state: &mut AppState) {
    let tabs = [
        ("Principal", ActiveTab::Main),
        ("Templates", ActiveTab::Templates),
        ("Config", ActiveTab::Settings),
    ];

    for (label, tab) in tabs {
        let is_active = state.active_tab == tab;
        let button = egui::Button::new(egui::RichText::new(label).size(13.0))
            .fill(if is_active {
                egui::Color32::from_rgb(60, 100, 140)
            } else {
                egui::Color32::TRANSPARENT
            })
            .stroke(if is_active {
                egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 130, 180))
            } else {
                egui::Stroke::NONE
            })
            .min_size(egui::vec2(100.0, 28.0));

        if ui.add(button).clicked() {
            state.active_tab = tab;
        }

        ui.add_space(2.0);
    }
}
