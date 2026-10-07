use super::*;
use crate::{
    repositories::Catalog,
    storage::{database_path, Theme},
};
fn setup() -> (tempfile::TempDir, tempfile::TempDir, Storage) {
    let app = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let mut db = Storage::open(app.path()).unwrap();
    db.configure_backup_destination(dest.path()).unwrap();
    (app, dest, db)
}
fn owner(db: &mut Storage, id: &str) {
    db.catalogs()
        .save(
            id,
            None,
            true,
            Catalog::Owner {
                name: "Synthetic Backup Owner",
            },
        )
        .unwrap();
}
fn owner_count(c: &Connection) -> i64 {
    c.query_row("SELECT count(*) FROM owners", [], |r| r.get(0))
        .unwrap()
}
fn files(path: &Path) -> Vec<PathBuf> {
    fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "sqlite3"))
        .collect()
}
#[test]
fn initialized_backup_is_valid_unique_and_generational() {
    let (_app, dest, mut db) = setup();
    db.backup_now().unwrap();
    db.backup_now().unwrap();
    let backups = files(dest.path());
    assert_eq!(backups.len(), 2);
    assert_ne!(backups[0], backups[1]);
    for file in backups {
        assert_eq!(validate_file(&file).unwrap(), 4);
        assert!(manifest_path(&file).exists());
    }
    let status = db.backup_status().unwrap();
    assert_eq!(status.runs.len(), 2);
    assert!(status
        .runs
        .iter()
        .all(|r| r.status == "SUCCESS" && r.schema_version == Some(4) && r.sha256.is_some()));
}
#[test]
fn wal_snapshot_contains_committed_data_and_not_uncommitted_data() {
    let (_app, dest, mut db) = setup();
    owner(&mut db, "committed");
    db.connection
        .pragma_update(None, "wal_autocheckpoint", 0)
        .unwrap();
    let mut writer = Connection::open(&db.path).unwrap();
    let tx = writer.transaction().unwrap();
    tx.execute(
        "INSERT INTO owners(id,name) VALUES('uncommitted','Synthetic Pending')",
        [],
    )
    .unwrap();
    let target = dest.path().join("isolated.sqlite3");
    snapshot(&db.connection, &target).unwrap();
    let copy = read_only(&target).unwrap();
    assert_eq!(owner_count(&copy), 1);
    assert_eq!(
        db.connection
            .pragma_query_value(None, "journal_mode", |r| r.get::<_, String>(0))
            .unwrap(),
        "wal"
    );
    tx.rollback().unwrap();
}
#[test]
fn automatic_policy_skips_trivial_changes_and_records_metadata() {
    let (_app, _dest, mut db) = setup();
    assert!(db.automatic_backup(1_800_000_000).unwrap());
    assert!(!db.automatic_backup(1_800_000_001).unwrap());
    db.set_theme(Theme::Dark).unwrap();
    assert!(!db.automatic_backup(1_800_086_401).unwrap());
    owner(&mut db, "changed");
    assert!(db.automatic_backup(1_800_086_402).unwrap());
    let status = db.backup_status().unwrap();
    assert_eq!(status.runs.len(), 2);
    assert!(status
        .runs
        .iter()
        .all(|r| r.kind == "AUTOMATIC" && r.status == "SUCCESS"));
    db.set_automatic_backup(false).unwrap();
    assert!(db.backup_status().unwrap().notice.contains("are off"));
    owner(&mut db, "disabled");
    assert!(!db.automatic_backup(1_800_999_999).unwrap());
}
#[test]
fn missing_destination_is_explicit_and_never_claims_protection() {
    let app = tempfile::tempdir().unwrap();
    let mut db = Storage::open(app.path()).unwrap();
    assert!(!db.automatic_backup(now()).unwrap());
    assert!(db.backup_now().is_err());
    let status = db.backup_status().unwrap();
    assert!(status.notice.contains("Choose a backup folder"));
    assert!(status.last_successful_backup.is_none());
    assert!(db
        .configure_backup_destination(&app.path().join("data"))
        .is_err());
}
#[test]
fn retention_keeps_manual_unrecognized_and_recent_automatic_generations() {
    let (_app, dest, mut db) = setup();
    db.connection
        .execute("UPDATE backup_settings SET retention_count=2", [])
        .unwrap();
    let manual = db.make_backup("MANUAL", 1_800_000_000, None).unwrap();
    let unrelated = dest.path().join("twx-payroll-unrelated.sqlite3");
    fs::write(&unrelated, b"unrelated file").unwrap();
    for n in 1..5 {
        owner(&mut db, &format!("o{n}"));
        assert!(db.automatic_backup(1_800_000_000 + n * 86401).unwrap());
    }
    assert!(manual.exists());
    assert!(unrelated.exists());
    assert_eq!(files(dest.path()).len(), 4);
    let rows = all_runs(&db.connection).unwrap();
    assert_eq!(
        rows.iter()
            .filter(|r| r.validation.contains("removed by retention"))
            .count(),
        2
    );
}
#[test]
fn failed_and_inaccessible_destinations_are_not_successful() {
    let (_app, dest, mut db) = setup();
    let missing = dest.path().join("missing");
    db.connection
        .execute(
            "UPDATE backup_settings SET destination=?1",
            [missing.to_string_lossy().as_ref()],
        )
        .unwrap();
    assert!(db.automatic_backup(1_800_000_000).is_err());
    assert!(!db.automatic_backup(1_800_000_001).unwrap());
    assert_eq!(db.backup_status().unwrap().runs[0].status, "FAILED");
    let file = dest.path().join("not-a-folder");
    fs::write(&file, b"synthetic").unwrap();
    assert!(db.configure_backup_destination(&file).is_err());
    assert!(db.backup_status().unwrap().last_successful_backup.is_none());
}
#[test]
fn rejects_corrupt_unrelated_future_and_invalid_foreign_keys() {
    let (_app, dest, mut db) = setup();
    let bad = dest.path().join("corrupt.sqlite3");
    fs::write(&bad, b"not a database").unwrap();
    assert!(db.prepare_restore(&bad).is_err());
    let other = dest.path().join("other.sqlite3");
    let c = Connection::open(&other).unwrap();
    c.execute_batch("CREATE TABLE unrelated(id INTEGER); PRAGMA user_version=3; PRAGMA application_id=1415002192;").unwrap();
    drop(c);
    assert!(db.prepare_restore(&other).is_err());
    let future = dest.path().join("future.sqlite3");
    snapshot(&db.connection, &future).unwrap();
    let c = Connection::open(&future).unwrap();
    c.pragma_update(None, "user_version", 99).unwrap();
    drop(c);
    assert!(db.prepare_restore(&future).is_err());
    let broken = dest.path().join("fk.sqlite3");
    snapshot(&db.connection, &broken).unwrap();
    let c = Connection::open(&broken).unwrap();
    c.pragma_update(None, "foreign_keys", false).unwrap();
    c.execute(
        "INSERT INTO drivers(id,name,driver_type_id) VALUES('broken','Synthetic','absent')",
        [],
    )
    .unwrap();
    drop(c);
    assert!(db.prepare_restore(&broken).is_err());
    assert!(db.prepare_restore(&db.path.clone()).is_err());
    assert_eq!(owner_count(&db.connection), 0);
}
#[test]
fn successful_restore_preserves_backup_data_history_and_safety_copy() {
    let (_app, dest, mut db) = setup();
    owner(&mut db, "original");
    db.set_theme(Theme::Dark).unwrap();
    let saved = db.make_backup("MANUAL", now(), None).unwrap();
    owner(&mut db, "later");
    db.set_theme(Theme::Light).unwrap();
    let preview = db.prepare_restore(&saved).unwrap();
    assert!(db.restore_backup(&preview.token, false).is_err());
    assert!(db.restore_backup("wrong-token", true).is_err());
    let status = db.restore_backup(&preview.token, true).unwrap();
    assert_eq!(owner_count(&db.connection), 1);
    assert_eq!(db.preferences().unwrap().theme, Theme::Dark);
    assert_eq!(
        status.destination,
        Some(
            fs::canonicalize(dest.path())
                .unwrap()
                .to_string_lossy()
                .into_owned()
        )
    );
    let safety = status
        .runs
        .iter()
        .find(|r| r.kind == "PRE_RESTORE")
        .unwrap();
    assert_eq!(safety.status, "SUCCESS");
    assert_eq!(owner_count(&read_only(Path::new(&safety.path)).unwrap()), 2);
    assert!(status
        .runs
        .iter()
        .any(|r| r.kind == "RESTORE" && r.status == "SUCCESS"));
    assert!(status.runs.iter().any(|r| r.kind == "MANUAL"));
    assert!(db.restore_backup(&preview.token, true).is_err());
    assert!(!db.app_data.join("recovery/restore-pending.json").exists());
}
#[test]
fn post_apply_validation_failure_rolls_back_and_retains_candidate() {
    let (_app, _dest, mut db) = setup();
    owner(&mut db, "before");
    let saved = db.make_backup("MANUAL", now(), None).unwrap();
    owner(&mut db, "current");
    let preview = db.prepare_restore(&saved).unwrap();
    assert!(db.restore_inner(&preview.token, true).is_err());
    assert_eq!(owner_count(&db.connection), 2);
    assert!(!db.recovery_blocked);
    assert!(db
        .backup_status()
        .unwrap()
        .runs
        .iter()
        .any(|r| r.status == "ROLLED_BACK"));
    assert!(db
        .app_data
        .join(format!("recovery/candidate-{}.sqlite3", preview.token))
        .exists());
}
#[test]
fn restore_old_schema_migrates_only_staged_copy() {
    let (_app, dest, mut db) = setup();
    let legacy = dest.path().join("legacy.sqlite3");
    let mut c = Connection::open(&legacy).unwrap();
    MIGRATIONS.to_version(&mut c, 1).unwrap();
    c.execute("UPDATE app_settings SET value='dark'", [])
        .unwrap();
    drop(c);
    let preview = db.prepare_restore(&legacy).unwrap();
    assert_eq!(preview.schema_version, 1);
    db.restore_backup(&preview.token, true).unwrap();
    assert_eq!(db.info().unwrap().schema_version, 4);
    assert_eq!(db.preferences().unwrap().theme, Theme::Dark);
    assert_eq!(validate_file(&legacy).unwrap(), 1);
}
#[test]
fn restore_phase3_business_snapshot_migrates_without_rewriting_source() {
    let (_app, dest, mut db) = setup();
    let legacy = dest.path().join("phase3.sqlite3");
    let mut c = configured(Connection::open(&legacy).unwrap()).unwrap();
    MIGRATIONS.to_version(&mut c, 3).unwrap();
    c.execute(
        "INSERT INTO owners(id,name) VALUES('legacy-owner','Synthetic legacy owner')",
        [],
    )
    .unwrap();
    drop(c);
    let original = fs::read(&legacy).unwrap();
    let preview = db.prepare_restore(&legacy).unwrap();
    assert_eq!(preview.schema_version, 3);
    db.restore_backup(&preview.token, true).unwrap();
    assert_eq!(db.info().unwrap().schema_version, 4);
    assert_eq!(owner_count(&db.connection), 1);
    assert_eq!(fs::read(&legacy).unwrap(), original);
    assert_eq!(validate_file(&legacy).unwrap(), 3);
}
#[test]
fn interrupted_operation_history_and_pending_restore_recover_on_startup() {
    let (app, _dest, mut db) = setup();
    owner(&mut db, "safe");
    let directory = app.path().join("recovery");
    fs::create_dir_all(&directory).unwrap();
    let directory = fs::canonicalize(directory).unwrap();
    let safety = db
        .make_backup("PRE_RESTORE", now(), Some(&directory))
        .unwrap();
    let marker = RecoveryMarker {
        application: APP.into(),
        operation_id: Uuid::new_v4().to_string(),
        safety_filename: safety.file_name().unwrap().to_string_lossy().into_owned(),
        safety_sha256: checksum(&safety).unwrap(),
        candidate_filename: "candidate-retained.sqlite3".into(),
    };
    publish_json(&directory.join("restore-pending.json"), &marker).unwrap();
    owner(&mut db, "partially-restored");
    drop(db);
    let db = Storage::open(app.path()).unwrap();
    assert_eq!(owner_count(&db.connection), 1);
    assert!(!directory.join("restore-pending.json").exists());
    assert!(safety.exists());
    assert!(db
        .backup_status()
        .unwrap()
        .runs
        .iter()
        .any(|r| r.status == "ROLLED_BACK"));
    new_run(
        &db.connection,
        "interrupted",
        "MANUAL",
        Path::new("synthetic-partial"),
        now(),
    )
    .unwrap();
    drop(db);
    let db = Storage::open(app.path()).unwrap();
    assert!(db
        .backup_status()
        .unwrap()
        .runs
        .iter()
        .any(|r| r.id == "interrupted" && r.status == "INTERRUPTED"));
}
#[test]
fn candidate_tampering_and_bad_marker_never_replace_current_database() {
    let (app, _dest, mut db) = setup();
    owner(&mut db, "current");
    let backup = db.make_backup("MANUAL", now(), None).unwrap();
    let preview = db.prepare_restore(&backup).unwrap();
    fs::write(&db.prepared_restore.as_ref().unwrap().path, b"tampered").unwrap();
    assert!(db.restore_backup(&preview.token, true).is_err());
    assert_eq!(owner_count(&db.connection), 1);
    db.cancel_restore().unwrap();
    let directory = app.path().join("recovery");
    let marker = RecoveryMarker {
        application: APP.into(),
        operation_id: "test".into(),
        safety_filename: "../escape.sqlite3".into(),
        safety_sha256: String::new(),
        candidate_filename: String::new(),
    };
    publish_json(&directory.join("restore-pending.json"), &marker).unwrap();
    drop(db);
    assert!(Storage::open(app.path()).is_err());
    assert_eq!(
        owner_count(&read_only(&database_path(app.path())).unwrap()),
        1
    );
}
#[cfg(windows)]
#[test]
fn locked_destination_file_cannot_be_opened_as_restore_source() {
    use std::os::windows::fs::OpenOptionsExt;
    let (_app, dest, mut db) = setup();
    let path = dest.path().join("locked.sqlite3");
    snapshot(&db.connection, &path).unwrap();
    let _locked = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&path)
        .unwrap();
    assert!(db.prepare_restore(&path).is_err());
    assert_eq!(owner_count(&db.connection), 0);
}
