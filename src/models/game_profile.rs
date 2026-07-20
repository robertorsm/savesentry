/// Perfil de um jogo para monitoramento e backup
#[derive(Debug, Clone)]
pub struct GameProfile {
    pub id: i64,
    pub template_id: Option<i64>, // FK para GameTemplate (se baseado em template)
    pub name: String,             // Nome do jogo
    pub save_path: String,        // Caminho do arquivo de save
    pub backup_dir: String,       // Diretório onde backups serão salvos
    pub backup_delay_minutes: u32, // Intervalo mínimo entre backups (em minutos)
    pub exclude_pattern: Option<String>, // Padrão glob de exclusão (pode sobrescrever template)
    pub save_pattern: Option<String>, // Padrão glob de inclusão (ex: *.sav)
    pub is_active: bool,          // Se o monitoramento está ativo
    pub process_name: Option<String>, // Nome do processo para monitoramento (ex: game.exe)
    pub created_at: String,       // Data de criação do perfil
    pub backup_max_count: u32,    // Máximo de backups a manter (default: 50)
    pub backup_recursive: bool,   // Se faz backup recursivo de subdiretórios
}

impl GameProfile {
    /// Cria um novo perfil de jogo
    #[allow(dead_code)]
    pub fn new(
        name: String,
        save_path: String,
        backup_dir: String,
        backup_delay_minutes: u32,
    ) -> Self {
        Self {
            id: 0,
            template_id: None,
            name,
            save_path,
            backup_dir,
            backup_delay_minutes,
            exclude_pattern: None,
            save_pattern: None,
            is_active: false,
            process_name: None,
            created_at: chrono::Local::now().to_rfc3339(),
            backup_max_count: 50,
            backup_recursive: false,
        }
    }

    /// Cria um perfil baseado em um template
    #[allow(dead_code)]
    pub fn from_template(
        template_id: i64,
        template: &crate::models::GameTemplate,
        backup_dir: String,
        backup_delay_minutes: u32,
    ) -> Self {
        Self {
            id: 0,
            template_id: Some(template_id),
            name: template.name.clone(),
            save_path: template.expand_save_directory(),
            backup_dir,
            backup_delay_minutes,
            exclude_pattern: template.exclude_pattern.clone(),
            save_pattern: Some(template.save_pattern.clone()),
            is_active: false,
            process_name: Some(template.process_name.clone()),
            created_at: chrono::Local::now().to_rfc3339(),
            backup_max_count: template.backup_max_count,
            backup_recursive: false,
        }
    }
}
