use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Gerenciador de estado e lógica de backup para um perfil
#[derive(Debug)]
pub struct FileWatcher {
    #[allow(dead_code)]
    save_path: PathBuf,
    backup_dir: PathBuf,
    backup_delay_minutes: u32,
    last_backup: Option<SystemTime>,
    exclude_regex: Option<regex::Regex>,
    save_pattern: Option<glob::Pattern>,
    last_backup_time: Arc<AtomicU64>,
}

impl FileWatcher {
    pub fn new(
        save_path: PathBuf,
        backup_dir: PathBuf,
        backup_delay_minutes: u32,
        exclude_regex_str: Option<String>,
        save_pattern_str: Option<String>,
        last_backup_time: Arc<AtomicU64>,
    ) -> Self {
        let exclude_regex = exclude_regex_str.and_then(|s| regex::Regex::new(&s).ok());
        let save_pattern = save_pattern_str.and_then(|s| glob::Pattern::new(&s).ok());

        Self {
            save_path,
            backup_dir,
            backup_delay_minutes,
            last_backup: None,
            exclude_regex,
            save_pattern,
            last_backup_time,
        }
    }

    /// Verifica se um arquivo deve ser excluído do backup
    pub fn should_exclude(&self, path: &std::path::Path) -> bool {
        if let Some(regex) = &self.exclude_regex {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                return regex.is_match(file_name);
            }
        }
        false
    }

    pub fn matches_pattern(&self, path: &std::path::Path) -> bool {
        if let Some(pattern) = &self.save_pattern {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let opts = glob::MatchOptions {
                    case_sensitive: cfg!(not(target_os = "windows")),
                    require_literal_separator: false,
                    require_literal_leading_dot: false,
                };
                return pattern.matches_with(file_name, opts);
            }
            return false;
        }
        true
    }

    /// Verifica se o timeout expirou desde o último backup
    pub fn should_backup(&self) -> bool {
        match self.last_backup {
            None => true, // Primeiro backup
            Some(last) => {
                let elapsed = SystemTime::now()
                    .duration_since(last)
                    .unwrap_or(Duration::from_secs(0));

                elapsed >= Duration::from_secs(self.backup_delay_minutes as u64 * 60)
            }
        }
    }

    pub fn create_backup(
        &mut self,
        _source_path: &std::path::Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if !self.should_backup() {
            return Err("Timeout ainda nao expirou".into());
        }

        fs::create_dir_all(&self.backup_dir)?;

        let now = chrono::Local::now();
        let backup_name = format!("backup_{}.zip", now.format("%d-%m-%Y_%H-%M-%S"));
        let backup_path = self.backup_dir.join(&backup_name);

        let file = fs::File::create(&backup_path)?;
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let save_dir = std::path::Path::new(&self.save_path);
        let match_opts = glob::MatchOptions {
            case_sensitive: cfg!(not(target_os = "windows")),
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let mut files_added = 0;
        if let Ok(entries) = fs::read_dir(save_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if let Some(pattern) = &self.save_pattern {
                                if !pattern.matches_with(name, match_opts) {
                                    continue;
                                }
                            }
                            if self.should_exclude(&entry.path()) {
                                continue;
                            }

                            zip.start_file(name, options)?;
                            let mut source = fs::File::open(entry.path())?;
                            io::copy(&mut source, &mut zip)?;
                            files_added += 1;
                        }
                    }
                }
            }
        }

        zip.finish()?;

        if files_added == 0 {
            let _ = fs::remove_file(&backup_path);
            return Err("Nenhum arquivo encontrado para backup".into());
        }

        let now = SystemTime::now();
        self.last_backup = Some(now);
        self.last_backup_time.store(
            now.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            Ordering::Relaxed,
        );
        self.rotate_backups()?;

        Ok(backup_path)
    }

    /// Remove backups antigos mantendo apenas os 50 mais recentes
    fn rotate_backups(&self) -> Result<(), Box<dyn std::error::Error>> {
        const MAX_BACKUPS: usize = 50;

        let mut entries: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("zip"))
                    .unwrap_or(false)
            })
            .collect();

        if entries.len() <= MAX_BACKUPS {
            return Ok(());
        }

        // Ordena por data de modificação (mais antigo primeiro)
        entries.sort_by(|a, b| {
            let time_a = a
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let time_b = b
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            time_a.cmp(&time_b)
        });

        // Remove os mais antigos
        let to_remove = entries.len() - MAX_BACKUPS;
        for entry in entries.iter().take(to_remove) {
            let _ = fs::remove_file(entry.path());
        }

        Ok(())
    }
}
