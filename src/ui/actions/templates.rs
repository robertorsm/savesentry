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
            self.template_form.backup_max_count = template.backup_max_count;
            self.template_form.process = template.process_name.clone();
            self.template_form.pattern = template.save_pattern.clone();
            self.template_form.exclude = template.exclude_pattern.clone().unwrap_or_default();
            self.template_form.is_new = false;
            self.template_form.original_save_dir = template.save_directory.clone();
            self.template_form.original_backup_dir = template.backup_dir.clone();
            self.template_form.original_backup_delay_minutes = template.backup_delay_minutes;
            self.template_form.original_backup_max_count = template.backup_max_count;
            self.template_form.original_process = template.process_name.clone();
            self.template_form.original_pattern = template.save_pattern.clone();
            self.template_form.original_exclude =
                template.exclude_pattern.clone().unwrap_or_default();
        }
    }

    /// Cria um novo template
    pub fn create_template(&mut self) {
        // Validação
        if self.template_form.name.trim().is_empty() {
            self.set_error_message("Nome do template é obrigatório".to_string());
            return;
        }
        if self.template_form.save_dir.trim().is_empty() {
            self.set_error_message("Diretório de save é obrigatório".to_string());
            return;
        }
        if self.template_form.backup_dir.trim().is_empty() {
            self.set_error_message("Diretório de backup é obrigatório".to_string());
            return;
        }
        if self.template_form.process.trim().is_empty() {
            self.set_error_message("Nome do processo é obrigatório".to_string());
            return;
        }

        let exclude_pattern = if self.template_form.exclude.is_empty() {
            None
        } else {
            Some(self.template_form.exclude.clone())
        };

        match self.db.insert_game_template(
            &self.template_form.name,
            &self.template_form.save_dir,
            &self.template_form.process,
            &self.template_form.pattern,
            exclude_pattern.as_deref(),
            None, // default_exclude_pattern não é configurável pelo usuário
            &self.template_form.backup_dir,
            self.template_form.backup_delay_minutes,
            self.template_form.backup_max_count,
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
                self.set_error_message(format!("Erro ao criar template: {}", e));
            }
        }
    }

    /// Atualiza um template existente
    pub fn update_template(&mut self) {
        if let Some(template_id) = self.template_form.selected_for_edit {
            if self.template_form.name.trim().is_empty() {
                self.set_error_message("Nome do template é obrigatório".to_string());
                return;
            }

            let exclude_pattern = if self.template_form.exclude.is_empty() {
                None
            } else {
                Some(self.template_form.exclude.clone())
            };

            let needs_watcher_restart = self.template_form.save_dir
                != self.template_form.original_save_dir
                || self.template_form.backup_dir != self.template_form.original_backup_dir
                || self.template_form.process != self.template_form.original_process
                || self.template_form.pattern != self.template_form.original_pattern
                || self.template_form.exclude != self.template_form.original_exclude
                || self.template_form.backup_delay_minutes
                    != self.template_form.original_backup_delay_minutes
                || self.template_form.backup_max_count
                    != self.template_form.original_backup_max_count;

            match self.db.update_game_template(
                template_id,
                &self.template_form.name,
                &self.template_form.save_dir,
                &self.template_form.process,
                &self.template_form.pattern,
                exclude_pattern.as_deref(),
                None, // default_exclude_pattern não é configurável pelo usuário (apenas oficiais)
                &self.template_form.backup_dir,
                self.template_form.backup_delay_minutes,
                self.template_form.backup_max_count,
            ) {
                Ok(_) => {
                    self.success_message =
                        Some(format!("Template '{}' atualizado", self.template_form.name));
                    self.reload_templates();

                    if self.selected_template_id == Some(template_id) {
                        if needs_watcher_restart {
                            self.select_template(template_id);
                        } else if let Some(ref mut profile) = self.active_profile {
                            profile.name = self.template_form.name.clone();
                        }
                    }

                    self.clear_template_form();
                }
                Err(e) => {
                    self.set_error_message(format!("Erro ao atualizar template: {}", e));
                }
            }
        }
    }

    /// Deleta um template
    pub fn delete_template(&mut self, template_id: i64) {
        // Verifica se não é template oficial
        if let Some(template) = self.templates.iter().find(|t| t.id == template_id) {
            if template.is_official {
                self.set_error_message("Templates oficiais não podem ser excluídos".to_string());
                return;
            }

            match self.db.delete_game_template(template_id) {
                Ok(_) => {
                    self.set_success_message("Template excluído com sucesso".to_string());
                    self.reload_templates();
                    if self.template_form.selected_for_edit == Some(template_id) {
                        self.clear_template_form();
                    }
                }
                Err(e) => {
                    self.set_error_message(format!("Erro ao excluir template: {}", e));
                }
            }
        }
    }
}
