use crate::models::GameProfile;
use crate::ui::state::AppState;
use crate::watcher;

/// Ações da aplicação - lógica de negócio
impl AppState {
    /// Seleciona um template e preenche o formulário
    pub fn select_template(&mut self, index: usize) {
        if let Some(template) = self.templates.get(index) {
            self.selected_template = Some(index);
            self.form_game_name = template.name.clone();
            self.form_save_path = template.expand_save_directory();
            self.form_timeout = 5;
        }
    }

    /// Cria um novo perfil
    pub fn create_profile(&mut self) {
        // Validação
        if self.form_game_name.is_empty() {
            self.error_message = Some("Nome do jogo é obrigatório".to_string());
            return;
        }
        if self.form_save_path.is_empty() {
            self.error_message = Some("Caminho do save é obrigatório".to_string());
            return;
        }
        if self.form_backup_dir.is_empty() {
            self.error_message = Some("Diretório de backup é obrigatório".to_string());
            return;
        }

        let timeout = self.form_timeout;

        // Cria o perfil (com ou sem template)
        let profile = if let Some(idx) = self.selected_template {
            if let Some(template) = self.templates.get(idx) {
                GameProfile::from_template(
                    template.id,
                    template,
                    self.form_backup_dir.clone(),
                    timeout,
                )
            } else {
                GameProfile::new(
                    self.form_game_name.clone(),
                    self.form_save_path.clone(),
                    self.form_backup_dir.clone(),
                    timeout,
                )
            }
        } else {
            GameProfile::new(
                self.form_game_name.clone(),
                self.form_save_path.clone(),
                self.form_backup_dir.clone(),
                timeout,
            )
        };

        // Insere no banco
        if let Err(e) = self.db.insert_game_profile(&profile) {
            self.error_message = Some(format!("Erro ao criar perfil: {}", e));
            return;
        }

        // Recarrega perfis
        self.reload_profiles();

        self.success_message = Some("Perfil criado com sucesso!".to_string());
        self.clear_form();
    }

    /// Alterna o monitoramento de um perfil
    pub fn toggle_monitoring(&mut self, profile_id: i64) {
        let mut profiles = match self.profiles.lock() {
            Ok(p) => p,
            Err(_) => return,
        };

        if let Some(profile) = profiles.iter_mut().find(|p| p.id == profile_id) {
            profile.is_active = !profile.is_active;

            // Atualiza no banco
            if let Err(e) = self.db.update_profile_status(profile_id, profile.is_active) {
                self.error_message = Some(format!("Erro ao atualizar status: {}", e));
                return;
            }

            // Se ativou, inicia watcher
            if profile.is_active {
                if let Ok(handle) = watcher::start_watching(profile.clone()) {
                    if let Ok(mut w) = self.watchers.lock() {
                        w.push(handle);
                    }
                }
            } else {
                // Se desativou, para o watcher correspondente
                if let Ok(mut w) = self.watchers.lock() {
                    w.retain(|h| h.profile_id() != profile_id);
                }
            }
        }
    }

    /// Exclui um perfil
    pub fn delete_profile(&mut self, profile_id: i64) {
        // Para o watcher se estiver ativo
        if let Ok(mut w) = self.watchers.lock() {
            w.retain(|h| h.profile_id() != profile_id);
        }

        // Remove do banco
        if let Err(e) = self.db.delete_game_profile(profile_id) {
            self.error_message = Some(format!("Erro ao excluir perfil: {}", e));
            return;
        }

        // Recarrega perfis
        self.reload_profiles();

        self.success_message = Some("Perfil excluído com sucesso!".to_string());
    }

    /// Recarrega perfis do banco
    fn reload_profiles(&mut self) {
        if let Ok(profiles) = self.db.list_game_profiles() {
            if let Ok(mut p) = self.profiles.lock() {
                *p = profiles;
            }
        }
    }
}

