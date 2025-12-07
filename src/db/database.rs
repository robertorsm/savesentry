use rusqlite::{Connection, Result};
use std::path::Path;

// Embarca migrations no executável
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("src/db/migrations");
}

/// Gerenciador de banco de dados SQLite
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Cria uma nova conexão com o banco de dados
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut conn = Connection::open(path)?;

        // Aplica migrations
        embedded::migrations::runner()
            .run(&mut conn)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        println!("Migrations aplicadas com sucesso!");

        Ok(Database { conn })
    }

    // Métodos legados mantidos para referência futura (não estão em uso)
    // delete_item, update_item - podem ser removidos se não forem necessários

    // ===== Métodos para GameProfile =====

    /// Insere um novo perfil de jogo
    pub fn insert_game_profile(&self, profile: &crate::models::GameProfile) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO game_profiles (template_id, name, save_path, backup_dir, timeout_minutes, exclude_regex, is_active, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                profile.template_id,
                profile.name,
                profile.save_path,
                profile.backup_dir,
                profile.timeout_minutes,
                profile.exclude_regex,
                profile.is_active as i32,
                profile.created_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Lista todos os perfis de jogos
    pub fn list_game_profiles(&self) -> Result<Vec<crate::models::GameProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, template_id, name, save_path, backup_dir, timeout_minutes, exclude_regex, is_active, created_at 
             FROM game_profiles ORDER BY created_at DESC",
        )?;

        let profiles = stmt
            .query_map([], |row| {
                Ok(crate::models::GameProfile {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    name: row.get(2)?,
                    save_path: row.get(3)?,
                    backup_dir: row.get(4)?,
                    timeout_minutes: row.get(5)?,
                    exclude_regex: row.get(6)?,
                    is_active: row.get::<_, i32>(7)? != 0,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(profiles)
    }

    /// Atualiza o status de monitoramento de um perfil
    pub fn update_profile_status(&self, id: i64, is_active: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE game_profiles SET is_active = ?1 WHERE id = ?2",
            rusqlite::params![is_active as i32, id],
        )?;
        Ok(())
    }

    /// Deleta um perfil de jogo
    pub fn delete_game_profile(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM game_profiles WHERE id = ?1", [id])?;
        Ok(())
    }

    // ===== Métodos para GameTemplate =====

    /// Lista todos os templates de jogos
    pub fn list_game_templates(&self) -> Result<Vec<crate::models::GameTemplate>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, save_directory, process_name, save_pattern, exclude_regex, version, is_official, created_at 
             FROM game_templates ORDER BY name ASC",
        )?;

        let templates = stmt
            .query_map([], |row| {
                Ok(crate::models::GameTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    save_directory: row.get(2)?,
                    process_name: row.get(3)?,
                    save_pattern: row.get(4)?,
                    exclude_regex: row.get(5)?,
                    version: row.get(6)?,
                    is_official: row.get::<_, i32>(7)? != 0,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(templates)
    }

    /// Busca um template por ID
    #[allow(dead_code)]
    pub fn get_game_template(&self, id: i64) -> Result<crate::models::GameTemplate> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, save_directory, process_name, save_pattern, exclude_regex, version, is_official, created_at 
             FROM game_templates WHERE id = ?1",
        )?;

        stmt.query_row([id], |row| {
            Ok(crate::models::GameTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                save_directory: row.get(2)?,
                process_name: row.get(3)?,
                save_pattern: row.get(4)?,
                exclude_regex: row.get(5)?,
                version: row.get(6)?,
                is_official: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
            })
        })
    }
}
