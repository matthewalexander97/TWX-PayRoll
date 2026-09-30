use rusqlite::{params, Connection};
use rusqlite_migration::{Migrations, SchemaVersion, M};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

static MIGRATION_STEPS: [M<'static>; 1] = [M::up(include_str!("../migrations/001_settings.sql"))];
static MIGRATIONS: Migrations<'static> = Migrations::from_slice(&MIGRATION_STEPS);

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}
impl Theme {
    fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}
#[derive(Serialize)]
pub struct Preferences {
    pub theme: Theme,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: &'static str,
    pub database_path: String,
    pub schema_version: i64,
    pub sqlite_version: String,
}

pub struct Storage {
    connection: Connection,
    path: PathBuf,
}
pub fn database_path(app_data: &Path) -> PathBuf {
    app_data.join("data").join("twx-payroll.sqlite3")
}
impl Storage {
    pub fn open(app_data: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let path = database_path(app_data);
        let directory = path.parent().ok_or("Invalid application data directory")?;
        std::fs::create_dir_all(directory)?;
        let mut connection = Connection::open(&path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update(None, "trusted_schema", false)?;
        let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if matches!(
            MIGRATIONS.current_version(&connection)?,
            SchemaVersion::Outside(_)
        ) {
            return Err(
                "Database was created by a newer application. No migrations were applied.".into(),
            );
        }
        if version < 0 {
            return Err("Invalid schema version".into());
        }
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "FULL")?;
        // Extension loading is disabled by SQLite by default; no extension API is exposed.
        MIGRATIONS.to_latest(&mut connection)?;
        let storage = Self { connection, path };
        storage.preferences()?; // Corrupt/invalid preferences fail startup rather than overwrite data.
        Ok(storage)
    }
    pub fn preferences(&self) -> Result<Preferences, rusqlite::Error> {
        let value: String = self.connection.query_row(
            "SELECT value FROM app_settings WHERE key = 'theme'",
            [],
            |row| row.get(0),
        )?;
        let theme = match value.as_str() {
            "system" => Theme::System,
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        Ok(Preferences { theme })
    }
    pub fn set_theme(&mut self, theme: Theme) -> Result<Preferences, rusqlite::Error> {
        let transaction = self.connection.transaction()?;
        let changed = transaction.execute("UPDATE app_settings SET value = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE key = 'theme'", params![theme.as_str()])?;
        if changed != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        transaction.commit()?;
        self.preferences()
    }
    pub fn info(&self) -> Result<AppInfo, rusqlite::Error> {
        Ok(AppInfo {
            version: env!("CARGO_PKG_VERSION"),
            database_path: self.path.to_string_lossy().into_owned(),
            schema_version: self
                .connection
                .pragma_query_value(None, "user_version", |row| row.get(0))?,
            sqlite_version: self
                .connection
                .query_row("SELECT sqlite_version()", [], |row| row.get(0))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_survive_reopen_and_migrations_are_idempotent() {
        let directory = tempfile::tempdir().unwrap();
        let mut storage = Storage::open(directory.path()).unwrap();
        assert_eq!(storage.preferences().unwrap().theme, Theme::System);
        storage.set_theme(Theme::Dark).unwrap();
        assert_eq!(storage.info().unwrap().schema_version, 1);
        drop(storage);
        let storage = Storage::open(directory.path()).unwrap();
        assert_eq!(storage.preferences().unwrap().theme, Theme::Dark);
        let tables: i64 = storage.connection.query_row("SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |row| row.get(0)).unwrap();
        assert_eq!(tables, 1);
        assert_eq!(
            storage
                .connection
                .pragma_query_value(None, "foreign_keys", |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            storage
                .connection
                .pragma_query_value(None, "journal_mode", |row| row.get::<_, String>(0))
                .unwrap(),
            "wal"
        );
    }
    #[test]
    fn future_database_is_rejected_without_reset() {
        let directory = tempfile::tempdir().unwrap();
        let storage = Storage::open(directory.path()).unwrap();
        storage
            .connection
            .pragma_update(None, "user_version", 99)
            .unwrap();
        drop(storage);
        assert!(Storage::open(directory.path()).is_err());
        let connection = Connection::open(database_path(directory.path())).unwrap();
        assert_eq!(
            connection
                .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
                .unwrap(),
            99
        );
    }
    #[test]
    fn migration_failure_rolls_back_schema_and_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        let migrations = Migrations::new(vec![M::up(
            "CREATE TABLE example(id INTEGER); INVALID SQL;",
        )]);
        assert!(migrations.to_latest(&mut connection).is_err());
        assert_eq!(
            connection
                .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM sqlite_schema WHERE name='example'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }
    #[test]
    fn schema_rejects_unsupported_settings() {
        let directory = tempfile::tempdir().unwrap();
        let storage = Storage::open(directory.path()).unwrap();
        assert!(storage
            .connection
            .execute("UPDATE app_settings SET value='invalid'", [])
            .is_err());
        assert!(storage
            .connection
            .execute(
                "INSERT INTO app_settings(key,value) VALUES('payroll','light')",
                []
            )
            .is_err());
        MIGRATIONS.validate().unwrap();
        assert_eq!(
            database_path(Path::new("root")),
            Path::new("root").join("data/twx-payroll.sqlite3")
        );
    }
}
