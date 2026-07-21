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
            let save_pattern = Some(t.save_pattern.clone());
            let process_name = Some(t.process_name.clone());

            // P3: Merge de exclusões: template default + user custom
            let exclude_pattern = match (&t.default_exclude_pattern, &t.exclude_pattern) {
                (Some(default), Some(user)) => {
                    let merged = format!("{}|{}", default, user);
                    Some(merged)
                }
                (Some(default), None) => Some(default.clone()),
                (None, Some(user)) => Some(user.clone()),
                (None, None) => None,
            };

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
                backup_max_count: t.backup_max_count,
                backup_recursive: false,
            };

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

            self.active_profile = Some(profile.clone());
            self.current_save_path = save_dir;
            self.current_save_file.clear();
            self.update_save_info();

            self.set_success_message(format!("Template '{}' selecionado", template_name));

            if let Some(ref proc_name) = profile.process_name {
                if is_process_running(proc_name) {
                    let mut profile_for_watcher = profile.clone();
                    profile_for_watcher.backup_dir = self.get_backup_dir();
                    match crate::watcher::start_watching(profile_for_watcher, self.egui_ctx.clone(), None) {
                        Ok(handle) => {
                            self.active_watcher = Some(handle);
                            if let Some(ref mut active_profile) = self.active_profile {
                                active_profile.is_active = true;
                            }
                            self.invalidate_backup_cache();
                            self.reload_backup_history();
                            self.set_success_message(format!(
                                "Template '{}' selecionado — monitoramento ativo",
                                template_name
                            ));
                        }
                        Err(_e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("❌ Failed to auto-start: {}", _e);
                        }
                    }
                } else {
                    self.invalidate_backup_cache();
                    self.selected_backup_filename = None;
                    self.reload_backup_history();
                    self.set_success_message(format!(
                        "Template '{}' selecionado — aguardando '{}'",
                        template_name, proc_name
                    ));
                }
            }
        } else {
            self.set_error_message("Template não encontrado".to_string());
        }
    }

    pub fn set_backup_directory(&mut self, dir: String) {
        self.config.backup_dir = dir;
        self.reload_backup_history();
    }

    /// Inicia o monitoramento
    pub fn start_monitoring(&mut self) {
        let last_backup_time = self.last_backup_time_before_restore.take();
        self.start_monitoring_internal(last_backup_time, false);
    }

    pub fn restart_monitoring(&mut self) {
        let last_backup_time = self.last_backup_time_before_restore.take();
        self.start_monitoring_internal(last_backup_time, true);
    }

    fn start_monitoring_internal(&mut self, last_backup_time: Option<u64>, silent: bool) {
        if self.active_profile.is_none() {
            if !silent {
                self.set_error_message("Selecione um template primeiro".to_string());
            }
            return;
        }

        if self.active_watcher.is_some() {
            if !silent {
                self.set_error_message("Monitoramento já está ativo".to_string());
            }
            return;
        }

        let resolved_backup_dir = self.get_backup_dir();

        if let Some(profile) = &mut self.active_profile {
            if resolved_backup_dir.is_empty() {
                if !silent {
                    self.error_message =
                        Some("Configure o diretório de backup padrão em Configurações".to_string());
                }
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
                        if !silent {
                            self.set_error_message(format!("Erro ao salvar perfil: {}", e));
                        }
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

            let mut profile_for_watcher = profile.clone();
            profile_for_watcher.backup_dir = resolved_backup_dir;

            match crate::watcher::start_watching(profile_for_watcher, self.egui_ctx.clone(), last_backup_time) {
                Ok(handle) => {
                    self.active_watcher = Some(handle);
                    if !silent {
                        self.set_success_message("Monitoramento iniciado".to_string());
                    }
                    self.invalidate_backup_cache();
                    self.update_save_info();
                    self.reload_backup_history();
                }
                Err(e) => {
                    if !silent {
                        self.set_error_message(format!("Erro ao iniciar monitoramento: {}", e));
                    }
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

            self.set_success_message("Monitoramento parado".to_string());
        }
    }

    /// Restaura um backup
    pub fn restore_backup(&mut self, filename: &str) {
        if self.current_save_path.is_empty() {
            self.set_error_message("Configuração incompleta".to_string());
            return;
        }

        let backup_dir = self.get_backup_dir();
        if backup_dir.is_empty() {
            self.set_error_message("Diretório de backup não configurado".to_string());
            return;
        }

        let backup_path = std::path::Path::new(&backup_dir).join(filename);

        // Para o monitoramento temporariamente
        let was_monitoring = self.active_watcher.is_some();
        if was_monitoring {
            self.last_backup_time_before_restore = self.active_watcher.as_ref().map(|h| h.last_backup_time());
            self.stop_monitoring();
        }

        if let Err(_e) = self.create_safety_backup() {
            #[cfg(debug_assertions)]
            eprintln!("⚠️ Falha ao criar safety backup: {}", _e);
        }

        // Extrai o ZIP
        match extract_zip(&backup_path, &self.current_save_path) {
            Ok(_) => {
                self.set_success_message(format!("Backup '{}' restaurado", filename));
                self.update_save_info();

                let backup_dir_path = std::path::Path::new(&backup_dir);
                if let Ok(entries) = std::fs::read_dir(backup_dir_path) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.starts_with("BeforeRestore_") && name.ends_with(".zip") {
                                let path = entry.path();
                                let _ = std::fs::remove_file(&path);
                                let png_path = path.with_extension("png");
                                if png_path.exists() {
                                    let _ = std::fs::remove_file(&png_path);
                                }
                            }
                        }
                    }
                }

                // Reinicia monitoramento se estava ativo
                if was_monitoring {
                    // Agenda reinício após 2 segundos via timer não-bloqueante
                    self.restart_monitoring_after =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(2));
                }
            }
            Err(e) => {
                self.set_error_message(format!("Erro ao restaurar: {}", e));
            }
        }
    }

    /// Exclui um backup permanentemente (arquivo ZIP e screenshot PNG)
    pub fn delete_backup(&mut self, filename: &str) {
        let backup_dir = self.get_backup_dir();
        if backup_dir.is_empty() {
            self.set_error_message("Diretório de backup não configurado".to_string());
            return;
        }

        let backup_dir_path = std::path::Path::new(&backup_dir);
        let zip_path = backup_dir_path.join(filename);
        let png_path = backup_dir_path.join(filename).with_extension("png");

        let mut deleted_any = false;

        if zip_path.exists() {
            if let Err(e) = std::fs::remove_file(&zip_path) {
                self.set_error_message(format!("Erro ao excluir backup: {}", e));
                return;
            }
            deleted_any = true;
        }

        if png_path.exists() {
            if let Err(e) = std::fs::remove_file(&png_path) {
                self.set_error_message(format!("Erro ao excluir screenshot: {}", e));
                return;
            }
            deleted_any = true;
        }

        if deleted_any {
            if self.selected_backup_filename.as_deref() == Some(filename) {
                self.selected_backup_filename = None;
            }
            self.screenshot_textures.remove(filename);
            self.invalidate_backup_cache();
            self.reload_backup_history();
            self.set_success_message(format!("Backup '{}' excluído", filename));
        } else {
            self.set_error_message("Backup não encontrado".to_string());
        }
    }

    pub fn rename_backup(&mut self, old_filename: &str, new_name: &str) {
        let backup_dir = self.get_backup_dir();
        if backup_dir.is_empty() {
            self.set_error_message("Diretório de backup não configurado".to_string());
            return;
        }

        let backup_dir_path = std::path::Path::new(&backup_dir);
        let old_zip = backup_dir_path.join(old_filename);
        let old_png = backup_dir_path.join(old_filename).with_extension("png");

        let safe_name = new_name.trim().replace(" ", "_");
        if safe_name.is_empty() {
            self.set_error_message("Nome inválido".to_string());
            return;
        }

        let new_zip_name = format!("{}.zip", safe_name);
        let new_zip = backup_dir_path.join(&new_zip_name);

        if new_zip.exists() {
            self.set_error_message("Já existe um backup com esse nome".to_string());
            return;
        }

        if let Err(e) = std::fs::rename(&old_zip, &new_zip) {
            self.set_error_message(format!("Erro ao renomear backup: {}", e));
            return;
        }

        let new_png = backup_dir_path.join(&safe_name).with_extension("png");
        if old_png.exists() {
            let _ = std::fs::rename(&old_png, &new_png);
        }

        if self.selected_backup_filename.as_deref() == Some(old_filename) {
            self.selected_backup_filename = Some(new_zip_name.clone());
        }
        self.screenshot_textures.remove(old_filename);
        self.invalidate_backup_cache();
        self.reload_backup_history();
        self.set_success_message(format!("Backup renomeado para '{}'", safe_name));
    }

    /// Cria um safety backup (BeforeRestore) do estado atual do save
    fn create_safety_backup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let save_path = std::path::Path::new(&self.current_save_path);
        if !save_path.exists() {
            return Err("Diretório de save não existe".into());
        }

        let backup_dir_str = self.get_backup_dir();
        if backup_dir_str.is_empty() {
            return Err("Diretório de backup não configurado".into());
        }
        let backup_dir = std::path::Path::new(&backup_dir_str);

        let now = chrono::Local::now();
        let timestamp = now.format("%d-%m-%Y_%H-%M-%S").to_string();
        let safety_name = format!("BeforeRestore_{}", timestamp);

        if let Ok(entries) = std::fs::read_dir(backup_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("BeforeRestore_") && name.ends_with(".zip") {
                        let path = entry.path();
                        let _ = std::fs::remove_file(&path);
                        let png_path = path.with_extension("png");
                        if png_path.exists() {
                            let _ = std::fs::remove_file(&png_path);
                        }
                    }
                }
            }
        }

        let exclude_pattern = self
            .active_profile
            .as_ref()
            .and_then(|p| p.exclude_pattern.as_ref())
            .and_then(|s| glob::Pattern::new(s).ok());
        let save_pattern = self
            .active_profile
            .as_ref()
            .and_then(|p| p.save_pattern.as_ref())
            .and_then(|s| glob::Pattern::new(s).ok());

        crate::watcher::file_watcher::FileWatcher::create_backup_from_dir(
            save_path,
            backup_dir,
            exclude_pattern.as_ref(),
            save_pattern.as_ref(),
            Some(&safety_name),
            50,
            false,
        )?;

        self.set_success_message(format!("Safety backup '{}' criado", safety_name));
        self.invalidate_backup_cache();
        self.reload_backup_history();

        Ok(())
    }
}

use std::sync::Mutex;
use std::time::{Duration, Instant};

static PROCESS_CHECK_CACHE: Mutex<Option<(Instant, String, bool)>> = Mutex::new(None);

pub(crate) fn is_process_running(name: &str) -> bool {
    let target = name.to_lowercase();

    {
        let cache = PROCESS_CHECK_CACHE.lock().unwrap();
        if let Some((timestamp, cached_name, result)) = cache.as_ref() {
            if *cached_name == target && timestamp.elapsed() < Duration::from_secs(1) {
                return *result;
            }
        }
    }

    use sysinfo::{ProcessesToUpdate, System};
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    let result = system
        .processes()
        .values()
        .any(|p| p.name().to_string_lossy().to_lowercase() == target);

    {
        let mut cache = PROCESS_CHECK_CACHE.lock().unwrap();
        *cache = Some((Instant::now(), target, result));
    }

    result
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
