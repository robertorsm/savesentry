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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Repintar continuamente para atualizar UI
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Cabeçalho
            ui.heading("SaveGameWatcher - Backup Automático de Save Games");
            ui.add_space(10.0);

            // Mensagens de erro/sucesso
            components::render_messages(ui, &mut self.state);

            ui.separator();

            // Seção de Templates
            components::render_templates(ui, &mut self.state);

            ui.add_space(10.0);
            ui.separator();

            // Formulário de Novo Perfil
            components::render_profile_form(ui, &mut self.state);

            ui.add_space(10.0);
            ui.separator();

            // Lista de Perfis
            components::render_profiles_list(ui, &mut self.state);
        });
    }
}

