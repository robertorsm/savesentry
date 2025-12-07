use crate::models::{GameProfile, GameTemplate};
use crate::watcher;
use std::sync::{Arc, Mutex};

/// Estado da aplicação
pub struct AppState {
    // Banco de dados
    pub db: crate::db::Database,

    // Templates disponíveis
    pub templates: Vec<GameTemplate>,
    pub selected_template: Option<usize>,

    // Perfis de jogos
    pub profiles: Arc<Mutex<Vec<GameProfile>>>,

    // Watchers ativos
    pub watchers: Arc<Mutex<Vec<watcher::WatcherHandle>>>,

    // Formulário de novo perfil
    pub form_game_name: String,
    pub form_save_path: String,
    pub form_backup_dir: String,
    pub form_timeout: u32,

    // UI state
    pub error_message: Option<String>,
    pub success_message: Option<String>,
}

impl AppState {
    /// Cria um novo estado da aplicação
    pub fn new(db_path: std::path::PathBuf) -> Self {
        // Inicializa banco de dados
        let db = crate::db::Database::new(&db_path)
            .expect("Falha ao inicializar banco de dados");

        // Carrega perfis e templates existentes
        let profiles = db.list_game_profiles().unwrap_or_default();
        let templates = db.list_game_templates().unwrap_or_default();

        Self {
            db,
            templates,
            selected_template: None,
            profiles: Arc::new(Mutex::new(profiles)),
            watchers: Arc::new(Mutex::new(Vec::new())),
            form_game_name: String::new(),
            form_save_path: String::new(),
            form_backup_dir: String::new(),
            form_timeout: 5,
            error_message: None,
            success_message: None,
        }
    }

    /// Limpa o formulário
    pub fn clear_form(&mut self) {
        self.form_game_name.clear();
        self.form_save_path.clear();
        self.form_backup_dir.clear();
        self.form_timeout = 5;
        self.selected_template = None;
    }
}

