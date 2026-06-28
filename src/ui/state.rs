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
    pub timeout_minutes: u32,
    pub backup_dir: String,
}

/// Estado do formulário de template
pub struct TemplateForm {
    pub selected_for_edit: Option<i64>,
    pub name: String,
    pub save_dir: String,
    pub process: String,
    pub pattern: String,
    pub exclude: String,
    pub is_new: bool,
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
    pub fn new(db_path: std::path::PathBuf) -> Self {
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
            current_save_modified: None,
            last_save_info_update: std::time::Instant::now(),
            config: AppConfig {
                timeout_minutes: 5,
                backup_dir: String::new(),
            },
            error_message: None,
            success_message: None,
            active_tab: ActiveTab::Main,
            template_form: TemplateForm {
                selected_for_edit: None,
                name: String::new(),
                save_dir: String::new(),
                process: String::new(),
                pattern: String::from("*.*"),
                exclude: String::new(),
                is_new: true,
            },
            restart_monitoring_after: None,
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
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        self.backup_history = backups;
        self.backup_history_last_reload = Some(std::time::Instant::now());
    }

    /// Invalida o cache de backup history (forçar reload no próximo acesso)
    pub fn invalidate_backup_cache(&mut self) {
        self.backup_history_last_reload = None;
    }

    /// Atualiza informações do save atual (throttled - máximo a cada 2 segundos)
    pub fn update_save_info(&mut self) {
        // Throttling: atualiza no máximo a cada 2 segundos para evitar I/O excessivo
        let now = std::time::Instant::now();
        if now.duration_since(self.last_save_info_update) < std::time::Duration::from_secs(2) {
            return;
        }
        self.last_save_info_update = now;

        if self.current_save_path.is_empty() {
            return;
        }

        let save_path = std::path::Path::new(&self.current_save_path);
        if let Ok(metadata) = std::fs::metadata(save_path) {
            self.current_save_modified = metadata.modified().ok();
        }
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
        self.template_form.process.clear();
        self.template_form.pattern = String::from("*.*");
        self.template_form.exclude.clear();
        self.template_form.is_new = true;
    }

    /// Recarrega lista de templates do banco
    pub fn reload_templates(&mut self) {
        self.templates = self.db.list_game_templates().unwrap_or_default();
    }

    /// Restaura último perfil usado ao iniciar aplicação
    fn restore_last_profile(&mut self) {
        if let Ok((last_profile_id, last_backup_dir, last_timeout)) = self.db.get_app_state() {
            // Restaura configurações
            if let Some(dir) = last_backup_dir {
                self.config.backup_dir = dir;
            }
            self.config.timeout_minutes = last_timeout;

            // Restaura perfil
            if let Some(profile_id) = last_profile_id {
                if let Ok(profile) = self.db.get_game_profile(profile_id) {
                    self.active_profile = Some(profile.clone());
                    self.selected_template_id = profile.template_id;
                    self.current_save_path = profile.save_path.clone();
                    self.update_save_info();

                    #[cfg(debug_assertions)]
                    println!("📋 Restored last profile: {}", profile.name);

                    // 🚀 Auto-start watcher se tem process_name
                    if profile.process_name.is_some() {
                        match crate::watcher::start_watching(profile) {
                            Ok(handle) => {
                                self.active_watcher = Some(handle);

                                #[cfg(debug_assertions)]
                                println!("✅ Auto-started watcher for process: {:?}",
                                    self.active_profile.as_ref().unwrap().process_name);
                            }
                            Err(_e) => {
                                #[cfg(debug_assertions)]
                                eprintln!("❌ Failed to auto-start watcher: {}", _e);
                            }
                        }
                    }
                }
            }
        }
    }
}
