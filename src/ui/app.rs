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

        // Top panel - Barra superior com título e status
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

        // Barra de abas
        eframe::egui::TopBottomPanel::top("tab_bar_panel").show(ctx, |ui| {
            components::render_tab_bar(ui, &mut self.state);
        });

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
