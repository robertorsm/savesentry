use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza banner global de mensagens (chamado do app.rs antes do conteúdo)
pub fn render_messages_banner(ui: &mut egui::Ui, state: &mut AppState) {
    if let Some(msg) = state.error_message.clone() {
        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_rgb(80, 20, 20))
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100),
                        format!("ERRO: {}", msg),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("X").clicked() {
                            state.clear_messages();
                        }
                    });
                });
            });
    }

    if let Some(msg) = state.success_message.clone() {
        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_rgb(20, 80, 20))
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 255, 100),
                        format!("OK: {}", msg),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("X").clicked() {
                            state.clear_messages();
                        }
                    });
                });
            });
    }
}
