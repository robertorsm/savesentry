mod db;
mod models;
mod ui;
mod watcher;

use iced::Size;

fn main() -> anyhow::Result<()> {
    // Obter diretório do executável para modo portátil
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Não foi possível determinar o diretório do executável"))?;

    // Caminho do banco de dados na mesma pasta do executável
    let db_path = exe_dir.join("savegame_watcher.db");
    let _db = db::Database::new(&db_path)?;
    println!("Banco de dados inicializado em: {:?}", db_path);

    // Configurar e executar a aplicação
    iced::application(ui::App::title, ui::App::update, ui::App::view)
        .theme(ui::App::theme)
        .subscription(ui::App::subscription)
        .window_size(Size::new(800.0, 600.0))
        .run_with(ui::App::new)?;

    Ok(())
}
