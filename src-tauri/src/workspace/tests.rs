use super::*;
use crate::storage::{database_path, MIGRATIONS};
fn reference(s: &mut Storage, kind: &str, fields: Value) -> Value {
    let mut value = json!({"kind":kind,"id":null,"expectedVersion":null,"active":true});
    value
        .as_object_mut()
        .unwrap()
        .extend(fields.as_object().unwrap().clone());
    save_reference(s, serde_json::from_value(value).unwrap()).unwrap()
}
fn fixture() -> (tempfile::TempDir, Storage, SettlementWrite) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Storage::open(dir.path()).unwrap();
    let owner = reference(&mut s, "owners", json!({"name":"Synthetic Owner"}));
    let dtype = reference(
        &mut s,
        "driver_types",
        json!({"name":"Synthetic Driver Type","payoutPercentage":"85"}),
    );
    let driver = reference(
        &mut s,
        "drivers",
        json!({"name":"Synthetic Driver","driverTypeId":dtype["id"]}),
    );
    let truck = reference(
        &mut s,
        "trucks",
        json!({"truckNumber":"FAKE-001","ownerId":owner["id"],"defaultDriverId":driver["id"]}),
    );
    reference(
        &mut s,
        "trucks",
        json!({"truckNumber":"FAKE-002","ownerId":owner["id"]}),
    );
    let deduction = reference(
        &mut s,
        "adjustment_types",
        json!({"name":"Insurance","defaultDirection":"DEDUCT"}),
    );
    let addition = reference(
        &mut s,
        "adjustment_types",
        json!({"name":"Reimbursement","defaultDirection":"ADD"}),
    );
    let load = |tons: &str, rate: &str| json!({"id":null,"driverId":driver["id"],"truckId":truck["id"],"plantId":null,"jobId":null,"loadNumber":"FAKE-LOAD","bolNumber":"FAKE-BOL","poNumber":"FAKE-PO","tons":tons,"completionRate":rate,"mileage":"12.3456789"});
    let w=serde_json::from_value(json!({"id":null,"expectedVersion":null,"driverId":driver["id"],"truckId":truck["id"],"periodStart":"2026-10-01","periodEnd":"2026-10-19","notes":"Synthetic fixture","payoutPercentage":null,"loads":[load("10","5"),load("20","4")],"adjustments":[{"id":null,"adjustmentTypeId":deduction["id"],"twxCodeId":null,"explanation":"Synthetic deduction","direction":"DEDUCT","amount":"10"},{"id":null,"adjustmentTypeId":addition["id"],"twxCodeId":null,"explanation":"Synthetic addition","direction":"ADD","amount":"2"}]})).unwrap();
    (dir, s, w)
}
fn saved_input(a: &Value) -> SettlementWrite {
    serde_json::from_value(json!({"id":a["header"]["id"],"expectedVersion":a["header"]["row_version"],"driverId":a["header"]["driver_id"],"truckId":a["header"]["truck_id"],"periodStart":"2026-10-01","periodEnd":"2026-10-19","notes":"Edited synthetic fixture","payoutPercentage":null,"loads":a["loads"].as_array().unwrap().iter().map(|l|json!({"id":l["id"],"driverId":l["driver_id"],"truckId":l["truck_id"],"plantId":null,"jobId":null,"loadNumber":l["load_number"],"bolNumber":"FAKE-BOL","poNumber":"FAKE-PO","tons":if l["tons_milli"]=="10000" {"10"} else {"20"},"completionRate":if l["completion_rate_scaled"]=="50000" {"5"} else {"4"},"mileage":l["mileage"]})).collect::<Vec<_>>(),"adjustments":a["adjustments"].as_array().unwrap().iter().map(|v|json!({"id":v["id"],"adjustmentTypeId":v["adjustment_type_id"],"twxCodeId":null,"explanation":v["explanation"],"direction":v["direction"],"amount":if v["amount_cents"]=="1000" {"10"} else {"2"}})).collect::<Vec<_>>()})).unwrap()
}
#[test]
fn desktop_workflow_preview_finalize_ach_edit_history_and_reopen() {
    let (dir, mut s, w) = fixture();
    let p = preview(&s, &w).unwrap();
    assert_eq!(p["loadTotalsCents"], json!(["5000", "8000"]));
    assert_eq!(p["grossRouteCents"], "13000");
    assert_eq!(p["driverGrossCents"], "11050");
    assert_eq!(p["finalPayoutCents"], "10250");
    let a = save_settlement(&mut s, w, false).unwrap();
    let id = a["header"]["id"].as_str().unwrap().to_owned();
    let a = finalize(&mut s, &id, "1", false).unwrap();
    assert_eq!(a["header"]["status"], "FINALIZED");
    let pay = payrolls(&s, RecordFilter::default())
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(pay["amount_cents"], "10250");
    let code = reference(
        &mut s,
        "twx_codes",
        json!({"code":"FAKE-CODE","label":"Synthetic Code"}),
    );
    let pay=save_driver(&mut s,serde_json::from_value(json!({"id":pay["id"],"expectedVersion":pay["row_version"],"achCharge":"1.25","twxCodeId":code["id"],"notes":"Synthetic ACH note"})).unwrap()).unwrap();
    assert_eq!(pay["total_cents"], "10375");
    let mut w = saved_input(&a);
    w.loads[0].completion_rate = "6".into();
    let stale = w.clone();
    let edited = save_settlement(&mut s, w, false).unwrap();
    assert_eq!(edited["header"]["final_payout_cents"], "11100");
    assert_eq!(
        save_settlement(&mut s, stale, false).unwrap_err().code,
        "CONFLICT"
    );
    let pay = payrolls(&s, RecordFilter::default())
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(pay["amount_cents"], "11100");
    assert_eq!(pay["total_cents"], "11225");
    assert_eq!(pay["notes"], "Synthetic ACH note");
    assert_eq!(pay["twx_code_snapshot"], "FAKE-CODE");
    assert_eq!(events(&s, Entity::Settlement, &id).unwrap().len(), 3);
    assert!(s
        .connection
        .query_row(
            "SELECT change_generation>saved_generation FROM backup_settings",
            [],
            |r| r.get::<_, bool>(0)
        )
        .unwrap());
    drop(s);
    let s = Storage::open(dir.path()).unwrap();
    assert_eq!(
        settlement(&s, &id).unwrap()["loads"][0]["mileage"],
        "12.3456789"
    );
    assert_eq!(
        payrolls(&s, RecordFilter::default()).unwrap()[0]["total_cents"],
        "11225"
    );
}
#[test]
fn negative_payout_requires_confirmation_and_never_clamps() {
    let (_dir, mut s, mut w) = fixture();
    w.adjustments[0].amount = "500".into();
    let a = save_settlement(&mut s, w, false).unwrap();
    let id = a["header"]["id"].as_str().unwrap();
    assert_eq!(
        finalize(&mut s, id, "1", false).unwrap_err().code,
        "NEGATIVE_CONFIRMATION"
    );
    assert!(payrolls(&s, RecordFilter::default()).unwrap().is_empty());
    let a = finalize(&mut s, id, "1", true).unwrap();
    assert_eq!(a["header"]["final_payout_cents"], "-38750");
    let mut edit = saved_input(&a);
    edit.adjustments[0].amount = "500".into();
    assert_eq!(
        save_settlement(&mut s, edit, false).unwrap_err().code,
        "NEGATIVE_CONFIRMATION"
    );
}
#[test]
fn exact_ipc_rejects_numbers_overprecision_bad_dates_and_amount_overwrite() {
    let (_dir, s, mut w) = fixture();
    w.loads[0].tons = "0.0001".into();
    assert_eq!(
        preview(&s, &w).unwrap_err().field.as_deref(),
        Some("loads.0.tons")
    );
    w.loads[0].tons = "NaN".into();
    assert!(preview(&s, &w).is_err());
    w.period_end = "2026-09-30".into();
    assert_eq!(
        preview(&s, &w).unwrap_err().field.as_deref(),
        Some("periodEnd")
    );
    assert!(serde_json::from_value::<DriverPayrollWrite>(
        json!({"id":"x","expectedVersion":"1","achCharge":1.25,"twxCodeId":null,"notes":""})
    )
    .is_err());
    assert!(serde_json::from_value::<DriverPayrollWrite>(json!({"id":"x","expectedVersion":"1","achCharge":"1.25","twxCodeId":null,"notes":"","amount":"999"})).is_err());
}
#[test]
fn owner_manual_no_ach_and_parameterized_history_filters() {
    let (_dir, mut s, w) = fixture();
    let r = references(&s).unwrap();
    assert_eq!(r["trucks"].as_array().unwrap().len(), 2);
    let pay=save_owner(&mut s,serde_json::from_value(json!({"id":null,"expectedVersion":null,"ownerId":r["owners"][0]["id"],"truckId":r["trucks"][0]["id"],"twxCodeId":null,"periodStart":"2026-10-01","periodEnd":"2026-10-19","amount":"123.45","notes":"Manual synthetic owner amount"})).unwrap()).unwrap();
    assert!(pay["ach_charge_cents"].is_null());
    assert_eq!(pay["total_cents"], "12345");
    save_settlement(&mut s, w.clone(), false).unwrap();
    assert_eq!(
        settlements(
            &s,
            RecordFilter {
                driver_id: Some(w.driver_id),
                truck_id: Some(w.truck_id),
                period_start: Some("2026-10-18".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .len(),
        1
    );
    assert!(settlements(
        &s,
        RecordFilter {
            driver_id: Some("' OR 1=1 --".into()),
            ..Default::default()
        }
    )
    .unwrap()
    .is_empty());
    assert!(payrolls(
        &s,
        RecordFilter {
            period_start: Some("2027-01-01".into()),
            ..Default::default()
        }
    )
    .unwrap()
    .is_empty());
}
#[test]
fn default_driver_and_rate_edits_preserve_selected_history() {
    let (_dir, mut s, w) = fixture();
    let a = save_settlement(&mut s, w, false).unwrap();
    let id = a["header"]["id"].as_str().unwrap();
    let a = finalize(&mut s, id, "1", false).unwrap();
    let r = references(&s).unwrap();
    let truck = &r["trucks"][0];
    save_reference(&mut s,serde_json::from_value(json!({"kind":"trucks","id":truck["id"],"expectedVersion":truck["row_version"],"active":true,"truckNumber":truck["truck_number"],"ownerId":truck["owner_id"],"defaultDriverId":null})).unwrap()).unwrap();
    let dtype = &r["driver_types"][0];
    save_reference(&mut s,serde_json::from_value(json!({"kind":"driver_types","id":dtype["id"],"expectedVersion":dtype["row_version"],"active":true,"name":"Changed Type","payoutPercentage":"50"})).unwrap()).unwrap();
    let old = settlement(&s, id).unwrap();
    assert_eq!(old["header"]["driver_id"], a["header"]["driver_id"]);
    assert_eq!(old["header"]["payout_rate_ppm"], "850000");
    assert_eq!(
        preview(&s, &saved_input(&a)).unwrap()["payoutRatePpm"],
        "850000"
    );
}
#[test]
fn phase3_migration_and_backup_compatibility_preserve_source_and_optional_mileage() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("data")).unwrap();
    let path = database_path(dir.path());
    let mut c = rusqlite::Connection::open(&path).unwrap();
    MIGRATIONS.to_version(&mut c, 3).unwrap();
    c.execute(
        "INSERT INTO owners(id,name) VALUES('old-owner','Synthetic legacy owner')",
        [],
    )
    .unwrap();
    drop(c);
    let s = Storage::open(dir.path()).unwrap();
    assert_eq!(s.info().unwrap().schema_version, 4);
    assert_eq!(
        references(&s).unwrap()["owners"][0]["name"],
        "Synthetic legacy owner"
    );
    assert!(s.connection.execute("INSERT INTO loads(id,settlement_id,position,driver_id,truck_id,driver_name_snapshot,truck_number_snapshot,mileage) VALUES('bad','none',0,'none','none','x','x','1e5')",[]).is_err());
    for v in ["-1", "NaN", "1e5", ".5", "1.", "1.2.3"] {
        assert!(domain::validate_mileage(Some(v)).is_err());
    }
    assert!(domain::validate_mileage(Some("12.1234567890123456789")).is_ok());
    assert!(domain::validate_mileage(None).is_ok());
}
