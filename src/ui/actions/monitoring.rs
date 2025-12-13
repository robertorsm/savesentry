//! Ações relacionadas a monitoramento e backup de saves

use crate::models::GameProfile;
use crate::ui::state::AppState;

impl AppState {
    /// Seleciona um template e configura o perfil ativo
    pub fn select_template(&mut self, template_id: i64) {
        // Encontra o template
        let template = self.templates.iter().find(|t| t.id == template_id);

        if let Some(t) = template {
            // Captura dados do template antes de usar &mut self
            let template_name = t.name.clone();
            let save_dir = t.expand_save_directory();
            let exclude_regex = t.exclude_regex.clone();

            self.selected_template_id = Some(template_id);

            // Cria perfil ativo baseado no template
            let profile = GameProfile {
                id: 0, // ID temporário (não salvo no banco)
                name: template_name.clone(),
                save_path: save_dir.clone(),
                backup_dir: self.config_backup_dir.clone(),
                timeout_minutes: self.config_timeout,
                exclude_regex,
                is_active: false,
                template_id: Some(template_id),
                created_at: chrono::Local::now().to_rfc3339(),
            };

            self.active_profile = Some(profile);
            self.current_save_path = save_dir;
            self.update_save_info();

            self.success_message = Some(format!("Template '{}' selecionado", template_name));
        } else {
            self.error_message = Some("Template não encontrado".to_string());
        }
    }

    /// Configura diretório de backup
    pub fn set_backup_directory(&mut self, dir: String) {
        self.config_backup_dir = dir;

        // Atualiza perfil ativo se existir
        if let Some(ref mut profile) = self.active_profile {
            profile.backup_dir = self.config_backup_dir.clone();
        }

        // Recarrega histórico de backups
        self.reload_backup_history();
    }

    /// Configura timeout de backup
    pub fn set_timeout(&mut self, minutes: u32) {
        self.config_timeout = minutes;

        // Atualiza perfil ativo se existir
        if let Some(ref mut profile) = self.active_profile {
            profile.timeout_minutes = minutes;
        }
    }

    /// Inicia o monitoramento
    pub fn start_monitoring(&mut self) {
        // Valida se tem perfil ativo
        if self.active_profile.is_none() {
            self.error_message = Some("Selecione um template primeiro".to_string());
            return;
        }

        // Valida se tem diretório de backup
        if self.config_backup_dir.is_empty() {
            self.error_message = Some("Configure o diretório de backup".to_string());
            return;
        }

        // Valida se já está monitorando
        if self.active_watcher.is_some() {
            self.error_message = Some("Monitoramento já está ativo".to_string());
            return;
        }

        // Pega o perfil ativo
        if let Some(mut profile) = self.active_profile.clone() {
            profile.is_active = true;

            // Inicia watcher
            match crate::watcher::start_watching(profile.clone()) {
                Ok(handle) => {
                    self.active_watcher = Some(handle);
                    self.active_profile = Some(profile);
                    self.success_message = Some("Monitoramento iniciado".to_string());
                    self.reload_backup_history();
                }
                Err(e) => {
                    self.error_message = Some(format!("Erro ao iniciar monitoramento: {}", e));
                }
            }
        }
    }

    /// Para o monitoramento
    pub fn stop_monitoring(&mut self) {
        if self.active_watcher.is_some() {
            self.active_watcher = None;

            if let Some(ref mut profile) = self.active_profile {
                profile.is_active = false;
            }

            self.success_message = Some("Monitoramento parado".to_string());
        }
    }

    /// Restaura um backup
    pub fn restore_backup(&mut self, filename: &str) {
        if self.config_backup_dir.is_empty() || self.current_save_path.is_empty() {
            self.error_message = Some("Configuração incompleta".to_string());
            return;
        }

        let backup_path = std::path::Path::new(&self.config_backup_dir).join(filename);

        // Para o monitoramento temporariamente
        let was_monitoring = self.active_watcher.is_some();
        if was_monitoring {
            self.stop_monitoring();
        }

        // Extrai o ZIP
        match extract_zip(&backup_path, &self.current_save_path) {
            Ok(_) => {
                self.success_message = Some(format!("Backup '{}' restaurado", filename));
                self.update_save_info();

                // Reinicia monitoramento se estava ativo
                if was_monitoring {
                    // Aguarda 2 segundos antes de reiniciar (evita backup imediato)
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    self.start_monitoring();
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Erro ao restaurar: {}", e));
            }
        }
    }
}

/// Extrai um arquivo ZIP para o caminho de destino
fn extract_zip(
    zip_path: &std::path::Path,
    dest_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Extrai o primeiro arquivo do ZIP
    let mut file = archive.by_index(0)?;
    let mut outfile = std::fs::File::create(dest_path)?;
    std::io::copy(&mut file, &mut outfile)?;

    Ok(())
}
