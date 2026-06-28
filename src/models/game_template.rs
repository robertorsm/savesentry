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

        for (var, key) in [
            ("APPDATA", "%APPDATA%"),
            ("USERPROFILE", "%USERPROFILE%"),
            ("LOCALAPPDATA", "%LOCALAPPDATA%"),
        ] {
            if let Ok(value) = std::env::var(var) {
                path = path.replace(key, &value);
            }
        }

        path
    }
}
