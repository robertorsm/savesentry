use crate::ui::views;
use crate::ui::Message;
use iced::{
    widget::{column, container, text},
    Element, Length,
};

use super::App;

impl App {
    /// Renderiza a interface principal
    pub fn view(&self) -> Element<'_, Message> {
        let title = text("SaveGameWatcher").size(32);

        // Seção de Templates
        let templates_section =
            views::render_template_selection(&self.templates, &self.selected_template);

        // Formulário de novo perfil
        let form = views::render_profile_form(
            &self.form_game_name,
            &self.form_save_path,
            &self.form_backup_dir,
            &self.form_timeout,
        );

        // Lista de perfis
        let profiles_list = views::render_profiles_list(&self.profiles);

        // Layout principal
        let content = column![
            title,
            text("Gerenciador de Backups Automáticos de Save Games").size(16),
            text("1. Selecione um Template (Opcional)").size(20),
            templates_section,
            text("2. Configurar Perfil").size(20),
            form,
            text("Perfis Criados").size(24),
            profiles_list,
        ]
        .spacing(20)
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

