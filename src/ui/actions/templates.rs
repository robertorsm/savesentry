//! Ações CRUD para gerenciamento de templates

use crate::ui::state::AppState;

impl AppState {
    /// Seleciona um template para edição
    pub fn select_template_for_edit(&mut self, template_id: i64) {
        if let Some(template) = self.templates.iter().find(|t| t.id == template_id) {
            self.template_form.selected_for_edit = Some(template_id);
            self.template_form.name = template.name.clone();
            self.template_form.save_dir = template.save_directory.clone();
            self.template_form.backup_dir = template.backup_dir.clone();
            self.template_form.backup_delay_minutes = template.backup_delay_minutes;
            self.template_form.process = template.process_name.clone();
            self.template_form.pattern = template.save_pattern.clone();
            self.template_form.exclude = template.exclude_regex.clone().unwrap_or_default();
            self.template_form.is_new = false;
        }
    }

    /// Cria um novo template
    pub fn create_template(&mut self) {
        // Validação
        if self.template_form.name.trim().is_empty() {
            self.error_message = Some("Nome do template é obrigatório".to_string());
            return;
        }
        if self.template_form.save_dir.trim().is_empty() {
            self.error_message = Some("Diretório de save é obrigatório".to_string());
            return;
        }
        if self.template_form.backup_dir.trim().is_empty() {
            self.error_message = Some("Diretório de backup é obrigatório".to_string());
            return;
        }
        if self.template_form.process.trim().is_empty() {
            self.error_message = Some("Nome do processo é obrigatório".to_string());
            return;
        }

        let exclude_regex = if self.template_form.exclude.is_empty() {
            None
        } else {
            Some(self.template_form.exclude.clone())
        };

        match self.db.insert_game_template(
            &self.template_form.name,
            &self.template_form.save_dir,
            &self.template_form.process,
            &self.template_form.pattern,
            exclude_regex.as_deref(),
            &self.template_form.backup_dir,
            self.template_form.backup_delay_minutes,
        ) {
            Ok(_) => {
                self.success_message = Some(format!(
                    "Template '{}' criado com sucesso",
                    self.template_form.name
                ));
                self.reload_templates();
                self.clear_template_form();
            }
            Err(e) => {
                self.error_message = Some(format!("Erro ao criar template: {}", e));
            }
        }
    }

    /// Atualiza um template existente
    pub fn update_template(&mut self) {
        if let Some(template_id) = self.template_form.selected_for_edit {
            // Validação
            if self.template_form.name.trim().is_empty() {
                self.error_message = Some("Nome do template é obrigatório".to_string());
                return;
            }

            let exclude_regex = if self.template_form.exclude.is_empty() {
                None
            } else {
                Some(self.template_form.exclude.clone())
            };

            // Atualiza no banco
            match self.db.update_game_template(
                template_id,
                &self.template_form.name,
                &self.template_form.save_dir,
                &self.template_form.process,
                &self.template_form.pattern,
                exclude_regex.as_deref(),
                &self.template_form.backup_dir,
                self.template_form.backup_delay_minutes,
            ) {
                Ok(_) => {
                    self.success_message =
                        Some(format!("Template '{}' atualizado", self.template_form.name));
                    self.reload_templates();

                    if self.selected_template_id == Some(template_id) {
                        self.select_template(template_id);
                    }

                    self.clear_template_form();
                }
                Err(e) => {
                    self.error_message = Some(format!("Erro ao atualizar template: {}", e));
                }
            }
        }
    }

    /// Deleta um template
    pub fn delete_template(&mut self, template_id: i64) {
        // Verifica se não é template oficial
        if let Some(template) = self.templates.iter().find(|t| t.id == template_id) {
            if template.is_official {
                self.error_message = Some("Templates oficiais não podem ser excluídos".to_string());
                return;
            }

            match self.db.delete_game_template(template_id) {
                Ok(_) => {
                    self.success_message = Some("Template excluído com sucesso".to_string());
                    self.reload_templates();
                    if self.template_form.selected_for_edit == Some(template_id) {
                        self.clear_template_form();
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Erro ao excluir template: {}", e));
                }
            }
        }
    }
}
