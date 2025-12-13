use crate::models::{GameProfile, GameTemplate};
use crate::watcher;

/// Estado da aplicação (simplificado para single profile)
pub struct AppState {
    // Banco de dados
    #[allow(dead_code)]
    pub db: crate::db::Database,

    // Templates disponíveis
    pub templates: Vec<GameTemplate>,
    pub selected_template_id: Option<i64>,

    // Perfil ativo atual (apenas 1)
    pub active_profile: Option<GameProfile>,

    // Watcher ativo
    pub active_watcher: Option<watcher::WatcherHandle>,

    // Lista de backups criados (histórico)
    pub backup_history: Vec<BackupEntry>,

    // Informações do save atual
    pub current_save_path: String,
    pub current_save_modified: Option<std::time::SystemTime>,

    // Configuração
    pub config_timeout: u32,
    pub config_backup_dir: String,

    // UI state
    pub error_message: Option<String>,
    pub success_message: Option<String>,
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

        Self {
            db,
            templates,
            selected_template_id: None,
            active_profile: None,
            active_watcher: None,
            backup_history: Vec::new(),
            current_save_path: String::new(),
            current_save_modified: None,
            config_timeout: 5,
            config_backup_dir: String::new(),
            error_message: None,
            success_message: None,
        }
    }

    /// Carrega histórico de backups do diretório
    pub fn reload_backup_history(&mut self) {
        if self.config_backup_dir.is_empty() {
            return;
        }

        let backup_dir = std::path::Path::new(&self.config_backup_dir);
        if !backup_dir.exists() {
            return;
        }

        let mut backups = Vec::new();

        if let Ok(entries) = std::fs::read_dir(backup_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.ends_with(".zip") {
                                let save_name = self
                                    .current_save_path
                                    .split(&['/', '\\'][..])
                                    .next_back()
                                    .unwrap_or("save")
                                    .to_string();

                                backups.push(BackupEntry {
                                    filename: filename.to_string(),
                                    created_at: metadata
                                        .modified()
                                        .unwrap_or(std::time::SystemTime::now()),
                                    size_bytes: metadata.len(),
                                    save_name,
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
    }

    /// Atualiza informações do save atual
    pub fn update_save_info(&mut self) {
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
}
