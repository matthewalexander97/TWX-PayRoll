//! Native picker only. No frontend-supplied path and no filesystem plugin grant.
#[cfg(windows)]
pub fn google_client(owner: isize) -> crate::backup::Result<Option<std::path::PathBuf>> {
    use windows::{
        core::w,
        Win32::{
            Foundation::HWND,
            System::Com::{
                CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
            },
            UI::Shell::{
                Common::COMDLG_FILTERSPEC, FileOpenDialog, IFileOpenDialog, FOS_FILEMUSTEXIST,
                FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, SIGDN_FILESYSPATH,
            },
        },
    };
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() }
        }
    }
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let _apartment = Apartment;
        let dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
        dialog.SetOptions(FOS_FILEMUSTEXIST | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST)?;
        dialog.SetTitle(w!(
            "Connect Google — choose YOUR Google OAuth Desktop client JSON"
        ))?;
        dialog.SetFileTypes(&[COMDLG_FILTERSPEC {
            pszName: w!("Google Desktop OAuth client"),
            pszSpec: w!("*.json"),
        }])?;
        if let Err(e) = dialog.Show(Some(HWND(owner as *mut std::ffi::c_void))) {
            if e.code().0 as u32 == 0x800704c7 {
                return Ok(None);
            }
            return Err(e.into());
        }
        let allocated = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
        let path = allocated.to_string();
        CoTaskMemFree(Some(allocated.0.cast()));
        Ok(Some(path?.into()))
    }
}
#[cfg(not(windows))]
pub fn google_client(_: isize) -> crate::backup::Result<Option<std::path::PathBuf>> {
    Err("Google client selection requires Windows".into())
}
#[cfg(windows)]
pub fn pick(folder: bool) -> crate::backup::Result<Option<std::path::PathBuf>> {
    pick_owned(folder, false, 0)
}
#[cfg(windows)]
pub fn pick_owned(
    folder: bool,
    export: bool,
    owner: isize,
) -> crate::backup::Result<Option<std::path::PathBuf>> {
    use windows::{
        core::w,
        Win32::{
            System::Com::{
                CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
            },
            UI::Shell::{
                Common::COMDLG_FILTERSPEC, FileOpenDialog, IFileOpenDialog, FOS_FILEMUSTEXIST,
                FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
            },
        },
    };
    struct ComApartment;
    impl Drop for ComApartment {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }
    // Called on a dedicated blocking worker. COM interfaces and allocated strings stay on this thread.
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let _apartment = ComApartment;
        let dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
        dialog.SetOptions(
            FOS_FORCEFILESYSTEM
                | FOS_PATHMUSTEXIST
                | if folder {
                    FOS_PICKFOLDERS
                } else {
                    FOS_FILEMUSTEXIST
                },
        )?;
        if folder {
            dialog.SetTitle(if export {
                w!("Choose TWX Payroll export folder")
            } else {
                w!("Choose TWX Payroll backup folder")
            })?;
        } else {
            dialog.SetTitle(w!("Select TWX Payroll backup to restore"))?;
            dialog.SetFileTypes(&[COMDLG_FILTERSPEC {
                pszName: w!("TWX Payroll SQLite backup"),
                pszSpec: w!("*.sqlite3"),
            }])?;
        }
        if let Err(error) = dialog.Show(Some(windows::Win32::Foundation::HWND(
            owner as *mut std::ffi::c_void,
        ))) {
            if error.code().0 as u32 == 0x800704c7 {
                return Ok(None);
            }
            return Err(error.into());
        }
        let allocated = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
        let selected = allocated.to_string();
        CoTaskMemFree(Some(allocated.0.cast()));
        Ok(Some(std::path::PathBuf::from(selected?)))
    }
}
#[cfg(not(windows))]
pub fn pick(_folder: bool) -> crate::backup::Result<Option<std::path::PathBuf>> {
    Err("Native backup selection currently requires Windows".into())
}

#[cfg(windows)]
pub fn save_report(
    default_name: &str,
    extension: &str,
    owner: isize,
) -> crate::backup::Result<Option<std::path::PathBuf>> {
    use windows::{
        core::{w, PCWSTR},
        Win32::{
            Foundation::HWND,
            System::Com::{
                CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
            },
            UI::Shell::{
                Common::COMDLG_FILTERSPEC, FileSaveDialog, IFileSaveDialog, FOS_FORCEFILESYSTEM,
                FOS_NOREADONLYRETURN, FOS_PATHMUSTEXIST, SIGDN_FILESYSPATH,
            },
        },
    };
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() }
        }
    }
    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let name = wide(default_name);
    let ext = wide(extension);
    let label = wide(&format!("TWX {} report", extension.to_uppercase()));
    let pattern = wide(&format!("*.{extension}"));
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let _apartment = Apartment;
        let dialog: IFileSaveDialog =
            CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)?;
        dialog.SetOptions(FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST | FOS_NOREADONLYRETURN)?;
        dialog.SetTitle(w!("Save TWX Payroll report - choose a new filename"))?;
        dialog.SetFileName(PCWSTR(name.as_ptr()))?;
        dialog.SetDefaultExtension(PCWSTR(ext.as_ptr()))?;
        dialog.SetFileTypes(&[COMDLG_FILTERSPEC {
            pszName: PCWSTR(label.as_ptr()),
            pszSpec: PCWSTR(pattern.as_ptr()),
        }])?;
        if let Err(e) = dialog.Show(Some(HWND(owner as *mut std::ffi::c_void))) {
            if e.code().0 as u32 == 0x800704c7 {
                return Ok(None);
            }
            return Err(e.into());
        }
        let allocated = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
        let path = allocated.to_string();
        CoTaskMemFree(Some(allocated.0.cast()));
        Ok(Some(path?.into()))
    }
}
#[cfg(not(windows))]
pub fn save_report(
    _name: &str,
    _extension: &str,
    _owner: isize,
) -> crate::backup::Result<Option<std::path::PathBuf>> {
    Err("Native export selection currently requires Windows".into())
}

#[cfg(not(windows))]
pub fn pick_owned(_: bool, _: bool, _: isize) -> crate::backup::Result<Option<std::path::PathBuf>> {
    Err("Native folder selection requires Windows".into())
}
