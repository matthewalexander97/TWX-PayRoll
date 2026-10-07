//! Optional outputs: no business writes, no secrets in operation metadata, no automatic sends.
mod oauth;
pub mod output;
mod transport;
pub mod vault;
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine as _,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Mutex},
    time::{Duration, Instant},
};
use transport::{Endpoint, Payload, Transport};
use uuid::Uuid;
use vault::Vault;
pub type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    Sheets,
    Email,
}
impl Service {
    pub fn key(self) -> &'static str {
        match self {
            Self::Sheets => "sheets",
            Self::Email => "email",
        }
    }
    pub fn api_scope(self) -> &'static str {
        match self {
            Self::Sheets => "https://www.googleapis.com/auth/spreadsheets",
            Self::Email => "https://www.googleapis.com/auth/gmail.send",
        }
    }
    pub fn scope(self) -> &'static str {
        match self {
            Self::Sheets => "openid email https://www.googleapis.com/auth/spreadsheets",
            Self::Email => "openid email https://www.googleapis.com/auth/gmail.send",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Failure {
    Unavailable,
    Rejected,
    RateLimited,
    Uncertain,
}
impl Failure {
    fn message(self, sending: bool) -> String {
        match self{Self::Rejected=>"Google rejected the request or permission expired/revoked. Review the account/project and reconnect.",Self::RateLimited=>"Google rate limit reached. No automatic retry was made.",Self::Unavailable=>"Network access is required. Local reports remain available.",Self::Uncertain if sending=>"Delivery/creation is uncertain. Check your Google account before any new action; this operation will not be retried.",Self::Uncertain=>"Google could not be reached securely. Check connectivity; local workflows remain available."}.into()
    }
}
pub struct IntegrationState {
    pub engine: Mutex<Result<Engine>>,
    pub cancel: AtomicBool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub service: Service,
    pub connected: bool,
    pub account: Option<String>,
    pub message: String,
}
#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    jobs: Vec<Job>,
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Job {
    id: String,
    key: String,
    service: Service,
    report_type: crate::reports::ReportKind,
    time: String,
    status: String,
    destination: Option<String>,
}
struct Pending {
    service: Service,
    report: crate::reports::Report,
    email: Option<output::Email>,
    attachment: Vec<u8>,
    key: String,
    created: Instant,
    account: String,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Prepared {
    pub token: String,
    pub service: Service,
    pub title: String,
    pub period: String,
    pub draft: bool,
    pub account: String,
    pub recipient: Option<String>,
    pub subject: Option<String>,
    pub file_name: Option<String>,
    pub format: Option<crate::reports::Format>,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub message: String,
    pub destination: Option<String>,
}
pub struct Engine {
    root: PathBuf,
    vault: Box<dyn Vault>,
    http: Box<dyn Transport>,
    journal: Journal,
    pending: BTreeMap<String, Pending>,
}
impl Engine {
    pub fn new(root: &Path) -> Result<Self> {
        Self::with(
            root,
            Box::new(vault::WindowsVault(root.into())),
            Box::new(transport::Https),
        )
    }
    fn with(root: &Path, vault: Box<dyn Vault>, http: Box<dyn Transport>) -> Result<Self> {
        let path = root.join("integration-operations.json");
        let mut journal = if path.exists() {
            let mut bytes = Vec::new();
            std::fs::File::open(path).map_err(|_|"Integration history unavailable; external operations disabled. Local data is unaffected.")?.take(8_000_001).read_to_end(&mut bytes).map_err(|_|"Integration history could not be read; external operations disabled")?;
            if bytes.len() > 8_000_000 {
                return Err("Integration history too large; external operations disabled".into());
            }
            serde_json::from_slice::<Journal>(&bytes).map_err(|_|"Integration history is corrupt; external operations disabled to prevent duplicate sends.")?
        } else {
            Journal {
                version: 1,
                jobs: vec![],
            }
        };
        if journal.version != 1 {
            return Err("Unsupported integration history version".into());
        }
        for job in &mut journal.jobs {
            if !["SENDING", "UNCERTAIN", "SUCCESS", "FAILED"].contains(&job.status.as_str())
                || job.key.len() != 64
                || !job.key.bytes().all(|b| b.is_ascii_hexdigit())
                || Uuid::parse_str(&job.id).is_err()
            {
                return Err("Integration history has invalid operations; external sends disabled to prevent duplicates.".into());
            }
            if job.status == "SENDING" {
                job.status = "UNCERTAIN".into();
            }
        }
        Ok(Self {
            root: root.into(),
            vault,
            http,
            journal,
            pending: BTreeMap::new(),
        })
    }
    fn persist(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)
            .map_err(|_| "Integration metadata destination is unavailable")?;
        let mut stage = tempfile::NamedTempFile::new_in(&self.root)
            .map_err(|_| "Integration history could not be staged")?;
        let bytes = serde_json::to_vec(&self.journal)
            .map_err(|_| "Integration history serialization failed")?;
        if bytes.len() > 8_000_000 {
            return Err("Integration history is full; new external actions are blocked to preserve duplicate protection. Keep the journal. Review Sheets/Sent Mail and use local exports; there is no automatic reset or retry.".into());
        }
        stage
            .write_all(&bytes)
            .map_err(|_| "Integration history write failed")?;
        stage
            .as_file()
            .sync_all()
            .map_err(|_| "Integration history sync failed")?;
        stage
            .persist(self.root.join("integration-operations.json"))
            .map_err(|_| "Integration history publication failed; no retry will be made")?;
        Ok(())
    }
    fn credential(&self, service: Service) -> Result<oauth::Credential> {
        let bytes=self.vault.read(service)?.ok_or("Integration is disconnected. Configure it in Settings; local exports remain available.")?;
        let credential: oauth::Credential = serde_json::from_slice(&bytes)
            .map_err(|_| "Secure credential is corrupt. Clear the connection and reconnect.")?;
        if credential.refresh_token.is_empty() || !output::recipient(&credential.email) {
            return Err("Secure credential is corrupt. Disconnect and reconnect.".into());
        }
        Ok(credential)
    }
    pub fn status(&self) -> Vec<Status> {
        [Service::Sheets, Service::Email]
            .into_iter()
            .map(|service| match self.credential(service) {
                Ok(c) => Status {
                    service,
                    connected: true,
                    account: Some(c.email),
                    message:
                        "Configured locally. Google permission and network are checked when used."
                            .into(),
                },
                Err(e) => Status {
                    service,
                    connected: false,
                    account: None,
                    message: e,
                },
            })
            .collect()
    }
    pub fn connect(&mut self, service: Service, bytes: &[u8], cancel: &AtomicBool) -> Result<()> {
        let client = oauth::client_file(bytes)?;
        let credential = oauth::connect(client, service, self.http.as_ref(), cancel)?;
        self.vault.write(
            service,
            &serde_json::to_vec(&credential).map_err(|_| "Credential encoding failed")?,
        )?;
        self.pending.retain(|_, p| p.service != service);
        Ok(())
    }
    pub fn disconnect(&mut self, service: Service) -> Result<Receipt> {
        let credential = self.credential(service).ok();
        // Delete locally before any external failure. Corrupt blobs can still be cleared.
        self.vault.delete(service)?;
        self.pending.retain(|_, p| p.service != service);
        let revoked = credential.as_ref().is_some_and(|c| {
            self.http
                .request(
                    Endpoint::Revoke,
                    None,
                    Payload::Form(vec![("token".into(), c.refresh_token.clone())]),
                )
                .is_ok()
        });
        Ok(Receipt{message:if revoked{"Local credential removed and Google revocation accepted. Google may revoke both services for this project; reconnect any remaining service if needed."}else{"Local credential removed. Remote revocation was not confirmed; remove the grant in your Google Account permissions."}.into(),destination:None})
    }
    pub fn prepare(
        &mut self,
        report: crate::reports::Report,
        service: Service,
        email: Option<output::Email>,
    ) -> Result<Prepared> {
        let credential = self.credential(service)?;
        let attachment = match service {
            Service::Sheets => {
                if email.is_some() {
                    return Err("Email details do not belong in a Sheets request".into());
                }
                output::sheets(&report)?;
                vec![]
            }
            Service::Email => {
                let e = email.as_ref().ok_or("Email details required")?;
                e.validate()?;
                crate::reports::render(&report,e.format).map_err(|_|"Attachment could not be generated; nothing was sent. Try a local export to review the report.")?
            }
        };
        if attachment.len() > 18_000_000 {
            return Err("Attachment is too large".into());
        }
        let mut digest = Sha256::new();
        digest.update(serde_json::to_vec(&report).map_err(|_| "Report encoding failed")?);
        digest.update(service.key());
        digest.update(credential.email.as_bytes());
        if let Some(e) = &email {
            digest.update(serde_json::to_vec(&json!({"recipient":e.recipient,"subject":e.subject,"body":e.body,"format":e.format})).map_err(|_|"Email encoding failed")?);
        }
        let key = format!("{:x}", digest.finalize());
        if self.journal.jobs.iter().any(|j| {
            j.key == key && ["SENDING", "UNCERTAIN", "SUCCESS"].contains(&j.status.as_str())
        }) {
            return Err("This exact report action already succeeded or has an uncertain result. Check Google Sheets/Sent Mail; automatic duplicates are blocked.".into());
        }
        self.pending
            .retain(|_, p| p.created.elapsed() < Duration::from_secs(600));
        if self.pending.len() >= 8 {
            return Err("Cancel previous report confirmations first".into());
        }
        let token = Uuid::new_v4().to_string();
        let prepared = Prepared {
            token: token.clone(),
            service,
            title: report.title.clone(),
            period: report.period.clone(),
            draft: report.draft,
            account: credential.email.clone(),
            recipient: email.as_ref().map(|e| e.recipient.clone()),
            subject: email.as_ref().map(|e| e.subject.clone()),
            file_name: email
                .as_ref()
                .map(|e| format!("{}.{}", report.default_name, e.format.extension())),
            format: email.as_ref().map(|e| e.format),
        };
        self.pending.insert(
            token,
            Pending {
                service,
                report,
                email,
                attachment,
                key,
                created: Instant::now(),
                account: credential.email,
            },
        );
        Ok(prepared)
    }
    pub fn cancel(&mut self, token: &str) {
        self.pending.remove(token);
    }
    pub fn confirm(&mut self, token: &str, confirmed: bool) -> Result<Receipt> {
        if !confirmed {
            return Err("Explicit confirmation is required".into());
        }
        let p = self
            .pending
            .remove(token)
            .ok_or("Confirmation expired or already used. Prepare the report again.")?;
        if p.created.elapsed() > Duration::from_secs(600) {
            return Err("Confirmation expired. Prepare again.".into());
        }
        if self.journal.jobs.iter().any(|j| {
            j.key == p.key && ["SENDING", "UNCERTAIN", "SUCCESS"].contains(&j.status.as_str())
        }) {
            return Err("Duplicate operation blocked".into());
        }
        let credential = self.credential(p.service)?;
        if credential.email != p.account {
            return Err("Connected account changed. Prepare again before sending.".into());
        }
        let access = oauth::access(&credential, self.http.as_ref())?;
        let payload = match p.service {
            Service::Sheets => output::sheets(&p.report)?,
            Service::Email => {
                json!({"raw":output::mime(&credential.email,p.email.as_ref().ok_or("Missing email")?,&p.report,&p.attachment,token)?})
            }
        };
        let endpoint = match p.service {
            Service::Sheets => Endpoint::Sheets,
            Service::Email => Endpoint::Gmail,
        };
        let job = Job {
            id: token.into(),
            key: p.key,
            service: p.service,
            report_type: p.report.kind,
            time: crate::backup::now().to_string(),
            status: "SENDING".into(),
            destination: None,
        };
        self.journal.jobs.push(job);
        self.persist()?; // Durable marker BEFORE a non-idempotent request.
        let result = self
            .http
            .request(endpoint, Some(&access), Payload::Json(payload));
        let last = self
            .journal
            .jobs
            .last_mut()
            .ok_or("Integration operation missing")?;
        let receipt = match result {
            Ok(v) => {
                let id = v[if p.service == Service::Sheets {
                    "spreadsheetId"
                } else {
                    "id"
                }]
                .as_str()
                .filter(|s| {
                    !s.is_empty()
                        && s.len() < 200
                        && s.bytes()
                            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
                });
                if let Some(id) = id {
                    last.status = "SUCCESS".into();
                    last.destination = Some(id.into());
                    Ok(Receipt{message:if p.service==Service::Sheets{"New Google spreadsheet created from the saved snapshot. Editing it does not change TWX Payroll."}else{"Gmail accepted the report message. Recipient delivery is not verified."}.into(),destination:if p.service==Service::Sheets{Some(format!("https://docs.google.com/spreadsheets/d/{id}/edit"))}else{None}})
                } else {
                    last.status = "UNCERTAIN".into();
                    Err(Failure::Uncertain.message(true))
                }
            }
            Err(e) => {
                last.status = if matches!(
                    e,
                    Failure::Rejected | Failure::RateLimited | Failure::Unavailable
                ) {
                    "FAILED"
                } else {
                    "UNCERTAIN"
                }
                .into();
                Err(e.message(true))
            }
        };
        if self.persist().is_err() {
            return Err("Provider result could not be recorded. Check Sheets/Sent Mail; the durable marker blocks retry to prevent duplicates.".into());
        }
        receipt
    }
}
#[cfg(test)]
mod tests;
