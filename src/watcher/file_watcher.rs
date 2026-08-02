use chrono::{Datelike, Timelike};
use image::ImageEncoder;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Gerenciador de estado e lógica de backup para um perfil
#[derive(Debug)]
pub struct FileWatcher {
    save_path: PathBuf,
    backup_dir: PathBuf,
    backup_delay_minutes: u32,
    last_backup: Option<SystemTime>,
    exclude_pattern: Option<glob::Pattern>,
    save_pattern: Option<glob::Pattern>,
    last_backup_time: Arc<AtomicU64>,
    backup_max_count: u32,
    pending: bool,
}

impl FileWatcher {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        save_path: PathBuf,
        backup_dir: PathBuf,
        backup_delay_minutes: u32,
        exclude_pattern_str: Option<String>,
        save_pattern_str: Option<String>,
        last_backup_time: Arc<AtomicU64>,
        backup_max_count: u32,
        initial_last_backup_time: Option<u64>,
    ) -> Self {
        let exclude_pattern = exclude_pattern_str.and_then(|s| glob::Pattern::new(&s).ok());
        let save_pattern = save_pattern_str.and_then(|s| glob::Pattern::new(&s).ok());

        let last_backup = initial_last_backup_time
            .and_then(|t| SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(t)));

        Self {
            save_path,
            backup_dir,
            backup_delay_minutes,
            last_backup,
            exclude_pattern,
            save_pattern,
            last_backup_time,
            backup_max_count,
            pending: false,
        }
    }

    /// Marca que houve modificação pendente de backup
    pub fn set_pending(&mut self, value: bool) {
        self.pending = value;
    }

    /// Verifica se há modificação pendente aguardando backup
    pub fn has_pending(&self) -> bool {
        self.pending
    }

    /// Verifica se um arquivo deve ser excluído do backup
    pub fn should_exclude(&self, path: &std::path::Path) -> bool {
        if let Some(pattern) = &self.exclude_pattern {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                let opts = glob::MatchOptions {
                    case_sensitive: cfg!(not(target_os = "windows")),
                    require_literal_separator: false,
                    require_literal_leading_dot: false,
                };
                return pattern.matches_with(file_name, opts);
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
        custom_name: Option<&str>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if !self.should_backup() {
            return Err("Timeout ainda nao expirou".into());
        }

        let backup_path = Self::create_backup_from_dir(
            &self.save_path,
            &self.backup_dir,
            self.exclude_pattern.as_ref(),
            self.save_pattern.as_ref(),
            custom_name,
            self.backup_max_count,
        )?;

        let now = SystemTime::now();
        self.last_backup = Some(now);
        self.last_backup_time.store(
            now.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            Ordering::Relaxed,
        );

        Ok(backup_path)
    }

    /// Cria um backup ZIP a partir de um diretório de save
    pub fn create_backup_from_dir(
        save_path: &std::path::Path,
        backup_dir: &std::path::Path,
        exclude_pattern: Option<&glob::Pattern>,
        save_pattern: Option<&glob::Pattern>,
        custom_name: Option<&str>,
        backup_max_count: u32,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        fs::create_dir_all(backup_dir)?;

        let now = chrono::Local::now();
        let backup_name = if let Some(name) = custom_name {
            format!("{}.zip", name)
        } else {
            format!("backup_{}.zip", now.format("%d-%m-%Y_%H-%M-%S"))
        };
        let backup_path = backup_dir.join(&backup_name);

        let file = fs::File::create(&backup_path)?;
        let mut zip = zip::ZipWriter::new(file);

        let match_opts = glob::MatchOptions {
            case_sensitive: cfg!(not(target_os = "windows")),
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let mut files_added = 0;

        if let Ok(entries) = fs::read_dir(save_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if !metadata.is_file() {
                        continue;
                    }
                    if let Some(name) = entry.file_name().to_str() {
                        if let Some(pattern) = save_pattern {
                            if !pattern.matches_with(name, match_opts) {
                                continue;
                            }
                        }
                        if let Some(pattern) = exclude_pattern {
                            if pattern.matches_with(name, match_opts) {
                                continue;
                            }
                        }

                        let mut options = zip::write::FileOptions::default()
                            .compression_method(zip::CompressionMethod::Deflated);

                        if let Ok(modified) = metadata.modified() {
                            if let Some(zip_time) = system_time_to_zip_datetime(modified) {
                                options = options.last_modified_time(zip_time);
                            }
                        }

                        let entry_path = entry.path();
                        let relative_path =
                            entry_path.strip_prefix(save_path).unwrap_or(&entry_path);
                        let zip_name = relative_path.to_string_lossy().replace('\\', "/");

                        zip.start_file(&zip_name, options)?;
                        let mut source = fs::File::open(entry.path())?;
                        io::copy(&mut source, &mut zip)?;
                        files_added += 1;
                    }
                }
            }
        }

        zip.finish()?;

        if files_added == 0 {
            let _ = fs::remove_file(&backup_path);
            return Err("Nenhum arquivo encontrado para backup".into());
        }

        if Self::is_auto_backup_name(&backup_name) {
            Self::rotate_backups_with_count(backup_dir, backup_max_count as usize)?;
        }

        Ok(backup_path)
    }

    fn is_auto_backup_name(name: &str) -> bool {
        name.starts_with("backup_") && name.ends_with(".zip")
    }

    fn rotate_backups_with_count(
        backup_dir: &std::path::Path,
        max_backups: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries: Vec<_> = fs::read_dir(backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("zip"))
                    .unwrap_or(false)
            })
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(Self::is_auto_backup_name)
                    .unwrap_or(false)
            })
            .collect();

        if entries.len() <= max_backups {
            return Ok(());
        }

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

        let to_remove = entries.len() - max_backups;
        for entry in entries.iter().take(to_remove) {
            let path = entry.path();
            let _ = fs::remove_file(&path);
            let png_path = path.with_extension("png");
            if png_path.exists() {
                let _ = fs::remove_file(&png_path);
            }
        }

        Ok(())
    }
}

fn system_time_to_zip_datetime(time: SystemTime) -> Option<zip::DateTime> {
    let datetime = chrono::DateTime::<chrono::Local>::from(time);
    let year = datetime.year() as u16;
    let month = datetime.month() as u8;
    let day = datetime.day() as u8;
    let hour = datetime.hour() as u8;
    let minute = datetime.minute() as u8;
    let second = datetime.second() as u8;
    zip::DateTime::from_date_and_time(year, month, day, hour, minute, second).ok()
}

pub fn capture_screenshot(
    backup_dir: &std::path::Path,
    stem: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use screenshots::Screen;

    let screens = Screen::all()?;
    if screens.is_empty() {
        return Err("Nenhum monitor encontrado".into());
    }

    let screen = &screens[0];
    let image_buffer = screen.capture()?;
    let dynamic = image::DynamicImage::ImageRgba8(image_buffer);

    let screenshot_path = backup_dir.join(stem).with_extension("png");

    let file = std::fs::File::create(&screenshot_path)?;
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        file,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::Adaptive,
    );
    encoder.write_image(
        dynamic.as_bytes(),
        dynamic.width(),
        dynamic.height(),
        image::ColorType::Rgba8,
    )?;

    let thumb_path = backup_dir.join(stem).with_extension("thumb.png");
    let thumb = dynamic.resize(320, 180, image::imageops::FilterType::Triangle);
    let _ = thumb.save(&thumb_path);

    Ok(())
}

pub fn cleanup_screenshots_by_stem(backup_dir: &std::path::Path, stem: &str) {
    let png = backup_dir.join(stem).with_extension("png");
    let thumb = backup_dir.join(stem).with_extension("thumb.png");
    if png.exists() {
        let _ = std::fs::remove_file(&png);
    }
    if thumb.exists() {
        let _ = std::fs::remove_file(&thumb);
    }
}

/// Remove screenshots orfaos (sem ZIP correspondente) do diretorio de backup.
/// Chamado no startup do watcher para garantir consistencia apos falhas anteriores.
pub fn cleanup_orphan_screenshots(backup_dir: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(backup_dir) else {
        return;
    };
    let mut zips = std::collections::HashSet::new();
    let mut screenshots = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.eq_ignore_ascii_case("zip") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    zips.insert(stem.to_string());
                }
            } else if ext.eq_ignore_ascii_case("png") {
                screenshots.push(path);
            }
        }
    }
    for path in screenshots {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            // ignora .thumb.png — verifica o stem base
            let base_stem = stem.strip_suffix(".thumb").unwrap_or(stem);
            if !zips.contains(base_stem) {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

pub struct ScreenshotWorker {
    sender: std::sync::mpsc::Sender<(std::path::PathBuf, String, std::time::Duration)>,
}

impl ScreenshotWorker {
    pub fn new(new_screenshot_ready: Arc<AtomicBool>, egui_ctx: eframe::egui::Context) -> Self {
        let (sender, receiver) =
            std::sync::mpsc::channel::<(std::path::PathBuf, String, std::time::Duration)>();
        std::thread::Builder::new()
            .name("screenshot_worker".into())
            .stack_size(512 * 1024)
            .spawn(move || {
                while let Ok((backup_dir, stem, delay)) = receiver.recv() {
                    if delay.as_secs() > 0 {
                        std::thread::sleep(delay);
                    }
                    let _ = capture_screenshot(&backup_dir, &stem);
                    new_screenshot_ready.store(true, Ordering::Relaxed);
                    egui_ctx.request_repaint();
                }
            })
            .unwrap();
        Self { sender }
    }

    pub fn capture(
        &self,
        backup_dir: std::path::PathBuf,
        stem: String,
        delay: std::time::Duration,
    ) {
        let _ = self.sender.send((backup_dir, stem, delay));
    }
}
