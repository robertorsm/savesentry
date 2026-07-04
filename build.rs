use refinery::embed_migrations;

// Embarca migrations no executável
embed_migrations!("src/db/migrations");

fn main() {
    // Build script - migrations são embarcadas em tempo de compilação
    println!("cargo:rerun-if-changed=src/db/migrations");

    #[cfg(windows)]
    {
        let icon_path = std::path::Path::new("assets").join("icon.ico");
        if icon_path.exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon(icon_path.to_str().unwrap());
            res.compile().expect("Falha ao compilar recurso de ícone");
            println!("cargo:rerun-if-changed=assets/icon.ico");
        }

        // Desabilita janela de console no Windows (apenas release)
        #[cfg(not(debug_assertions))]
        {
            let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
            if target_env == "msvc" {
                println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
                println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
            } else {
                println!("cargo:rustc-link-arg=-Wl,--subsystem,windows");
            }
        }
    }
}
