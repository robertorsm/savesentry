use crate::models::{GameProfile, GameTemplate};
use crate::ui::Message;
use iced::{Subscription, Task, Theme};
use std::path::PathBuf;

/// Aplicação principal
pub struct App {
    // Banco de dados
    pub(super) db: crate::db::Database,

    // Templates disponíveis
    pub(super) templates: Vec<GameTemplate>,
    pub(super) selected_template: Option<GameTemplate>,

    // Perfis de jogos
    pub(super) profiles: Vec<GameProfile>,

    // Formulário de novo perfil
    pub(super) form_game_name: String,
    pub(super) form_save_path: Option<PathBuf>,
    pub(super) form_backup_dir: Option<PathBuf>,
    pub(super) form_timeout: String,
}

impl App {
    /// Cria uma nova instância da aplicação
    pub fn new() -> (Self, Task<Message>) {
        // Inicializa banco de dados
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let db_path = exe_dir.join("sgw.db");

        // Se o banco não existir, extrai o embarcado
        if !db_path.exists() {
            let embedded_db = include_bytes!("../../resources/sgw.db");
            if let Err(e) = std::fs::write(&db_path, embedded_db) {
                eprintln!("Erro ao extrair banco de dados embarcado: {}", e);
            }
        }

        let db = crate::db::Database::new(&db_path).expect("Falha ao inicializar banco de dados");

        // Carrega perfis e templates existentes
        let profiles = db.list_game_profiles().unwrap_or_default();
        let templates = db.list_game_templates().unwrap_or_default();

        (
            Self {
                db,
                templates,
                selected_template: None,
                profiles,
                form_game_name: String::new(),
                form_save_path: None,
                form_backup_dir: None,
                form_timeout: String::from("5"),
            },
            Task::none(),
        )
    }

    /// Define o título da janela
    pub fn title(&self) -> String {
        String::from("SaveGameWatcher - Backup Automático de Save Games")
    }

    /// Atualiza o estado da aplicação baseado em mensagens
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectTemplate(id) => {
                if let Ok(template) = self.db.get_game_template(id) {
                    // Pré-preenche formulário com dados do template
                    self.form_game_name = template.name.clone();
                    self.form_save_path = Some(PathBuf::from(template.expand_save_directory()));
                    self.form_timeout = String::from("5");
                    self.selected_template = Some(template);
                }
            }
            Message::ClearTemplate => {
                self.selected_template = None;
                self.form_game_name.clear();
                self.form_save_path = None;
                self.form_backup_dir = None;
                self.form_timeout = String::from("5");
            }
            Message::GameNameChanged(value) => {
                self.form_game_name = value;
            }
            Message::TimeoutChanged(value) => {
                self.form_timeout = value;
            }
            Message::SelectSaveFile => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Selecionar arquivo de save")
                            .pick_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Message::SaveFileSelected,
                );
            }
            Message::SaveFileSelected(path) => {
                self.form_save_path = path;
            }
            Message::SelectBackupDir => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Selecionar diretório de backups")
                            .pick_folder()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Message::BackupDirSelected,
                );
            }
            Message::BackupDirSelected(path) => {
                self.form_backup_dir = path;
            }
            Message::CreateProfile => {
                if let (Some(save_path), Some(backup_dir)) =
                    (&self.form_save_path, &self.form_backup_dir)
                {
                    if !self.form_game_name.is_empty() {
                        let timeout = self.form_timeout.parse::<u32>().unwrap_or(5);

                        let profile = if let Some(template) = &self.selected_template {
                            GameProfile::from_template(
                                template.id,
                                template,
                                backup_dir.to_string_lossy().to_string(),
                                timeout,
                            )
                        } else {
                            GameProfile::new(
                                self.form_game_name.clone(),
                                save_path.to_string_lossy().to_string(),
                                backup_dir.to_string_lossy().to_string(),
                                timeout,
                            )
                        };

                        if self.db.insert_game_profile(&profile).is_ok() {
                            // Recarrega perfis
                            self.profiles = self.db.list_game_profiles().unwrap_or_default();

                            // Limpa formulário
                            self.form_game_name.clear();
                            self.form_save_path = None;
                            self.form_backup_dir = None;
                            self.form_timeout = String::from("5");
                            self.selected_template = None;
                        }
                    }
                }
            }
            Message::ToggleMonitoring(id) => {
                if let Some(profile) = self.profiles.iter_mut().find(|p| p.id == id) {
                    profile.is_active = !profile.is_active;
                    let _ = self.db.update_profile_status(id, profile.is_active);
                }
            }
            Message::DeleteProfile(id) => {
                if let Ok(()) = self.db.delete_game_profile(id) {
                    self.profiles = self.db.list_game_profiles().unwrap_or_default();
                }
            }
            Message::BackupCreated(_id, _backup_path) => {
                // TODO: Atualizar UI com notificação de backup criado
                // Por enquanto apenas ignoramos, o backup já foi feito em background
            }
        }
        Task::none()
    }

    /// Configura assinaturas de eventos (background tasks)
    pub fn subscription(&self) -> Subscription<Message> {
        crate::watcher::watch(self.profiles.clone())
    }

    /// Define o tema da aplicação
    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}
