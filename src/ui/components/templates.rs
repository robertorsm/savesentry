use crate::ui::state::AppState;

/// Renderiza a seção de templates de jogos
pub fn render_templates(ui: &mut egui::Ui, state: &mut AppState) {
    ui.collapsing("📋 Templates de Jogos", |ui| {
        if state.templates.is_empty() {
            ui.label("Nenhum template disponível");
        } else {
            // Clona lista de templates para evitar borrow checker
            let templates_snapshot: Vec<_> = state
                .templates
                .iter()
                .enumerate()
                .map(|(idx, t)| (idx, t.name.clone()))
                .collect();

            let selected = state.selected_template;

            ui.horizontal_wrapped(|ui| {
                for (idx, name) in templates_snapshot.iter() {
                    let is_selected = selected == Some(*idx);
                    let button = if is_selected {
                        egui::Button::new(format!("✓ {}", name))
                            .fill(egui::Color32::from_rgb(0, 100, 200))
                    } else {
                        egui::Button::new(name)
                    };

                    if ui.add(button).clicked() {
                        state.select_template(*idx);
                    }
                }
            });

            if state.selected_template.is_some() {
                if ui.button("🗑 Limpar Seleção").clicked() {
                    state.clear_form();
                }
            }
        }
    });
}

