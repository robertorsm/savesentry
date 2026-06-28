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
            let process_name = Some(t.process_name.clone());

            // 🔄 NOVO: Para watcher anterior se estiver ativo (troca de perfil)
            if self.active_watcher.is_some() {
                #[cfg(debug_assertions)]
                println!("🔄 Parando watcher anterior para trocar de perfil...");

                self.active_watcher = None; // Para watcher anterior

                if let Some(ref mut old_profile) = self.active_profile {
                    old_profile.is_active = false;
                }
            }

            self.selected_template_id = Some(template_id);

            // Cria subdiretório específico para este jogo dentro do backup_dir global
            let game_backup_dir = if self.config.backup_dir.is_empty() {
                String::new()
            } else {
                std::path::Path::new(&self.config.backup_dir)
                    .join(&template_name)
                    .to_string_lossy()
                    .to_string()
            };

            // Cria perfil ativo baseado no template
            let mut profile = GameProfile {
                id: 0, // ID temporário (não salvo no banco)
                name: template_name.clone(),
                save_path: save_dir.clone(),
                backup_dir: game_backup_dir,
                timeout_minutes: self.config.timeout_minutes,
                exclude_regex,
                is_active: false,
                template_id: Some(template_id),
                process_name,
                created_at: chrono::Local::now().to_rfc3339(),
            };

            // 💾 Salva perfil no banco imediatamente se diretório de backup está configurado
            if !self.config.backup_dir.is_empty() {
                match self.db.insert_game_profile(&profile) {
                    Ok(new_id) => {
                        profile.id = new_id;

                        // Salva como último perfil usado
                        let _ = self.db.update_last_profile(
                            profile.id,
                            &profile.backup_dir,
                            profile.timeout_minutes
                        );

                        #[cfg(debug_assertions)]
                        println!("💾 Perfil salvo e registrado como último usado (ID: {})", new_id);
                    }
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("⚠️ Não foi possível salvar perfil: {}", _e);
                        // Continua mesmo se falhar (perfil temporário)
                    }
                }
            }

            self.active_profile = Some(profile.clone());
            self.current_save_path = save_dir;
            self.update_save_info();

            self.success_message = Some(format!("Template '{}' selecionado", template_name));

            // 🚀 Auto-start watcher se tem process_name (aguardando processo)
            if profile.process_name.is_some() {
                match crate::watcher::start_watching(profile) {
                    Ok(handle) => {
                        self.active_watcher = Some(handle);

                        #[cfg(debug_assertions)]
                        println!("🚀 Auto-started watcher for {} (awaiting process)", template_name);
                    }
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("❌ Failed to auto-start: {}", _e);
                    }
                }
            }
        } else {
            self.error_message = Some("Template não encontrado".to_string());
        }
    }

    /// Configura diretório de backup
    pub fn set_backup_directory(&mut self, dir: String) {
        self.config.backup_dir = dir;

        // Atualiza perfil ativo se existir
        if let Some(ref mut profile) = self.active_profile {
            // Cria subdiretório específico para o jogo
            if self.config.backup_dir.is_empty() {
                profile.backup_dir = String::new();
            } else {
                profile.backup_dir = std::path::Path::new(&self.config.backup_dir)
                    .join(&profile.name)
                    .to_string_lossy()
                    .to_string();
            }
        }

        // Recarrega histórico de backups
        self.reload_backup_history();
    }

    /// Configura timeout de backup
    pub fn set_timeout(&mut self, minutes: u32) {
        self.config.timeout_minutes = minutes;

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
        if self.config.backup_dir.is_empty() {
            self.error_message = Some("Configure o diretório de backup".to_string());
            return;
        }

        // Valida se já está monitorando
        if self.active_watcher.is_some() {
            self.error_message = Some("Monitoramento já está ativo".to_string());
            return;
        }

        // Pega o perfil ativo
        if let Some(profile) = &mut self.active_profile {
            // Salva perfil no banco se ainda não foi salvo (id == 0)
            if profile.id == 0 {
                match self.db.insert_game_profile(profile) {
                    Ok(new_id) => {
                        profile.id = new_id;

                        #[cfg(debug_assertions)]
                        println!("💾 Perfil salvo no banco com ID: {}", new_id);
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Erro ao salvar perfil: {}", e));
                        return;
                    }
                }
            }

            // 🚀 Salva como último perfil usado
            let _ = self.db.update_last_profile(
                profile.id,
                &profile.backup_dir,
                profile.timeout_minutes
            );

            profile.is_active = true;

            // Clone apenas uma vez para enviar para thread (necessário)
            match crate::watcher::start_watching(profile.clone()) {
                Ok(handle) => {
                    self.active_watcher = Some(handle);
                    self.success_message = Some("Monitoramento iniciado".to_string());
                    self.invalidate_backup_cache(); // Invalida cache para forçar reload
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
        if self.current_save_path.is_empty() {
            self.error_message = Some("Configuração incompleta".to_string());
            return;
        }

        // Obtém backup_dir do perfil ativo
        let backup_dir = if let Some(ref profile) = self.active_profile {
            profile.backup_dir.clone()
        } else {
            self.error_message = Some("Nenhum perfil ativo".to_string());
            return;
        };

        if backup_dir.is_empty() {
            self.error_message = Some("Diretório de backup não configurado".to_string());
            return;
        }

        let backup_path = std::path::Path::new(&backup_dir).join(filename);

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
                    // Agenda reinício após 2 segundos via timer não-bloqueante
                    self.restart_monitoring_after = Some(
                        std::time::Instant::now() + std::time::Duration::from_secs(2)
                    );
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
