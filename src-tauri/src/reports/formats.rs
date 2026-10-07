use super::*;
use rust_xlsxwriter::{Format as XFormat, Workbook};
use std::io::Cursor;

fn spreadsheet_text(value: &str) -> String {
    // Quoting alone doesn't stop CSV spreadsheet formula interpretation.
    if value
        .trim_start()
        .starts_with(['=', '+', '-', '@', '\t', '\r'])
    {
        format!("'{value}")
    } else {
        value.into()
    }
}
pub fn csv(report: &Report) -> Result<Vec<u8>> {
    // One rectangular CSV; section identifies load/adjustment/summary/payroll rows.
    let mut columns = vec![
        "Report Type".into(),
        "Section".into(),
        "Record".into(),
        "Report State".into(),
    ];
    for section in &report.sections {
        for column in &section.columns {
            if !columns.contains(column) {
                columns.push(column.clone());
            }
        }
    }
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_writer(vec![]);
    writer.write_record(&columns)?;
    for section in &report.sections {
        for (index, row) in section.rows.iter().enumerate() {
            let mut values = vec![String::new(); columns.len()];
            values[0] = report.title.clone();
            values[1] = section.name.clone();
            values[2] = (index + 1).to_string();
            values[3] = if report.draft {
                "DRAFT / NOT FINALIZED"
            } else {
                "SAVED / FINALIZED"
            }
            .into();
            for (column, cell) in section.columns.iter().zip(row) {
                let pos = columns
                    .iter()
                    .position(|v| v == column)
                    .ok_or("Missing report column")?;
                values[pos] = match cell {
                    Cell::Text { value } => spreadsheet_text(value),
                    _ => cell.raw(),
                };
            }
            writer.write_record(values)?;
        }
    }
    Ok(writer.into_inner()?)
}
fn number(cell: &Cell) -> Option<f64> {
    let (value, scale) = match cell {
        Cell::Money { value } => (value, 2),
        Cell::Decimal { value, scale } => (value, *scale),
        Cell::Percentage { value } => (value, 4),
        Cell::Text { .. } => return None,
    };
    // Only export presentation uses f64. It never drives any financial calculation.
    // Excel promises 15 significant decimal digits; require exact fixed-scale round-trip too.
    if value.trim_start_matches('-').trim_start_matches('0').len() > 15 {
        return None;
    }
    let raw = cell.raw();
    let n = raw.parse::<f64>().ok()?;
    if n.is_finite() && format!("{n:.precision$}", precision = scale as usize) == raw {
        Some(n)
    } else {
        None
    }
}
pub fn xlsx(report: &Report) -> Result<Vec<u8>> {
    let mut workbook = Workbook::new();
    let title = XFormat::new().set_bold().set_font_size(16);
    let header = XFormat::new()
        .set_bold()
        .set_background_color("#E5EEE9")
        .set_text_wrap();
    let normal = XFormat::new().set_text_wrap().set_font_size(10);
    let currency = XFormat::new()
        .set_num_format("\"$\"#,##0.00;-\"$\"#,##0.00")
        .set_font_size(10);
    let mut groups: Vec<(String, Vec<&Section>)> = vec![];
    for section in &report.sections {
        let name = if report.kind == ReportKind::SettlementDetail {
            "Settlement"
        } else {
            section.name.as_str()
        };
        if let Some((_, sections)) = groups.iter_mut().find(|(n, _)| n == name) {
            sections.push(section);
        } else {
            groups.push((name.into(), vec![section]));
        }
    }
    for (name, sections) in groups {
        let mut widths: Vec<f64> = vec![];
        for section in &sections {
            for (c, label) in section.columns.iter().enumerate() {
                if widths.len() <= c {
                    widths.resize(c + 1, 20.0);
                }
                let base = if label == "Notes" || label == "Explanation" {
                    35.0
                } else if label.contains("Period") {
                    32.0
                } else {
                    20.0
                };
                let numeric = section
                    .rows
                    .iter()
                    .filter_map(|r| r.get(c))
                    .filter(|cell| !matches!(cell, Cell::Text { .. }))
                    .map(|cell| cell.display().chars().count() as f64 + 2.0)
                    .fold(base, f64::max);
                widths[c] = widths[c].max(numeric);
            }
        }
        let sheet = workbook.add_worksheet();
        sheet.set_name(&name)?;
        sheet.set_landscape();
        sheet.set_print_fit_to_pages(1, 0);
        sheet.set_freeze_panes(4, 0)?;
        for (c, width) in widths.iter().enumerate() {
            sheet.set_column_width(c as u16, *width)?;
        }
        sheet.write_string_with_format(0, 0, &report.title, &title)?;
        sheet.write_string(1, 0, &report.period)?;
        sheet.write_string(
            2,
            0,
            if report.draft {
                "DRAFT / NOT FINALIZED - saved inputs only; unavailable totals are not calculated"
            } else {
                "Saved authoritative values - USD"
            },
        )?;
        let mut r = 4;
        for section in sections {
            sheet.write_string_with_format(r, 0, &section.name, &header)?;
            r += 1;
            for (c, label) in section.columns.iter().enumerate() {
                sheet.write_string_with_format(r, c as u16, label, &header)?;
            }
            sheet.set_row_height(r, 32.0)?;
            r += 1;
            for row in &section.rows {
                let lines = row
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        c.display()
                            .split('\n')
                            .map(|line| {
                                line.chars()
                                    .count()
                                    .div_ceil((widths[i] * 0.6) as usize)
                                    .max(1)
                            })
                            .sum::<usize>()
                    })
                    .max()
                    .unwrap_or(1)
                    .max(1);
                let height = (lines as f64 * 14.0).max(18.0);
                if height > 409.0 {
                    return Err("Saved text exceeds Excel's row-height limit. Choose CSV, PDF or Word to preserve all text without clipping.".into());
                }
                sheet.set_row_height(r, height)?;
                for (c, cell) in row.iter().enumerate() {
                    if let Some(n) = number(cell) {
                        let fmt = match cell {
                            Cell::Money { .. } => currency.clone(),
                            Cell::Percentage { .. } => XFormat::new().set_num_format("0.0000\"%\""),
                            Cell::Decimal { scale, .. } => XFormat::new()
                                .set_num_format(format!("0.{}", "0".repeat(*scale as usize))),
                            _ => normal.clone(),
                        };
                        sheet.write_number_with_format(r, c as u16, n, &fmt)?;
                    } else {
                        sheet.write_string_with_format(r, c as u16, cell.display(), &normal)?;
                    }
                }
                r += 1;
            }
            r += 2;
        }
        sheet.write_string(
            r,
            0,
            "Large values are exact text when Excel's numeric precision cannot preserve them.",
        )?;
    }
    Ok(workbook.save_to_buffer()?)
}
fn paragraph(text: &str, size: usize, bold: bool) -> docx_rs::Paragraph {
    let mut run = docx_rs::Run::new().size(size);
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            run = run.add_break(docx_rs::BreakType::TextWrapping);
        }
        run = run.add_text(line);
    }
    docx_rs::Paragraph::new().add_run(if bold { run.bold() } else { run })
}
pub fn docx(report: &Report) -> Result<Vec<u8>> {
    use docx_rs::*;
    let mut doc = Docx::new()
        .page_size(16838, 11906)
        .page_margin(PageMargin::new().top(600).bottom(600).left(600).right(600))
        .default_size(18)
        .add_paragraph(paragraph(&report.title, 32, true).style("Title"))
        .add_paragraph(paragraph(&report.period, 22, false))
        .add_paragraph(paragraph(
            if report.draft {
                "DRAFT / NOT FINALIZED - saved inputs only; unavailable totals are not calculated"
            } else {
                "Saved authoritative values - USD"
            },
            18,
            report.draft,
        ));
    doc = doc
        .header(Header::new().add_paragraph(paragraph(
            &format!("TWX Payroll   {}", report.title),
            16,
            false,
        )))
        .footer(Footer::new().add_paragraph(paragraph(&report.period, 16, false)));
    for section in &report.sections {
        doc = doc.add_paragraph(paragraph(&section.name, 24, true).keep_next(true));
        let header = TableRow::new(
            section
                .columns
                .iter()
                .map(|v| TableCell::new().add_paragraph(paragraph(v, 16, true)))
                .collect(),
        )
        .cant_split();
        let rows = std::iter::once(header)
            .chain(section.rows.iter().map(|row| {
                TableRow::new(
                    row.iter()
                        .map(|c| TableCell::new().add_paragraph(paragraph(&c.display(), 16, false)))
                        .collect(),
                )
            }))
            .collect();
        doc = doc.add_table(Table::new(rows).width(15638, WidthType::Dxa));
    }
    let mut output = Cursor::new(vec![]);
    doc.build().pack(&mut output)?;
    Ok(output.into_inner())
}
