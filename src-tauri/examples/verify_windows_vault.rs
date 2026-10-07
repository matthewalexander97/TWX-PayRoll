#![cfg_attr(windows, windows_subsystem = "windows")]
//! Interactive Windows platform probe; no network, real accounts, or real credentials.
fn main() {
    use twx_payroll_lib::integrations::{
        vault::{Vault, WindowsVault},
        Service,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("work/phase6-vault-probe");
    let result = (|| -> Result<(), String> {
        let vault = WindowsVault(root.clone());
        vault.write(Service::Sheets, b"synthetic-only-secret")?;
        let read = vault.read(Service::Sheets);
        if read? != Some(b"synthetic-only-secret".to_vec()) {
            return Err("Round trip mismatch".into());
        }
        let path = root.join("integration-credentials/sheets.dpapi");
        let mut encrypted = std::fs::read(&path).map_err(|_| "Read failed")?;
        if encrypted.windows(21).any(|v| v == b"synthetic-only-secret") {
            return Err("Plaintext found".into());
        }
        encrypted[20] ^= 1;
        std::fs::write(&path, encrypted).map_err(|_| "Tamper fixture write failed")?;
        if vault.read(Service::Sheets).is_ok() {
            return Err("Tampering was accepted".into());
        }
        vault.delete(Service::Sheets)?;
        if vault.read(Service::Sheets)?.is_some() {
            return Err("Removal failed".into());
        }
        Ok(())
    })();
    let _ = std::fs::create_dir_all(&root);
    let _=std::fs::write(root.join("result.json"),serde_json::to_vec(&serde_json::json!({"passed":result.is_ok(),"error":result.err(),"networkRequests":0,"syntheticOnly":true})).unwrap());
}
