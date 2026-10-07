//! Rust-owned destinations, no-clobber publication, and local PDF preview.
use super::*;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
pub fn safe_filename(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_control() || "<>:\"/\\|?*".contains(c) {
                '-'
            } else {
                c
            }
        })
        .take(160)
        .collect();
    let clean = clean.trim_matches([' ', '.']).to_string();
    let stem = clean.split('.').next().unwrap_or("").to_uppercase();
    if clean.is_empty() {
        "TWX-Report".into()
    } else if [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
    .contains(&stem.as_str())
    {
        format!("TWX-{clean}")
    } else {
        clean
    }
}
pub fn publish(path: &Path, format: Format, bytes: &[u8], app_data: &Path) -> Result<PathBuf> {
    publish_with(path, format, app_data, |stage| {
        stage.write_all(bytes)?;
        Ok(())
    })
}
pub fn publish_numbered(
    folder: &Path,
    name: &str,
    format: Format,
    bytes: &[u8],
    app_data: &Path,
) -> Result<PathBuf> {
    let folder = crate::export_settings::validate_folder(folder, app_data)?;
    let name = safe_filename(name);
    let mut stage = NamedTempFile::new_in(&folder)?;
    stage.write_all(bytes)?;
    stage.as_file().sync_all()?;
    for number in 1..=10000 {
        let suffix = if number == 1 {
            String::new()
        } else {
            format!("-{number}")
        };
        let destination = folder.join(format!("{name}{suffix}.{}", format.extension()));
        match stage.persist_noclobber(&destination) {
            Ok(_) => return Ok(destination),
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => stage = e.file,
            Err(e) => return Err(e.error.into()),
        }
    }
    Err("Too many filename collisions. Choose another export folder.".into())
}
pub(super) fn publish_with(
    path: &Path,
    format: Format,
    app_data: &Path,
    write: impl FnOnce(&mut NamedTempFile) -> Result<()>,
) -> Result<PathBuf> {
    if path
        .extension()
        .and_then(|v| v.to_str())
        .map(|v| v.to_ascii_lowercase())
        .as_deref()
        != Some(format.extension())
    {
        return Err("Choose a filename with the selected report extension".into());
    }
    let parent = path.parent().ok_or("Invalid destination")?;
    let parent = std::fs::canonicalize(parent)?;
    let protected = std::fs::canonicalize(app_data)?;
    if parent.starts_with(&protected) {
        return Err("Choose an export destination outside application data".into());
    }
    let dest = parent.join(path.file_name().ok_or("Invalid filename")?);
    if dest.exists() {
        return Err("A file already exists at this destination. Choose a new filename; existing exports are never overwritten.".into());
    }
    let mut stage = NamedTempFile::new_in(&parent)?;
    write(&mut stage)?;
    stage.as_file().sync_all()?;
    stage.persist_noclobber(&dest).map_err(|e| e.error)?;
    Ok(dest)
}
pub fn preview_pdf(report: &Report, app_data: &Path) -> Result<PathBuf> {
    let folder = app_data.join("report-previews");
    std::fs::create_dir_all(&folder)?;
    let bytes = render(report, Format::Pdf)?;
    crate::preview_cache::reserve(&folder, bytes.len() as u64)?;
    let mut stage = NamedTempFile::new_in(&folder)?;
    stage.write_all(&bytes)?;
    stage.as_file().sync_all()?;
    let destination = folder.join(format!(
        "{}-{}.pdf",
        report.default_name,
        uuid::Uuid::new_v4()
    ));
    stage.persist_noclobber(&destination).map_err(|e| e.error)?;
    Ok(destination)
}
#[cfg(windows)]
pub fn open_pdf(path: &Path) -> Result<()> {
    open_local(path)
}
#[cfg(windows)]
pub(crate) fn open_local(path: &Path) -> Result<()> {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::HWND,
            UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
        },
    };
    let file: Vec<u16> = path
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
    // Only the app-generated PDF path reaches this private OS adapter. No frontend shell arguments.
    let result = unsafe {
        ShellExecuteW(
            Some(HWND::default()),
            PCWSTR(verb.as_ptr()),
            PCWSTR(file.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        return Err("Windows could not open the local folder or PDF. Check its availability and the default PDF viewer.".into());
    }
    Ok(())
}
#[cfg(not(windows))]
pub fn open_pdf(_path: &Path) -> Result<()> {
    Err("Local print preview currently requires Windows".into())
}
#[cfg(not(windows))]
pub(crate) fn open_local(_path: &Path) -> Result<()> {
    Err("Opening local folders requires Windows".into())
}
