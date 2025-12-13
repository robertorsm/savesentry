use crate::ui::state::AppState;

/// Renderiza mensagens de erro e sucesso
pub fn render_messages(ui: &mut egui::Ui, state: &mut AppState) {
    if let Some(ref msg) = state.error_message {
        ui.colored_label(egui::Color32::RED, format!("❌ {}", msg));
        if ui.button("Fechar").clicked() {
            state.error_message = None;
        }
        ui.add_space(5.0);
    }

    if let Some(ref msg) = state.success_message {
        ui.colored_label(egui::Color32::GREEN, format!("✅ {}", msg));
        if ui.button("Fechar").clicked() {
            state.success_message = None;
        }
        ui.add_space(5.0);
    }
}
