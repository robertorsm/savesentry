use crate::ui::state::AppState;

/// Renderiza a lista de perfis criados
pub fn render_profiles_list(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("📂 Perfis Criados");
    ui.add_space(5.0);

    // Clona profiles para evitar problemas de borrow
    let profiles_snapshot = state.profiles.lock().ok().map(|p| p.clone());

    if let Some(profiles) = profiles_snapshot {
        if profiles.is_empty() {
            ui.label("Nenhum perfil criado ainda");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for profile in profiles.iter() {
                    ui.group(|ui| {
                        // Cabeçalho do perfil
                        ui.horizontal(|ui| {
                            // Status
                            let status_icon = if profile.is_active { "🟢" } else { "⚫" };
                            let status_text = if profile.is_active {
                                "Monitorando"
                            } else {
                                "Inativo"
                            };
                            ui.label(format!("{} {}", status_icon, status_text));

                            ui.separator();

                            // Nome
                            ui.strong(&profile.name);

                            // Botões à direita
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // Botão excluir
                                    if ui.button("🗑 Excluir").clicked() {
                                        state.delete_profile(profile.id);
                                    }

                                    // Botão iniciar/parar
                                    let toggle_text = if profile.is_active {
                                        "⏸ Parar"
                                    } else {
                                        "▶ Iniciar"
                                    };
                                    if ui.button(toggle_text).clicked() {
                                        state.toggle_monitoring(profile.id);
                                    }
                                },
                            );
                        });

                        ui.separator();

                        // Detalhes do perfil
                        ui.horizontal(|ui| {
                            ui.label(format!("Save: {}", profile.save_path));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Backup: {}", profile.backup_dir));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Timeout: {} min", profile.timeout_minutes));
                        });
                    });
                    ui.add_space(5.0);
                }
            });
        }
    }
}

