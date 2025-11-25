use serde::{Deserialize, Serialize};

/// Template de jogo pré-configurado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTemplate {
    pub id: i64,
    pub name: String,
    pub save_directory: String, // Pode conter variáveis como %APPDATA%
    pub process_name: String,   // Nome do processo do jogo
    pub save_pattern: String,   // Padrão de arquivos (ex: *.sav)
    pub exclude_regex: Option<String>, // Regex para excluir arquivos
    pub version: i32,
    pub is_official: bool, // Template oficial ou customizado
    pub created_at: String,
}

impl GameTemplate {
    /// Expande variáveis de ambiente no caminho
    pub fn expand_save_directory(&self) -> String {
        let mut path = self.save_directory.clone();

        // Expande %APPDATA%
        if let Ok(appdata) = std::env::var("APPDATA") {
            path = path.replace("%APPDATA%", &appdata);
        }

        // Expande %USERPROFILE%
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            path = path.replace("%USERPROFILE%", &userprofile);
        }

        // Expande %LOCALAPPDATA%
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            path = path.replace("%LOCALAPPDATA%", &localappdata);
        }

        path
    }
}
