-- Seed initial game templates
-- Popular games with pre-configured settings
INSERT INTO game_templates (
        name,
        save_directory,
        process_name,
        save_pattern,
        exclude_regex,
        version,
        is_official,
        created_at
    )
VALUES (
        'Minecraft',
        '%APPDATA%\.minecraft\saves',
        'javaw.exe',
        '*',
        '.*\.lock$|.*\.tmp$|.*session\.lock$',
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
        1,
        1,
        datetime('now')
    ),
    (
        'Stardew Valley',
        '%APPDATA%\StardewValley\Saves',
        'StardewValley.exe',
        '*',
        '.*_old$|.*\.tmp$',
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
        1,
        1,
        datetime('now')
    ),
    (
        'Skyrim',
        '%USERPROFILE%\Documents\My Games\Skyrim\Saves',
        'TESV.exe',
        '*.ess',
        '.*autosave.*|.*quicksave.*',
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
        1,
        1,
        datetime('now')
    ),
    (
        'Cyberpunk 2077',
        '%USERPROFILE%\Saved Games\CD Projekt Red\Cyberpunk 2077',
        'Cyberpunk2077.exe',
        '*.dat',
        '.*\.old$',
        1,
        1,
        datetime('now')
    ),
    (
        'Valheim',
        '%USERPROFILE%\AppData\LocalLow\IronGate\Valheim\worlds',
        'valheim.exe',
        '*',
        '.*\.old$|.*\.new$',
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
        1,
        1,
        datetime('now')
    );