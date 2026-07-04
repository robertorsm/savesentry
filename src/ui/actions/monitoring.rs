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
            let exclude_pattern = t.exclude_pattern.clone();
            let save_pattern = Some(t.save_pattern.clone());
            let process_name = Some(t.process_name.clone());

            // 🔄 NOVO: Para watcher anterior se estiver ativo (troca de perfil)
            if self.active_watcher.is_some() {
                #[cfg(debug_assertions)]
                println!("🔄 Parando watcher anterior para trocar de perfil...");

                self.active_watcher = None;

                if let Some(ref mut old_profile) = self.active_profile {
                    old_profile.is_active = false;
                }
            }

            self.selected_template_id = Some(template_id);

            let game_backup_dir = t.expand_backup_directory();

            // Cria perfil ativo baseado no template
            let mut profile = GameProfile {
                id: 0,
                name: template_name.clone(),
                save_path: save_dir.clone(),
                backup_dir: game_backup_dir,
                backup_delay_minutes: t.backup_delay_minutes,
                exclude_pattern,
                save_pattern,
                is_active: false,
                template_id: Some(template_id),
                process_name,
                created_at: chrono::Local::now().to_rfc3339(),
            };

            if !profile.backup_dir.is_empty() {
                match self.db.insert_game_profile(&profile) {
                    Ok(new_id) => {
                        profile.id = new_id;

                        // Salva como último perfil usado
                        let _ = self.db.update_last_profile(
                            profile.id,
                            &profile.backup_dir,
                            profile.backup_delay_minutes,
                        );

                        #[cfg(debug_assertions)]
                        println!(
                            "💾 Perfil salvo e registrado como último usado (ID: {})",
                            new_id
                        );
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
            self.current_save_file.clear();
            self.update_save_info();

            self.success_message = Some(format!("Template '{}' selecionado", template_name));

            // 🚀 Auto-start watcher se tem process_name (aguardando processo)
            if profile.process_name.is_some() {
                match crate::watcher::start_watching(profile) {
                    Ok(handle) => {
                        self.active_watcher = Some(handle);

                        #[cfg(debug_assertions)]
                        println!(
                            "🚀 Auto-started watcher for {} (awaiting process)",
                            template_name
                        );
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

    pub fn set_backup_directory(&mut self, dir: String) {
        self.config.backup_dir = dir;
        self.reload_backup_history();
    }

    /// Inicia o monitoramento
    pub fn start_monitoring(&mut self) {
        // Valida se tem perfil ativo
        if self.active_profile.is_none() {
            self.error_message = Some("Selecione um template primeiro".to_string());
            return;
        }

        // Valida se já está monitorando
        if self.active_watcher.is_some() {
            self.error_message = Some("Monitoramento já está ativo".to_string());
            return;
        }

        // Pega o perfil ativo
        if let Some(profile) = &mut self.active_profile {
            if profile.backup_dir.is_empty() {
                self.error_message = Some("Configure o diretório de backup no template".to_string());
                return;
            }

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

            let _ = self.db.update_last_profile(
                profile.id,
                &profile.backup_dir,
                profile.backup_delay_minutes,
            );

            profile.is_active = true;

            // Clone apenas uma vez para enviar para thread (necessário)
            match crate::watcher::start_watching(profile.clone()) {
                Ok(handle) => {
                    self.active_watcher = Some(handle);
                    self.success_message = Some("Monitoramento iniciado".to_string());
                    self.invalidate_backup_cache();
                    self.update_save_info();
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
                    self.restart_monitoring_after =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(2));
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Erro ao restaurar: {}", e));
            }
        }
    }
}

/// Extrai todos os arquivos de um ZIP para o diretório de destino
fn extract_zip(
    zip_path: &std::path::Path,
    dest_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let dest = std::path::Path::new(dest_dir);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest.join(file.name());

        if let Some(parent) = outpath.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut outfile = std::fs::File::create(&outpath)?;
        std::io::copy(&mut file, &mut outfile)?;
    }

    Ok(())
}
