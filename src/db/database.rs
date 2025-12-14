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
    #[allow(dead_code)]
    pub fn insert_game_profile(&self, profile: &crate::models::GameProfile) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO game_profiles (template_id, name, save_path, backup_dir, timeout_minutes, exclude_regex, is_active, process_name, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                profile.template_id,
                profile.name,
                profile.save_path,
                profile.backup_dir,
                profile.timeout_minutes,
                profile.exclude_regex,
                profile.is_active as i32,
                &profile.process_name,
                profile.created_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Lista todos os perfis de jogos
    #[allow(dead_code)]
    pub fn list_game_profiles(&self) -> Result<Vec<crate::models::GameProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, template_id, name, save_path, backup_dir, timeout_minutes, exclude_regex, is_active, process_name, created_at 
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
                    process_name: row.get(8).ok(),
                    created_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(profiles)
    }

    /// Atualiza o status de monitoramento de um perfil
    #[allow(dead_code)]
    pub fn update_profile_status(&self, id: i64, is_active: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE game_profiles SET is_active = ?1 WHERE id = ?2",
            rusqlite::params![is_active as i32, id],
        )?;
        Ok(())
    }

    /// Deleta um perfil de jogo
    #[allow(dead_code)]
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

    /// Insere um novo template de jogo
    pub fn insert_game_template(
        &self,
        name: &str,
        save_directory: &str,
        process_name: &str,
        save_pattern: &str,
        exclude_regex: Option<&str>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO game_templates (name, save_directory, process_name, save_pattern, exclude_regex, version, is_official, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 0, datetime('now'))",
            rusqlite::params![
                name,
                save_directory,
                process_name,
                save_pattern,
                exclude_regex,
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
        exclude_regex: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE game_templates 
             SET name = ?1, save_directory = ?2, process_name = ?3, save_pattern = ?4, exclude_regex = ?5 
             WHERE id = ?6",
            rusqlite::params![
                name,
                save_directory,
                process_name,
                save_pattern,
                exclude_regex,
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
            return Err(rusqlite::Error::InvalidQuery);
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
            "SELECT last_profile_id, last_backup_dir, last_timeout_minutes FROM app_state WHERE id = 1"
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
    pub fn update_last_profile(&self, profile_id: i64, backup_dir: &str, timeout: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE app_state SET last_profile_id = ?1, last_backup_dir = ?2, last_timeout_minutes = ?3, updated_at = datetime('now') WHERE id = 1",
            rusqlite::params![profile_id, backup_dir, timeout]
        )?;
        Ok(())
    }

    /// Obtém um perfil específico por ID
    pub fn get_game_profile(&self, id: i64) -> Result<crate::models::GameProfile> {
        let mut stmt = self.conn.prepare(
            "SELECT id, template_id, name, save_path, backup_dir, timeout_minutes, exclude_regex, is_active, process_name, created_at 
             FROM game_profiles WHERE id = ?1"
        )?;
        
        stmt.query_row([id], |row| {
            Ok(crate::models::GameProfile {
                id: row.get(0)?,
                template_id: row.get(1)?,
                name: row.get(2)?,
                save_path: row.get(3)?,
                backup_dir: row.get(4)?,
                timeout_minutes: row.get(5)?,
                exclude_regex: row.get(6)?,
                is_active: row.get::<_, i32>(7)? != 0,
                process_name: row.get(8).ok(),
                created_at: row.get(9)?,
            })
        })
    }
}
