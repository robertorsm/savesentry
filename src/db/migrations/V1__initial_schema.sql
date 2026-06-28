-- Initial schema for SaveSentry
-- Creates base tables for game profiles and templates
-- Game templates (pre-configured game settings)
CREATE TABLE IF NOT EXISTS game_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    save_directory TEXT NOT NULL,
    process_name TEXT NOT NULL,
    save_pattern TEXT NOT NULL,
    exclude_regex TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    is_official INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);
-- Game profiles (user-created profiles based on templates or custom)
CREATE TABLE IF NOT EXISTS game_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_id INTEGER,
    name TEXT NOT NULL,
    save_path TEXT NOT NULL,
    backup_dir TEXT NOT NULL,
    timeout_minutes INTEGER NOT NULL,
    exclude_regex TEXT,
    is_active INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (template_id) REFERENCES game_templates(id) ON DELETE
    SET NULL
);
-- Migration tracking
CREATE TABLE IF NOT EXISTS refinery_schema_history (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_on TEXT NOT NULL,
    checksum TEXT NOT NULL
);