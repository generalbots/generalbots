//! Neutral input and output types for Office structure-aware chunking.

use serde::{Deserialize, Serialize};

/// Precise locator of one parsed block inside its source file.
///
/// Fields are inherited by following blocks until a new marker is seen, so a
/// paragraph after a page marker stays on that page without repeating it on
/// every block.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockLocator {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slide: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sheet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clause: Option<String>,
}

impl BlockLocator {
    pub fn page(page: u32) -> Self {
        Self {
            page: Some(page),
            ..Self::default()
        }
    }

    pub fn slide(slide: u32) -> Self {
        Self {
            slide: Some(slide),
            ..Self::default()
        }
    }

    pub fn sheet(name: impl Into<String>) -> Self {
        Self {
            sheet: Some(name.into()),
            ..Self::default()
        }
    }

    pub fn clause(number: impl Into<String>) -> Self {
        Self {
            clause: Some(number.into()),
            ..Self::default()
        }
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn with_cell(mut self, cell: impl Into<String>) -> Self {
        self.cell = Some(cell.into());
        self
    }
}

/// One block from a parsed DOCX/PPTX/XLSX structure.
///
/// A heading opens a new section scope at `level`; a text block is content in
/// the currently open scope. Slides and sheets use level-0 headings as roots
/// (see [`OfficeBlock::slide`], [`OfficeBlock::sheet`], [`OfficeBlock::cell`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficeBlock {
    pub level: u32,
    pub is_heading: bool,
    pub heading: Option<String>,
    pub text: String,
    pub locator: BlockLocator,
}

impl OfficeBlock {
    pub fn heading(level: u32, label: impl Into<String>, locator: BlockLocator) -> Self {
        Self {
            level,
            is_heading: true,
            heading: Some(label.into()),
            text: String::new(),
            locator,
        }
    }

    pub fn text(text: impl Into<String>, locator: BlockLocator) -> Self {
        Self {
            level: 0,
            is_heading: false,
            heading: None,
            text: text.into(),
            locator,
        }
    }

    /// Level-0 root heading for a slide; following text blocks are its content.
    pub fn slide(index: u32) -> Self {
        Self::heading(0, format!("Slide {index}"), BlockLocator::slide(index))
    }

    /// Level-0 root heading for a worksheet.
    pub fn sheet(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::heading(0, name.clone(), BlockLocator::sheet(name))
    }

    /// A cell value under the most recent sheet heading.
    pub fn cell(value: impl Into<String>, cell_ref: impl Into<String>) -> Self {
        Self::text(value, BlockLocator::default().with_cell(cell_ref))
    }
}

/// Identity and parsed blocks of one Office document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficeDocument {
    pub id: String,
    pub file: String,
    pub version: String,
    pub blocks: Vec<OfficeBlock>,
}

impl OfficeDocument {
    pub fn new(
        id: impl Into<String>,
        file: impl Into<String>,
        version: impl Into<String>,
        blocks: Vec<OfficeBlock>,
    ) -> Self {
        Self {
            id: id.into(),
            file: file.into(),
            version: version.into(),
            blocks,
        }
    }
}

/// Full citation anchor attached to every chunk.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAnchor {
    pub file: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slide: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sheet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cell: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clause: Option<String>,
    #[serde(default)]
    pub section_path: Vec<String>,
}

impl SourceAnchor {
    /// Canonical machine-readable address, used as part of the stable id.
    pub fn address(&self) -> String {
        let mut parts = vec![self.file.clone()];
        if let Some(page) = self.page {
            parts.push(format!("p{page}"));
        }
        if let Some(slide) = self.slide {
            parts.push(format!("s{slide}"));
        }
        if let Some(sheet) = &self.sheet {
            parts.push(format!("sh:{sheet}"));
        }
        if let Some(cell) = &self.cell {
            parts.push(format!("c:{cell}"));
        }
        if let Some(clause) = &self.clause {
            parts.push(format!("clause:{clause}"));
        }
        if !self.section_path.is_empty() {
            parts.push(format!("sec:{}", self.section_path.join("/")));
        }
        parts.push(format!("v{}", self.version));
        parts.join("|")
    }

    /// Human-readable citation: file, heading path, locator and version.
    pub fn citation(&self) -> String {
        let mut out = self.file.clone();
        if !self.section_path.is_empty() {
            out.push_str(" - ");
            out.push_str(&self.section_path.join(" > "));
        }
        if let Some(clause) = &self.clause {
            out.push_str(&format!(" [clause {clause}]"));
        }
        let mut locators = Vec::new();
        if let Some(page) = self.page {
            locators.push(format!("p.{page}"));
        }
        if let Some(slide) = self.slide {
            locators.push(format!("slide {slide}"));
        }
        if let Some(sheet) = &self.sheet {
            locators.push(match &self.cell {
                Some(cell) => format!("{sheet}!{cell}"),
                None => sheet.clone(),
            });
        }
        if !locators.is_empty() {
            out.push_str(" (");
            out.push_str(&locators.join(", "));
            out.push(')');
        }
        out.push_str(&format!(" v{}", self.version));
        out
    }
}

/// A KB-ready chunk: embed text plus the hierarchy and anchor for citation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficeKbChunk {
    pub id: String,
    pub document_id: String,
    pub file: String,
    pub version: String,
    /// Text to embed: ancestor headings on the first line, then the body.
    pub text: String,
    /// Body only, without the heading breadcrumb.
    pub body: String,
    pub anchor: SourceAnchor,
    pub heading_path: Vec<String>,
    pub level: u32,
}
