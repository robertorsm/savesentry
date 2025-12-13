use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Gerenciador de estado e lógica de backup para um perfil
#[derive(Debug)]
pub struct FileWatcher {
    save_path: PathBuf,
    backup_dir: PathBuf,
    timeout_minutes: u32,
    last_backup: Option<SystemTime>,
    exclude_regex: Option<regex::Regex>,
}

impl FileWatcher {
    /// Cria um novo gerenciador de backup
    pub fn new(
        save_path: PathBuf,
        backup_dir: PathBuf,
        timeout_minutes: u32,
        exclude_regex_str: Option<String>,
    ) -> Self {
        let exclude_regex = exclude_regex_str.and_then(|s| regex::Regex::new(&s).ok());

        Self {
            save_path,
            backup_dir,
            timeout_minutes,
            last_backup: None,
            exclude_regex,
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

    /// Verifica se o timeout expirou desde o último backup
    pub fn should_backup(&self) -> bool {
        match self.last_backup {
            None => true, // Primeiro backup
            Some(last) => {
                let elapsed = SystemTime::now()
                    .duration_since(last)
                    .unwrap_or(Duration::from_secs(0));

                elapsed >= Duration::from_secs(self.timeout_minutes as u64 * 60)
            }
        }
    }

    /// Cria um backup do arquivo de save em formato ZIP
    pub fn create_backup(&mut self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Verifica se deve fazer backup
        if !self.should_backup() {
            return Err("Timeout ainda não expirou".into());
        }

        // Cria diretório de backup se não existir
        fs::create_dir_all(&self.backup_dir)?;

        // Verifica se o arquivo principal deve ser excluído (caso raro, mas possível se o regex for muito abrangente)
        if self.should_exclude(&self.save_path) {
            return Err("Arquivo ignorado pelo filtro de exclusão".into());
        }

        // Gera nome do backup com timestamp pt-BR
        let now = chrono::Local::now();
        let backup_name = format!("backup_{}.zip", now.format("%d-%m-%Y_%H-%M-%S"));
        let backup_path = self.backup_dir.join(&backup_name);

        // Cria arquivo ZIP
        let file = fs::File::create(&backup_path)?;
        let mut zip = zip::ZipWriter::new(file);

        // Adiciona o arquivo de save ao ZIP
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        let save_filename = self
            .save_path
            .file_name()
            .ok_or("Arquivo de save não tem nome")?
            .to_string_lossy();

        zip.start_file(save_filename.as_ref(), options)?;

        let save_content = fs::read(&self.save_path)?;
        zip.write_all(&save_content)?;

        zip.finish()?;

        // Atualiza timestamp do último backup
        self.last_backup = Some(SystemTime::now());

        Ok(backup_path)
    }
}
