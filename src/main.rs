mod db;
mod models;
mod ui;
mod watcher;

fn main() -> anyhow::Result<()> {
    // Configurações de janela do eframe
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("SaveGameWatcher - Backup Automático de Save Games"),
        ..Default::default()
    };

    // Executa a aplicação
    eframe::run_native(
        "SaveGameWatcher",
        options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Erro ao executar aplicação: {}", e))?;

    Ok(())
}

