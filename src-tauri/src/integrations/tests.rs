use super::*;
use crate::reports::{self, Format, ReportKind, Request};
use std::sync::{Arc, Mutex};
#[derive(Clone, Default)]
struct FakeVault(Arc<Mutex<BTreeMap<String, Vec<u8>>>>);
impl Vault for FakeVault {
    fn read(&self, s: Service) -> Result<Option<Vec<u8>>> {
        Ok(self.0.lock().unwrap().get(s.key()).cloned())
    }
    fn write(&self, s: Service, b: &[u8]) -> Result<()> {
        self.0.lock().unwrap().insert(s.key().into(), b.to_vec());
        Ok(())
    }
    fn delete(&self, s: Service) -> Result<()> {
        self.0.lock().unwrap().remove(s.key());
        Ok(())
    }
}
type Response = std::result::Result<Value, Failure>;
#[derive(Clone, Default)]
struct FakeHttp {
    responses: Arc<Mutex<std::collections::VecDeque<Response>>>,
    calls: Arc<Mutex<Vec<(String, Value)>>>,
}
impl Transport for FakeHttp {
    fn request(&self, e: Endpoint, _: Option<&str>, p: Payload) -> Response {
        let data = match p {
            Payload::Json(v) => v,
            Payload::Form(_) => json!({"form":"redacted"}),
            Payload::Get => json!({}),
        };
        self.calls.lock().unwrap().push((e.url().into(), data));
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected request; no default live transport")
    }
}
fn config() -> oauth::ClientConfig {
    oauth::ClientConfig {
        client_id: "synthetic.apps.googleusercontent.com".into(),
        client_secret: "synthetic-client".into(),
    }
}
fn fixture() -> (tempfile::TempDir, crate::Storage, crate::reports::Report) {
    let (dir, s, id) = reports::tests::fixture();
    let r = reports::build(
        &s,
        &Request {
            kind: ReportKind::SettlementDetail,
            settlement_id: Some(id),
            period_start: None,
            period_end: None,
            driver_id: None,
            owner_id: None,
            truck_id: None,
        },
    )
    .unwrap();
    (dir, s, r)
}
fn setup(root: &Path) -> (Engine, FakeHttp, FakeVault) {
    let v = FakeVault::default();
    let c = oauth::Credential {
        client: config(),
        refresh_token: "synthetic-refresh".into(),
        email: "test@example.invalid".into(),
    };
    for s in [Service::Sheets, Service::Email] {
        v.write(s, &serde_json::to_vec(&c).unwrap()).unwrap();
    }
    let h = FakeHttp::default();
    let e = Engine::with(root, Box::new(v.clone()), Box::new(h.clone())).unwrap();
    (e, h, v)
}
fn email(format: Format) -> output::Email {
    output::Email {
        recipient: "recipient@example.invalid".into(),
        subject: "Synthetic payroll — review".into(),
        body: "Synthetic message\nSecond line".into(),
        format,
    }
}
fn replies(h: &FakeHttp, values: Vec<Response>) {
    h.responses.lock().unwrap().extend(values);
}
#[test]
fn disconnected_and_corrupt_vault_do_not_touch_transport() {
    let (d, _, r) = fixture();
    let v = FakeVault::default();
    let h = FakeHttp::default();
    let mut e = Engine::with(d.path(), Box::new(v.clone()), Box::new(h.clone())).unwrap();
    assert!(e
        .prepare(r.clone(), Service::Sheets, None)
        .unwrap_err()
        .contains("disconnected"));
    v.write(Service::Sheets, b"bad-json").unwrap();
    assert!(!e.status()[0].connected);
    assert!(e
        .prepare(r, Service::Sheets, None)
        .unwrap_err()
        .contains("corrupt"));
    assert!(h.calls.lock().unwrap().is_empty());
}
#[test]
fn oauth_pkce_state_scope_client_and_identity_contract() {
    assert_eq!(
        oauth::challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
    assert_eq!(
        oauth::callback("/oauth/callback?code=x&state=y", "y").unwrap(),
        "x"
    );
    for target in [
        "/oauth/callback?code=x&state=bad",
        "/oauth/callback?state=y&state=y&code=x",
        "/oauth/callback?state=y&error=access_denied",
        "/wrong?state=y&code=x",
    ] {
        assert!(oauth::callback(target, "y").is_err());
    }
    assert!(oauth::client_file(br#"{"web":{}}"#).is_err());
    assert!(oauth::client_file(&vec![b'a'; 16385]).is_err());
    let url = oauth::authorization(
        &config(),
        Service::Sheets,
        "http://127.0.0.1:1234/oauth/callback",
        "state",
        "verifier",
    );
    assert!(!url.contains("drive"));
    assert!(url.contains("S256"));
    assert!(!url.contains("client_secret"));
    let h = FakeHttp::default();
    replies(
        &h,
        vec![Ok(
            json!({"email":"test@example.invalid","email_verified":true}),
        )],
    );
    let credential=oauth::credential_from_response(config(),Service::Sheets,json!({"access_token":"access","refresh_token":"refresh","scope":Service::Sheets.api_scope()}),&h).unwrap();
    assert_eq!(credential.email, "test@example.invalid");
    assert!(oauth::credential_from_response(config(),Service::Email,json!({"access_token":"access","refresh_token":"refresh","scope":Service::Sheets.api_scope()}),&h).is_err());
}
#[test]
fn sheets_exact_historical_values_draft_owner_and_no_formulas() {
    let (_d, s, mut r) = fixture();
    let body = output::sheets(&r).unwrap();
    let text = body.to_string();
    assert!(text.contains("stringValue"));
    assert!(!text.contains("formulaValue"));
    for section in &r.sections {
        for cell in section.rows.iter().flatten() {
            assert!(text.contains(&serde_json::to_string(&cell.display()).unwrap()));
        }
    }
    r.draft = true;
    assert!(output::sheets(&r)
        .unwrap()
        .to_string()
        .contains("DRAFT / NOT FINALIZED"));
    let owner = reports::build(
        &s,
        &Request {
            kind: ReportKind::OwnerPayroll,
            settlement_id: None,
            period_start: None,
            period_end: None,
            driver_id: None,
            owner_id: None,
            truck_id: None,
        },
    )
    .unwrap();
    assert!(!output::sheets(&owner).unwrap().to_string().contains("ACH"));
}
#[test]
fn confirmed_sheets_snapshot_success_deduplicates_across_restart() {
    let (d, s, r) = fixture();
    let changes = s.connection.total_changes();
    let (e, h, v) = setup(d.path());
    let mut e = e;
    let p = e.prepare(r.clone(), Service::Sheets, None).unwrap();
    assert!(h.calls.lock().unwrap().is_empty());
    replies(
        &h,
        vec![
            Ok(json!({"access_token":"access"})),
            Ok(json!({"spreadsheetId":"synthetic-id"})),
        ],
    );
    assert!(e
        .confirm(&p.token, true)
        .unwrap()
        .destination
        .unwrap()
        .contains("synthetic-id"));
    assert!(e.confirm(&p.token, true).is_err());
    assert!(e.prepare(r.clone(), Service::Sheets, None).is_err());
    let mut restart = Engine::with(d.path(), Box::new(v), Box::new(h.clone())).unwrap();
    assert!(restart.prepare(r, Service::Sheets, None).is_err());
    assert_eq!(h.calls.lock().unwrap().len(), 2);
    assert_eq!(s.connection.total_changes(), changes);
    let journal = std::fs::read_to_string(d.path().join("integration-operations.json")).unwrap();
    for secret in [
        "synthetic-refresh",
        "recipient@example.invalid",
        "test@example.invalid",
        "access",
    ] {
        assert!(!journal.contains(secret));
    }
}
#[test]
fn email_pdf_xlsx_docx_attachments_equal_local_bytes() {
    let (d, s, r) = fixture();
    let before = s.connection.total_changes();
    for format in [Format::Pdf, Format::Xlsx, Format::Docx] {
        let root = d.path().join(format.extension());
        let (mut e, h, _) = setup(&root);
        let details = email(format);
        let p = e.prepare(r.clone(), Service::Email, Some(details)).unwrap();
        replies(
            &h,
            vec![
                Ok(json!({"access_token":"access"})),
                Ok(json!({"id":"synthetic-message"})),
            ],
        );
        e.confirm(&p.token, true).unwrap();
        let calls = h.calls.lock().unwrap();
        let raw = URL_SAFE_NO_PAD
            .decode(calls[1].1["raw"].as_str().unwrap())
            .unwrap();
        let mime = String::from_utf8(raw).unwrap();
        let attachment_part = mime
            .split("Content-Disposition: attachment")
            .nth(1)
            .unwrap();
        let encoded = attachment_part
            .split("\r\n\r\n")
            .nth(1)
            .unwrap()
            .split("\r\n--")
            .next()
            .unwrap()
            .replace("\r\n", "");
        assert_eq!(
            STANDARD.decode(encoded).unwrap(),
            reports::render(&r, format).unwrap()
        );
        assert!(!mime.contains("sqlite3"));
    }
    assert_eq!(s.connection.total_changes(), before);
}
#[test]
fn recipient_header_injection_and_mime_validation() {
    for value in ["x@example.invalid", "a+b@sub.example.invalid"] {
        assert!(output::recipient(value));
    }
    for value in [
        "bad",
        "a@",
        "a@example.invalid\r\nBcc: x@example.invalid",
        "a..b@example.invalid",
        "a,b@example.invalid",
        "a@-example.invalid",
        "a@example..invalid",
    ] {
        assert!(!output::recipient(value));
    }
    let mut details = email(Format::Pdf);
    details.subject = "bad\r\nBcc: someone".into();
    assert!(details.validate().is_err());
}
#[test]
fn cancel_confirmation_expiry_and_explicit_confirmation_send_nothing() {
    let (d, _, r) = fixture();
    let (mut e, h, _) = setup(d.path());
    let p = e
        .prepare(r.clone(), Service::Email, Some(email(Format::Pdf)))
        .unwrap();
    assert!(e.confirm(&p.token, false).is_err());
    e.cancel(&p.token);
    assert!(e.confirm(&p.token, true).is_err());
    let p = e.prepare(r, Service::Sheets, None).unwrap();
    e.pending.get_mut(&p.token).unwrap().created = Instant::now() - Duration::from_secs(601);
    assert!(e.confirm(&p.token, true).is_err());
    assert!(h.calls.lock().unwrap().is_empty());
}
#[test]
fn provider_network_rate_limit_and_ambiguous_send_never_auto_retry() {
    let (d, s, r) = fixture();
    let changes = s.connection.total_changes();
    for (index, failure) in [
        Failure::Uncertain,
        Failure::Rejected,
        Failure::RateLimited,
        Failure::Unavailable,
    ]
    .into_iter()
    .enumerate()
    {
        for service in [Service::Email, Service::Sheets] {
            let (mut e, h, v) = setup(&d.path().join(format!("{index}{}", service.key())));
            let email = if service == Service::Email {
                Some(email(Format::Pdf))
            } else {
                None
            };
            let p = e.prepare(r.clone(), service, email.clone()).unwrap();
            replies(&h, vec![Ok(json!({"access_token":"access"})), Err(failure)]);
            assert!(e.confirm(&p.token, true).is_err());
            assert!(e.confirm(&p.token, true).is_err());
            assert_eq!(h.calls.lock().unwrap().len(), 2);
            if matches!(failure, Failure::Uncertain) {
                assert!(e.prepare(r.clone(), service, email.clone()).is_err());
                let mut restart = Engine::with(&e.root, Box::new(v), Box::new(h)).unwrap();
                assert!(restart.prepare(r.clone(), service, email).is_err());
            }
        }
    }
    assert_eq!(s.connection.total_changes(), changes);
}
#[test]
fn revoked_expired_refresh_fails_before_send_and_disconnect_clears_offline() {
    let (d, _, r) = fixture();
    let (mut e, h, v) = setup(d.path());
    let p = e.prepare(r, Service::Sheets, None).unwrap();
    replies(&h, vec![Err(Failure::Rejected), Err(Failure::Uncertain)]);
    assert!(e.confirm(&p.token, true).unwrap_err().contains("revoked"));
    assert_eq!(h.calls.lock().unwrap().len(), 1);
    assert!(e
        .disconnect(Service::Sheets)
        .unwrap()
        .message
        .contains("not confirmed"));
    assert!(v.read(Service::Sheets).unwrap().is_none());
    assert!(v.read(Service::Email).unwrap().is_some());
}
#[test]
fn attachment_failure_and_metadata_write_failure_happen_before_external_send() {
    let (d, _, mut r) = fixture();
    let (mut e, h, _) = setup(d.path());
    r.sections[0].rows[0].push(reports::Cell::Text {
        value: "extra".into(),
    });
    assert!(e
        .prepare(r, Service::Email, Some(email(Format::Pdf)))
        .is_err());
    assert!(h.calls.lock().unwrap().is_empty());
    let (d, _, r) = fixture();
    let root = d.path().join("blocked");
    std::fs::write(&root, b"file").unwrap();
    let (mut e, h, _) = setup(&root);
    let p = e.prepare(r, Service::Sheets, None).unwrap();
    replies(&h, vec![Ok(json!({"access_token":"access"}))]);
    assert!(e.confirm(&p.token, true).is_err());
    assert_eq!(h.calls.lock().unwrap().len(), 1);
}
#[test]
fn interrupted_marker_blocks_duplicate_and_malformed_history_fails_closed() {
    let (d, _, r) = fixture();
    let (mut e, h, v) = setup(d.path());
    let p = e.prepare(r.clone(), Service::Sheets, None).unwrap();
    e.journal.jobs.push(Job {
        id: p.token.clone(),
        key: e.pending[&p.token].key.clone(),
        service: Service::Sheets,
        report_type: r.kind,
        time: "synthetic".into(),
        status: "SENDING".into(),
        destination: None,
    });
    e.persist().unwrap();
    let mut reopen = Engine::with(d.path(), Box::new(v), Box::new(h.clone())).unwrap();
    assert!(reopen.prepare(r, Service::Sheets, None).is_err());
    std::fs::write(d.path().join("integration-operations.json"), b"corrupt").unwrap();
    assert!(Engine::with(
        d.path(),
        Box::new(FakeVault::default()),
        Box::new(h.clone())
    )
    .is_err());
    let oversized = std::fs::File::create(d.path().join("integration-operations.json")).unwrap();
    oversized.set_len(8_000_001).unwrap();
    drop(oversized);
    assert!(
        Engine::with(d.path(), Box::new(FakeVault::default()), Box::new(h))
            .err()
            .unwrap()
            .contains("too large")
    );
}
#[cfg(windows)]
#[test]
fn windows_vault_protects_or_fails_closed_in_restricted_host() {
    let folder = tempfile::tempdir().unwrap();
    let v = vault::WindowsVault(folder.path().into());
    if let Err(error) = v.write(Service::Sheets, b"synthetic-only-secret") {
        assert!(error.contains("No plaintext fallback was used"));
        assert!(
            error.contains("-2147024894") || error.contains("-2147023584"),
            "Unexpected platform failure: {error}"
        );
        assert!(!folder
            .path()
            .join("integration-credentials/sheets.dpapi")
            .exists());
        // Strict interactive success/tamper/removal acceptance lives in examples/verify_windows_vault.rs.
        return;
    }
    let result = v.read(Service::Sheets);
    let removed = v.delete(Service::Sheets);
    assert_eq!(result.unwrap().unwrap(), b"synthetic-only-secret");
    removed.unwrap();
    assert!(v.read(Service::Sheets).unwrap().is_none());
    assert!(v.write(Service::Sheets, &vec![0; 8193]).is_err());
    v.write(Service::Sheets, b"synthetic-only-secret").unwrap();
    let path = folder.path().join("integration-credentials/sheets.dpapi");
    let mut bytes = std::fs::read(&path).unwrap();
    assert!(!bytes.windows(21).any(|v| v == b"synthetic-only-secret"));
    bytes[20] ^= 1;
    std::fs::write(&path, bytes).unwrap();
    assert!(v.read(Service::Sheets).is_err());
    v.delete(Service::Sheets).unwrap();
}
