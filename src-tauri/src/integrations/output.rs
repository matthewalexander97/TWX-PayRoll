use super::*;
use crate::reports::{Cell, Report};
pub fn sheets(report: &Report) -> Result<Value> {
    let mut sheets = Vec::new();
    let mut cells = 0usize;
    for (index, section) in report.sections.iter().enumerate() {
        let mut rows = vec![
            vec![report.title.clone()],
            vec![report.period.clone()],
            vec![
                if report.draft {
                    "DRAFT / NOT FINALIZED"
                } else {
                    "SAVED / FINALIZED"
                }
                .into(),
                "Snapshot: editing this Sheet does not change TWX Payroll.".into(),
            ],
            section.columns.clone(),
        ];
        for row in &section.rows {
            if row.len() != section.columns.len() {
                return Err("Malformed saved report".into());
            }
            rows.push(row.iter().map(Cell::display).collect());
        }
        cells += rows.iter().map(Vec::len).sum::<usize>();
        if cells > 100_000 || rows.iter().flatten().any(|v| v.chars().count() > 50_000) {
            return Err("Report exceeds the safe Sheets export size; use local exports or a narrower period.".into());
        }
        let width = section.columns.len().max(2);
        let data=rows.into_iter().map(|row|json!({"values":row.into_iter().map(|v|json!({"userEnteredValue":{"stringValue":v},"userEnteredFormat":{"wrapStrategy":"WRAP"}})).collect::<Vec<_>>()})).collect::<Vec<_>>();
        // String cells preserve exact saved values, Unicode and formula-like labels; no formulas.
        sheets.push(json!({"properties":{"sheetId":index,"title":format!("{} {}",index+1,section.name.chars().filter(|c|!"[]:*?/\\".contains(*c)).take(80).collect::<String>()),"gridProperties":{"rowCount":data.len().max(5),"columnCount":width,"frozenRowCount":4}},"data":[{"startRow":0,"startColumn":0,"rowData":data}]}));
    }
    let result = json!({"properties":{"title":format!("{} — {}{}",report.title,report.period,if report.draft{" — DRAFT"}else{""})},"sheets":sheets});
    if serde_json::to_vec(&result)
        .map_err(|_| "Report serialization failed")?
        .len()
        > 8_000_000
    {
        return Err("Report is too large for a single safe upload; narrow the period.".into());
    }
    Ok(result)
}
pub fn recipient(value: &str) -> bool {
    if value.len() > 254 || !value.is_ascii() {
        return false;
    }
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && local.len() <= 64
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && local
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~".contains(&c))
        && domain.contains('.')
        && domain.split('.').all(|p| {
            !p.is_empty()
                && p.len() <= 63
                && !p.starts_with('-')
                && !p.ends_with('-')
                && p.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Email {
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub format: crate::reports::Format,
}
impl Email {
    pub fn validate(&self) -> Result<()> {
        if !recipient(&self.recipient) {
            return Err(
                "Enter one syntactically valid email address. Deliverability is not verified."
                    .into(),
            );
        }
        if self.subject.is_empty()
            || self.subject.len() > 200
            || self.subject.chars().any(char::is_control)
            || self.body.len() > 20_000
            || self.body.contains('\0')
        {
            return Err("Use a subject of 1–200 bytes without control characters and a message under 20,000 bytes.".into());
        }
        Ok(())
    }
}
fn wrapped(bytes: &[u8]) -> String {
    STANDARD
        .encode(bytes)
        .as_bytes()
        .chunks(76)
        .map(|v| String::from_utf8_lossy(v).to_string())
        .collect::<Vec<_>>()
        .join("\r\n")
}
fn subject(value: &str) -> String {
    let mut parts = Vec::new();
    let mut chunk = String::new();
    for c in value.chars() {
        if chunk.len() + c.len_utf8() > 45 {
            parts.push(format!("=?UTF-8?B?{}?=", STANDARD.encode(chunk.as_bytes())));
            chunk.clear();
        }
        chunk.push(c);
    }
    if !chunk.is_empty() {
        parts.push(format!("=?UTF-8?B?{}?=", STANDARD.encode(chunk.as_bytes())));
    }
    parts.join("\r\n ")
}
pub fn mime(
    sender: &str,
    email: &Email,
    report: &Report,
    attachment: &[u8],
    id: &str,
) -> Result<String> {
    email.validate()?;
    if !recipient(sender) {
        return Err(
            "Connected Google account did not provide a valid sender address. Reconnect.".into(),
        );
    }
    if attachment.len() > 18_000_000 {
        return Err("Attachment is too large; export locally or narrow the report.".into());
    }
    let boundary = format!("twx-{}", Uuid::new_v4());
    let content_type = match email.format {
        crate::reports::Format::Pdf => "application/pdf",
        crate::reports::Format::Xlsx => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        crate::reports::Format::Docx => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        crate::reports::Format::Csv => "text/csv",
    };
    let filename = format!(
        "{}.{}",
        crate::reports::files::safe_filename(&report.default_name),
        email.format.extension()
    );
    // ASCII attachment name and base64 headers/body prevent MIME header/boundary injection.
    let filename: String = filename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._-".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    let message=format!("From: {sender}\r\nTo: {}\r\nSubject: {}\r\nMessage-ID: <{}@twx-payroll.local>\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n--{boundary}\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n--{boundary}\r\nContent-Type: {content_type}\r\nContent-Disposition: attachment; filename=\"{filename}\"\r\nContent-Transfer-Encoding: base64\r\n\r\n{}\r\n--{boundary}--\r\n",email.recipient,subject(&email.subject),id,wrapped(email.body.as_bytes()),wrapped(attachment));
    Ok(URL_SAFE_NO_PAD.encode(message.as_bytes()))
}
