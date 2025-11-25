/// Mensagens que a aplicação pode processar
#[derive(Debug, Clone)]
pub enum Message {
    // Templates
    // LoadTemplates, // Reservado para futuro sync via API
    // TemplatesLoaded(Vec<crate::models::GameTemplate>), // Reservado para futuro sync via API
    SelectTemplate(i64),
    ClearTemplate,

    // Formulário de perfil
    GameNameChanged(String),
    TimeoutChanged(String),
    SelectSaveFile,
    SaveFileSelected(Option<std::path::PathBuf>),
    SelectBackupDir,
    BackupDirSelected(Option<std::path::PathBuf>),
    CreateProfile,

    // Gerenciamento de perfis
    ToggleMonitoring(i64),
    DeleteProfile(i64),

    // Eventos de backup
    BackupCreated(i64, String), // profile_id, backup_path
}
