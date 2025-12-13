//! Gerenciador de Templates - Aba Templates
//!
//! Interface com layout de duas colunas:
//! - Esquerda: Lista de templates disponíveis
//! - Direita: Formulário de edição/criação

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza o gerenciador de templates (aba Templates)
pub fn render_templates_manager(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        // Painel esquerdo - Lista de templates
        egui::SidePanel::left("templates_list_panel")
            .resizable(true)
            .default_width(300.0)
            .width_range(250.0..=400.0)
            .show_inside(ui, |ui| {
                render_templates_list(ui, state);
            });

        // Painel direito - Formulário
        egui::CentralPanel::default().show_inside(ui, |ui| {
            render_template_form(ui, state);
        });
    });
}

/// Renderiza a lista de templates
fn render_templates_list(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(8.0);
    ui.heading("Templates Disponíveis");
    ui.separator();
    ui.add_space(8.0);

    // Botão novo template
    if ui
        .button("➕ Novo Template")
        .on_hover_text("Criar um novo template personalizado")
        .clicked()
    {
        state.clear_template_form();
    }

    ui.add_space(8.0);

    // Clone templates para evitar borrow checker
    let templates_snapshot = state.templates.clone();

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for template in &templates_snapshot {
                let is_selected = state.selected_template_for_edit == Some(template.id);

                egui::Frame::group(ui.style())
                    .fill(if is_selected {
                        egui::Color32::from_rgb(60, 80, 100)
                    } else {
                        ui.style().visuals.faint_bg_color
                    })
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());

                        ui.horizontal(|ui| {
                            // Nome do template
                            ui.label(egui::RichText::new(&template.name).strong().size(14.0));

                            if template.is_official {
                                ui.label(
                                    egui::RichText::new("✓ Oficial")
                                        .weak()
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(100, 200, 100)),
                                );
                            }
                        });

                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new(&template.process_name)
                                .weak()
                                .size(11.0),
                        );

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            // Botão editar
                            if ui
                                .button("✏️ Editar")
                                .on_hover_text("Editar este template")
                                .clicked()
                            {
                                state.select_template_for_edit(template.id);
                            }

                            ui.add_space(4.0);

                            // Botão excluir (apenas para customizados)
                            if !template.is_official
                                && ui
                                    .button("🗑️ Excluir")
                                    .on_hover_text("Excluir este template")
                                    .clicked()
                            {
                                state.delete_template(template.id);
                            }
                        });
                    });

                ui.add_space(4.0);
            }
        });
}

/// Renderiza o formulário de template
fn render_template_form(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(8.0);

    if state.template_form_is_new {
        ui.heading("Criar Novo Template");
    } else {
        ui.heading("Editar Template");
    }

    ui.separator();
    ui.add_space(12.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            egui::Grid::new("template_form_grid")
                .num_columns(2)
                .spacing([12.0, 12.0])
                .show(ui, |ui| {
                    // Nome
                    ui.label("Nome do Jogo:");
                    ui.text_edit_singleline(&mut state.template_form_name);
                    ui.end_row();

                    // Diretório de save
                    ui.label("Diretório de Save:");
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [ui.available_width() - 100.0, 20.0],
                            egui::TextEdit::singleline(&mut state.template_form_save_dir)
                                .hint_text("%APPDATA%\\Jogo\\saves"),
                        );
                        if ui.button("📁").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Selecionar diretório de saves")
                                .pick_folder()
                            {
                                state.template_form_save_dir = path.display().to_string();
                            }
                        }
                    });
                    ui.end_row();

                    // Nome do processo
                    ui.label("Nome do Processo:");
                    ui.text_edit_singleline(&mut state.template_form_process);
                    ui.end_row();

                    // Padrão de arquivos
                    ui.label("Padrão de Arquivos:");
                    ui.text_edit_singleline(&mut state.template_form_pattern);
                    ui.end_row();

                    // Regex de exclusão
                    ui.label("Regex de Exclusão:");
                    ui.text_edit_singleline(&mut state.template_form_exclude);
                    ui.end_row();
                });

            ui.add_space(16.0);

            // Botões de ação
            ui.horizontal(|ui| {
                if state.template_form_is_new {
                    let can_create = !state.template_form_name.trim().is_empty()
                        && !state.template_form_save_dir.trim().is_empty()
                        && !state.template_form_process.trim().is_empty();

                    if ui
                        .add_enabled(
                            can_create,
                            egui::Button::new("💾 Criar Template")
                                .fill(egui::Color32::from_rgb(40, 120, 40))
                                .min_size(egui::vec2(140.0, 32.0)),
                        )
                        .on_hover_text("Salvar novo template")
                        .clicked()
                    {
                        state.create_template();
                    }
                } else {
                    let save_button = egui::Button::new(
                        egui::RichText::new("💾 Salvar Alterações").color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(40, 120, 40))
                    .min_size(egui::vec2(140.0, 32.0));

                    if ui
                        .add(save_button)
                        .on_hover_text("Atualizar template")
                        .clicked()
                    {
                        state.update_template();
                    }
                }

                ui.add_space(8.0);

                let cancel_button =
                    egui::Button::new("❌ Cancelar").min_size(egui::vec2(100.0, 32.0));

                if ui.add(cancel_button).clicked() {
                    state.clear_template_form();
                }
            });

            ui.add_space(12.0);

            // Dicas
            ui.group(|ui| {
                ui.label(egui::RichText::new("💡 Dicas:").strong());
                ui.add_space(4.0);
                ui.label("• Use variáveis de ambiente: %APPDATA%, %USERPROFILE%, %LOCALAPPDATA%");
                ui.label("• Padrão de arquivos: *.sav, *.dat, save*.* etc.");
                ui.label("• Regex exclusão (opcional): .*\\.tmp$, .*\\.bak$");
            });
        });
}
