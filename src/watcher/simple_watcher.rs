use crate::models::GameProfile;
use crate::watcher::file_watcher::FileWatcher;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::SystemTime;

#[cfg(windows)]
mod winapi {
    use std::ffi::c_void;

    pub const SYNCHRONIZE: u32 = 0x00100000;
    pub const INFINITE: u32 = 0xFFFFFFFF;


    extern "system" {
        pub fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut c_void;
        pub fn WaitForSingleObject(hHandle: *mut c_void, dwMilliseconds: u32) -> u32;
        pub fn CloseHandle(hObject: *mut c_void) -> i32;
    }
}

/// Handle para controlar um watcher em background
pub struct WatcherHandle {
    #[allow(dead_code)]
    profile_id: i64,
    _handle: thread::JoinHandle<()>,
    _process_monitor_handle: Option<thread::JoinHandle<()>>,
    last_backup_time: Arc<AtomicU64>,
    pub recent_save: Arc<Mutex<Option<(String, SystemTime)>>>,
    pub process_running: Arc<AtomicBool>,
}

impl WatcherHandle {
    #[allow(dead_code)]
    pub fn profile_id(&self) -> i64 {
        self.profile_id
    }

    pub fn last_backup_time(&self) -> u64 {
        self.last_backup_time.load(Ordering::Relaxed)
    }

    pub fn remaining_backup_seconds(&self, delay_minutes: u32) -> Option<u64> {
        let last = self.last_backup_time.load(Ordering::Relaxed);
        if last == 0 {
            return None;
        }
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let elapsed = now.saturating_sub(last);
        let delay = delay_minutes as u64 * 60;
        if elapsed >= delay {
            Some(0)
        } else {
            Some(delay - elapsed)
        }
    }
}

/// Inicia o monitoramento de um perfil em background
pub fn start_watching(
    profile: GameProfile,
    ctx: eframe::egui::Context,
) -> Result<WatcherHandle, Box<dyn std::error::Error>> {
    let profile_id = profile.id;
    let _profile_name = profile.name.clone();
    let _profile_name_for_monitor = _profile_name.clone(); // Clone para usar na segunda closure
    let save_path = PathBuf::from(&profile.save_path);
    let backup_dir = PathBuf::from(&profile.backup_dir);
    let backup_delay_minutes = profile.backup_delay_minutes;
    let exclude_pattern = profile.exclude_pattern.clone();
    let save_pattern = profile.save_pattern.clone();
    let process_name = profile.process_name.clone();
    let ctx_clone = ctx.clone();

    // Flag compartilhada: indica se deve monitorar arquivo
    // Se process_name existe, começar desabilitado até processo ser detectado
    let should_monitor = Arc::new(AtomicBool::new(process_name.is_none()));
    let should_monitor_clone = Arc::clone(&should_monitor);

    let last_backup_time = Arc::new(AtomicU64::new(0));
    let last_backup_time_clone = Arc::clone(&last_backup_time);

    let recent_save = Arc::new(Mutex::new(None));
    let recent_save_clone = Arc::clone(&recent_save);

    let process_running = Arc::new(AtomicBool::new(true));
    let process_running_clone = Arc::clone(&process_running);

    // Thread de file watching
    let file_watcher_handle = thread::spawn(move || {
        // Cria o FileWatcher
        let mut file_watcher = FileWatcher::new(
            save_path.clone(),
            backup_dir.clone(),
            backup_delay_minutes,
            exclude_pattern,
            save_pattern,
            last_backup_time_clone,
            profile.backup_max_count,
            profile.backup_recursive,
        );

        // Cria canal para receber eventos do notify
        let (tx, rx) = mpsc::channel();

        // Cria watcher do sistema de arquivos
        let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
            Ok(w) => w,
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("Erro ao criar watcher para perfil {}: {}", profile_id, _e);
                return;
            }
        };

        if let Err(_e) = watcher.watch(&save_path, RecursiveMode::NonRecursive) {
            #[cfg(debug_assertions)]
            eprintln!(
                "Erro ao monitorar diretório {:?} para perfil {}: {}",
                save_path, profile_id, _e
            );
            return;
        }

        #[cfg(debug_assertions)]
        println!(
            "Monitorando {:?} para perfil {} (ID: {})",
            save_path, _profile_name, profile_id
        );

        let debounce_duration = std::time::Duration::from_secs(3);
        let mut deadline: Option<std::time::Instant> = None;
        let mut last_backup_path: Option<std::path::PathBuf> = None;

        loop {
            let should_process = should_monitor_clone.load(Ordering::Relaxed);

            let recv_result = if let Some(d) = deadline {
                let now = std::time::Instant::now();
                if d <= now {
                    // Deadline expirou: dispara backup apenas se houve modificação
                    if should_process && file_watcher.has_pending() && file_watcher.should_backup()
                    {
                        #[cfg(debug_assertions)]
                        match file_watcher.create_backup(&save_path) {
                            Ok(backup_path) => {
                                println!(
                                    "✅ Backup criado: {:?} (Perfil: {})",
                                    backup_path, _profile_name
                                );
                                last_backup_path = Some(backup_path);
                                ctx_clone.request_repaint();
                            }
                            Err(e) => {
                                eprintln!("❌ Erro ao criar backup para {}: {}", _profile_name, e);
                            }
                        }

                        #[cfg(not(debug_assertions))]
                        if let Ok(backup_path) = file_watcher.create_backup(&save_path) {
                            last_backup_path = Some(backup_path);
                            ctx_clone.request_repaint();
                        }
                        file_watcher.set_pending(false);
                    }
                    deadline = None;
                    continue;
                }
                let timeout = d.duration_since(now);
                rx.recv_timeout(timeout)
            } else {
                rx.recv()
                    .map_err(|_| std::sync::mpsc::RecvTimeoutError::Disconnected)
            };

            match recv_result {
                Ok(result) => {
                    if !should_process {
                        continue;
                    }

                    match result {
                        Ok(Event { kind, paths, .. }) => {
                            match kind {
                                EventKind::Create(_) | EventKind::Modify(_) => {}
                                _ => continue,
                            }

                            let mut has_relevant_event = false;
                            for path in paths {
                                if !path.is_file() {
                                    continue;
                                }
                                if !file_watcher.matches_pattern(&path) {
                                    continue;
                                }
                                if file_watcher.should_exclude(&path) {
                                    continue;
                                }
                                has_relevant_event = true;

                                if let Ok(metadata) = std::fs::metadata(&path) {
                                    if let Ok(modified) = metadata.modified() {
                                        let mut recent = recent_save_clone.lock().unwrap();
                                        *recent =
                                            Some((path.to_string_lossy().to_string(), modified));
                                    }
                                }
                            }

                            if has_relevant_event {
                                // Reseta deadline (sliding debounce)
                                deadline = Some(std::time::Instant::now() + debounce_duration);
                                file_watcher.set_pending(true);
                            }
                        }
                        Err(_e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("Erro no watcher do perfil {}: {}", profile_id, _e);
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout expirou: dispara backup apenas se houve modificação pendente
                    if should_process && file_watcher.has_pending() && file_watcher.should_backup()
                    {
                        #[cfg(debug_assertions)]
                        match file_watcher.create_backup(&save_path) {
                            Ok(backup_path) => {
                                println!(
                                    "✅ Backup criado (debounce): {:?} (Perfil: {})",
                                    backup_path, _profile_name
                                );
                                last_backup_path = Some(backup_path);
                                ctx_clone.request_repaint();
                            }
                            Err(e) => {
                                eprintln!("❌ Erro ao criar backup para {}: {}", _profile_name, e);
                            }
                        }

                        #[cfg(not(debug_assertions))]
                        if let Ok(backup_path) = file_watcher.create_backup(&save_path) {
                            last_backup_path = Some(backup_path);
                            ctx_clone.request_repaint();
                        }
                        file_watcher.set_pending(false);
                    }
                    deadline = None;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }

            // Tenta capturar screenshot após backup (não bloqueante)
            if let Some(ref path) = last_backup_path {
                let _ = crate::watcher::file_watcher::capture_screenshot(path);
            }
        }

        #[cfg(debug_assertions)]
        println!(
            "Watcher encerrado para perfil {} (ID: {})",
            _profile_name, profile_id
        );
    });

    let process_monitor_handle = if let Some(proc_name) = process_name {
        let should_monitor_clone = Arc::clone(&should_monitor);

        Some(thread::spawn(move || {
            let proc_name_lower = proc_name.to_lowercase();

            #[cfg(debug_assertions)]
            println!(
                "🔍 Aguardando processo {} para perfil {}...",
                proc_name, _profile_name_for_monitor
            );

            loop {
                use sysinfo::{ProcessesToUpdate, System};

                let mut system = System::new();
                system.refresh_processes(ProcessesToUpdate::All, true);

                let found = system.processes().values().find(|p| {
                    p.name().to_string_lossy().to_lowercase() == proc_name_lower
                });

                if let Some(process) = found {
                    let pid = process.pid().as_u32();

                    #[cfg(debug_assertions)]
                    println!(
                        "🎮 Processo {} detectado (PID {}). Iniciando monitoramento de arquivo",
                        proc_name, pid
                    );

                    should_monitor_clone.store(true, Ordering::Relaxed);

                    #[cfg(windows)]
                    unsafe {
                        let handle = winapi::OpenProcess(winapi::SYNCHRONIZE, 0, pid);
                        if !handle.is_null() {
                            winapi::WaitForSingleObject(handle, winapi::INFINITE);
                            winapi::CloseHandle(handle);
                        }
                    }

                    #[cfg(not(windows))]
                    {
                        loop {
                            thread::sleep(std::time::Duration::from_secs(5));
                            let mut sys = sysinfo::System::new();
                            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                            if !sys.processes().values().any(|p| {
                                p.name().to_string_lossy().to_lowercase() == proc_name_lower
                            }) {
                                break;
                            }
                        }
                    }

                    #[cfg(debug_assertions)]
                    println!("⛔ Processo {} fechado. Parando monitoramento", proc_name);

                    should_monitor_clone.store(false, Ordering::Relaxed);
                    process_running_clone.store(false, Ordering::Relaxed);
                    ctx.request_repaint();
                }

                thread::sleep(std::time::Duration::from_secs(2));
            }
        }))
    } else {
        None
    };

    Ok(WatcherHandle {
        profile_id,
        _handle: file_watcher_handle,
        _process_monitor_handle: process_monitor_handle,
        last_backup_time,
        recent_save,
        process_running,
    })
}
