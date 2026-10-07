//! Read-only saved-report models. Renderers never query catalogs or calculate financial rules.
pub mod files;
mod formats;
mod pdf;
use crate::{repositories as repo, workspace, Storage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub type Result<T> = crate::backup::Result<T>;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReportKind {
    SettlementDetail,
    DriverPayroll,
    OwnerPayroll,
    PayrollSummary,
    SettlementHistory,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub kind: ReportKind,
    pub settlement_id: Option<String>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub driver_id: Option<String>,
    pub owner_id: Option<String>,
    pub truck_id: Option<String>,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Csv,
    Xlsx,
    Pdf,
    Docx,
}
impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Xlsx => "xlsx",
            Self::Pdf => "pdf",
            Self::Docx => "docx",
        }
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Cell {
    Text { value: String },
    Money { value: String },
    Decimal { value: String, scale: u32 },
    Percentage { value: String },
}
impl Cell {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text { value: s.into() }
    }
    fn exact(v: &Value, key: &str, scale: u32, money: bool) -> Result<Self> {
        let integer = repo::integer(v, key)?;
        if ["tons_milli", "completion_rate_scaled", "ach_charge_cents"].contains(&key)
            && integer < 0
        {
            return Err(format!("Corrupt negative saved quantity: {key}").into());
        }
        let value = integer.to_string();
        Ok(if money {
            Self::Money { value }
        } else {
            Self::Decimal { value, scale }
        })
    }
    pub fn raw(&self) -> String {
        match self {
            Self::Text { value } => value.clone(),
            Self::Money { value } => decimal(value, 2),
            Self::Decimal { value, scale } => decimal(value, *scale),
            Self::Percentage { value } => decimal(value, 4),
        }
    }
    pub fn display(&self) -> String {
        match self {
            Self::Money { .. } => money(&self.raw()),
            Self::Percentage { .. } => format!("{}%", self.raw()),
            _ => self.raw(),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Section {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
}
impl Section {
    fn new(name: &str, columns: &[&str]) -> Self {
        Self {
            name: name.into(),
            columns: columns.iter().map(|v| v.to_string()).collect(),
            rows: vec![],
        }
    }
    fn field(&mut self, name: &str, value: Cell) {
        self.rows.push(vec![Cell::text(name), value]);
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub kind: ReportKind,
    pub title: String,
    pub period: String,
    pub draft: bool,
    pub currency: String,
    pub sections: Vec<Section>,
    pub source_versions: Vec<String>,
    pub default_name: String,
}
// Exact digit-string formatting; no floating-point arithmetic.
pub fn decimal(value: &str, scale: u32) -> String {
    let (sign, digits) = value.strip_prefix('-').map_or(("", value), |v| ("-", v));
    let padded = format!("{:0>width$}", digits, width = scale as usize + 1);
    if scale == 0 {
        return format!("{sign}{padded}");
    }
    let split = padded.len() - scale as usize;
    format!("{sign}{}.{}", &padded[..split], &padded[split..])
}
pub fn money(raw: &str) -> String {
    let (sign, digits) = raw.strip_prefix('-').map_or(("", raw), |v| ("-", v));
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, "00"));
    let grouped = whole
        .chars()
        .enumerate()
        .fold(String::new(), |mut out, (i, c)| {
            if i > 0 && (whole.len() - i).is_multiple_of(3) {
                out.push(',');
            }
            out.push(c);
            out
        });
    format!("{sign}${grouped}.{fraction}")
}
fn text(v: &Value, key: &str) -> Result<Cell> {
    Ok(Cell::text(repo::text(v, key)?))
}
fn optional(v: &Value, key: &str) -> Result<Cell> {
    if v[key].is_null() {
        Ok(Cell::text(""))
    } else {
        text(v, key)
    }
}
fn saved_money(v: &Value, key: &str, draft: bool) -> Result<Cell> {
    if v[key].is_null() && draft {
        Ok(Cell::text("Not finalized"))
    } else {
        Cell::exact(v, key, 2, true)
    }
}
fn percentage(v: &Value) -> Result<Cell> {
    let n = repo::integer(v, "payout_rate_ppm")?;
    if !(0..=1_000_000).contains(&n) {
        return Err("Corrupt saved payout percentage".into());
    }
    Ok(Cell::Percentage {
        value: n.to_string(),
    })
}
fn period(v: &Value) -> Result<String> {
    let start = repo::text(v, "period_start")?;
    let end = repo::text(v, "period_end")?;
    crate::domain::validate_period(start, end)?;
    Ok(format!("{} – {}", date(start), date(end)))
}
fn date(v: &str) -> String {
    format!("{}/{}/{}", &v[5..7], &v[8..10], &v[..4])
}
fn settlement(s: &Storage, id: &str) -> Result<Value> {
    let current = repo::header(&s.connection, id)?;
    if current["status"] == "DRAFT" {
        return Ok(repo::aggregate(&s.connection, id)?);
    }
    if current["status"] != "FINALIZED" {
        return Err("Unsupported settlement status".into());
    }
    let snapshot = s.history().settlement(id)?;
    if snapshot["snapshot_schema"] != "1"
        || snapshot["header"]["id"] != current["id"]
        || snapshot["header"]["row_version"] != current["row_version"]
        || snapshot["header"]["status"] != "FINALIZED"
    {
        return Err("Finalized settlement snapshot is missing, corrupt or incompatible".into());
    }
    // Known integer field scales are tied to the versioned saved policy.
    if snapshot["header"]["rule_version"] != crate::domain::RULE_VERSION {
        return Err("Unsupported saved financial policy".into());
    }
    Ok(snapshot)
}
fn saved_payroll(s: &Storage, row: &Value) -> Result<Value> {
    let id = repo::text(row, "id")?;
    let json: String = s.connection.query_row(
        "SELECT snapshot_json FROM payroll_entries WHERE id=?1",
        [id],
        |r| r.get(0),
    )?;
    let v: Value = serde_json::from_str(&json)?;
    if v["id"] != row["id"]
        || v["row_version"] != row["row_version"]
        || v["status"] != "FINALIZED"
        || v["rule_version"] != crate::domain::RULE_VERSION
    {
        return Err("Payroll snapshot is missing, corrupt or incompatible".into());
    }
    Ok(v)
}
fn payroll_section(name: &str, rows: &[Value], driver: bool) -> Result<Section> {
    let mut columns = if driver {
        vec![
            "Driver Name",
            "TWX Code",
            "ACH Charge",
            "Truck #",
            "Amount",
            "Total",
            "Period / Week Ending",
            "Notes",
        ]
    } else {
        vec![
            "Owner Name",
            "TWX Code",
            "Truck #",
            "Amount",
            "Period / Week Ending",
            "Notes",
        ]
    };
    columns.push("Status");
    let mut sec = Section::new(name, &columns);
    for row in rows {
        if row["entry_kind"] != if driver { "DRIVER" } else { "OWNER" } {
            return Err("Unexpected saved payee type".into());
        }
        let mut cells = vec![
            text(row, "payee_name_snapshot")?,
            optional(row, "twx_code_snapshot")?,
        ];
        if driver {
            cells.push(Cell::exact(row, "ach_charge_cents", 2, true)?);
        } else if !row["ach_charge_cents"].is_null() {
            return Err("Owner payroll must not contain ACH".into());
        }
        cells.push(text(row, "truck_number_snapshot")?);
        cells.push(Cell::exact(row, "amount_cents", 2, true)?);
        if driver {
            cells.push(Cell::exact(row, "total_cents", 2, true)?);
        }
        cells.extend([
            Cell::text(period(row)?),
            text(row, "notes")?,
            text(row, "status")?,
        ]);
        sec.rows.push(cells);
    }
    Ok(sec)
}
pub fn build(s: &Storage, request: &Request) -> Result<Report> {
    let filter = workspace::RecordFilter {
        period_start: request.period_start.clone(),
        period_end: request.period_end.clone(),
        driver_id: request.driver_id.clone(),
        truck_id: request.truck_id.clone(),
    };
    if let Some(start) = &request.period_start {
        crate::domain::validate_period(start, start)?;
    }
    if let Some(end) = &request.period_end {
        crate::domain::validate_period(end, end)?;
    }
    if let (Some(start), Some(end)) = (&request.period_start, &request.period_end) {
        crate::domain::validate_period(start, end)?;
    }
    if request.owner_id.is_some()
        && matches!(
            request.kind,
            ReportKind::SettlementDetail
                | ReportKind::DriverPayroll
                | ReportKind::SettlementHistory
        )
    {
        return Err("Owner filter is not supported for this report".into());
    }
    if request.kind == ReportKind::OwnerPayroll && request.driver_id.is_some() {
        return Err("Driver filter is not supported for owner payroll".into());
    }
    if request.settlement_id.is_some() && request.kind != ReportKind::SettlementDetail {
        return Err("Settlement ID is only used for settlement detail".into());
    }
    let title = match request.kind {
        ReportKind::SettlementDetail => "Weekly Settlement Detail",
        ReportKind::DriverPayroll => "Driver Payroll",
        ReportKind::OwnerPayroll => "Owner Payroll",
        ReportKind::PayrollSummary => "Payroll Summary",
        ReportKind::SettlementHistory => "Settlement History",
    };
    let mut report = Report {
        kind: request.kind,
        title: title.into(),
        period: match (&request.period_start, &request.period_end) {
            (Some(a), Some(b)) => format!("{} – {}", date(a), date(b)),
            (Some(a), None) => format!("From {}", date(a)),
            (None, Some(b)) => format!("Through {}", date(b)),
            _ => "All saved periods".into(),
        },
        draft: false,
        currency: "USD".into(),
        sections: vec![],
        source_versions: vec![],
        default_name: format!(
            "TWX-{}-{}-to-{}",
            title.replace(' ', "-"),
            request.period_start.as_deref().unwrap_or("all"),
            request.period_end.as_deref().unwrap_or("all")
        ),
    };
    if request.kind == ReportKind::SettlementDetail {
        let id = request
            .settlement_id
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .ok_or("Select a saved settlement")?;
        let a = settlement(s, id)?;
        let h = &a["header"];
        report.draft = h["status"] == "DRAFT";
        report.period = period(h)?;
        report
            .source_versions
            .push(format!("settlement:{id}:{}", repo::text(h, "row_version")?));
        report.default_name = format!(
            "TWX-Settlement-{}-to-{}-{}",
            repo::text(h, "period_start")?,
            repo::text(h, "period_end")?,
            repo::text(h, "truck_number_snapshot")?
        );
        let mut header = Section::new("Settlement", &["Field", "Value"]);
        for (label, key) in [
            ("Period start", "period_start"),
            ("Period end", "period_end"),
            ("Driver", "driver_name_snapshot"),
            ("Driver ID", "driver_id"),
            ("Truck #", "truck_number_snapshot"),
            ("Truck ID", "truck_id"),
            ("Driver Type", "driver_type_name_snapshot"),
            ("Status", "status"),
            ("Notes", "notes"),
        ] {
            header.field(label, text(h, key)?);
        }
        header.field("Payout Percentage", percentage(h)?);
        report.sections.push(header);
        let mut loads = Section::new(
            "Loads",
            &[
                "Load ID",
                "BOL Number",
                "PO Number",
                "Plant",
                "Job Name",
                "Truck #",
                "Driver Name",
                "Net Weight / Tons",
                "Mileage",
                "Completion Rate",
                "Total",
            ],
        );
        for l in a["loads"].as_array().ok_or("Corrupt load snapshot")? {
            loads.rows.push(vec![
                text(l, "load_number")?,
                text(l, "bol_number")?,
                text(l, "po_number")?,
                optional(l, "plant_name_snapshot")?,
                optional(l, "job_name_snapshot")?,
                text(l, "truck_number_snapshot")?,
                text(l, "driver_name_snapshot")?,
                Cell::exact(l, "tons_milli", 3, false)?,
                optional(l, "mileage")?,
                Cell::exact(l, "completion_rate_scaled", 4, false)?,
                saved_money(l, "total_cents", report.draft)?,
            ]);
        }
        report.sections.push(loads);
        let mut summary = Section::new("Summary", &["Field", "Value"]);
        summary.field(
            "Gross Route Total",
            saved_money(h, "gross_route_cents", report.draft)?,
        );
        summary.field("Driver Payout Percentage", percentage(h)?);
        summary.field(
            "Driver Gross Payout",
            saved_money(h, "driver_gross_cents", report.draft)?,
        );
        report.sections.push(summary);
        let mut adjustments = Section::new(
            "Adjustments",
            &["Type", "TWX Code", "Explanation", "Direction", "Amount"],
        );
        for a in a["adjustments"]
            .as_array()
            .ok_or("Corrupt adjustment snapshot")?
        {
            if a["direction"] != "ADD" && a["direction"] != "DEDUCT" {
                return Err("Corrupt adjustment direction".into());
            }
            if repo::integer(a, "amount_cents")? < 0 {
                return Err("Corrupt negative adjustment magnitude".into());
            }
            adjustments.rows.push(vec![
                text(a, "type_name_snapshot")?,
                optional(a, "twx_code_snapshot")?,
                text(a, "explanation")?,
                text(a, "direction")?,
                Cell::exact(a, "amount_cents", 2, true)?,
            ]);
        }
        report.sections.push(adjustments);
        let mut final_line = Section::new("Final payout", &["Field", "Value"]);
        final_line.field(
            "FINAL PAYOUT",
            saved_money(h, "final_payout_cents", report.draft)?,
        );
        report.sections.push(final_line);
        if !report.draft {
            let rows = repo::rows(
                &s.connection,
                "SELECT * FROM payroll_entries WHERE settlement_id=?1",
                id,
            )?;
            if rows.len() != 1 {
                return Err("Finalized settlement requires one linked payroll record".into());
            }
            let p = saved_payroll(s, &rows[0])?;
            report.source_versions.push(format!(
                "payroll:{}:{}",
                repo::text(&p, "id")?,
                repo::text(&p, "row_version")?
            ));
            report
                .sections
                .push(payroll_section("Linked Driver Payroll", &[p], true)?);
        }
    } else if request.kind == ReportKind::SettlementHistory {
        let rows = workspace::settlements(s, filter).map_err(|e| e.message)?;
        if rows.len() > 10000 {
            return Err("Select a smaller period (maximum 10,000 records)".into());
        }
        let mut sec = Section::new(
            "Settlements",
            &[
                "Period / Week Ending",
                "Driver Name",
                "Driver Type",
                "Truck #",
                "Payout Percentage",
                "Gross Route Total",
                "Driver Gross Payout",
                "FINAL PAYOUT",
                "Status",
            ],
        );
        for row in rows {
            let id = repo::text(&row, "id")?;
            let a = settlement(s, id)?;
            let h = &a["header"];
            let draft = h["status"] == "DRAFT";
            report.draft |= draft;
            report
                .source_versions
                .push(format!("settlement:{id}:{}", repo::text(h, "row_version")?));
            sec.rows.push(vec![
                Cell::text(period(h)?),
                text(h, "driver_name_snapshot")?,
                text(h, "driver_type_name_snapshot")?,
                text(h, "truck_number_snapshot")?,
                percentage(h)?,
                saved_money(h, "gross_route_cents", draft)?,
                saved_money(h, "driver_gross_cents", draft)?,
                saved_money(h, "final_payout_cents", draft)?,
                text(h, "status")?,
            ]);
        }
        report.sections.push(sec);
    } else {
        // Driver filters apply to driver rows; owner filters apply to owner rows. Period/truck apply to both.
        let owner_filter = workspace::RecordFilter {
            driver_id: None,
            ..filter.clone()
        };
        let rows = workspace::payrolls(s, owner_filter).map_err(|e| e.message)?;
        if rows.len() > 10000 {
            return Err("Select a smaller period (maximum 10,000 records)".into());
        }
        let mut drivers = vec![];
        let mut owners = vec![];
        for row in rows {
            let driver = row["entry_kind"] == "DRIVER";
            if driver
                && request
                    .driver_id
                    .as_ref()
                    .is_some_and(|id| row["driver_id"].as_str() != Some(id))
            {
                continue;
            }
            if !driver
                && request
                    .owner_id
                    .as_ref()
                    .is_some_and(|id| row["owner_id"].as_str() != Some(id))
            {
                continue;
            }
            if driver && request.kind == ReportKind::OwnerPayroll
                || !driver && request.kind == ReportKind::DriverPayroll
            {
                continue;
            }
            let p = saved_payroll(s, &row)?;
            report.source_versions.push(format!(
                "payroll:{}:{}",
                repo::text(&p, "id")?,
                repo::text(&p, "row_version")?
            ));
            if driver {
                drivers.push(p);
            } else {
                owners.push(p);
            }
        }
        if request.kind != ReportKind::OwnerPayroll {
            report
                .sections
                .push(payroll_section("Driver Payroll", &drivers, true)?);
        }
        if request.kind != ReportKind::DriverPayroll {
            report
                .sections
                .push(payroll_section("Owner Payroll", &owners, false)?);
        }
    }
    if report.source_versions.is_empty() {
        return Err("No saved records match the selected report filters".into());
    }
    report.default_name = files::safe_filename(&report.default_name);
    Ok(report)
}
pub fn render(report: &Report, format: Format) -> Result<Vec<u8>> {
    for section in &report.sections {
        for row in &section.rows {
            if row.len() != section.columns.len() {
                return Err("Corrupt report row shape".into());
            }
            for cell in row {
                match cell {
                    Cell::Text { value } => {
                        if value
                            .chars()
                            .any(|c| c.is_control() && !['\n', '\r', '\t'].contains(&c))
                        {
                            return Err(
                                "Saved report text contains unsupported control characters".into(),
                            );
                        }
                    }
                    Cell::Money { value }
                    | Cell::Percentage { value }
                    | Cell::Decimal { value, .. } => {
                        value.parse::<i64>().map_err(|_| "Corrupt report integer")?;
                    }
                }
            }
        }
    }
    match format {
        Format::Csv => formats::csv(report),
        Format::Xlsx => formats::xlsx(report),
        Format::Pdf => pdf::render(report),
        Format::Docx => formats::docx(report),
    }
}
#[cfg(test)]
pub(crate) mod tests;
