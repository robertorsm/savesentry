/// Template de jogo pré-configurado
#[derive(Debug, Clone)]
pub struct GameTemplate {
    pub id: i64,
    pub name: String,
    pub save_directory: String, // Pode conter variáveis como %APPDATA%
    pub process_name: String,   // Nome do processo do jogo
    pub save_pattern: String,   // Padrão de arquivos (ex: *.sav)
    pub exclude_pattern: Option<String>, // Padrão glob para excluir arquivos (ex: *.tmp)
    pub default_exclude_pattern: Option<String>, // Padrão built-in de exclusão automático
    pub backup_dir: String,     // Diretório de backup para este template
    pub backup_delay_minutes: u32, // Intervalo mínimo entre backups (em minutos)
    pub backup_max_count: u32,     // Número máximo de backups a manter
    pub version: i32,
    pub is_official: bool, // Template oficial ou customizado
    pub created_at: String,
}

/// Base constante para conversão AccountID → SteamID64
pub const STEAMID64_BASE: u64 = 76561197960265728;

use std::sync::Mutex;

static STEAM_USERDATA_CACHE: Mutex<Option<String>> = Mutex::new(None);
static STEAM_ID_CACHE: Mutex<Option<String>> = Mutex::new(None);

impl GameTemplate {
    fn read_registry_steam_path() -> Option<String> {
        let output = std::process::Command::new("reg")
            .args(["query", "HKCU\\Software\\Valve\\Steam", "/v", "SteamPath"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("SteamPath") && line.contains("REG_SZ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let trimmed = parts[2..].join(" ");
                    if !trimmed.is_empty() {
                        let userdata = std::path::Path::new(&trimmed).join("userdata");
                        if userdata.is_dir() {
                            #[cfg(debug_assertions)]
                            println!("SteamPath do registry: {}", userdata.display());
                            return Some(userdata.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn detect_steam_userdata() -> Option<String> {
        {
            let cache = STEAM_USERDATA_CACHE.lock().unwrap();
            if let Some(ref path) = *cache {
                return Some(path.clone());
            }
        }

        let result = if let Some(path) = Self::read_registry_steam_path() {
            Some(path)
        } else {
            let possible_paths = [
                std::env::var("LOCALAPPDATA")
                    .ok()
                    .map(|p| std::path::Path::new(&p).join("Steam").join("userdata")),
                std::env::var("ProgramFiles(x86)")
                    .ok()
                    .map(|p| std::path::Path::new(&p).join("Steam").join("userdata")),
                std::env::var("ProgramFiles")
                    .ok()
                    .map(|p| std::path::Path::new(&p).join("Steam").join("userdata")),
                std::env::var("USERPROFILE")
                    .ok()
                    .map(|p| std::path::Path::new(&p).join("Steam").join("userdata")),
            ];

            let mut found = None;
            for path in possible_paths.iter().flatten() {
                if path.is_dir() {
                    #[cfg(debug_assertions)]
                    println!("Steam userdata detectado em: {}", path.display());
                    found = Some(path.to_string_lossy().to_string());
                    break;
                }
            }

            if found.is_none() {
                #[cfg(debug_assertions)]
                eprintln!(
                    "⚠️ Não foi possível detectar Steam userdata em nenhum caminho conhecido"
                );
            }
            found
        };

        if let Some(ref path) = result {
            let mut cache = STEAM_USERDATA_CACHE.lock().unwrap();
            *cache = Some(path.clone());
        }
        result
    }

    fn steamid64_from_account_id(account_id: u32) -> String {
        let steam64 = (account_id as u64) + STEAMID64_BASE;
        steam64.to_string()
    }

    fn read_registry_active_user() -> Option<u32> {
        let output = std::process::Command::new("reg")
            .args([
                "query",
                "HKCU\\Software\\Valve\\Steam\\ActiveProcess",
                "/v",
                "ActiveUser",
            ])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("ActiveUser") && line.contains("REG_DWORD") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(hex_value) = parts.last() {
                    let clean_hex = hex_value.trim_start_matches("0x");
                    return u32::from_str_radix(clean_hex, 16).ok();
                }
            }
        }
        None
    }

    fn detect_steam_id() -> Option<String> {
        {
            let cache = STEAM_ID_CACHE.lock().unwrap();
            if let Some(ref id) = *cache {
                return Some(id.clone());
            }
        }

        let result = if let Some(account_id) = Self::read_registry_active_user() {
            let steam64 = Self::steamid64_from_account_id(account_id);
            #[cfg(debug_assertions)]
            println!("SteamID64 do registry (ActiveUser): {}", steam64);
            Some(steam64)
        } else {
            let userdata = Self::detect_steam_userdata()?;
            let entries = std::fs::read_dir(&userdata).ok()?;

            let mut candidates: Vec<(String, std::time::SystemTime)> = Vec::new();
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.chars().all(|c| c.is_ascii_digit()) {
                                let modified = metadata
                                    .modified()
                                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                                candidates.push((name.to_string(), modified));
                            }
                        }
                    }
                }
            }

            candidates.sort_by_key(|b| std::cmp::Reverse(b.1));
            candidates.first().map(|(id, _)| {
                let steam64 = Self::steamid64_from_account_id(id.parse::<u32>().unwrap_or(0));
                #[cfg(debug_assertions)]
                println!("SteamID64 do filesystem (fallback): {}", steam64);
                steam64
            })
        };

        if let Some(ref id) = result {
            let mut cache = STEAM_ID_CACHE.lock().unwrap();
            *cache = Some(id.clone());
        }
        result
    }

    fn expand_path(path: &str) -> String {
        let mut expanded = path.to_string();

        let env_vars = [
            ("APPDATA", "%APPDATA%"),
            ("LOCALAPPDATA", "%LOCALAPPDATA%"),
            ("USERPROFILE", "%USERPROFILE%"),
            ("USERNAME", "%USERNAME%"),
            ("HOMEDRIVE", "%HOMEDRIVE%"),
            ("HOMEPATH", "%HOMEPATH%"),
            ("PROGRAMDATA", "%PROGRAMDATA%"),
            ("PUBLIC", "%PUBLIC%"),
            ("TEMP", "%TEMP%"),
            ("TMP", "%TMP%"),
            ("PROGRAMFILES", "%PROGRAMFILES%"),
            ("PROGRAMFILES(X86)", "%PROGRAMFILES(X86)%"),
        ];

        for (var, key) in env_vars {
            if let Ok(value) = std::env::var(var) {
                expanded = expanded.replace(key, &value);
            }
        }

        if let Some(steam_userdata) = Self::detect_steam_userdata() {
            expanded = expanded.replace("%STEAM_USERDATA%", &steam_userdata);
        }
        if let Some(steam_id) = Self::detect_steam_id() {
            expanded = expanded.replace("%STEAMID%", &steam_id);
        }

        expanded
    }

    pub fn expand_save_directory(&self) -> String {
        Self::expand_path(&self.save_directory)
    }

    pub fn expand_backup_directory(&self) -> String {
        Self::expand_path(&self.backup_dir)
    }
}
