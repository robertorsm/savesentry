use refinery::embed_migrations;

// Embarca migrations no executável
embed_migrations!("src/db/migrations");

fn main() {
    // Build script - migrations são embarcadas em tempo de compilação
    println!("cargo:rerun-if-changed=src/db/migrations");

    // Desabilita janela de console no Windows (apenas release)
    #[cfg(all(windows, not(debug_assertions)))]
    {
        // Detecta toolchain (MSVC ou GNU/MinGW)
        let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

        if target_env == "msvc" {
            // Sintaxe MSVC
            println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
            println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
        } else {
            // Sintaxe MinGW/GNU
            println!("cargo:rustc-link-arg=-Wl,--subsystem,windows");
        }
    }
}
