use super::*;
use serde_json::json;
use std::io::{Cursor, Read};
#[test]
fn persisted_negative_snapshot_exports_read_only_and_rejects_corrupt_fields() {
    let (_dir, mut s, id) = fixture();
    let a = repo::aggregate(&s.connection, &id).unwrap();
    let input=serde_json::from_value(json!({"driverId":a["header"]["driver_id"],"truckId":a["header"]["truck_id"],"periodStart":"2026-10-01","periodEnd":"2026-10-19","notes":"Synthetic negative","payoutPercentage":"85","loads":[{"driverId":a["header"]["driver_id"],"truckId":a["header"]["truck_id"],"loadNumber":"NEG","bolNumber":"","poNumber":"","tons":"10","completionRate":"5"}],"adjustments":[{"adjustmentTypeId":a["adjustments"][0]["adjustment_type_id"],"explanation":"Synthetic deduction","direction":"DEDUCT","amount":"92.50"}]})).unwrap();
    let draft = workspace::save_settlement(&mut s, input, true).unwrap();
    let id = draft["header"]["id"].as_str().unwrap();
    workspace::finalize(&mut s, id, "1", true).unwrap();
    let changes = s.connection.total_changes();
    let report = build(&s, &request(ReportKind::SettlementDetail, Some(id))).unwrap();
    assert_eq!(
        report
            .sections
            .iter()
            .find(|s| s.name == "Final payout")
            .unwrap()
            .rows[0][1]
            .display(),
        "-$50.00"
    );
    for format in [Format::Csv, Format::Xlsx, Format::Pdf, Format::Docx] {
        render(&report, format).unwrap();
    }
    assert_eq!(s.connection.total_changes(), changes);
    let mut corrupt = report.clone();
    corrupt.sections[0].rows[0][1] = Cell::text("unsupported\u{0}");
    assert!(render(&corrupt, Format::Docx).is_err());
    assert!(Cell::exact(
        &json!({"ach_charge_cents":"-1"}),
        "ach_charge_cents",
        2,
        true
    )
    .is_err());
    let mut unknown = repo::aggregate(&s.connection, id).unwrap();
    unknown["header"]["rule_version"] = json!("future-policy");
    s.connection
        .execute(
            "UPDATE weekly_settlements SET snapshot_json=?1 WHERE id=?2",
            rusqlite::params![serde_json::to_string(&unknown).unwrap(), id],
        )
        .unwrap();
    assert!(build(&s, &request(ReportKind::SettlementDetail, Some(id))).is_err());
}
pub(crate) fn fixture() -> (tempfile::TempDir, Storage, String) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Storage::open(dir.path()).unwrap();
    let reference = |s: &mut Storage, kind: &str, fields: Value| {
        let mut v = json!({"kind":kind,"id":null,"expectedVersion":null,"active":true});
        v.as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        workspace::save_reference(s, serde_json::from_value(v).unwrap()).unwrap()
    };
    let owner = reference(&mut s, "owners", json!({"name":"Synthetic Owner"}));
    let dtype = reference(
        &mut s,
        "driver_types",
        json!({"name":"Synthetic Type","payoutPercentage":"85"}),
    );
    let driver = reference(
        &mut s,
        "drivers",
        json!({"name":"José Synthetic, \"Driver\"","driverTypeId":dtype["id"]}),
    );
    let truck = reference(
        &mut s,
        "trucks",
        json!({"truckNumber":"FAKE-001","ownerId":owner["id"],"defaultDriverId":driver["id"]}),
    );
    let code = reference(
        &mut s,
        "twx_codes",
        json!({"code":"SYN","label":"Synthetic code"}),
    );
    let deduct = reference(
        &mut s,
        "adjustment_types",
        json!({"name":"Insurance","defaultDirection":"DEDUCT"}),
    );
    let add = reference(
        &mut s,
        "adjustment_types",
        json!({"name":"Reimbursement","defaultDirection":"ADD"}),
    );
    let input:workspace::SettlementWrite=serde_json::from_value(json!({"driverId":driver["id"],"truckId":truck["id"],"periodStart":"2026-10-01","periodEnd":"2026-10-19","notes":"Synthetic notes, with \"quotes\"\nand Unicode José","payoutPercentage":"85","loads":[{"driverId":driver["id"],"truckId":truck["id"],"loadNumber":"FAKE-1","bolNumber":"FAKE-BOL","poNumber":"FAKE-PO","tons":"10","completionRate":"5","mileage":"12.3456789"},{"driverId":driver["id"],"truckId":truck["id"],"loadNumber":"FAKE-2","bolNumber":"","poNumber":"","tons":"20","completionRate":"4"}],"adjustments":[{"adjustmentTypeId":deduct["id"],"twxCodeId":code["id"],"explanation":"Insurance, synthetic","direction":"DEDUCT","amount":"10"},{"adjustmentTypeId":add["id"],"explanation":"Synthetic reimbursement","direction":"ADD","amount":"2"}]})).unwrap();
    let draft = workspace::save_settlement(&mut s, input, false).unwrap();
    let id = draft["header"]["id"].as_str().unwrap().to_string();
    workspace::finalize(&mut s, &id, "1", false).unwrap();
    let pay = workspace::payrolls(&s, workspace::RecordFilter::default())
        .unwrap()
        .pop()
        .unwrap();
    workspace::save_driver(&mut s,serde_json::from_value(json!({"id":pay["id"],"expectedVersion":pay["row_version"],"achCharge":"1.25","twxCodeId":code["id"],"notes":"Synthetic ACH note"})).unwrap()).unwrap();
    workspace::save_owner(&mut s,serde_json::from_value(json!({"ownerId":owner["id"],"truckId":truck["id"],"periodStart":"2026-10-01","periodEnd":"2026-10-19","amount":"123.45","notes":"Synthetic owner"})).unwrap()).unwrap();
    (dir, s, id)
}
fn request(kind: ReportKind, id: Option<&str>) -> Request {
    Request {
        kind,
        settlement_id: id.map(str::to_string),
        period_start: Some("2026-10-01".into()),
        period_end: Some("2026-10-19".into()),
        driver_id: None,
        owner_id: None,
        truck_id: None,
    }
}
fn zip_text(bytes: &[u8], part: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut text = String::new();
    zip.by_name(part)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}
#[test]
fn canonical_preserves_history_and_saved_totals_without_catalog_queries() {
    let (_dir, s, id) = fixture();
    let before = build(&s, &request(ReportKind::SettlementDetail, Some(&id))).unwrap();
    s.connection
        .execute("UPDATE drivers SET name='Current changed name'", [])
        .unwrap();
    s.connection
        .execute("UPDATE trucks SET truck_number='CHANGED'", [])
        .unwrap();
    s.connection
        .execute("UPDATE driver_types SET payout_rate_ppm=500000", [])
        .unwrap();
    s.connection
        .execute("UPDATE twx_codes SET code='CHANGED'", [])
        .unwrap();
    let report = build(&s, &request(ReportKind::SettlementDetail, Some(&id))).unwrap();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&report).unwrap()
    );
    let text = serde_json::to_string(&report).unwrap();
    for expected in [
        "José",
        "FAKE-001",
        "5000",
        "8000",
        "13000",
        "11050",
        "10250",
        "10375",
        "125",
        "850000",
        "SYN",
        "12.3456789",
    ] {
        assert!(text.contains(expected), "{expected}");
    }
    assert!(!text.contains("CHANGED"));
}
#[test]
fn canonical_owner_no_ach_filters_missing_corrupt_and_draft_state() {
    let (_dir, s, id) = fixture();
    let report = build(&s, &request(ReportKind::OwnerPayroll, None)).unwrap();
    assert!(!report.sections[0].columns.iter().any(|c| c.contains("ACH")));
    assert_eq!(report.sections[0].rows[0][3].display(), "$123.45");
    let mut r = request(ReportKind::DriverPayroll, None);
    r.period_start = Some("2027-01-01".into());
    r.period_end = Some("2027-02-01".into());
    assert!(build(&s, &r).is_err());
    assert!(build(&s, &request(ReportKind::SettlementDetail, Some("missing"))).is_err());
    s.connection
        .execute(
            "UPDATE weekly_settlements SET snapshot_json='{}' WHERE id=?1",
            [&id],
        )
        .unwrap();
    assert!(build(&s, &request(ReportKind::SettlementDetail, Some(&id))).is_err());
    s.connection.execute("UPDATE weekly_settlements SET status='DRAFT',finalized_at=NULL,rule_version=NULL,snapshot_json=NULL,gross_route_cents=NULL,driver_gross_cents=NULL,final_payout_cents=NULL WHERE id=?1",[&id]).unwrap();
    let report = build(&s, &request(ReportKind::SettlementDetail, Some(&id))).unwrap();
    assert!(report.draft);
    assert_eq!(
        report
            .sections
            .iter()
            .find(|s| s.name == "Final payout")
            .unwrap()
            .rows[0][1]
            .display(),
        "Not finalized"
    );
}
#[test]
fn csv_quoting_unicode_decimals_negative_and_injection() {
    let (_dir, s, id) = fixture();
    let mut report = build(&s, &request(ReportKind::SettlementDetail, Some(&id))).unwrap();
    report.sections.push(Section {
        name: "Safety".into(),
        columns: vec!["Notes".into(), "Amount".into()],
        rows: vec![vec![
            Cell::text("=HYPERLINK(\"evil\")"),
            Cell::Money {
                value: "-5000".into(),
            },
        ]],
    });
    let bytes = render(&report, Format::Csv).unwrap();
    let mut reader = csv::Reader::from_reader(bytes.as_slice());
    let records = reader
        .records()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();
    assert!(records.iter().any(|r| r
        .iter()
        .any(|v| v == "Synthetic notes, with \"quotes\"\nand Unicode José")));
    assert!(records.iter().any(|r| r.iter().any(|v| v == "-50.00")));
    assert!(records
        .iter()
        .any(|r| r.iter().any(|v| v.starts_with("'=HYPERLINK"))));
}
#[test]
fn real_xlsx_and_docx_packages_preserve_values_and_headers() {
    let (_dir, s, id) = fixture();
    let mut report = build(&s, &request(ReportKind::SettlementDetail, Some(&id))).unwrap();
    report.sections[0].field(
        "Large exact money",
        Cell::Money {
            value: i64::MAX.to_string(),
        },
    );
    let excel = render(&report, Format::Xlsx).unwrap();
    assert!(excel.starts_with(b"PK"));
    let sheet = zip_text(&excel, "xl/worksheets/sheet1.xml");
    let strings = zip_text(&excel, "xl/sharedStrings.xml");
    assert!(sheet.contains("<v>102.5</v>"));
    assert!(sheet.contains("<v>1.25</v>"));
    let column = sheet
        .split("<col ")
        .find(|v| v.starts_with("min=\"2\""))
        .unwrap();
    let width: f64 = column
        .split("width=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        width >= 26.0,
        "Large exact money must receive enough column width"
    );
    for label in ["FAKE-001", "Completion Rate", "92,233,720,368,547,758.07"] {
        assert!(strings.contains(label), "{label}");
    }
    let summary = render(
        &build(&s, &request(ReportKind::PayrollSummary, None)).unwrap(),
        Format::Xlsx,
    )
    .unwrap();
    let book = zip_text(&summary, "xl/workbook.xml");
    assert!(book.contains("Driver Payroll"));
    assert!(book.contains("Owner Payroll"));
    let word = render(&report, Format::Docx).unwrap();
    assert!(word.starts_with(b"PK"));
    let text = zip_text(&word, "word/document.xml");
    for label in [
        "Weekly Settlement Detail",
        "FAKE-001",
        "Completion Rate",
        "FINAL PAYOUT",
        "$102.50",
        "$103.75",
    ] {
        assert!(text.contains(label), "{label}");
    }
    assert!(zip_text(&word, "[Content_Types].xml").contains("wordprocessingml"));
    let mut oversized = report.clone();
    oversized.sections[0].rows[0][1] = Cell::text("line\n".repeat(100));
    assert!(
        render(&oversized, Format::Xlsx).is_err(),
        "Never silently clip text above Excel's row-height limit"
    );
}
#[test]
fn pdf_pagination_negative_draft_unicode_and_export_fixture_consistency() {
    let (_dir, s, id) = fixture();
    let report = build(&s, &request(ReportKind::SettlementDetail, Some(&id))).unwrap();
    let bytes = render(&report, Format::Pdf).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
    assert!(String::from_utf8_lossy(&bytes).contains("/ToUnicode"));
    let mut long = report.clone();
    let loads = long
        .sections
        .iter_mut()
        .find(|s| s.name == "Loads")
        .unwrap();
    let row = loads.rows[0].clone();
    loads.rows = vec![row; 100];
    long.draft = true;
    long.sections
        .iter_mut()
        .find(|s| s.name == "Final payout")
        .unwrap()
        .rows[0][1] = Cell::Money {
        value: "-5000".into(),
    };
    let pdf = render(&long, Format::Pdf).unwrap();
    let raw = String::from_utf8_lossy(&pdf);
    assert!(raw.matches("/Type /Page\n").count() > 3);
    if let Ok(folder) = std::env::var("TWX_REPORT_FIXTURES_DIR") {
        std::fs::create_dir_all(&folder).unwrap();
        for (name, r) in [
            ("settlement", report),
            ("multipage-draft-negative", long),
            (
                "payroll-summary",
                build(&s, &request(ReportKind::PayrollSummary, None)).unwrap(),
            ),
            (
                "owner-payroll",
                build(&s, &request(ReportKind::OwnerPayroll, None)).unwrap(),
            ),
        ] {
            std::fs::write(
                std::path::Path::new(&folder).join(format!("{name}.json")),
                serde_json::to_vec_pretty(&r).unwrap(),
            )
            .unwrap();
            for f in [Format::Csv, Format::Xlsx, Format::Pdf, Format::Docx] {
                std::fs::write(
                    std::path::Path::new(&folder).join(format!("{name}.{}", f.extension())),
                    render(&r, f).unwrap(),
                )
                .unwrap();
            }
        }
    }
}
#[test]
fn publication_never_overwrites_cleans_failed_staging_and_sanitizes_names() {
    let app = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let file = dest.path().join("test.pdf");
    files::publish(&file, Format::Pdf, b"complete", app.path()).unwrap();
    assert!(files::publish(&file, Format::Pdf, b"changed", app.path()).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), b"complete");
    assert!(files::publish(
        &dest.path().join("missing/test.pdf"),
        Format::Pdf,
        b"x",
        app.path()
    )
    .is_err());
    assert!(files::publish(&app.path().join("x.pdf"), Format::Pdf, b"x", app.path()).is_err());
    assert!(files::publish(&dest.path().join("x.csv"), Format::Pdf, b"x", app.path()).is_err());
    assert_eq!(std::fs::read_dir(dest.path()).unwrap().count(), 1);
    assert_eq!(files::safe_filename("CON"), "TWX-CON");
    assert_eq!(files::safe_filename(" .. "), "TWX-Report");
    assert!(!files::safe_filename("truck:/\\?*<bad>\n")
        .contains(['/', '\\', ':', '?', '*', '<', '>', '\n']));
}
#[cfg(windows)]
#[test]
fn publication_write_lock_failure_leaves_original_and_no_temp_files() {
    use std::os::windows::fs::OpenOptionsExt;
    let app = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("locked.pdf");
    std::fs::write(&file, b"safe").unwrap();
    let _lock = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&file)
        .unwrap();
    assert!(files::publish(&file, Format::Pdf, b"bad", app.path()).is_err());
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 1);
}
#[test]
fn interrupted_write_removes_stage_and_never_publishes_partial_file() {
    use std::io::Write;
    let app = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("failed.pdf");
    assert!(
        files::publish_with(&file, Format::Pdf, app.path(), |stage| {
            stage.write_all(b"partial")?;
            Err(std::io::Error::other("simulated interrupted device write").into())
        })
        .is_err()
    );
    assert!(!file.exists());
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);
}
