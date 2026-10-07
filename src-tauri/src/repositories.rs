//! Parameterized persistence and read models. No generic SQL is exposed over IPC.
use crate::domain::{self, Direction, Error, Result};
use rusqlite::{params, types::ValueRef, Connection, OptionalExtension};
use serde_json::{Map, Value};

pub enum Catalog<'a> {
    DriverType {
        name: &'a str,
        payout_rate_ppm: i64,
    },
    Driver {
        name: &'a str,
        driver_type_id: &'a str,
    },
    Owner {
        name: &'a str,
    },
    Truck {
        truck_number: &'a str,
        owner_id: &'a str,
        default_driver_id: Option<&'a str>,
    },
    TwxCode {
        code: &'a str,
        label: &'a str,
    },
    Plant {
        name: &'a str,
    },
    Job {
        name: &'a str,
        plant_id: Option<&'a str>,
    },
    AdjustmentType {
        name: &'a str,
        default_direction: Direction,
        twx_code_id: Option<&'a str>,
    },
}
impl Catalog<'_> {
    fn table(&self) -> &'static str {
        match self {
            Self::DriverType { .. } => "driver_types",
            Self::Driver { .. } => "drivers",
            Self::Owner { .. } => "owners",
            Self::Truck { .. } => "trucks",
            Self::TwxCode { .. } => "twx_codes",
            Self::Plant { .. } => "plants",
            Self::Job { .. } => "jobs",
            Self::AdjustmentType { .. } => "adjustment_types",
        }
    }
}
pub struct CatalogRepository<'a> {
    connection: &'a mut Connection,
}
impl<'a> CatalogRepository<'a> {
    pub(crate) fn new(connection: &'a mut Connection) -> Self {
        Self { connection }
    }
    /// expected_version=None creates; Some edits or archives. References already selected on financial rows retain their snapshots.
    pub fn save(
        &mut self,
        id: &str,
        expected_version: Option<i64>,
        active: bool,
        item: Catalog<'_>,
    ) -> Result<i64> {
        if id.trim().is_empty() {
            return Err(Error::Invalid("Nonblank identifier required"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let table = item.table();
        let old: Option<i64> = tx
            .query_row(
                &format!("SELECT row_version FROM {table} WHERE id=?1"),
                [id],
                |r| r.get(0),
            )
            .optional()?;
        if old != expected_version {
            return Err(Error::Conflict);
        }
        let next = old.unwrap_or(0).checked_add(1).ok_or(Error::Overflow)?;
        // Identifiers are fixed Rust enum mappings, never supplied SQL fragments.
        match item {
            Catalog::DriverType {
                name,
                payout_rate_ppm,
            } => {
                domain::payout_rate(payout_rate_ppm)?;
                tx.execute("INSERT INTO driver_types(id,name,payout_rate_ppm) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=excluded.name,payout_rate_ppm=excluded.payout_rate_ppm",params![id,name,payout_rate_ppm])?;
            }
            Catalog::Driver {
                name,
                driver_type_id,
            } => {
                tx.execute("INSERT INTO drivers(id,name,driver_type_id) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=excluded.name,driver_type_id=excluded.driver_type_id",params![id,name,driver_type_id])?;
            }
            Catalog::Owner { name } => {
                tx.execute("INSERT INTO owners(id,name) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET name=excluded.name",params![id,name])?;
            }
            Catalog::Truck {
                truck_number,
                owner_id,
                default_driver_id,
            } => {
                tx.execute("INSERT INTO trucks(id,truck_number,owner_id,default_driver_id) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET truck_number=excluded.truck_number,owner_id=excluded.owner_id,default_driver_id=excluded.default_driver_id",params![id,truck_number,owner_id,default_driver_id])?;
            }
            Catalog::TwxCode { code, label } => {
                tx.execute("INSERT INTO twx_codes(id,code,label) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET code=excluded.code,label=excluded.label",params![id,code,label])?;
            }
            Catalog::Plant { name } => {
                tx.execute("INSERT INTO plants(id,name) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET name=excluded.name",params![id,name])?;
            }
            Catalog::Job { name, plant_id } => {
                tx.execute("INSERT INTO jobs(id,name,plant_id) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=excluded.name,plant_id=excluded.plant_id",params![id,name,plant_id])?;
            }
            Catalog::AdjustmentType {
                name,
                default_direction,
                twx_code_id,
            } => {
                tx.execute("INSERT INTO adjustment_types(id,name,default_direction,twx_code_id) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,default_direction=excluded.default_direction,twx_code_id=excluded.twx_code_id",params![id,name,default_direction.as_str(),twx_code_id])?;
            }
        }
        tx.execute(&format!("UPDATE {table} SET is_active=?2,row_version=?3,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1"),params![id,active,next])?;
        tx.commit()?;
        Ok(next)
    }
}

/// Internal JSON read helper. Integers are encoded as strings to preserve all i64 bits.
pub(crate) fn rows(connection: &Connection, sql: &str, id: &str) -> Result<Vec<Value>> {
    let mut statement = connection.prepare(sql)?;
    let names = statement
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let mut cursor = statement.query([id])?;
    let mut values = Vec::new();
    while let Some(row) = cursor.next()? {
        let mut fields = Map::new();
        for (index, name) in names.iter().enumerate() {
            if name == "snapshot_json" {
                continue;
            }
            let value = match row.get_ref(index)? {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(value) => Value::String(value.to_string()),
                ValueRef::Text(value) => Value::String(String::from_utf8_lossy(value).into_owned()),
                _ => return Err(Error::Invalid("Unexpected non-exact stored value")),
            };
            fields.insert(name.clone(), value);
        }
        values.push(Value::Object(fields));
    }
    Ok(values)
}
pub(crate) fn header(connection: &Connection, id: &str) -> Result<Value> {
    rows(
        connection,
        "SELECT * FROM weekly_settlements WHERE id=?1",
        id,
    )?
    .pop()
    .ok_or(Error::Missing)
}
pub(crate) fn aggregate(connection: &Connection, id: &str) -> Result<Value> {
    Ok(
        serde_json::json!({"snapshot_schema":"1","header":header(connection,id)?,"loads":rows(connection,"SELECT * FROM loads WHERE settlement_id=?1 ORDER BY position",id)?,"adjustments":rows(connection,"SELECT * FROM settlement_adjustments WHERE settlement_id=?1 ORDER BY position",id)?}),
    )
}
pub(crate) fn payroll(connection: &Connection, id: &str) -> Result<Value> {
    rows(connection, "SELECT * FROM payroll_entries WHERE id=?1", id)?
        .pop()
        .ok_or(Error::Missing)
}
pub(crate) fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or(Error::Invalid("Required stored field missing"))
}
pub(crate) fn integer(value: &Value, key: &str) -> Result<i64> {
    text(value, key)?
        .parse()
        .map_err(|_| Error::Invalid("Invalid stored integer"))
}
pub(crate) fn label(
    connection: &Connection,
    table: &str,
    column: &str,
    id: &str,
) -> Result<String> {
    // crate-only callers use literal table/column identifiers.
    connection
        .query_row(
            &format!("SELECT {column} FROM {table} WHERE id=?1"),
            [id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(Error::Missing)
}

#[derive(Clone, Copy)]
pub enum FinancialEntity {
    Settlement,
    Payroll,
}
impl FinancialEntity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Settlement => "settlement",
            Self::Payroll => "payroll",
        }
    }
}

pub struct HistoryRepository<'a> {
    connection: &'a Connection,
}
impl<'a> HistoryRepository<'a> {
    pub(crate) fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }
    pub fn settlement(&self, id: &str) -> Result<Value> {
        let saved: Option<String> = self
            .connection
            .query_row(
                "SELECT snapshot_json FROM weekly_settlements WHERE id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()?
            .ok_or(Error::Missing)?;
        match saved {
            Some(json) => Ok(serde_json::from_str(&json)?),
            None => aggregate(self.connection, id),
        }
    }
    pub fn payroll(&self, id: &str) -> Result<Value> {
        payroll(self.connection, id)
    }
    pub fn events(&self, entity: FinancialEntity, entity_id: &str) -> Result<Vec<Value>> {
        rows(
            self.connection,
            &format!(
                "SELECT * FROM audit_events WHERE entity_type='{}' AND entity_id=?1 ORDER BY id",
                entity.as_str()
            ),
            entity_id,
        )
    }
    pub fn settlements_ending_on_or_after(&self, date: &str) -> Result<Vec<Value>> {
        domain::validate_period(date, date)?;
        rows(self.connection,"SELECT id,period_start,period_end,status,row_version,final_payout_cents FROM weekly_settlements WHERE period_end>=?1 ORDER BY period_end,id",date)
    }
}
