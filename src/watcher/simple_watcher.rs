use crate::models::GameProfile;
use crate::watcher::file_watcher::FileWatcher;
use crate::watcher::process_monitor::{ProcessMonitor, ProcessState};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::SystemTime;

/// Handle para controlar um watcher em background
pub struct WatcherHandle {
    #[allow(dead_code)]
    profile_id: i64,
    _handle: thread::JoinHandle<()>,
    _process_monitor_handle: Option<thread::JoinHandle<()>>,
    last_backup_time: Arc<AtomicU64>,
}

impl WatcherHandle {
    #[allow(dead_code)]
    pub fn profile_id(&self) -> i64 {
        self.profile_id
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
pub fn start_watching(profile: GameProfile) -> Result<WatcherHandle, Box<dyn std::error::Error>> {
    let profile_id = profile.id;
    let _profile_name = profile.name.clone();
    let _profile_name_for_monitor = _profile_name.clone(); // Clone para usar na segunda closure
    let save_path = PathBuf::from(&profile.save_path);
    let backup_dir = PathBuf::from(&profile.backup_dir);
    let backup_delay_minutes = profile.backup_delay_minutes;
    let exclude_regex = profile.exclude_regex.clone();
    let save_pattern = profile.save_pattern.clone();
    let process_name = profile.process_name.clone();

    // Flag compartilhada: indica se deve monitorar arquivo
    // Se process_name existe, começar desabilitado até processo ser detectado
    let should_monitor = Arc::new(AtomicBool::new(process_name.is_none()));
    let should_monitor_clone = Arc::clone(&should_monitor);

    let last_backup_time = Arc::new(AtomicU64::new(0));
    let last_backup_time_clone = Arc::clone(&last_backup_time);

    // Thread de file watching
    let file_watcher_handle = thread::spawn(move || {
        // Cria o FileWatcher
        let mut file_watcher = FileWatcher::new(
            save_path.clone(),
            backup_dir,
            backup_delay_minutes,
            exclude_regex,
            save_pattern,
            last_backup_time_clone,
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

        if let Err(_e) = watcher.watch(&save_path, RecursiveMode::Recursive) {
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

        while let Ok(result) = rx.recv() {
            let should_process = should_monitor_clone.load(Ordering::Relaxed);
            if !should_process {
                continue;
            }

            match result {
                Ok(Event { paths, .. }) => {
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

                        if file_watcher.should_backup() {
                            #[cfg(debug_assertions)]
                            match file_watcher.create_backup(&path) {
                                Ok(backup_path) => {
                                    println!(
                                        "✅ Backup criado: {:?} (Perfil: {})",
                                        backup_path, _profile_name
                                    );
                                }
                                Err(e) => {
                                    eprintln!(
                                        "❌ Erro ao criar backup para {}: {}",
                                        _profile_name, e
                                    );
                                }
                            }

                            #[cfg(not(debug_assertions))]
                            let _ = file_watcher.create_backup(&path);
                        }
                    }
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Erro no watcher do perfil {}: {}", profile_id, _e);
                }
            }
        }

        #[cfg(debug_assertions)]
        println!(
            "Watcher encerrado para perfil {} (ID: {})",
            _profile_name, profile_id
        );
    });

    // Thread de monitoramento de processo (opcional)
    let process_monitor_handle = if let Some(proc_name) = process_name {
        let should_monitor_clone = Arc::clone(&should_monitor);

        Some(thread::spawn(move || {
            let mut monitor = ProcessMonitor::new(proc_name.clone());

            #[cfg(debug_assertions)]
            println!(
                "🔍 Aguardando processo {} para perfil {}...",
                proc_name, _profile_name_for_monitor
            );

            loop {
                let state = monitor.check_process();
                let poll_interval = monitor.get_poll_interval();

                // Atualiza flag de monitoramento baseado no estado do processo
                match state {
                    ProcessState::Running => {
                        #[cfg(debug_assertions)]
                        println!(
                            "🎮 Processo {} detectado! Iniciando monitoramento de arquivo",
                            proc_name
                        );

                        should_monitor_clone.store(true, Ordering::Relaxed);
                    }
                    ProcessState::Stopped => {
                        #[cfg(debug_assertions)]
                        println!("⛔ Processo {} fechado. Pausando monitoramento", proc_name);

                        should_monitor_clone.store(false, Ordering::Relaxed);
                    }
                    ProcessState::Waiting => {
                        // Continue esperando
                    }
                }

                thread::sleep(poll_interval);
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
    })
}
