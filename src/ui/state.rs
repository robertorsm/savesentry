use crate::models::{GameProfile, GameTemplate};
use crate::watcher;

/// Abas da aplicação
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Main,
    Templates,
    Settings,
}

/// Configurações da aplicação
pub struct AppConfig {
    pub backup_delay_minutes: u32,
    pub backup_dir: String,
}

/// Estado do formulário de template
pub struct TemplateForm {
    pub selected_for_edit: Option<i64>,
    pub name: String,
    pub save_dir: String,
    pub backup_dir: String,
    pub backup_delay_minutes: u32,
    pub backup_max_count: u32,
    pub process: String,
    pub pattern: String,
    pub exclude: String,
    pub is_new: bool,
    pub original_save_dir: String,
    pub original_backup_dir: String,
    pub original_process: String,
    pub original_pattern: String,
    pub original_exclude: String,
    pub original_backup_delay_minutes: u32,
    pub original_backup_max_count: u32,
}

/// Estado da aplicação (single profile com auto-restore do último usado)
pub struct AppState {
    // Banco de dados
    pub db: crate::db::Database,

    // Templates disponíveis
    pub templates: Vec<GameTemplate>,
    pub selected_template_id: Option<i64>,

    // Perfil ativo atual (último usado - auto-restored ao iniciar)
    pub active_profile: Option<GameProfile>,

    // Watcher ativo (apenas 1)
    pub active_watcher: Option<watcher::WatcherHandle>,

    // Lista de backups criados (histórico)
    pub backup_history: Vec<BackupEntry>,
    backup_history_last_reload: Option<std::time::Instant>,

    // Informações do save atual
    pub current_save_path: String,
    pub current_save_file: String,
    pub current_save_modified: Option<std::time::SystemTime>,
    last_save_info_update: std::time::Instant,

    // Configuração
    pub config: AppConfig,

    // UI state
    pub error_message: Option<String>,
    pub success_message: Option<String>,

    // Navegação por abas
    pub active_tab: ActiveTab,

    // Gerenciamento de templates
    pub template_form: TemplateForm,

    // Timer não-bloqueante para reinício de monitoramento após restore
    pub restart_monitoring_after: Option<std::time::Instant>,

    // Rastreia o último backup já refletido na UI para atualização automática
    last_seen_backup_time: u64,

    pub last_ui_update: std::time::Instant,
    pub selected_backup_filename: Option<String>,
    pub screenshot_textures: std::collections::HashMap<String, eframe::egui::TextureHandle>,

    pub egui_ctx: eframe::egui::Context,
}

/// Entrada no histórico de backups
#[derive(Clone, Debug)]
pub struct BackupEntry {
    pub filename: String,
    pub created_at: std::time::SystemTime,
    pub size_bytes: u64,
    #[allow(dead_code)]
    pub save_name: String,
}

impl AppState {
    /// Cria um novo estado da aplicação
    pub fn new(db_path: std::path::PathBuf, egui_ctx: eframe::egui::Context) -> Self {
        // Inicializa banco de dados
        let db = crate::db::Database::new(&db_path).expect("Falha ao inicializar banco de dados");

        // Carrega templates existentes
        let templates = db.list_game_templates().unwrap_or_default();

        let mut state = Self {
            db,
            templates,
            selected_template_id: None,
            active_profile: None,
            active_watcher: None,
            backup_history: Vec::new(),
            backup_history_last_reload: None,
            current_save_path: String::new(),
            current_save_file: String::new(),
            current_save_modified: None,
            last_save_info_update: std::time::Instant::now(),
            config: AppConfig {
                backup_delay_minutes: 5,
                backup_dir: String::new(),
            },
            error_message: None,
            success_message: None,
            active_tab: ActiveTab::Main,
            template_form: TemplateForm {
                selected_for_edit: None,
                name: String::new(),
                save_dir: String::new(),
                backup_dir: String::new(),
                backup_delay_minutes: 5,
                backup_max_count: 50,
                process: String::new(),
                pattern: String::from("*.*"),
                exclude: String::new(),
                is_new: true,
                original_save_dir: String::new(),
                original_backup_dir: String::new(),
                original_process: String::new(),
                original_pattern: String::new(),
                original_exclude: String::new(),
                original_backup_delay_minutes: 5,
                original_backup_max_count: 50,
            },
            restart_monitoring_after: None,
            last_seen_backup_time: 0,
            last_ui_update: std::time::Instant::now(),
            selected_backup_filename: None,
            screenshot_textures: std::collections::HashMap::new(),
            egui_ctx,
        };

        // 🚀 Auto-restore último perfil usado
        state.restore_last_profile();

        state
    }

    /// Carrega histórico de backups do diretório (cached - TTL de 5 segundos)
    pub fn reload_backup_history(&mut self) {
        // Cache: recarrega apenas se passou mais de 5 segundos desde o último reload
        if let Some(last_reload) = self.backup_history_last_reload {
            if last_reload.elapsed() < std::time::Duration::from_secs(5) {
                return; // Usa cache
            }
        }

        // Obtém backup_dir do perfil ativo (ou usa config.backup_dir se não houver perfil)
        let backup_dir_str = if let Some(ref profile) = self.active_profile {
            if profile.backup_dir.is_empty() {
                return;
            }
            profile.backup_dir.clone()
        } else {
            if self.config.backup_dir.is_empty() {
                return;
            }
            self.config.backup_dir.clone()
        };

        let backup_dir = std::path::Path::new(&backup_dir_str);
        if !backup_dir.exists() {
            return;
        }

        let mut backups = Vec::new();

        // Calcula save_name uma única vez (não muda durante o loop)
        let save_name = self
            .current_save_path
            .split(&['/', '\\'][..])
            .next_back()
            .unwrap_or("save")
            .to_string();

        if let Ok(entries) = std::fs::read_dir(backup_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.ends_with(".zip") {
                                backups.push(BackupEntry {
                                    filename: filename.to_string(),
                                    created_at: metadata
                                        .modified()
                                        .unwrap_or(std::time::SystemTime::now()),
                                    size_bytes: metadata.len(),
                                    save_name: save_name.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Ordena por data (mais recente primeiro)
        backups.sort_by_key(|b| std::cmp::Reverse(b.created_at));

        self.backup_history = backups;
        self.backup_history_last_reload = Some(std::time::Instant::now());
    }

    /// Invalida o cache de backup history (forçar reload no próximo acesso)
    pub fn invalidate_backup_cache(&mut self) {
        self.backup_history_last_reload = None;
    }

    fn find_latest_save(&self) -> Option<(std::path::PathBuf, std::time::SystemTime)> {
        let save_dir = std::path::Path::new(&self.current_save_path);
        if !save_dir.is_dir() {
            #[cfg(debug_assertions)]
            eprintln!(
                "[find_latest_save] Nao eh diretorio: {:?}",
                self.current_save_path
            );
            return None;
        }

        let pattern_str = self
            .active_profile
            .as_ref()
            .and_then(|p| p.save_pattern.as_deref())
            .unwrap_or("*");
        let pattern = glob::Pattern::new(pattern_str).ok()?;
        let match_opts = glob::MatchOptions {
            case_sensitive: cfg!(not(target_os = "windows")),
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        #[cfg(debug_assertions)]
        println!(
            "[find_latest_save] Escaneando {:?} com padrao '{}'",
            save_dir, pattern_str
        );

        let mut latest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
        if let Ok(entries) = std::fs::read_dir(save_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            let matched = pattern.matches_with(name, match_opts);
                            #[cfg(debug_assertions)]
                            println!(
                                "[find_latest_save]   {} -> {}",
                                name,
                                if matched { "bate" } else { "nao bate" }
                            );
                            if matched {
                                if let Ok(modified) = metadata.modified() {
                                    if latest.as_ref().is_none_or(|(_, t)| modified > *t) {
                                        latest = Some((entry.path(), modified));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(debug_assertions)]
        if let Some((ref path, _)) = latest {
            println!("[find_latest_save] Selecionado: {:?}", path);
        } else {
            println!("[find_latest_save] Nenhum arquivo bateu no padrao");
        }

        latest
    }

    /// Atualiza informações do save atual (throttled - máximo a cada 2 segundos)
    pub fn update_save_info(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_save_info_update) < std::time::Duration::from_secs(2) {
            return;
        }
        self.last_save_info_update = now;

        if self.current_save_path.is_empty() {
            return;
        }

        // Event-driven: usa informação do watcher se disponível
        if let Some(ref watcher) = self.active_watcher {
            let recent = watcher.recent_save.lock().unwrap();
            if let Some((path, modified)) = recent.as_ref() {
                self.current_save_file = path.clone();
                self.current_save_modified = Some(*modified);
                return;
            }
        }

        // Fallback: escaneamento do diretório quando watcher não está ativo
        if let Some((path, modified)) = self.find_latest_save() {
            self.current_save_file = path.to_string_lossy().to_string();
            self.current_save_modified = Some(modified);
        } else {
            self.current_save_file.clear();
            let save_path = std::path::Path::new(&self.current_save_path);
            if let Ok(metadata) = std::fs::metadata(save_path) {
                self.current_save_modified = metadata.modified().ok();
            }
        }
    }

    /// Verifica se um novo backup foi criado pelo watcher e recarrega o histórico
    /// Retorna true se o histórico foi atualizado
    pub fn check_backup_updates(&mut self) -> bool {
        if let Some(ref watcher) = self.active_watcher {
            let last_backup = watcher.last_backup_time();
            if last_backup > 0 && last_backup != self.last_seen_backup_time {
                self.last_seen_backup_time = last_backup;
                self.invalidate_backup_cache();
                self.reload_backup_history();
                return true;
            }
        }
        false
    }

    /// Limpa mensagens de erro/sucesso
    #[allow(dead_code)]
    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }

    /// Limpa o formulário de template
    pub fn clear_template_form(&mut self) {
        self.template_form.selected_for_edit = None;
        self.template_form.name.clear();
        self.template_form.save_dir.clear();
        self.template_form.backup_dir.clear();
        self.template_form.backup_delay_minutes = 5;
        self.template_form.backup_max_count = 50;
        self.template_form.process.clear();
        self.template_form.pattern = String::from("*.*");
        self.template_form.exclude.clear();
        self.template_form.is_new = true;
        self.template_form.original_save_dir.clear();
        self.template_form.original_backup_dir.clear();
        self.template_form.original_process.clear();
        self.template_form.original_pattern.clear();
        self.template_form.original_exclude.clear();
        self.template_form.original_backup_delay_minutes = 5;
        self.template_form.original_backup_max_count = 50;
    }

    /// Recarrega lista de templates do banco
    pub fn reload_templates(&mut self) {
        self.templates = self.db.list_game_templates().unwrap_or_default();
    }

    fn restore_last_profile(&mut self) {
        if let Ok((last_profile_id, last_backup_dir, last_backup_delay)) = self.db.get_app_state() {
            if let Some(dir) = last_backup_dir {
                self.config.backup_dir = dir;
            }
            self.config.backup_delay_minutes = last_backup_delay;

            if let Some(profile_id) = last_profile_id {
                if let Ok(profile) = self.db.get_game_profile(profile_id) {
                    self.active_profile = Some(profile.clone());
                    self.selected_template_id = profile.template_id;
                    self.current_save_path = profile.save_path.clone();
                    self.current_save_file.clear();
                    self.update_save_info();

                    #[cfg(debug_assertions)]
                    println!("📋 Restored last profile: {}", profile.name);

                    if let Some(ref proc_name) = profile.process_name {
                        if crate::ui::actions::monitoring::is_process_running(proc_name) {
                            match crate::watcher::start_watching(profile, self.egui_ctx.clone()) {
                                Ok(handle) => {
                                    self.active_watcher = Some(handle);
                                    if let Some(ref mut active_profile) = self.active_profile {
                                        active_profile.is_active = true;
                                    }
                                    self.invalidate_backup_cache();
                                    self.reload_backup_history();

                                    #[cfg(debug_assertions)]
                                    println!(
                                        "✅ Auto-started watcher for process: {:?}",
                                        self.active_profile.as_ref().unwrap().process_name
                                    );
                                    return;
                                }
                                Err(e) => {
                                    self.error_message =
                                        Some(format!("Falha ao iniciar watcher: {}", e));
                                    self.active_profile = None;
                                    self.selected_template_id = None;
                                    self.current_save_path.clear();
                                }
                            }
                        } else {
                            self.success_message = Some(format!(
                                "Perfil '{}' restaurado — aguardando '{}'",
                                profile.name, proc_name
                            ));
                            return;
                        }
                    } else {
                        match crate::watcher::start_watching(profile, self.egui_ctx.clone()) {
                            Ok(handle) => {
                                self.active_watcher = Some(handle);
                                if let Some(ref mut active_profile) = self.active_profile {
                                    active_profile.is_active = true;
                                }
                                self.invalidate_backup_cache();
                                self.reload_backup_history();
                                return;
                            }
                            Err(e) => {
                                self.error_message =
                                    Some(format!("Falha ao iniciar watcher: {}", e));
                                self.active_profile = None;
                                self.selected_template_id = None;
                                self.current_save_path.clear();
                            }
                        }
                    }
                }
            }
        }

        self.try_auto_detect_running_game();
    }

    fn try_auto_detect_running_game(&mut self) {
        use sysinfo::{ProcessesToUpdate, System};

        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let running_processes: Vec<String> = system
            .processes()
            .values()
            .map(|p| p.name().to_string_lossy().to_lowercase())
            .collect();

        for template in &self.templates {
            let target = template.process_name.to_lowercase();
            if running_processes.iter().any(|name| name == &target) {
                #[cfg(debug_assertions)]
                println!(
                    "🎮 Jogo detectado no startup: {} ({})",
                    template.name, template.process_name
                );

                self.select_template(template.id);
                return;
            }
        }

        #[cfg(debug_assertions)]
        println!("🔍 Nenhum jogo detectado no startup");
    }

    pub fn get_backup_dir(&self) -> String {
        if let Some(ref profile) = self.active_profile {
            if !profile.backup_dir.is_empty() {
                return profile.backup_dir.clone();
            }
        }
        self.config.backup_dir.clone()
    }

    pub fn load_screenshot_texture(
        &mut self,
        ctx: &eframe::egui::Context,
        filename: &str,
    ) -> Option<eframe::egui::TextureHandle> {
        if let Some(tex) = self.screenshot_textures.get(filename) {
            return Some(tex.clone());
        }

        let backup_dir = self.get_backup_dir();
        if backup_dir.is_empty() {
            return None;
        }

        let path = std::path::Path::new(&backup_dir)
            .join(filename)
            .with_extension("png");
        if !path.exists() {
            return None;
        }

        let img = image::open(&path).ok()?;
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let color_image = eframe::egui::ColorImage::from_rgba_unmultiplied(size, &rgba);

        let texture = ctx.load_texture(
            filename,
            color_image,
            eframe::egui::TextureOptions::default(),
        );

        self.screenshot_textures
            .insert(filename.to_string(), texture.clone());
        Some(texture)
    }
}
