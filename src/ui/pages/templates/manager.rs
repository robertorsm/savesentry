//! Gerenciador de Templates - Aba Templates
//!
//! Interface com layout de duas colunas:
//! - Esquerda: Lista de templates disponíveis
//! - Direita: Formulário de edição/criação

use crate::ui::state::AppState;
use eframe::egui;

/// Renderiza a lista de templates
pub(super) fn render_templates_list(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(4.0);
    ui.heading("Templates");
    ui.separator();
    ui.add_space(4.0);

    if ui
        .button("Novo")
        .on_hover_text("Criar template personalizado")
        .clicked()
    {
        state.clear_template_form();
    }

    ui.add_space(6.0);

    let template_ids: Vec<i64> = state.templates.iter().map(|t| t.id).collect();
    let mut clicked_edit: Option<i64> = None;
    let mut clicked_delete: Option<i64> = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for &template_id in &template_ids {
                if let Some(template) = state.templates.iter().find(|t| t.id == template_id) {
                    let is_selected = state.template_form.selected_for_edit == Some(template.id);

                    egui::Frame::group(ui.style())
                        .fill(if is_selected {
                            egui::Color32::from_rgb(60, 80, 100)
                        } else {
                            ui.style().visuals.faint_bg_color
                        })
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&template.name).strong().size(13.0));
                                if template.is_official {
                                    ui.label(
                                        egui::RichText::new("(oficial)")
                                            .weak()
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(100, 200, 100)),
                                    );
                                }
                            });

                            ui.label(
                                egui::RichText::new(&template.process_name)
                                    .weak()
                                    .size(11.0),
                            );

                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                if ui.button("Editar").on_hover_text("Editar").clicked() {
                                    clicked_edit = Some(template.id);
                                }
                                ui.add_space(4.0);
                                if !template.is_official
                                    && ui.button("Excluir").on_hover_text("Excluir").clicked()
                                {
                                    clicked_delete = Some(template.id);
                                }
                            });
                        });

                    ui.add_space(3.0);
                }
            }
        });

    if let Some(template_id) = clicked_edit {
        state.select_template_for_edit(template_id);
    }
    if let Some(template_id) = clicked_delete {
        state.delete_template(template_id);
    }
}

/// Renderiza o formulário de template
pub(super) fn render_template_form(ui: &mut egui::Ui, state: &mut AppState) {
    ui.add_space(4.0);

    if state.template_form.is_new {
        ui.heading("Novo Template");
    } else {
        ui.heading("Editar Template");
    }

    ui.separator();
    ui.add_space(8.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            egui::Grid::new("template_form_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Nome:");
                    ui.text_edit_singleline(&mut state.template_form.name);
                    ui.end_row();

                    ui.label("Save em:");
                    ui.horizontal(|ui| {
                        let btn_width = 70.0;
                        let available_width = (ui.available_width() - btn_width).max(80.0);
                        ui.add_sized(
                            [available_width, 20.0],
                            egui::TextEdit::singleline(&mut state.template_form.save_dir)
                                .hint_text("%APPDATA%\\Jogo\\saves"),
                        );
                        if ui.button("Buscar").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Selecionar diretório de saves")
                                .pick_folder()
                            {
                                state.template_form.save_dir = path.display().to_string();
                            }
                        }
                    });
                    ui.end_row();

                    ui.label("Processo:");
                    ui.text_edit_singleline(&mut state.template_form.process);
                    ui.end_row();

                    ui.label("Padrão:");
                    ui.text_edit_singleline(&mut state.template_form.pattern);
                    ui.end_row();

                    ui.label("Exclusão:");
                    ui.text_edit_singleline(&mut state.template_form.exclude);
                    ui.end_row();
                });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if state.template_form.is_new {
                    let can_create = !state.template_form.name.trim().is_empty()
                        && !state.template_form.save_dir.trim().is_empty()
                        && !state.template_form.process.trim().is_empty();

                    if ui
                        .add_enabled(
                            can_create,
                            egui::Button::new(
                                egui::RichText::new("Criar").color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(40, 120, 40))
                            .min_size(egui::vec2(120.0, 30.0)),
                        )
                        .on_hover_text("Salvar novo template")
                        .clicked()
                    {
                        state.create_template();
                    }
                } else {
                    let save_button = egui::Button::new(
                        egui::RichText::new("Salvar").color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(40, 120, 40))
                    .min_size(egui::vec2(120.0, 30.0));

                    if ui
                        .add(save_button)
                        .on_hover_text("Atualizar template")
                        .clicked()
                    {
                        state.update_template();
                    }
                }

                ui.add_space(6.0);

                if ui
                    .add(egui::Button::new("Cancelar").min_size(egui::vec2(90.0, 30.0)))
                    .clicked()
                {
                    state.clear_template_form();
                }
            });

            ui.add_space(12.0);

            ui.group(|ui| {
                ui.label(egui::RichText::new("Dicas").strong());
                ui.add_space(2.0);
                ui.label("• Variáveis: %APPDATA%, %USERPROFILE%, %LOCALAPPDATA%");
                ui.label("• Padrões: *.sav, *.dat, save*.*");
                ui.label("• Regex exclusão: .*\\.tmp$, .*\\.bak$");
            });
        });
}
