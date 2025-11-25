// Subscription for file watching integrated with Iced
use iced::futures::Stream;
use iced::Subscription;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::models::GameProfile;
use crate::ui::Message;
use crate::watcher::file_watcher::FileWatcher;

/// Starts a subscription that watches the save files of active game profiles.
pub fn watch(profiles: Vec<GameProfile>) -> Subscription<Message> {
    Subscription::run_with_id(std::any::TypeId::of::<FileWatcher>(), stream(profiles))
}

fn stream(profiles: Vec<GameProfile>) -> impl Stream<Item = Message> {
    iced::futures::stream::unfold(
        (
            profiles,
            None::<mpsc::UnboundedReceiver<notify::Result<Event>>>,
            HashMap::<PathBuf, (i64, FileWatcher)>::new(),
        ),
        |state| async move {
            let (profiles, mut rx_opt, mut file_watchers) = state;

            // Initialization – create the async channel and spawn the blocking watcher thread
            if rx_opt.is_none() {
                let (tx, rx) = mpsc::unbounded_channel();
                let profiles_clone = profiles.clone();

                // Spawn a thread that uses the blocking `notify` crate and forwards events
                std::thread::spawn(move || {
                    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
                    let mut watcher = match RecommendedWatcher::new(notify_tx, Config::default()) {
                        Ok(w) => w,
                        Err(e) => {
                            eprintln!("Erro ao criar watcher: {}", e);
                            return;
                        }
                    };

                    // Watch the parent directory of each active profile's save file
                    for profile in &profiles_clone {
                        if profile.is_active {
                            let save_path = PathBuf::from(&profile.save_path);
                            if let Some(parent) = save_path.parent() {
                                let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
                            }
                        }
                    }

                    // Forward events to the async channel
                    while let Ok(event) = notify_rx.recv() {
                        if tx.send(event).is_err() {
                            break;
                        }
                    }
                });

                // Populate FileWatcher map for each active profile
                for profile in &profiles {
                    if profile.is_active {
                        let save_path = PathBuf::from(&profile.save_path);
                        let backup_dir = PathBuf::from(&profile.backup_dir);
                        let fw = FileWatcher::new(
                            save_path.clone(),
                            backup_dir,
                            profile.timeout_minutes,
                            profile.exclude_regex.clone(),
                        );
                        file_watchers.insert(save_path, (profile.id, fw));
                    }
                }

                rx_opt = Some(rx);
            }

            let mut rx = rx_opt.unwrap();

            // Process incoming file system events
            while let Some(res) = rx.recv().await {
                match res {
                    Ok(event) => {
                        for path in event.paths {
                            if let Some((id, fw)) = file_watchers.get_mut(&path) {
                                // Skip excluded files
                                if fw.should_exclude(&path) {
                                    continue;
                                }
                                if fw.should_backup() {
                                    if let Ok(backup_path) = fw.create_backup() {
                                        return Some((
                                            Message::BackupCreated(
                                                *id,
                                                backup_path.to_string_lossy().to_string(),
                                            ),
                                            (profiles, Some(rx), file_watchers),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Erro no watcher: {}", e),
                }
            }

            // Channel closed – end the subscription
            None
        },
    )
}
