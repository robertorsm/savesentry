use crate::ui::state::AppState;
use eframe::egui;

pub fn render_settings_panel(ui: &mut egui::Ui, state: &mut AppState) {
    ui.label(egui::RichText::new("Configurações").heading().strong());
    ui.separator();
    ui.add_space(6.0);

    ui.columns(2, |cols| {
        cols[0].group(|ui| {
            ui.label(egui::RichText::new("Backup").strong().size(15.0));
            ui.add_space(6.0);

            egui::Grid::new("settings_backup_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Diretório:");
                    ui.horizontal(|ui| {
                        let btn_width = 70.0;
                        let available_width = (ui.available_width() - btn_width).max(80.0);
                        ui.add_sized(
                            [available_width, 20.0],
                            egui::TextEdit::singleline(&mut state.config.backup_dir)
                                .hint_text("Onde salvar os backups"),
                        );
                        if ui.button("Buscar").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Selecionar diretório de backups")
                                .pick_folder()
                            {
                                state.set_backup_directory(path.display().to_string());
                            }
                        }
                    });
                    ui.end_row();
                });
        });

        cols[1].group(|ui| {
            ui.label(egui::RichText::new("Interface").strong().size(15.0));
            ui.add_space(6.0);
            ui.label("Tema: Escuro (padrão)");
            ui.label("Atualização: 1 segundo");
        });
    });

    ui.add_space(12.0);

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "SaveSentry v{} · Rust + egui",
                env!("CARGO_PKG_VERSION")
            ))
            .weak()
            .size(11.0),
        );
    });
}
