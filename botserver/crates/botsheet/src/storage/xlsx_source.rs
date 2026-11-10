//! Source-package snapshot and the preserve-vs-regenerate decision (#2043).
//!
//! A workbook opened from Drive keeps the bytes of the original package. Saving
//! patches that package ([`super::xlsx_passthrough`]) so charts, comments,
//! conditional formats, images, themes and styles survive an edit. When no
//! OOXML package is available the save/export must rebuild the workbook from
//! the model — a lossy operation that has to stay *explicit*: this module owns
//! the decision so save-back and `/api/sheet/export` report the same outcome and
//! nothing is dropped silently.

use crate::types::Spreadsheet;
use std::io::Cursor;

/// Why the workbook is rebuilt from the model instead of patching a package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegenerateReason {
    /// Nothing was retained: a new document, or a `csv`/`tsv` import.
    NoSnapshot,
    /// The snapshot is a different package format (`.xls`, `.xlsb`, `.ods`).
    NonOoxmlSource,
}

impl RegenerateReason {
    /// Stable machine-readable label (export header, save message).
    pub fn as_str(self) -> &'static str {
        match self {
            RegenerateReason::NoSnapshot => "no-snapshot",
            RegenerateReason::NonOoxmlSource => "non-ooxml-source",
        }
    }

    /// Human explanation of why fidelity cannot be preserved.
    pub fn description(self) -> &'static str {
        match self {
            RegenerateReason::NoSnapshot => "no source workbook was retained",
            RegenerateReason::NonOoxmlSource => "the source workbook is not an OOXML package",
        }
    }
}

/// What a save/export will do with the retained source package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XlsxPlan {
    /// Patch the retained package; every unmodelled part survives verbatim.
    Preserve,
    /// Rebuild from the model — the caller must say so (never silent).
    Regenerate(RegenerateReason),
}

impl XlsxPlan {
    /// Fidelity label for the export header / save message.
    pub fn as_str(self) -> &'static str {
        match self {
            XlsxPlan::Preserve => "preserved",
            XlsxPlan::Regenerate(_) => "regenerated",
        }
    }

    /// The reason, when this plan rebuilds the workbook.
    pub fn regenerate_reason(self) -> Option<RegenerateReason> {
        match self {
            XlsxPlan::Preserve => None,
            XlsxPlan::Regenerate(reason) => Some(reason),
        }
    }
}

/// xlsx bytes plus the path that produced them.
pub struct ProducedXlsx {
    pub bytes: Vec<u8>,
    pub plan: XlsxPlan,
}

/// Formats whose original package is worth retaining for save-back.
///
/// `csv`/`tsv` are plain text and have no package to patch, so keeping a copy
/// would only bloat the persisted model.
pub fn retains_snapshot(detected_format: &str) -> bool {
    matches!(detected_format, "xlsx" | "xlsm" | "xls" | "xlsb" | "ods")
}

/// True when `bytes` is an OOXML spreadsheet package (`xl/workbook.xml` in a
/// zip) — the only thing [`super::xlsx_passthrough`] can patch.
pub fn is_ooxml_package(bytes: &[u8]) -> bool {
    let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(bytes)) else {
        return false;
    };
    let found = archive.by_name("xl/workbook.xml").is_ok();
    found
}

/// Decides how a save/export produces the xlsx bytes for this sheet.
pub fn plan_for(sheet: &Spreadsheet) -> XlsxPlan {
    match sheet.source_bytes.as_deref() {
        Some(bytes) if is_ooxml_package(bytes) => XlsxPlan::Preserve,
        Some(_) => XlsxPlan::Regenerate(RegenerateReason::NonOoxmlSource),
        None => XlsxPlan::Regenerate(RegenerateReason::NoSnapshot),
    }
}

/// Produces the xlsx bytes for a save/export, honouring the plan.
///
/// A `Preserve` that cannot patch the package returns the error: the original
/// stays untouched and the caller reports the failure, instead of quietly
/// shipping a rebuilt workbook that lost charts/comments/styles (#2043).
pub fn produce_xlsx(sheet: &Spreadsheet) -> Result<ProducedXlsx, String> {
    match plan_for(sheet) {
        XlsxPlan::Preserve => {
            let original = sheet
                .source_bytes
                .as_deref()
                .ok_or_else(|| "no source package to patch".to_string())?;
            let bytes = super::xlsx_passthrough::merge_into_original(original, sheet)?;
            Ok(ProducedXlsx {
                bytes,
                plan: XlsxPlan::Preserve,
            })
        }
        plan @ XlsxPlan::Regenerate(_) => {
            let bytes = super::xlsx_write::convert_to_xlsx(sheet)?;
            Ok(ProducedXlsx { bytes, plan })
        }
    }
}

/// Where a save-back writes, and whether that is the file the user opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveBackTarget {
    pub bucket: String,
    pub path: String,
    /// True when `path` is the opened file (patched in place).
    pub in_place: bool,
    pub plan: XlsxPlan,
}

/// Destination of the save-back write, or `None` when the sheet has no source.
///
/// A package that can be patched is written back to the opened file; anything
/// else goes to the `<name>.gbsheet.xlsx` sibling so a rebuilt workbook never
/// replaces the original.
pub fn save_back_target(sheet: &Spreadsheet) -> Option<SaveBackTarget> {
    let bucket = sheet.source_bucket.clone()?;
    let source_path = sheet.source_path.clone()?;
    let plan = plan_for(sheet);
    let in_place = plan == XlsxPlan::Preserve;
    let path = if in_place {
        source_path
    } else {
        super::sibling_xlsx_path(&source_path)
    };
    Some(SaveBackTarget {
        bucket,
        path,
        in_place,
        plan,
    })
}

/// Save-response message naming the file that was actually written.
pub fn save_back_message(target: Option<&SaveBackTarget>) -> String {
    match target {
        None => "Sheet saved successfully".to_string(),
        Some(target) if target.in_place => format!("Saved to {}", target.path),
        Some(target) => {
            let reason = target
                .plan
                .regenerate_reason()
                .map(RegenerateReason::description)
                .unwrap_or("the source package could not be patched");
            format!(
                "Saved as {} (the opened file was left untouched: {reason})",
                target.path
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn zip_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut buffer);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            for (name, data) in entries {
                writer.start_file(*name, options).expect("entry");
                writer.write_all(data).expect("write");
            }
            writer.finish().expect("finish");
        }
        buffer.into_inner()
    }

    fn ooxml_fixture() -> Vec<u8> {
        zip_with(&[
            ("xl/workbook.xml", b"<workbook><sheets/></workbook>"),
            (
                "xl/worksheets/sheet1.xml",
                b"<worksheet><sheetData/></worksheet>",
            ),
        ])
    }

    fn ods_fixture() -> Vec<u8> {
        zip_with(&[
            ("content.xml", b"<office/>"),
            (
                "mimetype",
                b"application/vnd.oasis.opendocument.spreadsheet",
            ),
        ])
    }

    fn sheet_with_source(bytes: Vec<u8>) -> Spreadsheet {
        let mut sheet = crate::storage::create_new_spreadsheet("owner");
        sheet.source_bucket = Some("gbo".to_string());
        sheet.source_path = Some("drive/budget.xlsx".to_string());
        sheet.source_bytes = Some(bytes);
        sheet
    }

    /// #2043 — package formats keep the uploaded bytes; text formats do not.
    #[test]
    fn snapshot_is_retained_for_package_formats_only() {
        for format in ["xlsx", "xlsm", "xls", "xlsb", "ods"] {
            assert!(retains_snapshot(format), "{format} keeps its package");
        }
        for format in ["csv", "tsv", "unknown"] {
            assert!(!retains_snapshot(format), "{format} has no package");
        }
    }

    /// #2043 — the uploaded workbook is snapshotted so a later save can patch
    /// the original instead of rebuilding it from the model.
    #[test]
    fn import_retains_the_uploaded_package() {
        let mut workbook = rust_xlsxwriter::Workbook::new();
        workbook
            .add_worksheet()
            .write_string(0, 0, "Item")
            .expect("write");
        let xlsx = workbook.save_to_buffer().expect("save");

        let imported =
            crate::storage::import::import_spreadsheet_bytes(&xlsx, "budget.xlsx", "owner")
                .expect("import");
        assert_eq!(
            imported.source_bytes.as_deref(),
            Some(xlsx.as_slice()),
            "the uploaded package must be retained byte-for-byte"
        );
        assert_eq!(plan_for(&imported), XlsxPlan::Preserve);

        let csv =
            crate::storage::import::import_spreadsheet_bytes(b"a,b\n1,2\n", "data.csv", "owner")
                .expect("csv import");
        assert!(csv.source_bytes.is_none(), "csv has no package to patch");
    }

    /// #2043 — the plan is preserve when the snapshot is OOXML and an explicit
    /// regenerate (with a reason) otherwise.
    #[test]
    fn plan_prefers_the_retained_package() {
        assert_eq!(
            plan_for(&sheet_with_source(ooxml_fixture())),
            XlsxPlan::Preserve
        );
        assert_eq!(
            plan_for(&sheet_with_source(ods_fixture())),
            XlsxPlan::Regenerate(RegenerateReason::NonOoxmlSource)
        );
        let mut fresh = crate::storage::create_new_spreadsheet("owner");
        fresh.source_bytes = None;
        assert_eq!(
            plan_for(&fresh),
            XlsxPlan::Regenerate(RegenerateReason::NoSnapshot)
        );
        assert_eq!(XlsxPlan::Preserve.as_str(), "preserved");
        assert_eq!(
            XlsxPlan::Regenerate(RegenerateReason::NoSnapshot).as_str(),
            "regenerated"
        );
    }

    /// #2043 — a failed patch is an error, never a silent rebuild.
    #[test]
    fn patch_failure_is_an_error_not_a_rebuild() {
        let broken = zip_with(&[
            (
                "xl/workbook.xml",
                br#"<workbook><sheets><sheet name="Budget" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
            ),
            (
                "xl/_rels/workbook.xml.rels",
                br#"<Relationships><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
            ),
            // No <sheetData>: the cell patcher cannot place the edit.
            ("xl/worksheets/sheet1.xml", b"<worksheet><foo/></worksheet>"),
        ]);
        let mut sheet = sheet_with_source(broken);
        sheet.worksheets[0].name = "Budget".to_string();
        sheet.worksheets[0].data.insert(
            "0,0".to_string(),
            crate::types::CellData {
                value: Some("edited".to_string()),
                typed: None,
                formula: None,
                style: None,
                format: None,
                note: None,
                locked: None,
                has_comment: None,
                array_formula_id: None,
            },
        );
        let result = produce_xlsx(&sheet);
        assert!(
            result.is_err(),
            "an unpatched package must surface the error, not regenerate"
        );
    }

    /// #2043 — a patchable source is written back to the opened file; anything
    /// else lands in the sibling and says why.
    #[test]
    fn save_back_target_names_the_written_file() {
        let in_place = save_back_target(&sheet_with_source(ooxml_fixture())).expect("target");
        assert!(in_place.in_place);
        assert_eq!(in_place.path, "drive/budget.xlsx");
        assert_eq!(
            save_back_message(Some(&in_place)),
            "Saved to drive/budget.xlsx"
        );

        let sibling = save_back_target(&sheet_with_source(ods_fixture())).expect("target");
        assert!(!sibling.in_place);
        assert_eq!(sibling.path, "drive/budget.gbsheet.xlsx");
        let message = save_back_message(Some(&sibling));
        assert!(message.contains("drive/budget.gbsheet.xlsx"), "{message}");
        assert!(message.contains("left untouched"), "{message}");

        let mut no_source = crate::storage::create_new_spreadsheet("owner");
        no_source.source_path = None;
        assert!(save_back_target(&no_source).is_none());
        assert_eq!(save_back_message(None), "Sheet saved successfully");
    }
}
