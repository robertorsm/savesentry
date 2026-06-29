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
            state: AppState::new(db_path),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // Atualiza informações do save periodicamente
        self.state.update_save_info();

        // Verifica timer não-bloqueante para reinício de monitoramento após restore
        if let Some(instant) = self.state.restart_monitoring_after {
            if instant <= std::time::Instant::now() {
                self.state.restart_monitoring_after = None;
                self.state.start_monitoring();
            }
        }

        // Header unificado: abas (esquerda) + título + status (direita)
        eframe::egui::TopBottomPanel::top("header").show(ctx, |ui| {
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
            eframe::egui::TopBottomPanel::top("messages_banner").show(ctx, |ui| {
                components::render_messages_banner(ui, &mut self.state);
            });
        }

        // Renderiza página ativa baseada na aba selecionada
        pages::render_active_page(ctx, &mut self.state);

        // Otimização: Repaint adaptativo baseado no estado
        let repaint_interval = if self.state.active_watcher.is_some() {
            std::time::Duration::from_secs(1) // Monitorando: 1 FPS para UI responsiva
        } else {
            std::time::Duration::from_secs(5) // Parado: 0.2 FPS para economizar CPU
        };
        ctx.request_repaint_after(repaint_interval);
    }
}
