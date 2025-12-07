use crate::ui::state::AppState;

/// Renderiza o formulário de criação de perfil
pub fn render_profile_form(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("➕ Criar Novo Perfil");
    ui.add_space(5.0);

    // Nome do jogo
    ui.horizontal(|ui| {
        ui.label("Nome do Jogo:");
        ui.text_edit_singleline(&mut state.form_game_name);
    });

    // Arquivo de save
    ui.horizontal(|ui| {
        ui.label("Arquivo de Save:");
        ui.text_edit_singleline(&mut state.form_save_path);
        if ui.button("📁 Selecionar").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Selecionar arquivo de save")
                .pick_file()
            {
                state.form_save_path = path.display().to_string();
            }
        }
    });

    // Diretório de backup
    ui.horizontal(|ui| {
        ui.label("Diretório de Backup:");
        ui.text_edit_singleline(&mut state.form_backup_dir);
        if ui.button("📁 Selecionar").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Selecionar diretório de backups")
                .pick_folder()
            {
                state.form_backup_dir = path.display().to_string();
            }
        }
    });

    // Timeout
    ui.horizontal(|ui| {
        ui.label("Timeout (minutos):");
        ui.add(egui::DragValue::new(&mut state.form_timeout).speed(0.1));
    });

    ui.add_space(5.0);

    // Botão criar
    if ui.button("✅ Criar Perfil").clicked() {
        state.create_profile();
    }
}

