use crate::models::GameProfile;
use crate::watcher::file_watcher::FileWatcher;
use crate::watcher::process_monitor::{ProcessMonitor, ProcessState};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;

/// Handle para controlar um watcher em background
pub struct WatcherHandle {
    #[allow(dead_code)]
    profile_id: i64,
    _handle: thread::JoinHandle<()>,
    _process_monitor_handle: Option<thread::JoinHandle<()>>,
}

impl WatcherHandle {
    #[allow(dead_code)]
    pub fn profile_id(&self) -> i64 {
        self.profile_id
    }
}

/// Inicia o monitoramento de um perfil em background
pub fn start_watching(profile: GameProfile) -> Result<WatcherHandle, Box<dyn std::error::Error>> {
    let profile_id = profile.id;
    let _profile_name = profile.name.clone();
    let _profile_name_for_monitor = _profile_name.clone(); // Clone para usar na segunda closure
    let save_path = PathBuf::from(&profile.save_path);
    let backup_dir = PathBuf::from(&profile.backup_dir);
    let timeout_minutes = profile.timeout_minutes;
    let exclude_regex = profile.exclude_regex.clone();
    let process_name = profile.process_name.clone();

    // Verifica se o save path tem um diretório pai para monitorar
    let watch_dir = save_path
        .parent()
        .ok_or("Save path não tem diretório pai")?
        .to_path_buf();

    // Flag compartilhada: indica se deve monitorar arquivo
    // Se process_name existe, começar desabilitado até processo ser detectado
    let should_monitor = Arc::new(AtomicBool::new(process_name.is_none()));
    let should_monitor_clone = Arc::clone(&should_monitor);

    // Thread de file watching
    let file_watcher_handle = thread::spawn(move || {
        // Cria o FileWatcher
        let mut file_watcher = FileWatcher::new(
            save_path.clone(),
            backup_dir,
            timeout_minutes,
            exclude_regex,
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

        // Monitora o diretório pai do arquivo de save
        if let Err(_e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
            #[cfg(debug_assertions)]
            eprintln!(
                "Erro ao monitorar diretório {:?} para perfil {}: {}",
                watch_dir, profile_id, _e
            );
            return;
        }

        #[cfg(debug_assertions)]
        println!(
            "Monitorando {:?} para perfil {} (ID: {})",
            save_path, _profile_name, profile_id
        );

        // Loop de processamento de eventos
        while let Ok(result) = rx.recv() {
            // Verifica flag: só processa se deve monitorar
            let should_process = should_monitor_clone.load(Ordering::Relaxed);

            if !should_process {
                continue; // Pula processamento enquanto aguarda processo
            }

            match result {
                Ok(Event { paths, .. }) => {
                    // Verifica se algum dos paths é o arquivo de save
                    for path in paths {
                        if path == save_path {
                            // Verifica se deve excluir
                            if file_watcher.should_exclude(&path) {
                                continue;
                            }

                            // Tenta criar backup
                            if file_watcher.should_backup() {
                                #[cfg(debug_assertions)]
                                match file_watcher.create_backup() {
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
                                let _ = file_watcher.create_backup();
                            }
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
    })
}
