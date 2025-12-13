use crate::ui::components;
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

        let db_path = exe_dir.join("sgw.db");

        Self {
            state: AppState::new(db_path),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // Atualiza informações do save periodicamente
        self.state.update_save_info();

        // Top panel - Barra superior com título e configurações
        eframe::egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("🎮 SaveGameWatcher");
                ui.separator();

                // Status do monitoramento
                if self.state.active_watcher.is_some() {
                    ui.label(
                        eframe::egui::RichText::new("🟢 MONITORANDO")
                            .color(eframe::egui::Color32::GREEN),
                    );
                } else {
                    ui.label(
                        eframe::egui::RichText::new("⚫ PARADO").color(eframe::egui::Color32::GRAY),
                    );
                }
            });
            ui.add_space(4.0);
        });

        // Side panel - Lista de backups (histórico)
        eframe::egui::SidePanel::left("backup_history_panel")
            .resizable(true)
            .default_width(250.0)
            .width_range(200.0..=400.0)
            .show(ctx, |ui| {
                components::render_backup_history(ui, &mut self.state);
            });

        // Central panel - Área principal
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            // Mensagens de erro/sucesso
            components::render_messages(ui, &mut self.state);

            ui.add_space(8.0);

            // Painel superior: Template selection + configurações
            eframe::egui::Frame::group(ui.style())
                .fill(ui.style().visuals.faint_bg_color)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    components::render_config_panel(ui, &mut self.state);
                });

            ui.add_space(12.0);

            // Painel inferior: Informações do save atual
            eframe::egui::Frame::group(ui.style())
                .fill(if self.state.active_watcher.is_some() {
                    eframe::egui::Color32::from_rgb(20, 60, 30)
                } else {
                    ui.style().visuals.faint_bg_color
                })
                .inner_margin(12.0)
                .show(ui, |ui| {
                    components::render_save_info(ui, &mut self.state);
                });
        });

        // Otimização: Repaint adaptativo - reduz CPU quando inativo
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}
