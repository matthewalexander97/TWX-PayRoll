use super::*;
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str};
use std::collections::BTreeMap;
const FONT: &[u8] = include_bytes!("../../assets/NotoSans-Regular.ttf");
const WIDTH: f32 = 842.0;
const HEIGHT: f32 = 595.0;
const MARGIN: f32 = 30.0;
enum Paint {
    Text(f32, f32, f32, String),
    Rect(f32, f32, f32, f32, bool),
}
struct Layout<'a> {
    report: &'a Report,
    face: ttf_parser::Face<'static>,
    pages: Vec<Vec<Paint>>,
    y: f32,
}
impl<'a> Layout<'a> {
    fn new(report: &'a Report) -> Result<Self> {
        let mut l = Self {
            report,
            face: ttf_parser::Face::parse(FONT, 0)?,
            pages: vec![],
            y: 0.0,
        };
        l.page();
        Ok(l)
    }
    fn page(&mut self) {
        self.pages.push(vec![]);
        self.text(MARGIN, HEIGHT - 37.0, 17.0, self.report.title.clone());
        self.text(MARGIN, HEIGHT - 56.0, 9.0, self.report.period.clone());
        self.text(
            MARGIN,
            HEIGHT - 74.0,
            9.0,
            if self.report.draft {
                "DRAFT / NOT FINALIZED - saved inputs only"
            } else {
                "Saved authoritative values - USD"
            }
            .into(),
        );
        self.y = HEIGHT - 98.0;
    }
    fn text(&mut self, x: f32, y: f32, size: f32, text: String) {
        self.pages
            .last_mut()
            .unwrap()
            .push(Paint::Text(x, y, size, text));
    }
    fn width(&self, text: &str, size: f32) -> f32 {
        text.chars()
            .map(|c| {
                self.face
                    .glyph_index(c)
                    .and_then(|id| self.face.glyph_hor_advance(id))
                    .unwrap_or(0) as f32
            })
            .sum::<f32>()
            * size
            / self.face.units_per_em() as f32
    }
    fn wrap(&self, text: &str, width: f32, size: f32) -> Vec<String> {
        let mut lines = vec![];
        for paragraph in text.replace('\r', "").replace('\t', " ").split('\n') {
            let mut line = String::new();
            for ch in paragraph.chars() {
                let candidate = format!("{line}{ch}");
                if !line.is_empty() && self.width(&candidate, size) > width {
                    if let Some(split) = line.rfind(' ').filter(|i| *i > 0) {
                        let remainder = line[split + 1..].to_string();
                        lines.push(line[..split].to_string());
                        line = remainder;
                    } else {
                        lines.push(std::mem::take(&mut line));
                    }
                }
                if !line.is_empty() || ch != ' ' {
                    line.push(ch);
                }
            }
            lines.push(line);
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines
    }
    fn table_header(&mut self, section: &Section, widths: &[f32], size: f32) {
        self.text(MARGIN, self.y, 11.0, section.name.clone());
        self.y -= 18.0;
        let lines: Vec<_> = section
            .columns
            .iter()
            .zip(widths)
            .map(|(v, w)| self.wrap(v, w - 10.0, size))
            .collect();
        let height = lines.iter().map(Vec::len).max().unwrap_or(1) as f32 * (size + 3.0) + 10.0;
        self.pages.last_mut().unwrap().push(Paint::Rect(
            MARGIN,
            self.y - height + 4.0,
            WIDTH - 2.0 * MARGIN,
            height,
            true,
        ));
        let mut x = MARGIN;
        for (lines, width) in lines.iter().zip(widths) {
            for (i, line) in lines.iter().enumerate() {
                self.text(
                    x + 5.0,
                    self.y - 5.0 - i as f32 * (size + 3.0),
                    size,
                    line.clone(),
                );
            }
            x += width;
        }
        self.y -= height;
    }
    fn section(&mut self, section: &Section) -> Result<()> {
        let size = if section.columns.len() > 7 { 7.5 } else { 9.0 };
        let line_height = size + 3.0;
        let mut widths: Vec<f32> = section
            .columns
            .iter()
            .enumerate()
            .map(|(c, _)| {
                section
                    .rows
                    .iter()
                    .filter_map(|row| row.get(c))
                    .filter(|v| !matches!(v, Cell::Text { .. }))
                    .map(|v| self.width(&v.display(), size) + 12.0)
                    .fold(40.0, f32::max)
            })
            .collect();
        let extra = (WIDTH - 2.0 * MARGIN) - widths.iter().sum::<f32>();
        if extra < 0.0 {
            return Err(
                "Financial values require a wider PDF table; use Excel for this selection".into(),
            );
        }
        let weights: Vec<f32> = section
            .columns
            .iter()
            .map(|n| {
                if ["Value", "Explanation", "Notes", "Period / Week Ending"].contains(&n.as_str()) {
                    3.0
                } else {
                    1.0
                }
            })
            .collect();
        let total = weights.iter().sum::<f32>();
        for (w, weight) in widths.iter_mut().zip(weights) {
            *w += extra * weight / total;
        }
        if self.y < 130.0 {
            self.page();
        }
        self.table_header(section, &widths, size);
        for row in &section.rows {
            if row.len() != widths.len() {
                return Err("Invalid report row shape".into());
            }
            let cells: Vec<_> = row
                .iter()
                .zip(&widths)
                .map(|(v, w)| self.wrap(&v.display(), w - 10.0, size))
                .collect();
            let total_lines = cells.iter().map(Vec::len).max().unwrap_or(1).max(1);
            let header_lines = section
                .columns
                .iter()
                .zip(&widths)
                .map(|(v, w)| self.wrap(v, w - 10.0, size).len())
                .max()
                .unwrap_or(1);
            let fresh_y = HEIGHT - 98.0 - 18.0 - (header_lines as f32 * line_height + 10.0);
            let row_height = total_lines as f32 * line_height + 9.0;
            // Keep ordinary rows together. Only a row taller than a full page is split.
            if self.y - row_height < 50.0 && row_height <= fresh_y - 50.0 {
                self.page();
                self.table_header(section, &widths, size);
            }
            let mut offset = 0;
            while offset < total_lines {
                let available = ((self.y - 50.0) / line_height).floor() as usize;
                if available < 2 {
                    self.page();
                    self.table_header(section, &widths, size);
                    continue;
                }
                let count = (total_lines - offset).min(available.saturating_sub(1));
                let height = count as f32 * line_height + 9.0;
                let mut x = MARGIN;
                for (lines, width) in cells.iter().zip(&widths) {
                    for i in 0..count {
                        if let Some(line) = lines.get(offset + i) {
                            self.text(
                                x + 5.0,
                                self.y - 5.0 - i as f32 * line_height,
                                size,
                                line.clone(),
                            );
                        }
                    }
                    x += width;
                }
                self.pages.last_mut().unwrap().push(Paint::Rect(
                    MARGIN,
                    self.y - height + 3.0,
                    WIDTH - 2.0 * MARGIN,
                    0.5,
                    false,
                ));
                self.y -= height;
                offset += count;
                if offset < total_lines {
                    self.page();
                    self.table_header(section, &widths, size);
                }
            }
        }
        self.y -= 20.0;
        Ok(())
    }
}
pub fn render(report: &Report) -> Result<Vec<u8>> {
    let mut layout = Layout::new(report)?;
    for section in &report.sections {
        layout.section(section)?;
    }
    let count = layout.pages.len();
    for (i, page) in layout.pages.iter_mut().enumerate() {
        page.push(Paint::Text(
            MARGIN,
            22.0,
            8.0,
            format!(
                "TWX Payroll   {}   Page {} of {}",
                report.period,
                i + 1,
                count
            ),
        ));
    }
    let mut glyphs = BTreeMap::new();
    for page in &layout.pages {
        for paint in page {
            if let Paint::Text(_, _, _, text) = paint {
                for c in text.chars() {
                    let id=layout.face.glyph_index(c).ok_or_else(||format!("The local PDF font cannot represent character {c:?}. Use Excel/Word/CSV or revise the saved label."))?;
                    glyphs.entry(id.0).or_insert(c);
                }
            }
        }
    }
    let mut pdf = Pdf::new();
    pdf.catalog(Ref::new(1)).pages(Ref::new(2));
    pdf.pages(Ref::new(2))
        .kids((0..count).map(|i| Ref::new(10 + i as i32 * 2)))
        .count(count as i32);
    {
        let mut font = pdf.indirect(Ref::new(3)).dict();
        font.pair(Name(b"Type"), Name(b"Font"))
            .pair(Name(b"Subtype"), Name(b"Type0"))
            .pair(Name(b"BaseFont"), Name(b"NotoSans-Regular"))
            .pair(Name(b"Encoding"), Name(b"Identity-H"))
            .pair(Name(b"ToUnicode"), Ref::new(7));
        font.insert(Name(b"DescendantFonts"))
            .array()
            .item(Ref::new(4));
    }
    {
        let mut font = pdf.indirect(Ref::new(4)).dict();
        font.pair(Name(b"Type"), Name(b"Font"))
            .pair(Name(b"Subtype"), Name(b"CIDFontType2"))
            .pair(Name(b"BaseFont"), Name(b"NotoSans-Regular"))
            .pair(Name(b"FontDescriptor"), Ref::new(5))
            .pair(Name(b"CIDToGIDMap"), Name(b"Identity"));
        {
            let mut info = font.insert(Name(b"CIDSystemInfo")).dict();
            info.pair(Name(b"Registry"), Str(b"Adobe"))
                .pair(Name(b"Ordering"), Str(b"Identity"))
                .pair(Name(b"Supplement"), 0);
        }
        let mut widths = font.insert(Name(b"W")).array();
        for id in glyphs.keys() {
            widths.item(*id as i32);
            widths.push().array().item(
                layout
                    .face
                    .glyph_hor_advance(ttf_parser::GlyphId(*id))
                    .unwrap_or(0) as f32
                    * 1000.0
                    / layout.face.units_per_em() as f32,
            );
        }
    }
    {
        let mut desc = pdf.indirect(Ref::new(5)).dict();
        desc.pair(Name(b"Type"), Name(b"FontDescriptor"))
            .pair(Name(b"FontName"), Name(b"NotoSans-Regular"))
            .pair(Name(b"Flags"), 32)
            .pair(Name(b"Ascent"), 1069)
            .pair(Name(b"Descent"), -293)
            .pair(Name(b"CapHeight"), 714)
            .pair(Name(b"ItalicAngle"), 0)
            .pair(Name(b"StemV"), 80)
            .pair(Name(b"FontFile2"), Ref::new(6));
        desc.insert(Name(b"FontBBox"))
            .array()
            .items([-621, -389, 2800, 1069]);
    }
    pdf.stream(Ref::new(6), FONT)
        .pair(Name(b"Length1"), FONT.len() as i32);
    let mut cmap=String::from("/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /TWXUnicode def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n");
    let mappings: Vec<_> = glyphs.iter().collect();
    for chunk in mappings.chunks(100) {
        cmap.push_str(&format!("{} beginbfchar\n", chunk.len()));
        for (id, ch) in chunk {
            let mut units = [0; 2];
            let encoded = ch.encode_utf16(&mut units);
            let unicode = encoded
                .iter()
                .map(|v| format!("{v:04X}"))
                .collect::<String>();
            cmap.push_str(&format!("<{id:04X}> <{unicode}>\n"));
        }
        cmap.push_str("endbfchar\n");
    }
    cmap.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    pdf.stream(Ref::new(7), cmap.as_bytes());
    for (i, paints) in layout.pages.iter().enumerate() {
        let page_ref = Ref::new(10 + i as i32 * 2);
        let stream_ref = Ref::new(11 + i as i32 * 2);
        let mut page = pdf.page(page_ref);
        page.media_box(Rect::new(0.0, 0.0, WIDTH, HEIGHT))
            .parent(Ref::new(2))
            .contents(stream_ref);
        page.resources().fonts().pair(Name(b"F1"), Ref::new(3));
        page.finish();
        let mut content = Content::new();
        for paint in paints {
            match paint {
                Paint::Text(x, y, size, text) => {
                    let encoded: Vec<u8> = text
                        .chars()
                        .flat_map(|c| layout.face.glyph_index(c).unwrap().0.to_be_bytes())
                        .collect();
                    content
                        .set_fill_rgb(0.08, 0.12, 0.10)
                        .begin_text()
                        .set_font(Name(b"F1"), *size)
                        .set_text_matrix([1.0, 0.0, 0.0, 1.0, *x, *y])
                        .show(Str(&encoded))
                        .end_text();
                }
                Paint::Rect(x, y, w, h, header) => {
                    let shade = if *header { 0.91 } else { 0.80 };
                    content
                        .set_fill_rgb(shade, shade + 0.02, shade + 0.01)
                        .rect(*x, *y, *w, *h)
                        .fill_nonzero();
                }
            }
        }
        pdf.stream(stream_ref, &content.finish());
    }
    Ok(pdf.finish())
}
