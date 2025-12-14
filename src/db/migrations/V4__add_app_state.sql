-- Migration V4: Add app_state table for persisting application state
-- Stores last active profile and other application preferences

CREATE TABLE IF NOT EXISTS app_state (
    id INTEGER PRIMARY KEY CHECK (id = 1), -- Apenas 1 linha
    last_profile_id INTEGER,
    last_backup_dir TEXT,
    last_timeout_minutes INTEGER DEFAULT 5,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (last_profile_id) REFERENCES game_profiles(id) ON DELETE SET NULL
);

-- Insere estado inicial
INSERT OR IGNORE INTO app_state (id, updated_at) VALUES (1, datetime('now'));

