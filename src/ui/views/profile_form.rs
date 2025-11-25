use crate::ui::Message;
use iced::widget::{button, column, row, text, text_input, Column};
use std::path::PathBuf;

/// Renderiza o formulário de criação de perfil
pub fn render_profile_form<'a>(
    form_game_name: &'a str,
    form_save_path: &'a Option<PathBuf>,
    form_backup_dir: &'a Option<PathBuf>,
    form_timeout: &'a str,
) -> Column<'a, Message> {
    let name_input = text_input("Nome do jogo...", form_game_name)
        .on_input(Message::GameNameChanged)
        .padding(10);

    let save_file_button = button(text("Selecionar Save")).on_press(Message::SelectSaveFile);

    let save_path_text = if let Some(ref path) = form_save_path {
        text(path.to_string_lossy()).size(14)
    } else {
        text("Nenhum arquivo selecionado").size(14)
    };

    let save_file_row = row![save_file_button, save_path_text].spacing(10);

    let backup_dir_button =
        button(text("Selecionar Diretório de Backup")).on_press(Message::SelectBackupDir);

    let backup_dir_text = if let Some(ref path) = form_backup_dir {
        text(path.to_string_lossy()).size(14)
    } else {
        text("Nenhum diretório selecionado").size(14)
    };

    let backup_dir_row = row![backup_dir_button, backup_dir_text].spacing(10);

    let timeout_input = text_input("Timeout (minutos)...", form_timeout)
        .on_input(Message::TimeoutChanged)
        .padding(10);

    let create_button = button(text("Criar Perfil"))
        .on_press(Message::CreateProfile)
        .padding(10);

    column![
        name_input,
        save_file_row,
        backup_dir_row,
        timeout_input,
        create_button,
    ]
    .spacing(10)
    .padding(20)
}

