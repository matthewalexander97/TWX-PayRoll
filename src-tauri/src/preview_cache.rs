//! Bounded private print copies. Old files are removed only with exclusive Windows delete access.
use crate::reports::Result;
use std::{
    path::Path,
    time::{Duration, SystemTime},
};
const LIMIT: u64 = 100 * 1024 * 1024;
const COUNT: usize = 100;
fn owned(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    path.extension().and_then(|s| s.to_str()) == Some("pdf")
        && stem.len() >= 37
        && stem.as_bytes()[stem.len() - 37] == b'-'
        && stem
            .get(stem.len() - 36..)
            .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
}
#[cfg(windows)]
fn remove_closed(path: &Path) -> std::io::Result<()> {
    use std::os::windows::fs::OpenOptionsExt;
    // DELETE access + zero sharing fails if ANY existing file handle is open.
    // DELETE_ON_CLOSE deletes under that same exclusive handle, avoiding a probe/delete race.
    let handle = std::fs::OpenOptions::new()
        .access_mode(0x00010000)
        .share_mode(0)
        .custom_flags(0x04000000)
        .open(path)?;
    drop(handle);
    Ok(())
}
#[cfg(not(windows))]
fn remove_closed(_: &Path) -> std::io::Result<()> {
    Err(std::io::Error::other("Exclusive cleanup requires Windows"))
}
pub fn reserve(folder: &Path, incoming: u64) -> Result<()> {
    let now = SystemTime::now();
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if entry.file_type()?.is_file()
            && owned(&entry.path())
            && now.duration_since(meta.modified()?).unwrap_or_default()
                > Duration::from_secs(7 * 86400)
        {
            let _ = remove_closed(&entry.path()); // Locked files remain and count toward quota.
        }
    }
    let mut count = 0;
    let mut size: u64 = 0;
    for entry in std::fs::read_dir(folder)? {
        let meta = entry?.metadata()?;
        count += 1;
        size = size.saturating_add(meta.len());
    }
    if count >= COUNT || size.saturating_add(incoming) > LIMIT {
        return Err("Print preview cache is full (100 files / 100 MiB). Close PDF viewers, then remove unneeded PDFs from report-previews beside the data folder shown in Settings. Export PDF remains available.".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_growth_and_preserves_recent_or_unrecognized_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("unrelated.pdf"), b"keep").unwrap();
        assert!(reserve(dir.path(), LIMIT).is_err());
        assert!(reserve(dir.path(), 1).is_ok());
        for n in 1..COUNT {
            std::fs::write(dir.path().join(format!("{n}.pdf")), b"x").unwrap();
        }
        assert!(reserve(dir.path(), 1).is_err());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), COUNT);
    }
    #[cfg(windows)]
    #[test]
    fn old_closed_files_removed_but_open_files_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(format!("TWX-{}.pdf", uuid::Uuid::new_v4()));
        std::fs::write(&file, b"preview").unwrap();
        let handle = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&file)
            .unwrap();
        handle
            .set_times(
                std::fs::FileTimes::new()
                    .set_modified(SystemTime::now() - Duration::from_secs(8 * 86400)),
            )
            .unwrap();
        reserve(dir.path(), 1).unwrap();
        assert!(file.exists());
        drop(handle);
        reserve(dir.path(), 1).unwrap();
        assert!(!file.exists());
    }
}
