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

        #[cfg(debug_assertions)]
        println!("Migrations aplicadas com sucesso!");

        Ok(Database { conn })
    }

    // Métodos legados mantidos para referência futura (não estão em uso)
    // delete_item, update_item - podem ser removidos se não forem necessários

    // ===== Métodos para GameProfile =====

    /// Insere um novo perfil de jogo
    pub fn insert_game_profile(&self, profile: &crate::models::GameProfile) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO game_profiles (template_id, name, save_path, backup_dir, backup_delay_minutes, exclude_pattern, save_pattern, is_active, process_name, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                profile.template_id,
                profile.name,
                profile.save_path,
                profile.backup_dir,
                profile.backup_delay_minutes,
                profile.exclude_pattern,
                &profile.save_pattern,
                profile.is_active as i32,
                &profile.process_name,
                profile.created_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // ===== Métodos para GameTemplate =====

    /// Lista todos os templates de jogos
    pub fn list_game_templates(&self) -> Result<Vec<crate::models::GameTemplate>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, save_directory, process_name, save_pattern, exclude_pattern, backup_dir, backup_delay_minutes, version, is_official, created_at 
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
                    backup_dir: row.get(6)?,
                    backup_delay_minutes: row.get(7)?,
                    version: row.get(8)?,
                    is_official: row.get::<_, i32>(9)? != 0,
                    created_at: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(templates)
    }

    /// Insere um novo template de jogo
    pub fn insert_game_template(
        &self,
        name: &str,
        save_directory: &str,
        process_name: &str,
        save_pattern: &str,
        exclude_pattern: Option<&str>,
        backup_dir: &str,
        backup_delay_minutes: u32,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO game_templates (name, save_directory, process_name, save_pattern, exclude_pattern, backup_dir, backup_delay_minutes, version, is_official, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, 0, datetime('now'))",
            rusqlite::params![
                name,
                save_directory,
                process_name,
                save_pattern,
                exclude_pattern,
                backup_dir,
                backup_delay_minutes,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Atualiza um template existente
    pub fn update_game_template(
        &self,
        id: i64,
        name: &str,
        save_directory: &str,
        process_name: &str,
        save_pattern: &str,
        exclude_pattern: Option<&str>,
        backup_dir: &str,
        backup_delay_minutes: u32,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE game_templates 
             SET name = ?1, save_directory = ?2, process_name = ?3, save_pattern = ?4, exclude_pattern = ?5, backup_dir = ?6, backup_delay_minutes = ?7, version = version + 1 
             WHERE id = ?8",
            rusqlite::params![
                name,
                save_directory,
                process_name,
                save_pattern,
                exclude_pattern,
                backup_dir,
                backup_delay_minutes,
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

    /// Obtém o estado da aplicação (último perfil usado, configurações)
    pub fn get_app_state(&self) -> Result<(Option<i64>, Option<String>, u32)> {
        let mut stmt = self.conn.prepare(
            "SELECT last_profile_id, last_backup_dir, last_backup_delay_minutes FROM app_state WHERE id = 1"
        )?;

        stmt.query_row([], |row| {
            Ok((
                row.get(0).ok(),
                row.get(1).ok(),
                row.get::<_, Option<u32>>(2)?.unwrap_or(5),
            ))
        })
    }

    /// Atualiza último perfil usado
    pub fn update_last_profile(
        &self,
        profile_id: i64,
        backup_dir: &str,
        timeout: u32,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE app_state SET last_profile_id = ?1, last_backup_dir = ?2, last_backup_delay_minutes = ?3, updated_at = datetime('now') WHERE id = 1",
            rusqlite::params![profile_id, backup_dir, timeout]
        )?;
        Ok(())
    }

    /// Obtém um perfil específico por ID
    pub fn get_game_profile(&self, id: i64) -> Result<crate::models::GameProfile> {
        let mut stmt = self.conn.prepare(
            "SELECT id, template_id, name, save_path, backup_dir, backup_delay_minutes, exclude_pattern, save_pattern, is_active, process_name, created_at 
             FROM game_profiles WHERE id = ?1"
        )?;

        stmt.query_row([id], |row| {
            Ok(crate::models::GameProfile {
                id: row.get(0)?,
                template_id: row.get(1)?,
                name: row.get(2)?,
                save_path: row.get(3)?,
                backup_dir: row.get(4)?,
                backup_delay_minutes: row.get(5)?,
                exclude_pattern: row.get(6)?,
                save_pattern: row.get(7)?,
                is_active: row.get::<_, i32>(8)? != 0,
                process_name: row.get(9).ok(),
                created_at: row.get(10)?,
            })
        })
    }
}
