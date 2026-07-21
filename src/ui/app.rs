use crate::ui::components;
use crate::ui::pages;
use crate::ui::state::AppState;
use std::path::PathBuf;

/// Aplicação principal - Orquestração
pub struct App {
    state: AppState,
}

impl App {
    /// Cria uma nova instância da aplicação
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Determina caminho do banco de dados
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let db_path = exe_dir.join("savesentry.db");

        Self {
            state: AppState::new(db_path, _cc.egui_ctx.clone()),
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let now = std::time::Instant::now();

        if self.state.last_ui_update.elapsed() >= std::time::Duration::from_millis(1000) {
            self.state.last_ui_update = now;
            self.state.update_save_info();

            if self.state.check_backup_updates() {
                ctx.request_repaint();
            }

            if self.state.active_watcher.is_none() {
                if let Some(ref profile) = self.state.active_profile {
                    if let Some(ref proc_name) = profile.process_name {
                        if crate::ui::actions::monitoring::is_process_running(proc_name) {
                            self.state.start_monitoring();
                        }
                    }
                }
            }
        }

        if let Some(instant) = self.state.restart_monitoring_after {
            if instant <= now {
                self.state.restart_monitoring_after = None;
                if self.state.active_watcher.is_none() {
                    self.state.start_monitoring();
                }
                ctx.request_repaint();
            }
        }

        if let Some(expires) = self.state.message_expires_at {
            if expires <= now {
                self.state.clear_messages();
                ctx.request_repaint();
            }
        }

        if let Some(ref watcher) = self.state.active_watcher {
            if !watcher.process_running.load(std::sync::atomic::Ordering::Relaxed) {
                self.state.stop_monitoring();
            }
        }

        if self.state.active_watcher.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        } else if self.state.active_profile.is_some()
            && self
                .state
                .active_profile
                .as_ref()
                .unwrap()
                .process_name
                .is_some()
        {
            ctx.request_repaint_after(std::time::Duration::from_secs(2));
        }
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        // Header unificado: abas (esquerda) + título + status (direita)
        eframe::egui::Panel::top("header").show(ui, |ui| {
            ui.horizontal(|ui| {
                // Abas na esquerda
                components::render_tab_bar(ui, &mut self.state);

                ui.separator();

                // Título e status à direita
                ui.label(eframe::egui::RichText::new("SaveSentry").heading().strong());

                let status_text = if self.state.active_watcher.is_some() {
                    eframe::egui::RichText::new("ATIVO")
                        .color(eframe::egui::Color32::GREEN)
                        .strong()
                } else {
                    eframe::egui::RichText::new("PARADO").color(eframe::egui::Color32::GRAY)
                };
                ui.label(status_text);

                // Empurra resto para a direita (vazio, apenas para alinhar)
                ui.with_layout(
                    eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                    |ui| {
                        ui.add_space(0.0);
                    },
                );
            });
            ui.add_space(2.0);
        });

        // Banner de mensagens global (condicional)
        if self.state.error_message.is_some() || self.state.success_message.is_some() {
            eframe::egui::Panel::top("messages_banner").show(ui, |ui| {
                components::render_messages_banner(ui, &mut self.state);
            });
        }

        // Renderiza página ativa baseada na aba selecionada
        pages::render_active_page(ui, &mut self.state);
    }
}
