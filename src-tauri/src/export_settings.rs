//! Machine-local export preference, deliberately independent of business backups.
use crate::reports::{files, Format, Result};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

pub struct ExportState(pub Mutex<()>);
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub directory: String,
    pub uses_documents: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Preference {
    directory: PathBuf,
}

fn config(app: &Path) -> PathBuf {
    app.join("export-directory.json")
}
pub fn directory(
    app: &Path,
    documents: impl FnOnce() -> Result<PathBuf>,
) -> Result<(PathBuf, bool)> {
    match std::fs::File::open(config(app)) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take(32769).read_to_end(&mut bytes)?;
            if bytes.len() > 32768 {
                return Err("Export setting is invalid. Choose another folder or Reset to Documents in Settings.".into());
            }
            let preference: Preference = serde_json::from_slice(&bytes)?;
            Ok((preference.directory, false))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok((
            documents().map_err(|_| {
                "Windows Documents could not be resolved. Choose an export folder in Settings."
            })?,
            true,
        )),
        Err(e) => Err(e.into()),
    }
}
pub fn status(app: &Path, documents: impl FnOnce() -> Result<PathBuf>) -> Result<Status> {
    let (path, uses_documents) = directory(app, documents)?;
    Ok(Status {
        directory: path.to_string_lossy().into(),
        uses_documents,
    })
}
pub fn validate_folder(path: &Path, app: &Path) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err("Choose an absolute export folder in Settings.".into());
    }
    let path = std::fs::canonicalize(path).map_err(|_| "Export folder is unavailable. Reconnect the drive, Change Folder, or Reset to Documents in Settings.")?;
    if !path.is_dir() || path.starts_with(std::fs::canonicalize(app)?) {
        return Err("Choose an existing export folder outside application data.".into());
    }
    Ok(path)
}
pub fn save(app: &Path, selected: &Path) -> Result<()> {
    let path = validate_folder(selected, app)?;
    // Probe only an explicitly selected directory; never create missing folders or fall back.
    tempfile::NamedTempFile::new_in(&path)
        .map_err(|_| "Export folder is not writable. Choose another folder in Settings.")?;
    let bytes = serde_json::to_vec(&Preference { directory: path })?;
    let mut stage = tempfile::NamedTempFile::new_in(app)?;
    stage.write_all(&bytes)?;
    stage.as_file().sync_all()?;
    stage.persist(config(app)).map_err(|e| e.error)?;
    Ok(())
}
pub fn reset(app: &Path, documents: impl FnOnce() -> Result<PathBuf>) -> Result<()> {
    let resolved = documents().map_err(|_| {
        "Windows Documents could not be resolved. Choose an export folder in Settings."
    })?;
    let resolved = validate_folder(&resolved, app)?;
    tempfile::NamedTempFile::new_in(resolved)
        .map_err(|_| "Documents is not writable. Choose another export folder in Settings.")?;
    match std::fs::remove_file(config(app)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}
pub fn publish(
    app: &Path,
    documents: impl FnOnce() -> Result<PathBuf>,
    name: &str,
    format: Format,
    bytes: &[u8],
) -> Result<PathBuf> {
    let (folder, _) = directory(app, documents)?;
    let folder = validate_folder(&folder, app)?;
    files::publish_numbered(&folder, name, format, bytes, app)
        .map_err(|e| format!("Export was not saved: {e}. Change Folder or Reset to Documents in Settings; no fallback was used.").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_preference_and_formats_preserve_finalized_finances_and_backup_destination() {
        let (app, mut storage, id) = crate::reports::tests::fixture();
        let backup = tempfile::tempdir().unwrap();
        let exports = tempfile::tempdir().unwrap();
        storage.configure_backup_destination(backup.path()).unwrap();
        let fingerprint = |s: &crate::Storage| {
            let financial: String = s
                .connection
                .query_row(
                    "SELECT json_group_array(snapshot_json) FROM weekly_settlements",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            let audit: i64 = s
                .connection
                .query_row("SELECT count(*) FROM audit_events", [], |r| r.get(0))
                .unwrap();
            (financial, audit, s.backup_status().unwrap().destination)
        };
        let before = fingerprint(&storage);
        save(app.path(), exports.path()).unwrap();
        let request: crate::reports::Request = serde_json::from_value(
            serde_json::json!({"kind":"settlement_detail","settlementId":id}),
        )
        .unwrap();
        let report = crate::reports::build(&storage, &request).unwrap();
        for format in [Format::Pdf, Format::Xlsx, Format::Csv, Format::Docx] {
            let bytes = crate::reports::render(&report, format).unwrap();
            publish(
                app.path(),
                || Err("must not resolve Documents for custom destination".into()),
                &report.default_name,
                format,
                &bytes,
            )
            .unwrap();
        }
        reset(app.path(), || Ok(exports.path().to_path_buf())).unwrap();
        assert_eq!(before, fingerprint(&storage));
    }
    #[test]
    fn documents_custom_restart_reset_and_no_business_changes() {
        let app = tempfile::tempdir().unwrap();
        let docs = tempfile::tempdir().unwrap();
        let custom = tempfile::tempdir().unwrap();
        let storage = crate::Storage::open(app.path()).unwrap();
        let before: String = storage
            .connection
            .query_row(
                "SELECT json_group_array(name) FROM sqlite_schema",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let resolver = || Ok(docs.path().to_path_buf());
        assert!(status(app.path(), resolver).unwrap().uses_documents);
        save(app.path(), custom.path()).unwrap();
        drop(storage);
        let storage = crate::Storage::open(app.path()).unwrap();
        assert_eq!(
            directory(app.path(), resolver).unwrap().0,
            std::fs::canonicalize(custom.path()).unwrap()
        );
        assert!(storage.backup_status().unwrap().destination.is_none());
        let first = publish(app.path(), resolver, "TWX-Test", Format::Pdf, b"first").unwrap();
        let second = publish(app.path(), resolver, "TWX-Test", Format::Pdf, b"second").unwrap();
        assert_eq!(second.file_name().unwrap(), "TWX-Test-2.pdf");
        assert_eq!(std::fs::read(first).unwrap(), b"first");
        // Save As is one-off publication, independent of the setting.
        files::publish(
            &docs.path().join("one-off.pdf"),
            Format::Pdf,
            b"one-off",
            app.path(),
        )
        .unwrap();
        assert!(!status(app.path(), resolver).unwrap().uses_documents);
        reset(app.path(), resolver).unwrap();
        assert!(status(app.path(), resolver).unwrap().uses_documents);
        assert!(
            publish(app.path(), resolver, "Default", Format::Csv, b"csv")
                .unwrap()
                .starts_with(std::fs::canonicalize(docs.path()).unwrap())
        );
        let after: String = storage
            .connection
            .query_row(
                "SELECT json_group_array(name) FROM sqlite_schema",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(before, after);
        assert_eq!(
            storage
                .connection
                .query_row("SELECT count(*) FROM audit_events", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn failures_never_fall_back_or_open_arbitrary_paths() {
        let app = tempfile::tempdir().unwrap();
        let docs = tempfile::tempdir().unwrap();
        let gone = tempfile::tempdir().unwrap();
        assert!(directory(app.path(), || Err("resolver unavailable".into())).is_err());
        assert!(validate_folder(app.path(), app.path()).is_err());
        assert!(validate_folder(Path::new("relative"), app.path()).is_err());
        save(app.path(), gone.path()).unwrap();
        gone.close().unwrap();
        assert!(publish(
            app.path(),
            || Ok(docs.path().to_path_buf()),
            "x",
            Format::Csv,
            b"x"
        )
        .is_err());
        assert_eq!(std::fs::read_dir(docs.path()).unwrap().count(), 0);
        assert!(reset(app.path(), || Err("unavailable".into())).is_err());
        assert!(config(app.path()).exists());
        std::fs::write(config(app.path()), b"invalid").unwrap();
        assert!(status(app.path(), || Ok(docs.path().to_path_buf())).is_err());
        reset(app.path(), || Ok(docs.path().to_path_buf())).unwrap();
    }
}
