-- Schema inicial consolidado para SaveSentry
-- Contém todo o schema + seeds em um único arquivo

-- Game templates (pre-configured game settings)
CREATE TABLE IF NOT EXISTS game_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    save_directory TEXT NOT NULL,
    process_name TEXT NOT NULL,
    save_pattern TEXT NOT NULL,
    exclude_pattern TEXT,
    backup_dir TEXT NOT NULL,
    backup_delay_minutes INTEGER NOT NULL DEFAULT 5,
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
    backup_delay_minutes INTEGER NOT NULL,
    exclude_pattern TEXT,
    save_pattern TEXT,
    is_active INTEGER NOT NULL DEFAULT 0,
    process_name TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (template_id) REFERENCES game_templates(id) ON DELETE SET NULL
);

-- App state (last active profile and preferences)
CREATE TABLE IF NOT EXISTS app_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_profile_id INTEGER,
    last_backup_dir TEXT,
    last_backup_delay_minutes INTEGER DEFAULT 5,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (last_profile_id) REFERENCES game_profiles(id) ON DELETE SET NULL
);

-- Insert initial app state
INSERT OR IGNORE INTO app_state (id, updated_at) VALUES (1, datetime('now'));

-- Seed initial game templates
INSERT INTO game_templates (
        name,
        save_directory,
        process_name,
        save_pattern,
        exclude_pattern,
        backup_dir,
        backup_delay_minutes,
        version,
        is_official,
        created_at
    )
VALUES (
        'Minecraft',
        '%APPDATA%\.minecraft\saves',
        'javaw.exe',
        '*',
        '*.lock|*.tmp|session.lock',
        '%USERPROFILE%\SaveSentry\Minecraft',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'Terraria',
        '%USERPROFILE%\Documents\My Games\Terraria\Worlds',
        'Terraria.exe',
        '*.wld',
        NULL,
        '%USERPROFILE%\SaveSentry\Terraria',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'Stardew Valley',
        '%APPDATA%\StardewValley\Saves',
        'StardewValley.exe',
        '*',
        '*_old|*.tmp',
        '%USERPROFILE%\SaveSentry\Stardew Valley',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'The Witcher 3',
        '%USERPROFILE%\Documents\The Witcher 3\gamesaves',
        'witcher3.exe',
        '*.sav',
        NULL,
        '%USERPROFILE%\SaveSentry\The Witcher 3',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'Skyrim',
        '%USERPROFILE%\Documents\My Games\Skyrim\Saves',
        'TESV.exe',
        '*.ess',
        '*autosave*|*quicksave*',
        '%USERPROFILE%\SaveSentry\Skyrim',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'Dark Souls III',
        '%APPDATA%\DarkSoulsIII',
        'DarkSoulsIII.exe',
        '*',
        NULL,
        '%USERPROFILE%\SaveSentry\Dark Souls III',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'Elden Ring',
        '%APPDATA%\EldenRing',
        'eldenring.exe',
        '*',
        NULL,
        '%USERPROFILE%\SaveSentry\Elden Ring',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'Cyberpunk 2077',
        '%USERPROFILE%\Saved Games\CD Projekt Red\Cyberpunk 2077',
        'Cyberpunk2077.exe',
        '*.dat',
        '*.old',
        '%USERPROFILE%\SaveSentry\Cyberpunk 2077',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'Valheim',
        '%USERPROFILE%\AppData\LocalLow\IronGate\Valheim\worlds',
        'valheim.exe',
        '*',
        '*.old|*.new',
        '%USERPROFILE%\SaveSentry\Valheim',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'Hollow Knight',
        '%APPDATA%\..\LocalLow\Team Cherry\Hollow Knight',
        'hollow_knight.exe',
        'user*.dat',
        NULL,
        '%USERPROFILE%\SaveSentry\Hollow Knight',
        5,
        1,
        1,
        datetime('now')
    ),
    (
        'Octopath Traveler',
        '%USERPROFILE%\Documents\My Games\Octopath_Traveler\%STEAMID%\SaveGames',
        'Octopath_Traveler-Win64-Shipping.exe',
        '*.sav',
        NULL,
        '%USERPROFILE%\SaveSentry\Octopath Traveler',
        1,
        1,
        1,
        datetime('now')
    );

-- Migration tracking
CREATE TABLE IF NOT EXISTS refinery_schema_history (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_on TEXT NOT NULL,
    checksum TEXT NOT NULL
);
