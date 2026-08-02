/// Perfil de um jogo em runtime (não persistido — construído a partir do template)
#[derive(Debug, Clone)]
pub struct GameProfile {
    pub id: i64,
    pub template_id: Option<i64>,
    pub name: String,
    pub save_path: String,
    pub backup_dir: String,
    pub backup_delay_minutes: u32,
    pub screenshot_delay_seconds: u32,
    pub exclude_pattern: Option<String>,
    pub save_pattern: Option<String>,
    pub is_active: bool,
    pub process_name: Option<String>,
    pub backup_max_count: u32,
}
