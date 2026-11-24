//! ODS XML -> worksheet parser (split from `odf.rs` to stay under 450 lines).

use super::*;
use botsheet_core::engine::value::CellValue;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::{Reader, XmlVersion};
use std::collections::HashMap;

use crate::types::CellData;

fn local_name(name: QName<'_>) -> &str {
    name.local_name().into_inner()
}

fn attr_value(start: &BytesStart<'_>, wanted: &str) -> Option<String> {
    for attribute in start.attributes().flatten() {
        if local_name(attribute.key) == wanted {
            return attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .ok()
                .map(|value| value.into_owned());
        }
    }
    None
}

pub(super) fn parse_repeat(raw: Option<&str>) -> u32 {
    // Parse into u64 first so an oversized count is CLAMPED to the cap instead
    // of failing to parse as u32 and silently collapsing to one cell.
    raw.and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .map(|value| value.min(u64::from(MAX_REPEAT)) as u32)
        .unwrap_or(1)
}

#[derive(Default)]
struct CellAttrs {
    repeat: u32,
    value_type: Option<String>,
    value: Option<String>,
    boolean_value: Option<String>,
}

fn read_cell_attrs(start: &BytesStart<'_>) -> CellAttrs {
    let mut attrs = CellAttrs {
        repeat: 1,
        ..CellAttrs::default()
    };
    for attribute in start.attributes().flatten() {
        match local_name(attribute.key) {
            "number-columns-repeated" => {
                let raw = attribute.normalized_value(XmlVersion::Implicit1_0).ok();
                attrs.repeat = parse_repeat(raw.as_deref());
            }
            "value-type" => attrs.value_type = unescape_attr(&attribute),
            "value" => attrs.value = unescape_attr(&attribute),
            "boolean-value" => attrs.boolean_value = unescape_attr(&attribute),
            _ => {}
        }
    }
    attrs
}

fn unescape_attr(attribute: &Attribute<'_>) -> Option<String> {
    attribute
        .normalized_value(XmlVersion::Implicit1_0)
        .ok()
        .map(|value| value.into_owned())
}

fn push_text(target: &mut String, value: &str) {
    let remaining = MAX_TEXT_CHARS.saturating_sub(target.len());
    if remaining == 0 {
        return;
    }
    if value.len() <= remaining {
        target.push_str(value);
        return;
    }
    let mut end = remaining.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    target.push_str(&value[..end]);
}

fn is_builtin_ref(name: &str) -> bool {
    matches!(name, "amp" | "lt" | "gt" | "quot" | "apos") || name.starts_with('#')
}

fn append_entity(target: &mut String, name: &str) {
    let wrapped = format!("&{name};");
    if let Ok(decoded) = quick_xml::escape::unescape(&wrapped) {
        push_text(target, &decoded);
    }
}

fn build_cell(text: &str, attrs: &CellAttrs) -> Option<CellData> {
    let typed = match attrs.value_type.as_deref() {
        Some("string") => CellValue::Text(text.to_string()),
        Some("float") | Some("currency") | Some("percentage") => attrs
            .value
            .as_deref()
            .and_then(|value| value.trim().parse::<f64>().ok())
            .map(CellValue::Number)
            .unwrap_or_else(|| CellValue::parse(text)),
        Some("boolean") => attrs
            .boolean_value
            .as_deref()
            .map(|value| CellValue::Bool(matches!(value, "true" | "TRUE" | "1")))
            .unwrap_or_else(|| CellValue::parse(text)),
        Some(_) => CellValue::Text(text.to_string()),
        None => CellValue::parse(text),
    };
    if text.is_empty() && typed.is_empty() && attrs.value.is_none() {
        return None;
    }
    Some(CellData {
        value: if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        },
        typed: Some(typed),
        ..CellData::default()
    })
}

pub(super) fn ensure_capacity(
    row_cells: usize,
    data_len: usize,
    row_repeat: u32,
) -> Result<(), String> {
    let projected = data_len.saturating_add(row_cells.saturating_mul(row_repeat as usize));
    if projected >= MAX_CELLS_PER_SHEET {
        return Err(reject(
            "cell cap exceeded",
            "ODS import failed: worksheet exceeds the 250000 cell import limit; export a smaller range",
        ));
    }
    Ok(())
}

#[derive(Default)]
struct OdsParser {
    worksheets: Vec<Worksheet>,
    in_table: bool,
    sheet_name: String,
    data: HashMap<String, CellData>,
    row_cells: HashMap<u32, CellData>,
    row_index: u32,
    row_repeat: u32,
    col_index: u32,
    in_row: bool,
    in_cell: bool,
    in_paragraph: bool,
    cell_text: String,
    cell_attrs: CellAttrs,
}

impl OdsParser {
    fn new() -> Self {
        Self {
            row_repeat: 1,
            cell_attrs: CellAttrs {
                repeat: 1,
                ..CellAttrs::default()
            },
            ..Self::default()
        }
    }

    fn begin_table(&mut self, start: &BytesStart<'_>) -> Result<(), String> {
        if self.in_table {
            self.end_table()?;
        }
        let name = attr_value(start, "name").filter(|value| !value.is_empty());
        self.sheet_name = name.unwrap_or_else(|| format!("Sheet{}", self.worksheets.len() + 1));
        self.data.clear();
        self.row_index = 0;
        self.in_table = true;
        Ok(())
    }

    fn end_table(&mut self) -> Result<(), String> {
        if !self.in_table {
            return Ok(());
        }
        self.in_table = false;
        if self.worksheets.len() >= MAX_SHEETS {
            return Err(reject(
                "sheet cap exceeded",
                "ODS import failed: too many worksheets in this package",
            ));
        }
        let mut name = std::mem::take(&mut self.sheet_name);
        if name.is_empty() {
            name = format!("Sheet{}", self.worksheets.len() + 1);
        }
        let data = std::mem::take(&mut self.data);
        self.worksheets.push(Worksheet {
            name,
            data,
            ..Worksheet::default()
        });
        Ok(())
    }

    fn begin_row(&mut self, start: &BytesStart<'_>) {
        self.row_cells.clear();
        let raw = attr_value(start, "number-rows-repeated");
        self.row_repeat = parse_repeat(raw.as_deref());
        self.col_index = 0;
        self.in_row = true;
    }

    fn end_row(&mut self) -> Result<(), String> {
        if !self.in_row {
            return Ok(());
        }
        self.in_row = false;
        let repeat = self.row_repeat.min(MAX_ROWS.saturating_sub(self.row_index));
        if repeat == 0 {
            return Ok(());
        }
        for offset in 0..repeat {
            let row = self.row_index + offset;
            for (column, cell) in &self.row_cells {
                self.data.insert(format!("{row},{column}"), cell.clone());
            }
        }
        self.row_index = self.row_index.saturating_add(repeat);
        Ok(())
    }

    fn begin_cell(&mut self, start: &BytesStart<'_>) {
        self.cell_attrs = read_cell_attrs(start);
        self.cell_text.clear();
        self.in_cell = true;
    }

    fn end_cell(&mut self) -> Result<(), String> {
        if !self.in_cell {
            return Ok(());
        }
        self.in_cell = false;
        self.in_paragraph = false;
        let repeated = self
            .cell_attrs
            .repeat
            .min(MAX_COLUMNS.saturating_sub(self.col_index));
        if repeated == 0 {
            return Ok(());
        }
        if let Some(cell) = build_cell(&self.cell_text, &self.cell_attrs) {
            for offset in 0..repeated {
                ensure_capacity(self.row_cells.len(), self.data.len(), self.row_repeat)?;
                self.row_cells.insert(self.col_index + offset, cell.clone());
            }
        }
        self.col_index = self.col_index.saturating_add(repeated);
        Ok(())
    }

    fn consume(&mut self, xml: &str) -> Result<(), String> {
        let mut reader = Reader::from_str(xml);
        let mut buffer = Vec::new();
        loop {
            let event = reader.read_event_into(&mut buffer).map_err(|error| {
                reject(
                    format!("xml parse error: {error}"),
                    "ODS import failed: malformed content.xml",
                )
            })?;
            match event {
                Event::DocType(_) => {
                    return Err(reject(
                        "DOCTYPE encountered",
                        "ODS import failed: XML DOCTYPE declarations are not allowed",
                    ));
                }
                Event::GeneralRef(name) => {
                    let reference = name.as_ref();
                    if !is_builtin_ref(reference) {
                        return Err(reject(
                            format!("custom entity: {reference}"),
                            "ODS import failed: custom XML entities are not supported",
                        ));
                    }
                    if self.in_paragraph {
                        append_entity(&mut self.cell_text, reference);
                    }
                }
                Event::Start(ref start) => {
                    let name = local_name(start.name());
                    if is_embedded_element_name(name) {
                        return Err(reject(
                            format!("embedded element: {name}"),
                            "ODS import failed: embedded objects are not supported; extract or save the object separately before importing",
                        ));
                    }
                    match name {
                        "table" => self.begin_table(start)?,
                        "table-row" if self.in_table => self.begin_row(start),
                        "table-cell" | "covered-table-cell" if self.in_table && self.in_row => {
                            self.begin_cell(start);
                        }
                        "p" if self.in_cell => {
                            if !self.cell_text.is_empty() {
                                push_text(&mut self.cell_text, "\n");
                            }
                            self.in_paragraph = true;
                        }
                        "line-break" if self.in_cell => push_text(&mut self.cell_text, "\n"),
                        "tab" if self.in_cell => push_text(&mut self.cell_text, "\t"),
                        _ => {}
                    }
                }
                Event::Empty(ref start) => {
                    let name = local_name(start.name());
                    if is_embedded_element_name(name) {
                        return Err(reject(
                            format!("embedded element: {name}"),
                            "ODS import failed: embedded objects are not supported; extract or save the object separately before importing",
                        ));
                    }
                    match name {
                        "table" => {
                            self.begin_table(start)?;
                            self.end_table()?;
                        }
                        "table-row" if self.in_table => {
                            self.begin_row(start);
                            self.end_row()?;
                        }
                        "table-cell" | "covered-table-cell" if self.in_table && self.in_row => {
                            self.begin_cell(start);
                            self.end_cell()?;
                        }
                        _ => {}
                    }
                }
                Event::Text(ref text) => {
                    if self.in_paragraph {
                        push_text(&mut self.cell_text, &text.xml10_content());
                    }
                }
                Event::CData(ref text) => {
                    if self.in_paragraph {
                        push_text(&mut self.cell_text, text.as_ref());
                    }
                }
                Event::End(ref end) => match local_name(end.name()) {
                    "table-cell" | "covered-table-cell" => self.end_cell()?,
                    "table-row" => self.end_row()?,
                    "table" => self.end_table()?,
                    "p" => self.in_paragraph = false,
                    _ => {}
                },
                Event::Eof => break,
                _ => {}
            }
            buffer.clear();
        }
        Ok(())
    }

    fn finish(mut self) -> Result<Vec<Worksheet>, String> {
        self.end_table()?;
        if self.worksheets.is_empty() {
            self.worksheets.push(Worksheet {
                name: "Sheet1".to_string(),
                ..Worksheet::default()
            });
        }
        Ok(self.worksheets)
    }
}

pub(super) fn parse_ods_content(xml: &str) -> Result<Vec<Worksheet>, String> {
    let mut parser = OdsParser::new();
    parser.consume(xml)?;
    parser.finish()
}
