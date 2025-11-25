use refinery::embed_migrations;

// Embarca migrations no executável
embed_migrations!("src/db/migrations");

fn main() {
    // Build script - migrations são embarcadas em tempo de compilação
    println!("cargo:rerun-if-changed=src/db/migrations");
}
