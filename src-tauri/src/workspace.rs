//! Typed desktop use cases. Decimal strings enter here; existing exact services own writes.
use crate::{
    application::{AdjustmentInput, LoadInput, OwnerPayrollInput, SettlementInput},
    domain::{self, Direction, Error},
    repositories::{self as repo, Catalog, FinancialEntity},
    Storage,
};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}
impl From<Error> for UiError {
    fn from(e: Error) -> Self {
        Self {
            code: match e {
                Error::Conflict => "CONFLICT",
                Error::Missing => "MISSING",
                Error::Database(_) => "DATABASE",
                _ => "VALIDATION",
            }
            .into(),
            message: e.to_string(),
            field: None,
        }
    }
}
pub type Result<T> = std::result::Result<T, UiError>;
pub fn unavailable(message: &str) -> UiError {
    UiError {
        code: "UNAVAILABLE".into(),
        message: message.into(),
        field: None,
    }
}
fn invalid(field: &str, message: &str) -> UiError {
    UiError {
        code: "VALIDATION".into(),
        message: message.into(),
        field: Some(field.into()),
    }
}
fn required<'a>(value: &'a str, field: &str) -> Result<&'a str> {
    if value.trim().is_empty() || value.len() > 10000 {
        return Err(invalid(
            field,
            "A value is required (maximum 10,000 characters).",
        ));
    }
    Ok(value)
}
fn scaled(value: &str, places: u32, field: &str) -> Result<i64> {
    domain::parse_scaled(value, places).map_err(|e| invalid(field, &e.to_string()))
}
fn percentage(value: &str) -> Result<i64> {
    domain::parse_percentage(value).map_err(|e| invalid("payoutPercentage", &e.to_string()))
}
fn version(value: Option<&str>) -> Result<Option<i64>> {
    value
        .map(|v| {
            v.parse::<i64>()
                .map_err(|_| invalid("expectedVersion", "Invalid record version"))
        })
        .transpose()
}
fn identity(id: Option<&str>) -> Result<String> {
    match id {
        Some(id) => Ok(required(id, "id")?.into()),
        None => Ok(Uuid::new_v4().to_string()),
    }
}
fn period(start: &str, end: &str) -> Result<()> {
    domain::validate_period(start, end).map_err(|_| {
        invalid(
            "periodEnd",
            "Choose valid dates with the end on or after the start.",
        )
    })
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceKind {
    DriverTypes,
    Drivers,
    Owners,
    Trucks,
    TwxCodes,
    Plants,
    Jobs,
    AdjustmentTypes,
}
impl ReferenceKind {
    fn table(self) -> &'static str {
        match self {
            Self::DriverTypes => "driver_types",
            Self::Drivers => "drivers",
            Self::Owners => "owners",
            Self::Trucks => "trucks",
            Self::TwxCodes => "twx_codes",
            Self::Plants => "plants",
            Self::Jobs => "jobs",
            Self::AdjustmentTypes => "adjustment_types",
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceWrite {
    pub kind: ReferenceKind,
    pub id: Option<String>,
    pub expected_version: Option<String>,
    pub active: bool,
    pub name: Option<String>,
    pub payout_percentage: Option<String>,
    pub driver_type_id: Option<String>,
    pub truck_number: Option<String>,
    pub owner_id: Option<String>,
    pub default_driver_id: Option<String>,
    pub code: Option<String>,
    pub label: Option<String>,
    pub plant_id: Option<String>,
    pub default_direction: Option<String>,
    pub twx_code_id: Option<String>,
}
pub fn references(s: &Storage) -> Result<Value> {
    let mut data = serde_json::Map::new();
    for kind in [
        ReferenceKind::DriverTypes,
        ReferenceKind::Drivers,
        ReferenceKind::Owners,
        ReferenceKind::Trucks,
        ReferenceKind::TwxCodes,
        ReferenceKind::Plants,
        ReferenceKind::Jobs,
        ReferenceKind::AdjustmentTypes,
    ] {
        let table = kind.table();
        data.insert(
            table.into(),
            json!(repo::rows(
                &s.connection,
                &format!("SELECT * FROM {table} WHERE ?1='' ORDER BY created_at,id"),
                ""
            )?),
        );
    }
    Ok(Value::Object(data))
}
pub fn save_reference(s: &mut Storage, w: ReferenceWrite) -> Result<Value> {
    let id = identity(w.id.as_deref())?;
    let need = |value: Option<&str>, field: &str| -> Result<String> {
        Ok(required(value.unwrap_or(""), field)?.to_owned())
    };
    let name = if matches!(w.kind, ReferenceKind::Trucks | ReferenceKind::TwxCodes) {
        String::new()
    } else {
        need(w.name.as_deref(), "name")?
    };
    let driver_type = if matches!(w.kind, ReferenceKind::Drivers) {
        need(w.driver_type_id.as_deref(), "driverTypeId")?
    } else {
        String::new()
    };
    let owner = if matches!(w.kind, ReferenceKind::Trucks) {
        need(w.owner_id.as_deref(), "ownerId")?
    } else {
        String::new()
    };
    let truck = if matches!(w.kind, ReferenceKind::Trucks) {
        need(w.truck_number.as_deref(), "truckNumber")?
    } else {
        String::new()
    };
    let code = if matches!(w.kind, ReferenceKind::TwxCodes) {
        need(w.code.as_deref(), "code")?
    } else {
        String::new()
    };
    let label = if matches!(w.kind, ReferenceKind::TwxCodes) {
        need(w.label.as_deref(), "label")?
    } else {
        String::new()
    };
    let item = match w.kind {
        ReferenceKind::DriverTypes => Catalog::DriverType {
            name: &name,
            payout_rate_ppm: percentage(w.payout_percentage.as_deref().unwrap_or(""))?,
        },
        ReferenceKind::Drivers => Catalog::Driver {
            name: &name,
            driver_type_id: &driver_type,
        },
        ReferenceKind::Owners => Catalog::Owner { name: &name },
        ReferenceKind::Trucks => Catalog::Truck {
            truck_number: &truck,
            owner_id: &owner,
            default_driver_id: w.default_driver_id.as_deref(),
        },
        ReferenceKind::TwxCodes => Catalog::TwxCode {
            code: &code,
            label: &label,
        },
        ReferenceKind::Plants => Catalog::Plant { name: &name },
        ReferenceKind::Jobs => Catalog::Job {
            name: &name,
            plant_id: w.plant_id.as_deref(),
        },
        ReferenceKind::AdjustmentTypes => Catalog::AdjustmentType {
            name: &name,
            default_direction: Direction::parse(w.default_direction.as_deref().unwrap_or(""))?,
            twx_code_id: w.twx_code_id.as_deref(),
        },
    };
    s.catalogs()
        .save(&id, version(w.expected_version.as_deref())?, w.active, item)?;
    Ok(repo::rows(
        &s.connection,
        &format!("SELECT * FROM {} WHERE id=?1", w.kind.table()),
        &id,
    )?
    .pop()
    .ok_or(Error::Missing)?)
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecordFilter {
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub driver_id: Option<String>,
    pub truck_id: Option<String>,
}
fn filter_text(f: &RecordFilter) -> Result<String> {
    if let Some(d) = &f.period_start {
        period(d, d)?;
    }
    if let Some(d) = &f.period_end {
        period(d, d)?;
    }
    if let (Some(a), Some(b)) = (&f.period_start, &f.period_end) {
        period(a, b)?;
    }
    Ok(serde_json::to_string(f).map_err(Error::from)?)
}
pub fn settlements(s: &Storage, f: RecordFilter) -> Result<Vec<Value>> {
    Ok(repo::rows(&s.connection,"SELECT id,period_start,period_end,driver_id,truck_id,driver_name_snapshot,truck_number_snapshot,status,row_version,final_payout_cents FROM weekly_settlements WHERE (json_extract(?1,'$.periodStart') IS NULL OR period_end>=json_extract(?1,'$.periodStart')) AND (json_extract(?1,'$.periodEnd') IS NULL OR period_start<=json_extract(?1,'$.periodEnd')) AND (json_extract(?1,'$.driverId') IS NULL OR driver_id=json_extract(?1,'$.driverId')) AND (json_extract(?1,'$.truckId') IS NULL OR truck_id=json_extract(?1,'$.truckId')) ORDER BY period_end DESC,id",&filter_text(&f)?)?)
}
pub fn settlement(s: &Storage, id: &str) -> Result<Value> {
    Ok(repo::aggregate(&s.connection, required(id, "id")?)?)
}
pub fn payrolls(s: &Storage, f: RecordFilter) -> Result<Vec<Value>> {
    Ok(repo::rows(&s.connection,"SELECT * FROM payroll_entries WHERE (json_extract(?1,'$.periodStart') IS NULL OR period_end>=json_extract(?1,'$.periodStart')) AND (json_extract(?1,'$.periodEnd') IS NULL OR period_start<=json_extract(?1,'$.periodEnd')) AND (json_extract(?1,'$.driverId') IS NULL OR driver_id=json_extract(?1,'$.driverId')) AND (json_extract(?1,'$.truckId') IS NULL OR truck_id=json_extract(?1,'$.truckId')) ORDER BY period_end DESC,id",&filter_text(&f)?)?)
}
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Entity {
    Settlement,
    Payroll,
}
pub fn events(s: &Storage, entity: Entity, id: &str) -> Result<Vec<Value>> {
    Ok(s.history().events(
        match entity {
            Entity::Settlement => FinancialEntity::Settlement,
            Entity::Payroll => FinancialEntity::Payroll,
        },
        required(id, "id")?,
    )?)
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoadWrite {
    pub id: Option<String>,
    pub driver_id: String,
    pub truck_id: String,
    pub plant_id: Option<String>,
    pub job_id: Option<String>,
    pub load_number: String,
    pub bol_number: String,
    pub po_number: String,
    pub tons: String,
    pub completion_rate: String,
    pub mileage: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdjustmentWrite {
    pub id: Option<String>,
    pub adjustment_type_id: String,
    pub twx_code_id: Option<String>,
    pub explanation: String,
    pub direction: String,
    pub amount: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettlementWrite {
    pub id: Option<String>,
    pub expected_version: Option<String>,
    pub driver_id: String,
    pub truck_id: String,
    pub period_start: String,
    pub period_end: String,
    pub notes: String,
    pub payout_percentage: Option<String>,
    pub loads: Vec<LoadWrite>,
    pub adjustments: Vec<AdjustmentWrite>,
}
fn parse_settlement(w: &SettlementWrite) -> Result<SettlementInput> {
    period(&w.period_start, &w.period_end)?;
    required(&w.driver_id, "driverId")?;
    required(&w.truck_id, "truckId")?;
    if w.loads.len() > 500 || w.adjustments.len() > 500 {
        return Err(invalid("loads", "Maximum 500 rows per section"));
    }
    let loads = w
        .loads
        .iter()
        .enumerate()
        .map(|(i, l)| {
            required(&l.driver_id, &format!("loads.{i}.driverId"))?;
            required(&l.truck_id, &format!("loads.{i}.truckId"))?;
            domain::validate_mileage(l.mileage.as_deref())
                .map_err(|e| invalid(&format!("loads.{i}.mileage"), &e.to_string()))?;
            Ok(LoadInput {
                id: identity(l.id.as_deref())?,
                driver_id: l.driver_id.clone(),
                truck_id: l.truck_id.clone(),
                plant_id: l.plant_id.clone(),
                job_id: l.job_id.clone(),
                load_number: l.load_number.clone(),
                bol_number: l.bol_number.clone(),
                po_number: l.po_number.clone(),
                tons_milli: scaled(&l.tons, 3, &format!("loads.{i}.tons"))?,
                completion_rate_scaled: scaled(
                    &l.completion_rate,
                    4,
                    &format!("loads.{i}.completionRate"),
                )?,
                mileage: l.mileage.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let adjustments = w
        .adjustments
        .iter()
        .enumerate()
        .map(|(i, a)| {
            required(
                &a.adjustment_type_id,
                &format!("adjustments.{i}.adjustmentTypeId"),
            )?;
            Ok(AdjustmentInput {
                id: identity(a.id.as_deref())?,
                adjustment_type_id: a.adjustment_type_id.clone(),
                twx_code_id: a.twx_code_id.clone(),
                explanation: a.explanation.clone(),
                direction: Direction::parse(&a.direction)?,
                amount_cents: scaled(&a.amount, 2, &format!("adjustments.{i}.amount"))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(SettlementInput {
        id: identity(w.id.as_deref())?,
        expected_version: version(w.expected_version.as_deref())?,
        driver_id: w.driver_id.clone(),
        truck_id: w.truck_id.clone(),
        period_start: w.period_start.clone(),
        period_end: w.period_end.clone(),
        notes: w.notes.clone(),
        payout_rate_ppm: w.payout_percentage.as_deref().map(percentage).transpose()?,
        loads,
        adjustments,
    })
}
fn input_rate(s: &Storage, i: &SettlementInput) -> Result<i64> {
    let old: Option<(i64, i64)> = s
        .connection
        .query_row(
            "SELECT row_version,payout_rate_ppm FROM weekly_settlements WHERE id=?1",
            [&i.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(Error::from)?;
    if old.map(|o| o.0) != i.expected_version {
        return Err(Error::Conflict.into());
    }
    if let Some(rate) = i.payout_rate_ppm {
        return Ok(rate);
    }
    if let Some((_, rate)) = old {
        return Ok(rate);
    }
    Ok(s.connection.query_row("SELECT t.payout_rate_ppm FROM drivers d JOIN driver_types t ON t.id=d.driver_type_id WHERE d.id=?1",[&i.driver_id],|r|r.get(0)).map_err(Error::from)?)
}
fn preview_input(s: &Storage, i: &SettlementInput) -> Result<Value> {
    let rate = input_rate(s, i)?;
    let loads = i
        .loads
        .iter()
        .map(|l| domain::load_total(l.tons_milli, l.completion_rate_scaled))
        .collect::<domain::Result<Vec<_>>>()?;
    let adjustments = i
        .adjustments
        .iter()
        .map(|a| (a.direction, a.amount_cents))
        .collect::<Vec<_>>();
    let total = domain::settlement(&loads, rate, &adjustments)?;
    Ok(
        json!({"loadTotalsCents":loads.iter().map(i64::to_string).collect::<Vec<_>>(),"payoutRatePpm":rate.to_string(),"grossRouteCents":total.gross_route_cents.to_string(),"driverGrossCents":total.driver_gross_cents.to_string(),"finalPayoutCents":total.final_payout_cents.to_string(),"negative":total.final_payout_cents<0}),
    )
}
pub fn preview(s: &Storage, w: &SettlementWrite) -> Result<Value> {
    preview_input(s, &parse_settlement(w)?)
}
fn confirm(p: &Value, confirmed: bool) -> Result<()> {
    if p["negative"] == true && !confirmed {
        return Err(UiError{code:"NEGATIVE_CONFIRMATION".into(),message:"Final payout is negative. Confirm the negative payout before finalizing or saving a finalized edit.".into(),field:None});
    }
    Ok(())
}
pub fn save_settlement(s: &mut Storage, w: SettlementWrite, confirmed: bool) -> Result<Value> {
    let i = parse_settlement(&w)?;
    let p = preview_input(s, &i)?;
    let is_finalized = s
        .connection
        .query_row(
            "SELECT status FROM weekly_settlements WHERE id=?1",
            [&i.id],
            |r| r.get::<_, String>(0),
        )
        .optional()
        .map_err(Error::from)?
        .is_some_and(|v| v == "FINALIZED");
    if is_finalized {
        confirm(&p, confirmed)?;
    }
    Ok(s.business().save_settlement(
        &i,
        if is_finalized {
            "Bookkeeper edited finalized settlement"
        } else {
            "Bookkeeper saved draft"
        },
    )?)
}
pub fn finalize(s: &mut Storage, id: &str, expected: &str, confirmed: bool) -> Result<Value> {
    let saved = repo::aggregate(&s.connection, id)?;
    let rate = repo::integer(&saved["header"], "payout_rate_ppm")?;
    let loads = saved["loads"]
        .as_array()
        .ok_or(Error::Missing)?
        .iter()
        .map(|l| {
            domain::load_total(
                repo::integer(l, "tons_milli")?,
                repo::integer(l, "completion_rate_scaled")?,
            )
        })
        .collect::<domain::Result<Vec<_>>>()?;
    let adjustments = saved["adjustments"]
        .as_array()
        .ok_or(Error::Missing)?
        .iter()
        .map(|a| {
            Ok((
                Direction::parse(repo::text(a, "direction")?)?,
                repo::integer(a, "amount_cents")?,
            ))
        })
        .collect::<domain::Result<Vec<_>>>()?;
    let total = domain::settlement(&loads, rate, &adjustments)?;
    confirm(&json!({"negative":total.final_payout_cents<0}), confirmed)?;
    Ok(s.business().finalize_settlement(
        id,
        version(Some(expected))?.ok_or(Error::Conflict)?,
        "Bookkeeper finalized settlement",
    )?)
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DriverPayrollWrite {
    pub id: String,
    pub expected_version: String,
    pub ach_charge: String,
    pub twx_code_id: Option<String>,
    pub notes: String,
}
pub fn save_driver(s: &mut Storage, w: DriverPayrollWrite) -> Result<Value> {
    Ok(s.business().update_driver_payroll(
        &w.id,
        version(Some(&w.expected_version))?.ok_or(Error::Conflict)?,
        scaled(&w.ach_charge, 2, "achCharge")?,
        w.twx_code_id.as_deref(),
        &w.notes,
        "Bookkeeper edited payroll ACH/code/notes",
    )?)
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnerPayrollWrite {
    pub id: Option<String>,
    pub expected_version: Option<String>,
    pub owner_id: String,
    pub truck_id: String,
    pub twx_code_id: Option<String>,
    pub period_start: String,
    pub period_end: String,
    pub amount: String,
    pub notes: String,
}
pub fn save_owner(s: &mut Storage, w: OwnerPayrollWrite) -> Result<Value> {
    period(&w.period_start, &w.period_end)?;
    required(&w.owner_id, "ownerId")?;
    required(&w.truck_id, "truckId")?;
    let i = OwnerPayrollInput {
        id: identity(w.id.as_deref())?,
        expected_version: version(w.expected_version.as_deref())?,
        owner_id: w.owner_id,
        truck_id: w.truck_id,
        twx_code_id: w.twx_code_id,
        period_start: w.period_start,
        period_end: w.period_end,
        notes: w.notes,
        amount_cents: scaled(&w.amount, 2, "amount")?,
    };
    Ok(s.business()
        .save_owner_payroll(&i, "Bookkeeper saved manual owner payroll")?)
}

#[cfg(test)]
mod tests;
