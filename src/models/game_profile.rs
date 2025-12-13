use serde::{Deserialize, Serialize};

/// Perfil de um jogo para monitoramento e backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProfile {
    pub id: i64,
    pub template_id: Option<i64>, // FK para GameTemplate (se baseado em template)
    pub name: String,             // Nome do jogo
    pub save_path: String,        // Caminho do arquivo de save
    pub backup_dir: String,       // Diretório onde backups serão salvos
    pub timeout_minutes: u32,     // Intervalo mínimo entre backups (em minutos)
    pub exclude_regex: Option<String>, // Regex de exclusão (pode sobrescrever template)
    pub is_active: bool,          // Se o monitoramento está ativo
    pub created_at: String,       // Data de criação do perfil
}

impl GameProfile {
    /// Cria um novo perfil de jogo
    #[allow(dead_code)]
    pub fn new(name: String, save_path: String, backup_dir: String, timeout_minutes: u32) -> Self {
        Self {
            id: 0,
            template_id: None,
            name,
            save_path,
            backup_dir,
            timeout_minutes,
            exclude_regex: None,
            is_active: false,
            created_at: chrono::Local::now().to_rfc3339(),
        }
    }

    /// Cria um perfil baseado em um template
    #[allow(dead_code)]
    pub fn from_template(
        template_id: i64,
        template: &crate::models::GameTemplate,
        backup_dir: String,
        timeout_minutes: u32,
    ) -> Self {
        Self {
            id: 0,
            template_id: Some(template_id),
            name: template.name.clone(),
            save_path: template.expand_save_directory(),
            backup_dir,
            timeout_minutes,
            exclude_regex: template.exclude_regex.clone(),
            is_active: false,
            created_at: chrono::Local::now().to_rfc3339(),
        }
    }
}
