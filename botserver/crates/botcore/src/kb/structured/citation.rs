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

    fn sample_citation(source: SourceRef, heading_path: Vec<String>) -> Citation {
        Citation::new("contract", "contract.docx", "3", source, heading_path)
    }

    #[test]
    fn source_ref_address_includes_page_when_present() {
        let reference = SourceRef::clause("7.3").with_page(12);
        assert_eq!(reference.address(), "clause=7.3 p12");
        assert_eq!(reference.kind, SourceKind::Clause);
    }

    #[test]
    fn source_ref_address_formats_every_kind_without_page() {
        assert_eq!(SourceRef::page(4).address(), "page=4");
        assert_eq!(SourceRef::section("Intro").address(), "section=Intro");
        assert_eq!(SourceRef::clause("7.3").address(), "clause=7.3");
        assert_eq!(SourceRef::slide(9).address(), "slide=9");
        assert_eq!(SourceRef::cell("B7").address(), "cell=B7");
    }

    #[test]
    fn source_ref_address_appends_page_for_every_kind() {
        assert_eq!(SourceRef::page(4).with_page(12).address(), "page=4 p12");
        assert_eq!(
            SourceRef::section("Intro").with_page(12).address(),
            "section=Intro p12"
        );
        assert_eq!(
            SourceRef::clause("7.3").with_page(12).address(),
            "clause=7.3 p12"
        );
        assert_eq!(SourceRef::slide(9).with_page(12).address(), "slide=9 p12");
        assert_eq!(SourceRef::cell("B7").with_page(12).address(), "cell=B7 p12");
    }

    #[test]
    fn with_page_preserves_kind_and_locator() {
        let reference = SourceRef::cell("C4").with_page(7);
        assert_eq!(reference.kind, SourceKind::Cell);
        assert_eq!(reference.locator, "C4");
        assert_eq!(reference.page, Some(7));
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
    fn citation_label_is_locale_neutral() {
        let expected = [
            ("page=4", "contract.docx · page=4 · v3"),
            ("section=Intro", "contract.docx · section=Intro · v3"),
            ("clause=7.3 p12", "contract.docx · clause=7.3 p12 · v3"),
            ("slide=9", "contract.docx · slide=9 · v3"),
            ("cell=B7", "contract.docx · cell=B7 · v3"),
        ];
        let sources = [
            SourceRef::page(4),
            SourceRef::section("Intro"),
            SourceRef::clause("7.3").with_page(12),
            SourceRef::slide(9),
            SourceRef::cell("B7"),
        ];
        for (source, (token, label)) in sources.iter().zip(expected.iter()) {
            assert_eq!(source.address(), *token);
            assert_eq!(sample_citation(source.clone(), Vec::new()).label(), *label);
            for ch in label.chars() {
                assert!(
                    ch.is_ascii() || ch == '·',
                    "label contains a non-neutral character: {ch:?}"
                );
            }
        }
    }

    #[test]
    fn citation_label_joins_only_last_heading_from_path() {
        let path = vec!["Employment".to_string(), "Termination".to_string()];
        let citation = sample_citation(SourceRef::clause("7.3"), path);
        assert_eq!(
            citation.label(),
            "contract.docx · clause=7.3 · [Termination] · v3"
        );
    }

    #[test]
    fn citation_label_omits_heading_when_empty_or_equal_to_locator() {
        let no_path = sample_citation(SourceRef::clause("7.3"), Vec::new());
        assert_eq!(no_path.label(), "contract.docx · clause=7.3 · v3");

        let duplicate = sample_citation(
            SourceRef::section("Termination"),
            vec!["Employment".to_string(), "Termination".to_string()],
        );
        assert_eq!(
            duplicate.label(),
            "contract.docx · section=Termination · v3"
        );
    }

    #[test]
    fn citation_label_preserves_version_verbatim() {
        for version in ["1", "1.2.3", "v2-final", "2026.11"] {
            let citation = Citation::new(
                "contract",
                "contract.docx",
                version,
                SourceRef::page(4),
                Vec::new(),
            );
            assert_eq!(
                citation.label(),
                format!("contract.docx · page=4 · v{version}")
            );
            assert_eq!(citation.version, version);
        }
    }

    #[test]
    fn citation_new_preserves_all_fields() {
        let source = SourceRef::cell("B7").with_page(5);
        let path = vec!["Sheet1".to_string()];
        let citation = Citation::new("doc-9", "sheet.xlsx", "2", source.clone(), path.clone());
        assert_eq!(citation.document_id, "doc-9");
        assert_eq!(citation.file, "sheet.xlsx");
        assert_eq!(citation.version, "2");
        assert_eq!(citation.source, source);
        assert_eq!(citation.heading_path, path);
    }

    #[test]
    fn source_ref_serialization_is_stable() {
        let cases = [
            (
                SourceRef::page(4),
                r#"{"kind":"page","locator":"4","page":4}"#,
            ),
            (
                SourceRef::section("Intro"),
                r#"{"kind":"section","locator":"Intro","page":null}"#,
            ),
            (
                SourceRef::clause("7.3").with_page(12),
                r#"{"kind":"clause","locator":"7.3","page":12}"#,
            ),
            (
                SourceRef::slide(9),
                r#"{"kind":"slide","locator":"9","page":null}"#,
            ),
            (
                SourceRef::cell("B7"),
                r#"{"kind":"cell","locator":"B7","page":null}"#,
            ),
        ];
        for (reference, expected) in cases.iter() {
            let encoded = serde_json::to_string(reference).expect("serialize SourceRef");
            assert_eq!(encoded, *expected);
            assert_eq!(
                serde_json::to_string(reference).expect("serialize SourceRef again"),
                encoded,
                "serialization must be deterministic"
            );
        }
    }

    #[test]
    fn citation_serialization_is_stable_and_round_trips() {
        let citation = sample_citation(
            SourceRef::clause("7.3").with_page(12),
            vec!["Employment".to_string(), "Termination".to_string()],
        );
        let expected = concat!(
            r#"{"document_id":"contract","file":"contract.docx","version":"3","#,
            r#""source":{"kind":"clause","locator":"7.3","page":12},"#,
            r#""heading_path":["Employment","Termination"]}"#
        );
        let encoded = serde_json::to_string(&citation).expect("serialize Citation");
        assert_eq!(encoded, expected);
        assert_eq!(
            serde_json::to_string(&citation).expect("serialize Citation again"),
            encoded,
            "serialization must be deterministic"
        );
        let decoded: Citation = serde_json::from_str(&encoded).expect("deserialize Citation");
        assert_eq!(decoded, citation);
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
