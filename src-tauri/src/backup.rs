//! SQLite Online Backup API, scoped destinations and recoverable restore coordination.
use crate::{storage::MIGRATIONS, Storage};
use rusqlite::{
    backup::{Backup, StepResult},
    params, Connection, OpenFlags,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tempfile::NamedTempFile;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const APP_ID: i64 = 0x54575850;
const VERSION: i64 = 4;
const APP: &str = "com.twx.payroll";
const DAY: i64 = 86400;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRun {
    pub id: String,
    pub started_at: i64,
    pub created_at: String,
    pub kind: String,
    pub path: String,
    pub status: String,
    pub schema_version: Option<i64>,
    pub size_bytes: Option<String>,
    pub sha256: Option<String>,
    pub validation: String,
    pub error_summary: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    pub destination: Option<String>,
    pub automatic_enabled: bool,
    pub retention_count: i64,
    pub notice: String,
    pub last_successful_backup: Option<String>,
    pub runs: Vec<BackupRun>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreview {
    pub token: String,
    pub filename: String,
    pub schema_version: i64,
    pub created_at: Option<String>,
}
pub(crate) struct PreparedRestore {
    preview: RestorePreview,
    path: PathBuf,
    checksum: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Manifest {
    application: String,
    id: String,
    filename: String,
    kind: String,
    created_at: String,
    schema_version: i64,
    size_bytes: u64,
    sha256: String,
}
#[derive(Serialize, Deserialize)]
struct RecoveryMarker {
    application: String,
    operation_id: String,
    safety_filename: String,
    safety_sha256: String,
    candidate_filename: String,
}
#[derive(Clone)]
struct Policy {
    destination: Option<String>,
    enabled: bool,
    retention: i64,
    generation: i64,
    saved: i64,
    success: i64,
    attempted: i64,
}
fn policy(c: &Connection) -> Result<Policy> {
    Ok(c.query_row("SELECT destination,automatic_enabled,retention_count,change_generation,saved_generation,last_success_at,last_attempt_at FROM backup_settings WHERE id=1",[],|r|Ok(Policy{destination:r.get(0)?,enabled:r.get(1)?,retention:r.get(2)?,generation:r.get(3)?,saved:r.get(4)?,success:r.get(5)?,attempted:r.get(6)?}))?)
}
pub(crate) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}
fn label(c: &Connection, seconds: i64) -> Result<String> {
    Ok(c.query_row(
        "SELECT strftime('%Y-%m-%dT%H:%M:%SZ',?1,'unixepoch')",
        [seconds],
        |r| r.get(0),
    )?)
}
fn fingerprint(c: &Connection) -> Result<Vec<(String, String, String, String)>> {
    let mut s=c.prepare("SELECT type,name,tbl_name,sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%' AND sql IS NOT NULL ORDER BY type,name")?;
    let result = Ok(s
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?);
    result
}
/// Recognize exact released schema, not just an arbitrary SQLite header or user_version.
pub(crate) fn validate(c: &Connection) -> Result<i64> {
    let version: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if !(1..=VERSION).contains(&version) {
        return Err("Unsupported database schema version".into());
    }
    let app: i64 = c.pragma_query_value(None, "application_id", |r| r.get(0))?;
    if (version >= 3 && app != APP_ID) || (version < 3 && app != 0) {
        return Err("Not a recognized TWX Payroll database".into());
    }
    c.pragma_update(None, "trusted_schema", false)?;
    let integrity: String = c.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity != "ok" {
        return Err("Database integrity validation failed".into());
    }
    if c.prepare("PRAGMA foreign_key_check")?
        .query([])?
        .next()?
        .is_some()
    {
        return Err("Database foreign-key validation failed".into());
    }
    let mut expected = configured(Connection::open_in_memory()?)?;
    MIGRATIONS.to_version(&mut expected, version as usize)?;
    if fingerprint(c)? != fingerprint(&expected)? {
        return Err("Database does not match a supported TWX Payroll schema".into());
    }
    let theme: String = c.query_row(
        "SELECT value FROM app_settings WHERE key='theme'",
        [],
        |r| r.get(0),
    )?;
    if !["system", "light", "dark"].contains(&theme.as_str()) {
        return Err("Invalid saved appearance preference".into());
    }
    Ok(version)
}
fn configured(connection: Connection) -> Result<Connection> {
    connection.pragma_update(None, "foreign_keys", true)?;
    connection.pragma_update(None, "trusted_schema", false)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(connection)
}
fn read_only(path: &Path) -> Result<Connection> {
    configured(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}
fn checksum(path: &Path) -> Result<String> {
    let mut input = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut bytes = [0_u8; 65536];
    loop {
        let n = input.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        hasher.update(&bytes[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
fn transfer(source: &Connection, destination: &mut Connection) -> Result<()> {
    let start = Instant::now();
    let backup = Backup::new(source, destination)?;
    loop {
        match backup.step(256)? {
            StepResult::Done => return Ok(()),
            StepResult::More => {}
            StepResult::Busy | StepResult::Locked => std::thread::sleep(Duration::from_millis(20)),
            _ => return Err("Unexpected snapshot state".into()),
        }
        if start.elapsed() > Duration::from_secs(120) {
            return Err("Snapshot timed out; previous data remains available".into());
        }
    }
}
/// Destination is a private empty file. Never copy the live main file or its WAL sidecars.
fn snapshot(source: &Connection, path: &Path) -> Result<()> {
    let mut destination = configured(Connection::open(path)?)?;
    destination.pragma_update(None, "journal_mode", "DELETE")?;
    destination.pragma_update(None, "synchronous", "FULL")?;
    transfer(source, &mut destination)?;
    destination.pragma_update(None, "journal_mode", "DELETE")?;
    validate(&destination)?;
    destination.close().map_err(|(_, e)| e)?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()?;
    Ok(())
}
fn manifest_path(path: &Path) -> PathBuf {
    path.with_extension("sqlite3.json")
}
fn read_manifest(path: &Path) -> Result<Manifest> {
    if fs::metadata(path)?.len() > 65536 {
        return Err("Invalid backup metadata size".into());
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn validate_file(path: &Path) -> Result<i64> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() || !path.is_file() {
        return Err("Select a regular SQLite backup file".into());
    }
    let version = validate(&read_only(path)?)?;
    let sidecar = manifest_path(path);
    if sidecar.exists() {
        let manifest = read_manifest(&sidecar)?;
        if manifest.application != APP
            || manifest.schema_version != version
            || Some(manifest.filename.as_str()) != path.file_name().and_then(|n| n.to_str())
            || manifest.size_bytes != fs::metadata(path)?.len()
            || manifest.sha256 != checksum(path)?
        {
            return Err("Backup completion metadata or checksum does not match".into());
        }
    }
    Ok(version)
}
fn publish_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut staged = NamedTempFile::new_in(path.parent().ok_or("Missing metadata parent")?)?;
    serde_json::to_writer(&mut staged, value)?;
    staged.flush()?;
    staged.as_file().sync_all()?;
    staged.persist_noclobber(path).map_err(|e| e.error)?;
    Ok(())
}
fn new_run(c: &Connection, id: &str, kind: &str, path: &Path, seconds: i64) -> Result<()> {
    c.execute("INSERT INTO backup_runs(id,started_at,created_at,kind,path,status) VALUES(?1,?2,?3,?4,?5,'RUNNING')",params![id,seconds,label(c,seconds)?,kind,path.to_string_lossy()])?;
    Ok(())
}
fn finish_run(c: &Connection, id: &str, status: &str, error: Option<&str>) -> Result<()> {
    c.execute(
        "UPDATE backup_runs SET status=?2,error_summary=?3 WHERE id=?1",
        params![id, status, error],
    )?;
    Ok(())
}
fn all_runs(c: &Connection) -> Result<Vec<BackupRun>> {
    let mut query=c.prepare("SELECT id,started_at,created_at,kind,path,status,schema_version,size_bytes,sha256,validation,error_summary FROM backup_runs ORDER BY started_at DESC,id DESC")?;
    let result = Ok(query
        .query_map([], |r| {
            Ok(BackupRun {
                id: r.get(0)?,
                started_at: r.get(1)?,
                created_at: r.get(2)?,
                kind: r.get(3)?,
                path: r.get(4)?,
                status: r.get(5)?,
                schema_version: r.get(6)?,
                size_bytes: r.get::<_, Option<i64>>(7)?.map(|n| n.to_string()),
                sha256: r.get(8)?,
                validation: r.get(9)?,
                error_summary: r.get(10)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?);
    result
}
fn merge_history(c: &mut Connection, runs: &[BackupRun], saved_policy: &Policy) -> Result<()> {
    let tx = c.transaction()?;
    for r in runs {
        tx.execute("INSERT INTO backup_runs(id,started_at,created_at,kind,path,status,schema_version,size_bytes,sha256,validation,error_summary) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(id) DO UPDATE SET status=excluded.status,schema_version=excluded.schema_version,size_bytes=excluded.size_bytes,sha256=excluded.sha256,validation=excluded.validation,error_summary=excluded.error_summary",params![r.id,r.started_at,r.created_at,r.kind,r.path,r.status,r.schema_version,r.size_bytes.as_ref().map(|n|n.parse::<i64>()).transpose()?,r.sha256,r.validation,r.error_summary])?;
    }
    // Destination belongs to this machine, not the historical snapshot's machine.
    tx.execute("UPDATE backup_settings SET destination=?1,automatic_enabled=?2,retention_count=?3,saved_generation=-1,last_success_at=0,last_attempt_at=0 WHERE id=1",params![saved_policy.destination,saved_policy.enabled,saved_policy.retention])?;
    tx.commit()?;
    Ok(())
}
fn reopen(storage: &mut Storage) -> Result<()> {
    let temporary = configured(Connection::open_in_memory()?)?;
    let old = std::mem::replace(&mut storage.connection, temporary);
    old.close().map_err(|(_, e)| e)?;
    let c = configured(Connection::open(&storage.path)?)?;
    c.pragma_update(None, "journal_mode", "WAL")?;
    c.pragma_update(None, "synchronous", "FULL")?;
    storage.connection = c;
    validate(&storage.connection)?;
    Ok(())
}
impl Storage {
    pub fn backup_status(&self) -> Result<BackupStatus> {
        let p = policy(&self.connection)?;
        let all = all_runs(&self.connection)?;
        let last = all
            .iter()
            .find(|r| r.status == "SUCCESS" && ["MANUAL", "AUTOMATIC"].contains(&r.kind.as_str()))
            .map(|r| r.created_at.clone());
        Ok(BackupStatus {
            destination: p.destination.clone(),
            automatic_enabled: p.enabled,
            retention_count: p.retention,
            notice: if self.recovery_blocked {
                "Recovery needs attention. Database writes are blocked.".into()
            } else if p.destination.is_none() {
                "Choose a backup folder to configure backup protection. A copy on the same physical drive does not protect against drive failure.".into()
            } else if !p.enabled {
                "Automatic backups are off. Use Backup Now to save a copy. Copies on the same drive do not protect against drive failure.".into()
            } else {
                "Automatic backups run while this app is open. Copies on the same drive do not protect against drive failure; a synced-folder save does not confirm cloud upload.".into()
            },
            last_successful_backup: last,
            runs: all.into_iter().take(100).collect(),
        })
    }
    pub(crate) fn configure_backup_destination(&mut self, path: &Path) -> Result<BackupStatus> {
        let destination = fs::canonicalize(path)?;
        let app_root = fs::canonicalize(&self.app_data)?;
        if !destination.is_dir() || destination.starts_with(app_root) {
            return Err(
                "Choose a backup folder outside the application's live-data directory".into(),
            );
        }
        // Probe the chosen directory with an app-created temporary file, never an existing file.
        let probe = NamedTempFile::new_in(&destination)?;
        probe.as_file().sync_all()?;
        self.connection.execute("UPDATE backup_settings SET destination=?1,saved_generation=-1,last_success_at=0,last_attempt_at=0 WHERE id=1",[destination.to_string_lossy().as_ref()])?;
        self.backup_status()
    }
    pub fn set_automatic_backup(&mut self, enabled: bool) -> Result<BackupStatus> {
        self.connection.execute(
            "UPDATE backup_settings SET automatic_enabled=?1 WHERE id=1",
            [enabled],
        )?;
        self.backup_status()
    }
    pub fn backup_now(&mut self) -> Result<BackupStatus> {
        self.make_backup("MANUAL", now(), None)?;
        self.backup_status()
    }
    pub(crate) fn automatic_backup(&mut self, seconds: i64) -> Result<bool> {
        if self.recovery_blocked {
            return Ok(false);
        }
        let p = policy(&self.connection)?;
        if !p.enabled
            || p.destination.is_none()
            || p.saved == p.generation
            || (p.success != 0 && seconds.saturating_sub(p.success) < DAY)
            || (p.attempted != 0 && seconds.saturating_sub(p.attempted) < 3600)
        {
            return Ok(false);
        }
        self.connection.execute(
            "UPDATE backup_settings SET last_attempt_at=?1 WHERE id=1",
            [seconds],
        )?;
        self.make_backup("AUTOMATIC", seconds, None)?;
        Ok(true)
    }
    fn make_backup(
        &mut self,
        kind: &str,
        seconds: i64,
        override_directory: Option<&Path>,
    ) -> Result<PathBuf> {
        if self.recovery_blocked {
            return Err("Recovery is pending; writes are blocked".into());
        }
        let p = policy(&self.connection)?;
        let directory = override_directory
            .map(Path::to_path_buf)
            .or_else(|| p.destination.map(PathBuf::from))
            .ok_or("Choose a backup folder first")?;
        let id = Uuid::new_v4().to_string();
        let timestamp: String = self.connection.query_row(
            "SELECT strftime('%Y-%m-%d-%H%M%S',?1,'unixepoch')",
            [seconds],
            |r| r.get(0),
        )?;
        let target = directory.join(format!("twx-payroll-{timestamp}-{id}.sqlite3"));
        new_run(&self.connection, &id, kind, &target, seconds)?;
        let result = (|| -> Result<()> {
            if fs::canonicalize(&directory)? != directory {
                return Err("Backup folder moved; select it again".into());
            }
            validate(&self.connection)?;
            let staged = tempfile::Builder::new()
                .prefix(".twx-partial-")
                .tempfile_in(&directory)?;
            snapshot(&self.connection, staged.path())?;
            let size = fs::metadata(staged.path())?.len();
            let hash = checksum(staged.path())?;
            let manifest = Manifest {
                application: APP.into(),
                id: id.clone(),
                filename: target
                    .file_name()
                    .ok_or("Missing filename")?
                    .to_string_lossy()
                    .into_owned(),
                kind: kind.into(),
                created_at: label(&self.connection, seconds)?,
                schema_version: VERSION,
                size_bytes: size,
                sha256: hash.clone(),
            };
            staged.persist_noclobber(&target).map_err(|e| e.error)?;
            publish_json(&manifest_path(&target), &manifest)?;
            validate_file(&target)?;
            let tx = self.connection.transaction()?;
            tx.execute("UPDATE backup_runs SET status='SUCCESS',schema_version=?2,size_bytes=?3,sha256=?4,validation='Integrity, foreign keys, schema and checksum verified' WHERE id=?1",params![id,VERSION,i64::try_from(size)?,hash])?;
            if kind != "PRE_RESTORE" {
                tx.execute("UPDATE backup_settings SET saved_generation=change_generation,last_success_at=?1,last_attempt_at=?1 WHERE id=1",[seconds])?;
            }
            tx.commit()?;
            Ok(())
        })();
        if let Err(error) = result {
            finish_run(&self.connection, &id, "FAILED", Some(&error.to_string()))?;
            return Err(error);
        }
        if kind == "AUTOMATIC" {
            if let Err(error) = self.retain_automatic(&directory) {
                self.connection.execute(
                    "UPDATE backup_runs SET error_summary=?2 WHERE id=?1",
                    params![
                        id,
                        format!("Backup complete; retention needs attention: {error}")
                    ],
                )?;
            }
        }
        Ok(target)
    }
    fn retain_automatic(&mut self, directory: &Path) -> Result<()> {
        let keep = usize::try_from(policy(&self.connection)?.retention)?;
        let mut recognized = Vec::new();
        for run in all_runs(&self.connection)? {
            let path = PathBuf::from(&run.path);
            if run.kind != "AUTOMATIC"
                || run.status != "SUCCESS"
                || path.parent() != Some(directory)
                || !path.exists()
            {
                continue;
            }
            if let Ok(m) = read_manifest(&manifest_path(&path)) {
                if m.application == APP
                    && m.kind == "AUTOMATIC"
                    && m.id == run.id
                    && path.file_name().and_then(|f| f.to_str()) == Some(&m.filename)
                    && m.filename.starts_with("twx-payroll-")
                    && m.filename.ends_with(&format!("-{}.sqlite3", m.id))
                    && run.sha256.as_deref() == Some(&m.sha256)
                    && validate_file(&path).is_ok()
                {
                    recognized.push(path);
                }
            }
        }
        for path in recognized.into_iter().skip(keep) {
            fs::remove_file(&path)?;
            fs::remove_file(manifest_path(&path))?;
            self.connection.execute("UPDATE backup_runs SET validation='Verified at creation; automatic generation removed by retention' WHERE path=?1",[path.to_string_lossy().as_ref()])?;
        }
        Ok(())
    }
    pub(crate) fn prepare_restore(&mut self, path: &Path) -> Result<RestorePreview> {
        match self.prepare_restore_candidate(path) {
            Ok(preview) => Ok(preview),
            Err(error) => {
                let id = Uuid::new_v4().to_string();
                new_run(&self.connection, &id, "RESTORE", path, now())?;
                finish_run(
                    &self.connection,
                    &id,
                    "FAILED",
                    Some(&format!(
                        "Selected backup rejected before live data changed: {error}"
                    )),
                )?;
                Err(error)
            }
        }
    }
    fn prepare_restore_candidate(&mut self, path: &Path) -> Result<RestorePreview> {
        if self.recovery_blocked {
            return Err("Resolve pending recovery before selecting another restore".into());
        }
        let path = fs::canonicalize(path)?;
        if path == fs::canonicalize(&self.path)? {
            return Err("Select a backup, not the active database".into());
        }
        if path
            .extension()
            .and_then(|e| e.to_str())
            .is_none_or(|e| !e.eq_ignore_ascii_case("sqlite3"))
        {
            return Err("Choose a .sqlite3 backup file".into());
        }
        let version = validate_file(&path)?;
        let directory = self.app_data.join("recovery");
        fs::create_dir_all(&directory)?;
        let token = Uuid::new_v4().to_string();
        let candidate = directory.join(format!("candidate-{token}.sqlite3"));
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)?;
        snapshot(&read_only(&path)?, &candidate)?;
        let mut staged = configured(Connection::open(&candidate)?)?;
        MIGRATIONS.to_latest(&mut staged)?;
        validate(&staged)?;
        drop(staged);
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&candidate)?
            .sync_all()?;
        let created_at = read_manifest(&manifest_path(&path))
            .ok()
            .map(|m| m.created_at);
        let preview = RestorePreview {
            token,
            filename: path
                .file_name()
                .ok_or("Missing selected filename")?
                .to_string_lossy()
                .into_owned(),
            schema_version: version,
            created_at,
        };
        self.cancel_restore()?;
        self.prepared_restore = Some(PreparedRestore {
            preview: preview.clone(),
            checksum: checksum(&candidate)?,
            path: candidate,
        });
        Ok(preview)
    }
    pub fn cancel_restore(&mut self) -> Result<()> {
        if let Some(p) = self.prepared_restore.take() {
            fs::remove_file(p.path)?;
        }
        Ok(())
    }
    pub fn restore_backup(&mut self, token: &str, confirmed: bool) -> Result<BackupStatus> {
        if !confirmed {
            return Err("Restore requires explicit confirmation".into());
        }
        self.restore_inner(token, false)?;
        self.backup_status()
    }
    fn restore_inner(&mut self, token: &str, fail_after_apply: bool) -> Result<()> {
        let prepared = self
            .prepared_restore
            .as_ref()
            .ok_or("Select and validate a backup first")?;
        if prepared.preview.token != token {
            return Err("Restore selection expired; select the backup again".into());
        }
        if checksum(&prepared.path)? != prepared.checksum {
            return Err("Prepared restore changed; select the backup again".into());
        }
        validate_file(&prepared.path)?;
        let candidate = self
            .prepared_restore
            .take()
            .ok_or("Missing prepared restore")?;
        let directory = fs::canonicalize(self.app_data.join("recovery"))?;
        let safety = self.make_backup("PRE_RESTORE", now(), Some(&directory))?;
        let saved_policy = policy(&self.connection)?;
        let id = Uuid::new_v4().to_string();
        new_run(&self.connection, &id, "RESTORE", &candidate.path, now())?;
        let history = all_runs(&self.connection)?;
        let marker = RecoveryMarker {
            application: APP.into(),
            operation_id: id.clone(),
            safety_filename: safety
                .file_name()
                .ok_or("Missing safety name")?
                .to_string_lossy()
                .into_owned(),
            safety_sha256: checksum(&safety)?,
            candidate_filename: candidate
                .path
                .file_name()
                .ok_or("Missing candidate name")?
                .to_string_lossy()
                .into_owned(),
        };
        let marker_path = directory.join("restore-pending.json");
        if let Err(error) = publish_json(&marker_path, &marker) {
            finish_run(
                &self.connection,
                &id,
                "FAILED",
                Some(&format!("Recovery marker could not be persisted: {error}")),
            )?;
            return Err(error);
        }
        let outcome = (|| -> Result<()> {
            transfer(&read_only(&candidate.path)?, &mut self.connection)?;
            reopen(self)?;
            if fail_after_apply {
                return Err("Synthetic post-restore validation failure".into());
            }
            merge_history(&mut self.connection, &history, &saved_policy)?;
            finish_run(&self.connection, &id, "SUCCESS", None)?;
            self.connection.execute("UPDATE backup_runs SET schema_version=?2,validation='Restored, reopened and validated' WHERE id=?1",params![&id,VERSION])?;
            Ok(())
        })();
        if let Err(error) = outcome {
            let rollback = (|| -> Result<()> {
                // Reopen a failed handle if necessary before applying the intact safety snapshot.
                self.connection = configured(Connection::open(&self.path)?)?;
                transfer(&read_only(&safety)?, &mut self.connection)?;
                reopen(self)?;
                merge_history(&mut self.connection, &history, &saved_policy)?;
                finish_run(
                    &self.connection,
                    &id,
                    "ROLLED_BACK",
                    Some(&error.to_string()),
                )?;
                fs::remove_file(&marker_path)?;
                Ok(())
            })();
            if let Err(rollback_error) = rollback {
                self.recovery_blocked = true;
                return Err(format!("Restore failed ({error}); recovery pending ({rollback_error}). Safety copy retained. Restart to recover.").into());
            }
            return Err(format!("Restore did not complete: {error}. Previous database restored; candidate and safety copy retained.").into());
        }
        // If cleanup fails, keep the guard: startup will conservatively recover the safety copy.
        if let Err(error) = fs::remove_file(&marker_path) {
            self.recovery_blocked = true;
            return Err(format!("Restore data verified but recovery marker cleanup failed: {error}. Restart to recover safely.").into());
        }
        Ok(())
    }
}

/// Runs before ordinary storage is exposed. Paths come only from an app-owned marker and validated basenames.
pub(crate) fn recover_pending(app_data: &Path, live: &Path) -> Result<()> {
    let directory = app_data.join("recovery");
    let marker_path = directory.join("restore-pending.json");
    if !marker_path.exists() {
        return Ok(());
    }
    if fs::metadata(&marker_path)?.len() > 65536 {
        return Err("Invalid recovery marker; manual recovery required".into());
    }
    let marker: RecoveryMarker = serde_json::from_slice(&fs::read(&marker_path)?)?;
    if marker.application != APP
        || Path::new(&marker.safety_filename)
            .file_name()
            .and_then(|s| s.to_str())
            != Some(&marker.safety_filename)
        || !marker.safety_filename.starts_with("twx-payroll-")
    {
        return Err("Invalid recovery marker paths".into());
    }
    let safety = directory.join(&marker.safety_filename);
    if checksum(&safety)? != marker.safety_sha256 {
        return Err("Safety snapshot checksum mismatch; existing files preserved".into());
    }
    validate_file(&safety)?;
    let source = read_only(&safety)?;
    let mut destination = configured(Connection::open(live)?)?;
    transfer(&source, &mut destination)?;
    validate(&destination)?;
    destination.execute("UPDATE backup_runs SET status='INTERRUPTED',error_summary='Interrupted operation; safety database recovered' WHERE status='RUNNING'",[])?;
    let completed = read_manifest(&manifest_path(&safety))?;
    destination.execute("UPDATE backup_runs SET status='SUCCESS',schema_version=?2,size_bytes=?3,sha256=?4,validation='Safety snapshot validated during startup recovery',error_summary=NULL WHERE id=?1",params![completed.id,completed.schema_version,i64::try_from(completed.size_bytes)?,completed.sha256])?;
    new_run(
        &destination,
        &marker.operation_id,
        "RESTORE",
        &safety,
        now(),
    )?;
    finish_run(
        &destination,
        &marker.operation_id,
        "ROLLED_BACK",
        Some("Recovered pre-restore database after interrupted restore; candidate retained"),
    )?;
    drop(destination);
    fs::remove_file(marker_path)?;
    Ok(())
}

#[cfg(test)]
mod tests;
