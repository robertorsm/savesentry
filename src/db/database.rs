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

        conn.execute_batch("PRAGMA cache_size = -512;")?;

        #[cfg(debug_assertions)]
        println!("Migrations aplicadas com sucesso!");

        Ok(Database { conn })
    }

    // Métodos legados mantidos para referência futura (não estão em uso)
    // delete_item, update_item - podem ser removidos se não forem necessários

    // ===== Métodos para GameTemplate =====

    /// Lista todos os templates de jogos
    pub fn list_game_templates(&self) -> Result<Vec<crate::models::GameTemplate>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, save_directory, process_name, save_pattern, exclude_pattern, default_exclude_pattern, backup_dir, backup_delay_minutes, screenshot_delay_seconds, backup_max_count, version, is_official, created_at 
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
                    exclude_pattern: row.get(5)?,
                    default_exclude_pattern: row.get(6)?,
                    backup_dir: row.get(7)?,
                    backup_delay_minutes: row.get(8)?,
                    screenshot_delay_seconds: row.get::<_, Option<u32>>(9)?.unwrap_or(0),
                    backup_max_count: row.get::<_, Option<u32>>(10)?.unwrap_or(50),
                    version: row.get(11)?,
                    is_official: row.get::<_, i32>(12)? != 0,
                    created_at: row.get(13)?,
                    expanded_save_directory: None,
                    expanded_backup_directory: None,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(templates)
    }

    #[allow(clippy::too_many_arguments)]
    /// Insere um novo template de jogo
    pub fn insert_game_template(
        &self,
        name: &str,
        save_directory: &str,
        process_name: &str,
        save_pattern: &str,
        exclude_pattern: Option<&str>,
        default_exclude_pattern: Option<&str>,
        backup_dir: &str,
        backup_delay_minutes: u32,
        screenshot_delay_seconds: u32,
        backup_max_count: u32,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO game_templates (name, save_directory, process_name, save_pattern, exclude_pattern, default_exclude_pattern, backup_dir, backup_delay_minutes, screenshot_delay_seconds, backup_max_count, version, is_official, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, 0, datetime('now'))",
            rusqlite::params![
                name,
                save_directory,
                process_name,
                save_pattern,
                exclude_pattern,
                default_exclude_pattern,
                backup_dir,
                backup_delay_minutes,
                screenshot_delay_seconds,
                backup_max_count,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    #[allow(clippy::too_many_arguments)]
    /// Atualiza um template existente
    pub fn update_game_template(
        &self,
        id: i64,
        name: &str,
        save_directory: &str,
        process_name: &str,
        save_pattern: &str,
        exclude_pattern: Option<&str>,
        default_exclude_pattern: Option<&str>,
        backup_dir: &str,
        backup_delay_minutes: u32,
        screenshot_delay_seconds: u32,
        backup_max_count: u32,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE game_templates 
             SET name = ?1, save_directory = ?2, process_name = ?3, save_pattern = ?4, exclude_pattern = ?5, default_exclude_pattern = ?6, backup_dir = ?7, backup_delay_minutes = ?8, screenshot_delay_seconds = ?9, backup_max_count = ?10, version = version + 1 
             WHERE id = ?11",
            rusqlite::params![
                name,
                save_directory,
                process_name,
                save_pattern,
                exclude_pattern,
                default_exclude_pattern,
                backup_dir,
                backup_delay_minutes,
                screenshot_delay_seconds,
                backup_max_count,
                id,
            ],
        )?;
        Ok(())
    }

    /// Deleta um template (apenas customizados, não oficiais)
    pub fn delete_game_template(&self, id: i64) -> Result<()> {
        // Verifica se não é oficial antes de deletar
        let is_official: i32 = self.conn.query_row(
            "SELECT is_official FROM game_templates WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;

        if is_official != 0 {
            return Err(rusqlite::Error::ToSqlConversionFailure(
                "Templates oficiais não podem ser excluídos".into(),
            ));
        }

        self.conn.execute(
            "DELETE FROM game_templates WHERE id = ?1 AND is_official = 0",
            [id],
        )?;
        Ok(())
    }

    // ===== Métodos para AppState =====

    /// Obtém o estado da aplicação (último template usado, configurações)
    pub fn get_app_state(&self) -> Result<(Option<i64>, Option<String>, u32)> {
        let mut stmt = self.conn.prepare(
            "SELECT last_template_id, last_backup_dir, last_backup_delay_minutes FROM app_state WHERE id = 1"
        )?;

        stmt.query_row([], |row| {
            Ok((
                row.get(0).ok(),
                row.get(1).ok(),
                row.get::<_, Option<u32>>(2)?.unwrap_or(5),
            ))
        })
    }

    /// Atualiza último template usado
    pub fn update_last_template(
        &self,
        template_id: i64,
        backup_dir: &str,
        timeout: u32,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE app_state SET last_template_id = ?1, last_backup_dir = ?2, last_backup_delay_minutes = ?3, updated_at = datetime('now') WHERE id = 1",
            rusqlite::params![template_id, backup_dir, timeout]
        )?;
        Ok(())
    }
}
