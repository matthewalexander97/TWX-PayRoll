use super::*;
use crate::{
    repositories::Catalog,
    storage::{database_path, Storage, Theme},
};

fn fixture() -> (tempfile::TempDir, Storage) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Storage::open(dir.path()).unwrap();
    let records = [
        (
            "type",
            Catalog::DriverType {
                name: "Synthetic Type",
                payout_rate_ppm: 850000,
            },
        ),
        (
            "driver",
            Catalog::Driver {
                name: "Synthetic Driver A",
                driver_type_id: "type",
            },
        ),
        (
            "other",
            Catalog::Driver {
                name: "Synthetic Driver B",
                driver_type_id: "type",
            },
        ),
        (
            "owner",
            Catalog::Owner {
                name: "Synthetic Owner",
            },
        ),
        (
            "truck",
            Catalog::Truck {
                truck_number: "TEST-001",
                owner_id: "owner",
                default_driver_id: Some("driver"),
            },
        ),
        (
            "truck2",
            Catalog::Truck {
                truck_number: "TEST-002",
                owner_id: "owner",
                default_driver_id: None,
            },
        ),
        (
            "code",
            Catalog::TwxCode {
                code: "54520",
                label: "Synthetic Insurance Code",
            },
        ),
        (
            "plant",
            Catalog::Plant {
                name: "Synthetic Plant",
            },
        ),
        (
            "job",
            Catalog::Job {
                name: "Synthetic Job",
                plant_id: Some("plant"),
            },
        ),
        (
            "insurance",
            Catalog::AdjustmentType {
                name: "Insurance",
                default_direction: Direction::Deduct,
                twx_code_id: Some("code"),
            },
        ),
        (
            "reimbursement",
            Catalog::AdjustmentType {
                name: "Reimbursement",
                default_direction: Direction::Add,
                twx_code_id: None,
            },
        ),
    ];
    for (id, item) in records {
        store.catalogs().save(id, None, true, item).unwrap();
    }
    (dir, store)
}
fn input() -> SettlementInput {
    SettlementInput {
        id: "s1".into(),
        expected_version: None,
        driver_id: "driver".into(),
        truck_id: "truck".into(),
        period_start: "2026-10-01".into(),
        period_end: "2026-10-19".into(),
        notes: "Synthetic test only".into(),
        payout_rate_ppm: None,
        loads: vec![LoadInput {
            id: "load1".into(),
            driver_id: "driver".into(),
            truck_id: "truck".into(),
            plant_id: Some("plant".into()),
            job_id: Some("job".into()),
            load_number: "TEST-L1".into(),
            bol_number: "TEST-BOL".into(),
            po_number: "TEST-PO".into(),
            tons_milli: 12345,
            completion_rate_scaled: 67891,
            mileage: None,
        }],
        adjustments: vec![AdjustmentInput {
            id: "adj1".into(),
            adjustment_type_id: "insurance".into(),
            twx_code_id: Some("code".into()),
            explanation: "Synthetic insurance".into(),
            direction: Direction::Deduct,
            amount_cents: 100,
        }],
    }
}
fn finalize(store: &mut Storage) -> Value {
    store
        .business()
        .save_settlement(&input(), "test create")
        .unwrap();
    store
        .business()
        .finalize_settlement("s1", 1, "test finalize")
        .unwrap()
}
fn count(store: &Storage, table: &str) -> i64 {
    store
        .connection
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn finalization_idempotency_ach_and_historical_reopen() {
    let (dir, mut store) = fixture();
    let saved = finalize(&mut store);
    assert_eq!(saved["loads"][0]["total_cents"], "8381");
    assert_eq!(saved["header"]["gross_route_cents"], "8381");
    assert_eq!(saved["header"]["driver_gross_cents"], "7124");
    assert_eq!(saved["header"]["final_payout_cents"], "7024");
    assert_eq!(saved["header"]["payout_rate_ppm"], "850000");
    assert_eq!(count(&store, "payroll_entries"), 1);
    let events = count(&store, "audit_events");
    assert_eq!(
        store
            .business()
            .finalize_settlement("s1", 2, "retry")
            .unwrap(),
        saved
    );
    assert_eq!(count(&store, "audit_events"), events);
    assert_eq!(count(&store, "payroll_entries"), 1);
    let payroll = store
        .business()
        .update_driver_payroll(
            "settlement:s1",
            1,
            125,
            Some("code"),
            "ACH test",
            "test ACH",
        )
        .unwrap();
    assert_eq!(payroll["amount_cents"], "7024");
    assert_eq!(payroll["total_cents"], "7149");
    assert_eq!(store.history().settlement("s1").unwrap(), saved);
    drop(store);
    let store = Storage::open(dir.path()).unwrap();
    assert_eq!(store.history().settlement("s1").unwrap(), saved);
    assert_eq!(
        store.history().payroll("settlement:s1").unwrap()["total_cents"],
        "7149"
    );
    assert_eq!(
        store
            .history()
            .settlements_ending_on_or_after("2026-10-19")
            .unwrap()
            .len(),
        1
    );
    assert!(store
        .history()
        .settlements_ending_on_or_after("2026-10-20")
        .unwrap()
        .is_empty());
}
#[test]
fn finalized_edit_syncs_payroll_and_preserves_before_after() {
    let (_dir, mut store) = fixture();
    let old = finalize(&mut store);
    store
        .business()
        .update_driver_payroll(
            "settlement:s1",
            1,
            125,
            Some("code"),
            "Preserve notes",
            "set ACH",
        )
        .unwrap();
    let mut edit = input();
    edit.expected_version = Some(2);
    edit.loads[0].tons_milli = 20000;
    edit.adjustments.push(AdjustmentInput {
        id: "adj2".into(),
        adjustment_type_id: "reimbursement".into(),
        twx_code_id: None,
        explanation: "Synthetic refund".into(),
        direction: Direction::Add,
        amount_cents: 500,
    });
    let new = store
        .business()
        .save_settlement(&edit, "Correct load weight")
        .unwrap();
    assert_eq!(new["header"]["gross_route_cents"], "13578");
    assert_eq!(new["header"]["driver_gross_cents"], "11541");
    assert_eq!(new["header"]["final_payout_cents"], "11941");
    let payroll = store.history().payroll("settlement:s1").unwrap();
    assert_eq!(payroll["amount_cents"], "11941");
    assert_eq!(payroll["ach_charge_cents"], "125");
    assert_eq!(payroll["total_cents"], "12066");
    assert_eq!(payroll["twx_code_snapshot"], "54520");
    assert_eq!(payroll["notes"], "Preserve notes");
    assert_eq!(count(&store, "payroll_entries"), 1);
    let events = store
        .history()
        .events(crate::repositories::FinancialEntity::Settlement, "s1")
        .unwrap();
    let last = events.last().unwrap();
    assert_eq!(last["category"], "CORRECTION");
    assert_eq!(last["reason"], "Correct load weight");
    assert_eq!(
        serde_json::from_str::<Value>(text(last, "before_json").unwrap()).unwrap(),
        old
    );
    assert_eq!(
        serde_json::from_str::<Value>(text(last, "after_json").unwrap()).unwrap(),
        new
    );
    assert!(last["occurred_at"].as_str().unwrap().ends_with('Z'));
}
#[test]
fn reference_edits_and_default_driver_do_not_rewrite_history() {
    let (_dir, mut store) = fixture();
    let old = finalize(&mut store);
    let edits = [
        (
            "type",
            Catalog::DriverType {
                name: "Changed Type",
                payout_rate_ppm: 500000,
            },
        ),
        (
            "driver",
            Catalog::Driver {
                name: "Changed Driver",
                driver_type_id: "type",
            },
        ),
        (
            "owner",
            Catalog::Owner {
                name: "Changed Owner",
            },
        ),
        (
            "truck",
            Catalog::Truck {
                truck_number: "CHANGED-001",
                owner_id: "owner",
                default_driver_id: Some("other"),
            },
        ),
        (
            "code",
            Catalog::TwxCode {
                code: "CHANGED",
                label: "Changed Label",
            },
        ),
        (
            "plant",
            Catalog::Plant {
                name: "Changed Plant",
            },
        ),
        (
            "job",
            Catalog::Job {
                name: "Changed Job",
                plant_id: Some("plant"),
            },
        ),
        (
            "insurance",
            Catalog::AdjustmentType {
                name: "Changed Type Label",
                default_direction: Direction::Add,
                twx_code_id: None,
            },
        ),
    ];
    for (id, item) in edits {
        store.catalogs().save(id, Some(1), true, item).unwrap();
    }
    assert_eq!(store.history().settlement("s1").unwrap(), old);
    let mut edit = input();
    edit.expected_version = Some(2);
    edit.notes = "New note".into();
    let new = store
        .business()
        .save_settlement(&edit, "Note correction")
        .unwrap();
    for field in [
        "driver_name_snapshot",
        "owner_name_snapshot",
        "truck_number_snapshot",
        "driver_type_name_snapshot",
        "payout_rate_ppm",
    ] {
        assert_eq!(new["header"][field], old["header"][field]);
    }
    assert_eq!(new["loads"], old["loads"]);
    assert_eq!(new["adjustments"], old["adjustments"]);
    assert_eq!(new["header"]["driver_id"], "driver");
    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT count(*) FROM trucks WHERE owner_id='owner'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
    let mut another = input();
    another.id = "s2".into();
    another.loads[0].id = "load2".into();
    another.adjustments[0].id = "newadj".into();
    let second = store
        .business()
        .save_settlement(&another, "New period")
        .unwrap();
    assert_eq!(second["header"]["payout_rate_ppm"], "500000");
    assert_eq!(second["header"]["driver_name_snapshot"], "Changed Driver");
}
#[test]
fn explicit_rate_and_actual_load_driver_are_preserved() {
    let (_dir, mut store) = fixture();
    let mut data = input();
    data.payout_rate_ppm = Some(0);
    data.loads[0].driver_id = "other".into();
    data.loads[0].truck_id = "truck2".into();
    store
        .business()
        .save_settlement(&data, "Synthetic alternate driver")
        .unwrap();
    let saved = store
        .business()
        .finalize_settlement("s1", 1, "Finalize")
        .unwrap();
    assert_eq!(saved["loads"][0]["driver_id"], "other");
    assert_eq!(saved["header"]["driver_gross_cents"], "0");
    assert_eq!(saved["header"]["final_payout_cents"], "-100");
    data.expected_version = Some(2);
    data.payout_rate_ppm = Some(1_000_000);
    assert_eq!(
        store
            .business()
            .save_settlement(&data, "Explicit rate change")
            .unwrap()["header"]["driver_gross_cents"],
        "8381"
    );
}
#[test]
fn stale_versions_invalid_inputs_and_fk_fail_without_partial_writes() {
    let (_dir, mut store) = fixture();
    let old = finalize(&mut store);
    let events = count(&store, "audit_events");
    assert!(matches!(
        store.business().finalize_settlement("s1", 1, "Stale"),
        Err(Error::Conflict)
    ));
    let mut edit = input();
    edit.expected_version = Some(1);
    assert!(matches!(
        store.business().save_settlement(&edit, "Stale"),
        Err(Error::Conflict)
    ));
    edit.expected_version = Some(2);
    edit.loads[0].tons_milli = -1;
    assert!(store.business().save_settlement(&edit, "Negative").is_err());
    edit.loads[0].tons_milli = 12345;
    edit.loads[0].driver_id = "missing".into();
    assert!(store
        .business()
        .save_settlement(&edit, "Missing FK")
        .is_err());
    edit.loads[0].driver_id = "driver".into();
    edit.period_end = "2026-09-30".into();
    assert!(store.business().save_settlement(&edit, "Reversed").is_err());
    assert_eq!(store.history().settlement("s1").unwrap(), old);
    assert_eq!(count(&store, "audit_events"), events);
    assert!(store
        .connection
        .execute("UPDATE trucks SET owner_id='missing' WHERE id='truck'", [])
        .is_err());
    assert!(store
        .connection
        .execute("DELETE FROM drivers WHERE id='driver'", [])
        .is_err());
    assert!(store
        .connection
        .execute(
            "UPDATE weekly_settlements SET period_end='2026-02-30' WHERE id='s1'",
            []
        )
        .is_err());
    assert!(store
        .connection
        .execute(
            "UPDATE weekly_settlements SET payout_rate_ppm=1000001 WHERE id='s1'",
            []
        )
        .is_err());
    assert!(store
        .connection
        .execute("UPDATE loads SET tons_milli=-1 WHERE id='load1'", [])
        .is_err());
}
#[test]
fn audit_failure_rolls_back_every_finalization_write_and_retry_succeeds() {
    let (_dir, mut store) = fixture();
    store
        .business()
        .save_settlement(&input(), "Create")
        .unwrap();
    let before = store.history().settlement("s1").unwrap();
    store.connection.execute_batch("CREATE TRIGGER test_abort BEFORE INSERT ON audit_events WHEN NEW.entity_type='settlement' AND NEW.category='FINALIZE' BEGIN SELECT RAISE(ABORT,'synthetic failure after payroll sync'); END;").unwrap();
    assert!(store
        .business()
        .finalize_settlement("s1", 1, "Fail")
        .is_err());
    assert_eq!(store.history().settlement("s1").unwrap(), before);
    assert_eq!(count(&store, "payroll_entries"), 0);
    assert_eq!(count(&store, "audit_events"), 1);
    store
        .connection
        .execute_batch("DROP TRIGGER test_abort")
        .unwrap();
    store
        .business()
        .finalize_settlement("s1", 1, "Retry")
        .unwrap();
    assert_eq!(count(&store, "payroll_entries"), 1);
}
#[test]
fn finalized_correction_failure_rolls_back_children_payroll_and_audit() {
    let (_dir, mut store) = fixture();
    let before = finalize(&mut store);
    let payroll = store.history().payroll("settlement:s1").unwrap();
    let events = count(&store, "audit_events");
    store.connection.execute_batch("CREATE TRIGGER test_abort BEFORE INSERT ON audit_events WHEN NEW.entity_type='settlement' AND NEW.category='CORRECTION' BEGIN SELECT RAISE(ABORT,'synthetic correction failure'); END;").unwrap();
    let mut edit = input();
    edit.expected_version = Some(2);
    edit.loads[0].tons_milli = 30000;
    assert!(store
        .business()
        .save_settlement(&edit, "Fail late")
        .is_err());
    assert_eq!(store.history().settlement("s1").unwrap(), before);
    assert_eq!(store.history().payroll("settlement:s1").unwrap(), payroll);
    assert_eq!(count(&store, "audit_events"), events);
}
#[test]
fn overflow_during_ach_sync_rolls_back_settlement_edit() {
    let (_dir, mut store) = fixture();
    let mut data = input();
    data.loads.clear();
    data.adjustments.clear();
    store.business().save_settlement(&data, "Zero").unwrap();
    let old = store
        .business()
        .finalize_settlement("s1", 1, "Zero finalize")
        .unwrap();
    store
        .business()
        .update_driver_payroll("settlement:s1", 1, i64::MAX, None, "", "Maximum ACH")
        .unwrap();
    data.expected_version = Some(2);
    data.loads = input().loads;
    assert!(matches!(
        store.business().save_settlement(&data, "Overflow"),
        Err(Error::Overflow)
    ));
    assert_eq!(store.history().settlement("s1").unwrap(), old);
    assert_eq!(
        store.history().payroll("settlement:s1").unwrap()["amount_cents"],
        "0"
    );
}
#[test]
fn owner_payroll_no_ach_and_xor_constraints() {
    let (_dir, mut store) = fixture();
    let mut data = OwnerPayrollInput {
        id: "owner-pay".into(),
        expected_version: None,
        owner_id: "owner".into(),
        truck_id: "truck".into(),
        twx_code_id: Some("code".into()),
        period_start: "2026-10-02".into(),
        period_end: "2026-10-02".into(),
        notes: "Synthetic".into(),
        amount_cents: 10000,
    };
    let old = store
        .business()
        .save_owner_payroll(&data, "Manual owner amount")
        .unwrap();
    assert_eq!(old["amount_cents"], "10000");
    assert_eq!(old["total_cents"], "10000");
    assert!(old["ach_charge_cents"].is_null());
    assert!(store
        .business()
        .update_driver_payroll("owner-pay", 1, 1, None, "", "Invalid")
        .is_err());
    assert!(store
        .connection
        .execute(
            "UPDATE payroll_entries SET ach_charge_cents=0 WHERE id='owner-pay'",
            []
        )
        .is_err());
    assert!(store
        .connection
        .execute(
            "UPDATE payroll_entries SET driver_id='driver' WHERE id='owner-pay'",
            []
        )
        .is_err());
    assert!(store
        .connection
        .execute(
            "UPDATE payroll_entries SET owner_id=NULL WHERE id='owner-pay'",
            []
        )
        .is_err());
    data.expected_version = Some(1);
    data.amount_cents = 12000;
    store
        .business()
        .save_owner_payroll(&data, "Owner correction")
        .unwrap();
    let history = store
        .history()
        .events(crate::repositories::FinancialEntity::Payroll, "owner-pay")
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(text(history.last().unwrap(), "before_json").unwrap())
            .unwrap(),
        old
    );
    finalize(&mut store);
    assert!(store
        .connection
        .execute(
            "UPDATE payroll_entries SET ach_charge_cents=NULL WHERE id='settlement:s1'",
            []
        )
        .is_err());
    assert!(store
        .connection
        .execute(
            "UPDATE payroll_entries SET owner_id='owner' WHERE id='settlement:s1'",
            []
        )
        .is_err());
    assert!(store
        .connection
        .execute(
            "UPDATE payroll_entries SET settlement_id=NULL WHERE id='settlement:s1'",
            []
        )
        .is_err());
}
#[test]
fn audit_append_only_finalized_delete_and_unknown_rules() {
    let (_dir, mut store) = fixture();
    finalize(&mut store);
    assert!(store
        .connection
        .execute("UPDATE audit_events SET reason='erase'", [])
        .is_err());
    assert!(store
        .connection
        .execute("DELETE FROM audit_events", [])
        .is_err());
    assert!(store.connection.execute("INSERT OR REPLACE INTO audit_events(id,entity_type,entity_id,category,reason,after_json) SELECT id,entity_type,entity_id,category,'replace history','{}' FROM audit_events LIMIT 1",[]).is_err());
    assert!(store
        .connection
        .execute("DELETE FROM weekly_settlements WHERE id='s1'", [])
        .is_err());
    assert!(store
        .connection
        .execute("DELETE FROM payroll_entries WHERE id='settlement:s1'", [])
        .is_err());
    store
        .connection
        .execute(
            "UPDATE weekly_settlements SET rule_version='future-policy' WHERE id='s1'",
            [],
        )
        .unwrap();
    assert!(store
        .business()
        .finalize_settlement("s1", 2, "Unknown rule")
        .is_err());
    let mut edit = input();
    edit.expected_version = Some(2);
    assert!(store
        .business()
        .save_settlement(&edit, "Unknown rule edit")
        .is_err());
}
#[test]
fn migrate_real_phase1_preserves_settings_and_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let path = database_path(dir.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("../../migrations/001_settings.sql"))
        .unwrap();
    connection
        .execute("UPDATE app_settings SET value='dark'", [])
        .unwrap();
    connection.pragma_update(None, "user_version", 1).unwrap();
    drop(connection);
    for _ in 0..2 {
        let store = Storage::open(dir.path()).unwrap();
        assert_eq!(store.info().unwrap().schema_version, 4);
        assert_eq!(store.preferences().unwrap().theme, Theme::Dark);
        assert_eq!(count(&store, "audit_events"), 0);
        assert_eq!(
            store
                .connection
                .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            0
        );
        assert_eq!(
            store
                .connection
                .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
    }
}
#[test]
fn failed_phase2_migration_preserves_phase1_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = database_path(dir.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(include_str!("../../migrations/001_settings.sql"))
        .unwrap();
    connection.execute_batch("PRAGMA user_version=1; UPDATE app_settings SET value='dark'; CREATE TABLE loads(synthetic TEXT); INSERT INTO loads VALUES('retain me');").unwrap();
    drop(connection);
    assert!(Storage::open(dir.path()).is_err());
    let connection = Connection::open(path).unwrap();
    assert_eq!(
        connection
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT value FROM app_settings", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "dark"
    );
    assert_eq!(
        connection
            .query_row("SELECT synthetic FROM loads", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "retain me"
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name='driver_types'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn multiple_rounded_loads_and_large_exact_values() {
    let (_dir, mut store) = fixture();
    let mut data = input();
    data.payout_rate_ppm = Some(1_000_000);
    data.adjustments.clear();
    data.loads[0].tons_milli = 1;
    data.loads[0].completion_rate_scaled = 50_000;
    let mut second = data.loads[0].clone();
    second.id = "second-load".into();
    data.loads.push(second);
    store
        .business()
        .save_settlement(&data, "Rounded lines")
        .unwrap();
    let saved = store
        .business()
        .finalize_settlement("s1", 1, "Finalize rounded lines")
        .unwrap();
    assert_eq!(saved["header"]["gross_route_cents"], "2");
    assert_eq!(saved["header"]["final_payout_cents"], "2");
    data.expected_version = Some(2);
    data.loads.truncate(1);
    data.loads[0].tons_milli = i64::MAX;
    data.loads[0].completion_rate_scaled = 100_000;
    let maximum = store
        .business()
        .save_settlement(&data, "Maximum exact output")
        .unwrap();
    assert_eq!(
        maximum["header"]["final_payout_cents"],
        i64::MAX.to_string()
    );
    assert_eq!(
        store.history().payroll("settlement:s1").unwrap()["total_cents"],
        i64::MAX.to_string()
    );
    data.expected_version = Some(3);
    data.loads[0].completion_rate_scaled = 100_001;
    assert!(store
        .business()
        .save_settlement(&data, "Overflow rejected")
        .is_err());
    assert_eq!(store.history().settlement("s1").unwrap(), maximum);
}

#[test]
fn history_distinguishes_payroll_and_settlement_with_same_id() {
    let (_dir, mut store) = fixture();
    finalize(&mut store);
    let owner = OwnerPayrollInput {
        id: "s1".into(),
        expected_version: None,
        owner_id: "owner".into(),
        truck_id: "truck".into(),
        twx_code_id: None,
        period_start: "2026-10-01".into(),
        period_end: "2026-10-01".into(),
        notes: String::new(),
        amount_cents: 0,
    };
    store
        .business()
        .save_owner_payroll(&owner, "Separate entity")
        .unwrap();
    assert_eq!(
        store
            .history()
            .events(crate::repositories::FinancialEntity::Settlement, "s1")
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        store
            .history()
            .events(crate::repositories::FinancialEntity::Payroll, "s1")
            .unwrap()
            .len(),
        1
    );
}
