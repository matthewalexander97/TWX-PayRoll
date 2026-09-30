CREATE TABLE app_settings (
    key TEXT PRIMARY KEY NOT NULL CHECK (key = 'theme'),
    value TEXT NOT NULL CHECK (value IN ('system', 'light', 'dark')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;
INSERT INTO app_settings (key, value) VALUES ('theme', 'system');
