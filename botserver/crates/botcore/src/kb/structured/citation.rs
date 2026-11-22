use serde::{Deserialize, Serialize};

/// The kind of address a citation points at inside a source document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Page,
    Section,
    Clause,
    Slide,
    Cell,
}

impl SourceKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Section => "section",
            Self::Clause => "clause",
            Self::Slide => "slide",
            Self::Cell => "cell",
        }
    }
}

/// A stable address inside a document, optionally carrying the physical page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub kind: SourceKind,
    pub locator: String,
    pub page: Option<usize>,
}

impl SourceRef {
    pub fn page(page: usize) -> Self {
        Self {
            kind: SourceKind::Page,
            locator: page.to_string(),
            page: Some(page),
        }
    }

    pub fn section(title: impl Into<String>) -> Self {
        Self {
            kind: SourceKind::Section,
            locator: title.into(),
            page: None,
        }
    }

    pub fn clause(number: impl Into<String>) -> Self {
        Self {
            kind: SourceKind::Clause,
            locator: number.into(),
            page: None,
        }
    }

    pub fn slide(number: usize) -> Self {
        Self {
            kind: SourceKind::Slide,
            locator: number.to_string(),
            page: None,
        }
    }

    pub fn cell(reference: impl Into<String>) -> Self {
        Self {
            kind: SourceKind::Cell,
            locator: reference.into(),
            page: None,
        }
    }

    pub fn with_page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }

    /// Stable, locale-neutral text used for point IDs, logs and tests.
    pub fn address(&self) -> String {
        let mut value = format!("{}={}", self.kind.as_str(), self.locator);
        if let Some(page) = self.page {
            value.push_str(&format!(" p{}", page));
        }
        value
    }
}

/// A pointer from an answer back to the document version and exact location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Citation {
    pub document_id: String,
    pub file: String,
    pub version: String,
    pub source: SourceRef,
    pub heading_path: Vec<String>,
}

impl Citation {
    pub fn new(
        document_id: impl Into<String>,
        file: impl Into<String>,
        version: impl Into<String>,
        source: SourceRef,
        heading_path: Vec<String>,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            file: file.into(),
            version: version.into(),
            source,
            heading_path,
        }
    }

    /// Stable, locale-neutral summary for logs and non-UI surfaces. The Docs
    /// UI builds its own localized label from these structured fields.
    pub fn label(&self) -> String {
        let mut parts = vec![self.file.clone(), self.source.address()];
        if let Some(section) = self.heading_path.last() {
            if section != &self.source.locator {
                parts.push(format!("[{}]", section));
            }
        }
        parts.push(format!("v{}", self.version));
        parts.join(" · ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_ref_address_includes_page_when_present() {
        let reference = SourceRef::clause("7.3").with_page(12);
        assert_eq!(reference.address(), "clause=7.3 p12");
        assert_eq!(reference.kind, SourceKind::Clause);
    }

    #[test]
    fn citation_label_carries_file_location_and_version() {
        let citation = Citation::new(
            "contract",
            "contract.docx",
            "3",
            SourceRef::clause("7.3").with_page(12),
            vec!["Employment".to_string(), "Termination".to_string()],
        );
        assert_eq!(
            citation.label(),
            "contract.docx · clause=7.3 p12 · [Termination] · v3"
        );
    }

    #[test]
    fn every_source_kind_round_trips_through_json() {
        let kinds = vec![
            SourceRef::page(4),
            SourceRef::section("Intro"),
            SourceRef::clause("7.3.1"),
            SourceRef::slide(9),
            SourceRef::cell("B7"),
        ];
        for reference in kinds {
            let json = serde_json::to_string(&reference);
            assert!(json.is_ok());
            if let Ok(text) = json {
                let decoded = serde_json::from_str::<SourceRef>(&text);
                assert!(decoded.is_ok());
                if let Ok(value) = decoded {
                    assert_eq!(value, reference);
                }
            }
        }
    }
}
