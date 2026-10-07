//! Windows current-user DPAPI encrypted blobs. Never plaintext or machine-wide protection.
use super::{Result, Service};
use std::path::{Path, PathBuf};
pub trait Vault: Send {
    fn read(&self, service: Service) -> Result<Option<Vec<u8>>>;
    fn write(&self, service: Service, bytes: &[u8]) -> Result<()>;
    fn delete(&self, service: Service) -> Result<()>;
}
pub struct WindowsVault(pub PathBuf);
impl WindowsVault {
    fn path(&self, service: Service) -> PathBuf {
        self.0
            .join("integration-credentials")
            .join(format!("{}.dpapi", service.key()))
    }
}
#[cfg(windows)]
fn protect(bytes: &[u8], root: &Path, service: Service, encrypt: bool) -> Result<Vec<u8>> {
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{LocalFree, HLOCAL},
            Security::Cryptography::{
                CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
            },
        },
    };
    if bytes.is_empty() || bytes.len() > 16_384 {
        return Err("Credential blob is invalid or too large. Reconnect.".into());
    }
    let entropy =
        format!("TWXPayroll/v1/{}/{}", root.to_string_lossy(), service.key()).into_bytes();
    let input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let additional = CRYPT_INTEGER_BLOB {
        cbData: entropy.len() as u32,
        pbData: entropy.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        let result = if encrypt {
            CryptProtectData(
                &input,
                PCWSTR::null(),
                Some(&additional),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input,
                None,
                Some(&additional),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        result.map_err(|e|format!("Windows user-protected credential unavailable or corrupt (HRESULT {}). Reconnect on the original Windows account. No plaintext fallback was used.",e.code().0))?;
        let result = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        if !encrypt {
            for index in 0..output.cbData as usize {
                std::ptr::write_volatile(output.pbData.add(index), 0);
            }
        }
        LocalFree(Some(HLOCAL(output.pbData.cast())));
        Ok(result)
    }
}
#[cfg(windows)]
impl Vault for WindowsVault {
    fn read(&self, service: Service) -> Result<Option<Vec<u8>>> {
        use std::io::Read;
        let file = match std::fs::File::open(self.path(service)) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => {
                return Err(
                    "Encrypted credential file unavailable. Disconnect and reconnect.".into(),
                )
            }
        };
        let mut encrypted = Vec::new();
        file.take(16_385)
            .read_to_end(&mut encrypted)
            .map_err(|_| "Encrypted credential could not be read")?;
        protect(&encrypted, &self.0, service, false).map(Some)
    }
    fn write(&self, service: Service, bytes: &[u8]) -> Result<()> {
        use std::io::Write;
        if bytes.len() > 8192 {
            return Err("Credential exceeds the secure-store size bound".into());
        }
        let encrypted = protect(bytes, &self.0, service, true)?;
        let path = self.path(service);
        let directory = path.parent().ok_or("Invalid secure-store location")?;
        std::fs::create_dir_all(directory).map_err(|_| "Secure-store folder unavailable")?;
        let mut stage = tempfile::NamedTempFile::new_in(directory)
            .map_err(|_| "Encrypted credential could not be staged")?;
        stage
            .write_all(&encrypted)
            .map_err(|_| "Encrypted credential write failed")?;
        stage
            .as_file()
            .sync_all()
            .map_err(|_| "Encrypted credential sync failed")?;
        stage
            .persist(path)
            .map_err(|_| "Encrypted credential could not be saved")?;
        Ok(())
    }
    fn delete(&self, service: Service) -> Result<()> {
        match std::fs::remove_file(self.path(service)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("Encrypted credential could not be removed. Try again.".into()),
        }
    }
}
#[cfg(not(windows))]
impl Vault for WindowsVault {
    fn read(&self, _: Service) -> Result<Option<Vec<u8>>> {
        Err("Secure integrations currently require Windows".into())
    }
    fn write(&self, _: Service, _: &[u8]) -> Result<()> {
        Err("Secure integrations currently require Windows".into())
    }
    fn delete(&self, _: Service) -> Result<()> {
        Err("Secure integrations currently require Windows".into())
    }
}
