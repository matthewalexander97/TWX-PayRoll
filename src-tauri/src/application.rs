//! Aggregate application boundary: all financial mutations, snapshots and audits commit together.
use crate::domain::{self, Direction, Error, Result, RULE_VERSION};
use crate::repositories::{self as repo, integer, text};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde_json::Value;

#[derive(Clone)]
pub struct LoadInput {
    pub id: String,
    pub driver_id: String,
    pub truck_id: String,
    pub plant_id: Option<String>,
    pub job_id: Option<String>,
    pub load_number: String,
    pub bol_number: String,
    pub po_number: String,
    pub tons_milli: i64,
    pub completion_rate_scaled: i64,
    pub mileage: Option<String>,
}
#[derive(Clone)]
pub struct AdjustmentInput {
    pub id: String,
    pub adjustment_type_id: String,
    pub twx_code_id: Option<String>,
    pub explanation: String,
    pub direction: Direction,
    pub amount_cents: i64,
}
#[derive(Clone)]
pub struct SettlementInput {
    pub id: String,
    pub expected_version: Option<i64>,
    pub driver_id: String,
    pub truck_id: String,
    pub period_start: String,
    pub period_end: String,
    pub notes: String,
    /// None selects current driver-type rate on creation and preserves saved rate on edit.
    pub payout_rate_ppm: Option<i64>,
    pub loads: Vec<LoadInput>,
    pub adjustments: Vec<AdjustmentInput>,
}
pub struct OwnerPayrollInput {
    pub id: String,
    pub expected_version: Option<i64>,
    pub owner_id: String,
    pub truck_id: String,
    pub twx_code_id: Option<String>,
    pub period_start: String,
    pub period_end: String,
    pub notes: String,
    pub amount_cents: i64,
}
pub struct BusinessService<'a> {
    connection: &'a mut Connection,
}
impl<'a> BusinessService<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        Self { connection }
    }
    /// Replace a draft aggregate; finalized edits automatically recalculate and audit.
    pub fn save_settlement(&mut self, input: &SettlementInput, reason: &str) -> Result<Value> {
        reason_required(reason)?;
        domain::validate_period(&input.period_start, &input.period_end)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = repo::rows(
            &tx,
            "SELECT * FROM weekly_settlements WHERE id=?1",
            &input.id,
        )?
        .pop();
        check_version(existing.as_ref(), input.expected_version)?;
        let before = existing
            .as_ref()
            .map(|_| repo::aggregate(&tx, &input.id))
            .transpose()?;
        let finalized = existing
            .as_ref()
            .is_some_and(|v| v["status"] == "FINALIZED");
        if finalized {
            check_rule(existing.as_ref().ok_or(Error::Missing)?)?;
        }
        let driver_name = preserved_label(
            &tx,
            existing.as_ref(),
            "driver_id",
            &input.driver_id,
            "driver_name_snapshot",
            "drivers",
            "name",
        )?;
        let truck_number = preserved_label(
            &tx,
            existing.as_ref(),
            "truck_id",
            &input.truck_id,
            "truck_number_snapshot",
            "trucks",
            "truck_number",
        )?;
        let same_truck = existing
            .as_ref()
            .is_some_and(|v| v["truck_id"] == input.truck_id);
        let owner_id = if same_truck {
            text(existing.as_ref().ok_or(Error::Missing)?, "owner_id")?.to_owned()
        } else {
            repo::label(&tx, "trucks", "owner_id", &input.truck_id)?
        };
        let owner_name = preserved_label(
            &tx,
            existing.as_ref(),
            "owner_id",
            &owner_id,
            "owner_name_snapshot",
            "owners",
            "name",
        )?;
        let (type_name,default_rate):(String,i64)=tx.query_row("SELECT t.name,t.payout_rate_ppm FROM drivers d JOIN driver_types t ON t.id=d.driver_type_id WHERE d.id=?1",[&input.driver_id],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let type_name = match existing
            .as_ref()
            .filter(|v| v["driver_id"] == input.driver_id)
        {
            Some(old) => text(old, "driver_type_name_snapshot")?.to_owned(),
            None => type_name,
        };
        let rate = input.payout_rate_ppm.unwrap_or(match existing.as_ref() {
            Some(old) => integer(old, "payout_rate_ppm")?,
            None => default_rate,
        });
        domain::payout_rate(rate)?;
        let version = input
            .expected_version
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        tx.execute("INSERT INTO weekly_settlements(id,driver_id,truck_id,owner_id,driver_name_snapshot,truck_number_snapshot,owner_name_snapshot,driver_type_name_snapshot,payout_rate_ppm,period_start,period_end,notes,row_version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13) ON CONFLICT(id) DO UPDATE SET driver_id=excluded.driver_id,truck_id=excluded.truck_id,owner_id=excluded.owner_id,driver_name_snapshot=excluded.driver_name_snapshot,truck_number_snapshot=excluded.truck_number_snapshot,owner_name_snapshot=excluded.owner_name_snapshot,driver_type_name_snapshot=excluded.driver_type_name_snapshot,payout_rate_ppm=excluded.payout_rate_ppm,period_start=excluded.period_start,period_end=excluded.period_end,notes=excluded.notes,row_version=excluded.row_version,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![input.id,input.driver_id,input.truck_id,owner_id,driver_name,truck_number,owner_name,type_name,rate,input.period_start,input.period_end,input.notes,version])?;
        tx.execute("DELETE FROM loads WHERE settlement_id=?1", [&input.id])?;
        tx.execute(
            "DELETE FROM settlement_adjustments WHERE settlement_id=?1",
            [&input.id],
        )?;
        for (position, load) in input.loads.iter().enumerate() {
            domain::load_total(load.tons_milli, load.completion_rate_scaled)?;
            domain::validate_mileage(load.mileage.as_deref())?;
            let old = child_before(before.as_ref(), "loads", &load.id);
            let driver = preserved_label(
                &tx,
                old,
                "driver_id",
                &load.driver_id,
                "driver_name_snapshot",
                "drivers",
                "name",
            )?;
            let truck = preserved_label(
                &tx,
                old,
                "truck_id",
                &load.truck_id,
                "truck_number_snapshot",
                "trucks",
                "truck_number",
            )?;
            let plant = optional_label(
                &tx,
                old,
                "plant_id",
                load.plant_id.as_deref(),
                "plant_name_snapshot",
                "plants",
                "name",
            )?;
            let job = optional_label(
                &tx,
                old,
                "job_id",
                load.job_id.as_deref(),
                "job_name_snapshot",
                "jobs",
                "name",
            )?;
            tx.execute("INSERT INTO loads(id,settlement_id,position,driver_id,truck_id,driver_name_snapshot,truck_number_snapshot,plant_id,job_id,plant_name_snapshot,job_name_snapshot,load_number,bol_number,po_number,tons_milli,completion_rate_scaled) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",params![load.id,input.id,index(position)?,load.driver_id,load.truck_id,driver,truck,load.plant_id,load.job_id,plant,job,load.load_number,load.bol_number,load.po_number,load.tons_milli,load.completion_rate_scaled])?;
            tx.execute(
                "UPDATE loads SET mileage=?2 WHERE id=?1",
                params![load.id, load.mileage],
            )?;
        }
        for (position, adjustment) in input.adjustments.iter().enumerate() {
            domain::nonnegative(adjustment.amount_cents)?;
            let old = child_before(before.as_ref(), "adjustments", &adjustment.id);
            let name = preserved_label(
                &tx,
                old,
                "adjustment_type_id",
                &adjustment.adjustment_type_id,
                "type_name_snapshot",
                "adjustment_types",
                "name",
            )?;
            let code = optional_label(
                &tx,
                old,
                "twx_code_id",
                adjustment.twx_code_id.as_deref(),
                "twx_code_snapshot",
                "twx_codes",
                "code",
            )?;
            let label = optional_label(
                &tx,
                old,
                "twx_code_id",
                adjustment.twx_code_id.as_deref(),
                "twx_label_snapshot",
                "twx_codes",
                "label",
            )?;
            tx.execute("INSERT INTO settlement_adjustments(id,settlement_id,position,adjustment_type_id,type_name_snapshot,twx_code_id,twx_code_snapshot,twx_label_snapshot,explanation,direction,amount_cents) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![adjustment.id,input.id,index(position)?,adjustment.adjustment_type_id,name,adjustment.twx_code_id,code,label,adjustment.explanation,adjustment.direction.as_str(),adjustment.amount_cents])?;
        }
        if finalized {
            calculate_and_sync(&tx, &input.id, reason)?;
        }
        let after = repo::aggregate(&tx, &input.id)?;
        audit(
            &tx,
            "settlement",
            &input.id,
            if finalized {
                "CORRECTION"
            } else if before.is_some() {
                "UPDATE"
            } else {
                "CREATE"
            },
            reason,
            before.as_ref(),
            &after,
        )?;
        tx.commit()?;
        Ok(after)
    }
    pub fn finalize_settlement(
        &mut self,
        id: &str,
        expected_version: i64,
        reason: &str,
    ) -> Result<Value> {
        reason_required(reason)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let before = repo::aggregate(&tx, id)?;
        check_version(Some(&before["header"]), Some(expected_version))?;
        if before["header"]["status"] == "FINALIZED" {
            check_rule(&before["header"])?;
            tx.commit()?;
            return Ok(before);
        }
        let version = expected_version.checked_add(1).ok_or(Error::Overflow)?;
        tx.execute("UPDATE weekly_settlements SET row_version=?2,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,version])?;
        calculate_and_sync(&tx, id, reason)?;
        let after = repo::aggregate(&tx, id)?;
        audit(
            &tx,
            "settlement",
            id,
            "FINALIZE",
            reason,
            Some(&before),
            &after,
        )?;
        tx.commit()?;
        Ok(after)
    }
    pub fn update_driver_payroll(
        &mut self,
        id: &str,
        expected_version: i64,
        ach_cents: i64,
        twx_code_id: Option<&str>,
        notes: &str,
        reason: &str,
    ) -> Result<Value> {
        reason_required(reason)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let before = repo::payroll(&tx, id)?;
        check_version(Some(&before), Some(expected_version))?;
        if before["entry_kind"] != "DRIVER" {
            return Err(Error::Invalid("ACH is only available for driver payroll"));
        }
        check_rule(&before)?;
        let total = domain::driver_payroll(integer(&before, "amount_cents")?, ach_cents)?;
        let code = optional_label(
            &tx,
            Some(&before),
            "twx_code_id",
            twx_code_id,
            "twx_code_snapshot",
            "twx_codes",
            "code",
        )?;
        let label = optional_label(
            &tx,
            Some(&before),
            "twx_code_id",
            twx_code_id,
            "twx_label_snapshot",
            "twx_codes",
            "label",
        )?;
        let next = expected_version.checked_add(1).ok_or(Error::Overflow)?;
        tx.execute("UPDATE payroll_entries SET ach_charge_cents=?2,total_cents=?3,twx_code_id=?4,twx_code_snapshot=?5,twx_label_snapshot=?6,notes=?7,row_version=?8,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,ach_cents,total,twx_code_id,code,label,notes,next])?;
        let after = save_payroll_snapshot(&tx, id)?;
        audit(
            &tx,
            "payroll",
            id,
            "CORRECTION",
            reason,
            Some(&before),
            &after,
        )?;
        tx.commit()?;
        Ok(after)
    }
    /// Owner amount is explicit/manual; no undocumented owner percentage calculation.
    pub fn save_owner_payroll(&mut self, input: &OwnerPayrollInput, reason: &str) -> Result<Value> {
        reason_required(reason)?;
        domain::validate_period(&input.period_start, &input.period_end)?;
        domain::nonnegative(input.amount_cents)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let before = repo::rows(&tx, "SELECT * FROM payroll_entries WHERE id=?1", &input.id)?.pop();
        check_version(before.as_ref(), input.expected_version)?;
        if let Some(old) = before.as_ref() {
            if old["entry_kind"] != "OWNER" {
                return Err(Error::Invalid("Payroll kind cannot change"));
            }
            check_rule(old)?;
        }
        let owner = preserved_label(
            &tx,
            before.as_ref(),
            "owner_id",
            &input.owner_id,
            "payee_name_snapshot",
            "owners",
            "name",
        )?;
        let truck = preserved_label(
            &tx,
            before.as_ref(),
            "truck_id",
            &input.truck_id,
            "truck_number_snapshot",
            "trucks",
            "truck_number",
        )?;
        let code = optional_label(
            &tx,
            before.as_ref(),
            "twx_code_id",
            input.twx_code_id.as_deref(),
            "twx_code_snapshot",
            "twx_codes",
            "code",
        )?;
        let label = optional_label(
            &tx,
            before.as_ref(),
            "twx_code_id",
            input.twx_code_id.as_deref(),
            "twx_label_snapshot",
            "twx_codes",
            "label",
        )?;
        let next = input
            .expected_version
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(Error::Overflow)?;
        tx.execute("INSERT INTO payroll_entries(id,entry_kind,owner_id,truck_id,payee_name_snapshot,truck_number_snapshot,twx_code_id,twx_code_snapshot,twx_label_snapshot,period_start,period_end,notes,amount_cents,total_cents,status,finalized_at,rule_version,snapshot_json,row_version) VALUES(?1,'OWNER',?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?12,'FINALIZED',strftime('%Y-%m-%dT%H:%M:%fZ','now'),?13,'{}',?14) ON CONFLICT(id) DO UPDATE SET owner_id=excluded.owner_id,truck_id=excluded.truck_id,payee_name_snapshot=excluded.payee_name_snapshot,truck_number_snapshot=excluded.truck_number_snapshot,twx_code_id=excluded.twx_code_id,twx_code_snapshot=excluded.twx_code_snapshot,twx_label_snapshot=excluded.twx_label_snapshot,period_start=excluded.period_start,period_end=excluded.period_end,notes=excluded.notes,amount_cents=excluded.amount_cents,total_cents=excluded.total_cents,row_version=excluded.row_version,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![input.id,input.owner_id,input.truck_id,owner,truck,input.twx_code_id,code,label,input.period_start,input.period_end,input.notes,input.amount_cents,RULE_VERSION,next])?;
        let after = save_payroll_snapshot(&tx, &input.id)?;
        audit(
            &tx,
            "payroll",
            &input.id,
            if before.is_some() {
                "CORRECTION"
            } else {
                "FINALIZE"
            },
            reason,
            before.as_ref(),
            &after,
        )?;
        tx.commit()?;
        Ok(after)
    }
}
fn index(value: usize) -> Result<i64> {
    i64::try_from(value).map_err(|_| Error::Overflow)
}
fn reason_required(reason: &str) -> Result<()> {
    if reason.trim().is_empty() {
        Err(Error::Invalid("Audit reason required"))
    } else {
        Ok(())
    }
}
fn check_rule(value: &Value) -> Result<()> {
    if value["rule_version"] != RULE_VERSION {
        Err(Error::Invalid(
            "Unknown financial policy; explicit policy migration required",
        ))
    } else {
        Ok(())
    }
}
fn check_version(value: Option<&Value>, expected: Option<i64>) -> Result<()> {
    let actual = value.map(|v| integer(v, "row_version")).transpose()?;
    if actual != expected {
        Err(Error::Conflict)
    } else {
        Ok(())
    }
}
fn child_before<'a>(before: Option<&'a Value>, collection: &str, id: &str) -> Option<&'a Value> {
    before
        .and_then(|v| v[collection].as_array())
        .and_then(|a| a.iter().find(|v| v["id"] == id))
}
fn preserved_label(
    connection: &Connection,
    old: Option<&Value>,
    reference: &str,
    id: &str,
    snapshot: &str,
    table: &str,
    column: &str,
) -> Result<String> {
    match old.filter(|v| v[reference] == id) {
        Some(value) => Ok(text(value, snapshot)?.to_owned()),
        None => repo::label(connection, table, column, id),
    }
}
fn optional_label(
    connection: &Connection,
    old: Option<&Value>,
    reference: &str,
    id: Option<&str>,
    snapshot: &str,
    table: &str,
    column: &str,
) -> Result<Option<String>> {
    id.map(|id| preserved_label(connection, old, reference, id, snapshot, table, column))
        .transpose()
}
fn audit(
    tx: &Transaction<'_>,
    entity: &str,
    id: &str,
    category: &str,
    reason: &str,
    before: Option<&Value>,
    after: &Value,
) -> Result<()> {
    tx.execute("INSERT INTO audit_events(entity_type,entity_id,category,reason,before_json,after_json) VALUES(?1,?2,?3,?4,?5,?6)",params![entity,id,category,reason,before.map(Value::to_string),after.to_string()])?;
    Ok(())
}
fn save_payroll_snapshot(tx: &Transaction<'_>, id: &str) -> Result<Value> {
    let after = repo::payroll(tx, id)?;
    tx.execute(
        "UPDATE payroll_entries SET snapshot_json=?2 WHERE id=?1",
        params![id, after.to_string()],
    )?;
    Ok(after)
}
fn calculate_and_sync(tx: &Transaction<'_>, id: &str, reason: &str) -> Result<()> {
    let header = repo::header(tx, id)?;
    let loads = repo::rows(
        tx,
        "SELECT * FROM loads WHERE settlement_id=?1 ORDER BY position",
        id,
    )?;
    let mut totals = Vec::new();
    for load in loads {
        let total = domain::load_total(
            integer(&load, "tons_milli")?,
            integer(&load, "completion_rate_scaled")?,
        )?;
        tx.execute(
            "UPDATE loads SET total_cents=?2 WHERE id=?1",
            params![text(&load, "id")?, total],
        )?;
        totals.push(total);
    }
    let adjustments=repo::rows(tx,"SELECT direction,amount_cents FROM settlement_adjustments WHERE settlement_id=?1 ORDER BY position",id)?
        .iter().map(|v|Ok((Direction::parse(text(v,"direction")?)?,integer(v,"amount_cents")?))).collect::<Result<Vec<_>>>()?;
    let totals = domain::settlement(&totals, integer(&header, "payout_rate_ppm")?, &adjustments)?;
    tx.execute("UPDATE weekly_settlements SET gross_route_cents=?2,driver_gross_cents=?3,final_payout_cents=?4,status='FINALIZED',finalized_at=COALESCE(finalized_at,strftime('%Y-%m-%dT%H:%M:%fZ','now')),rule_version=?5,snapshot_json='{}' WHERE id=?1",params![id,totals.gross_route_cents,totals.driver_gross_cents,totals.final_payout_cents,RULE_VERSION])?;
    let snapshot = repo::aggregate(tx, id)?;
    tx.execute(
        "UPDATE weekly_settlements SET snapshot_json=?2 WHERE id=?1",
        params![id, snapshot.to_string()],
    )?;
    let payroll_id: Option<String> = tx
        .query_row(
            "SELECT id FROM payroll_entries WHERE settlement_id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    let before = payroll_id
        .as_ref()
        .map(|id| repo::payroll(tx, id))
        .transpose()?;
    if let Some(old) = before.as_ref() {
        check_rule(old)?;
    }
    let ach = before
        .as_ref()
        .map(|v| integer(v, "ach_charge_cents"))
        .transpose()?
        .unwrap_or(0);
    let next = before
        .as_ref()
        .map(|v| integer(v, "row_version"))
        .transpose()?
        .unwrap_or(0)
        .checked_add(1)
        .ok_or(Error::Overflow)?;
    let payroll_id = payroll_id.unwrap_or_else(|| format!("settlement:{id}"));
    let total = domain::driver_payroll(totals.final_payout_cents, ach)?;
    tx.execute("INSERT INTO payroll_entries(id,entry_kind,driver_id,truck_id,payee_name_snapshot,truck_number_snapshot,settlement_id,period_start,period_end,notes,amount_cents,ach_charge_cents,total_cents,status,finalized_at,rule_version,snapshot_json,row_version) VALUES(?1,'DRIVER',?2,?3,?4,?5,?6,?7,?8,'',?9,?10,?11,'FINALIZED',strftime('%Y-%m-%dT%H:%M:%fZ','now'),?12,'{}',?13) ON CONFLICT(settlement_id) DO UPDATE SET driver_id=excluded.driver_id,truck_id=excluded.truck_id,payee_name_snapshot=excluded.payee_name_snapshot,truck_number_snapshot=excluded.truck_number_snapshot,period_start=excluded.period_start,period_end=excluded.period_end,amount_cents=excluded.amount_cents,total_cents=excluded.total_cents,row_version=excluded.row_version,rule_version=excluded.rule_version,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![payroll_id,text(&header,"driver_id")?,text(&header,"truck_id")?,text(&header,"driver_name_snapshot")?,text(&header,"truck_number_snapshot")?,id,text(&header,"period_start")?,text(&header,"period_end")?,totals.final_payout_cents,ach,total,RULE_VERSION,next])?;
    let after = save_payroll_snapshot(tx, &payroll_id)?;
    audit(
        tx,
        "payroll",
        &payroll_id,
        if before.is_some() {
            "CORRECTION"
        } else {
            "FINALIZE"
        },
        reason,
        before.as_ref(),
        &after,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
