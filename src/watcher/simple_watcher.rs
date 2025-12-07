use crate::models::GameProfile;
use crate::watcher::file_watcher::FileWatcher;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

/// Handle para controlar um watcher em background
pub struct WatcherHandle {
    profile_id: i64,
    _handle: thread::JoinHandle<()>,
}

impl WatcherHandle {
    pub fn profile_id(&self) -> i64 {
        self.profile_id
    }
}

/// Inicia o monitoramento de um perfil em background
pub fn start_watching(profile: GameProfile) -> Result<WatcherHandle, Box<dyn std::error::Error>> {
    let profile_id = profile.id;
    let save_path = PathBuf::from(&profile.save_path);
    let backup_dir = PathBuf::from(&profile.backup_dir);
    let timeout_minutes = profile.timeout_minutes;
    let exclude_regex = profile.exclude_regex.clone();

    // Verifica se o save path tem um diretório pai para monitorar
    let watch_dir = save_path
        .parent()
        .ok_or("Save path não tem diretório pai")?
        .to_path_buf();

    let handle = thread::spawn(move || {
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
            Err(e) => {
                eprintln!("Erro ao criar watcher para perfil {}: {}", profile_id, e);
                return;
            }
        };

        // Monitora o diretório pai do arquivo de save
        if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
            eprintln!(
                "Erro ao monitorar diretório {:?} para perfil {}: {}",
                watch_dir, profile_id, e
            );
            return;
        }

        println!(
            "Monitorando {:?} para perfil {} (ID: {})",
            save_path,
            profile.name,
            profile_id
        );

        // Loop de processamento de eventos
        while let Ok(result) = rx.recv() {
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
                                match file_watcher.create_backup() {
                                    Ok(backup_path) => {
                                        println!(
                                            "✅ Backup criado: {:?} (Perfil: {})",
                                            backup_path, profile.name
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "❌ Erro ao criar backup para {}: {}",
                                            profile.name, e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Erro no watcher do perfil {}: {}", profile_id, e);
                }
            }
        }

        println!("Watcher encerrado para perfil {} (ID: {})", profile.name, profile_id);
    });

    Ok(WatcherHandle {
        profile_id,
        _handle: handle,
    })
}

