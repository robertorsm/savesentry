mod db;
mod models;
mod ui;
mod watcher;

fn main() -> anyhow::Result<()> {
    let icon_data = eframe::icon_data::from_png_bytes(include_bytes!("../assets/exec_icon.png"))
        .ok()
        .map(std::sync::Arc::new);

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([850.0, 550.0])
        .with_min_inner_size([600.0, 400.0])
        .with_title("SaveSentry - Backup Automático de Save Games");
    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "SaveSentry",
        options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("Erro ao executar aplicação: {}", e))?;

    Ok(())
}
