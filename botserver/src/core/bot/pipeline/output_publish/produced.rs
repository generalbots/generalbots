//! The `files_produced` frame: every file this turn left in `output/`, with
//! its Drive copy when one was made.

use serde_json::{json, Value};

use super::{OutputFile, PublishReport};
use crate::core::bot::drive_bridge;

/// The file name inside a chat link `[name](vmfile:…)`.
fn link_name(link: &str) -> Option<&str> {
    let rest = link.strip_prefix('[')?;
    let end = rest.find("](")?;
    Some(&rest[..end])
}

/// `{"type":"files_produced","files":[{name, path, size, mime, drive_path}]}`
/// for the turn's report, or `None` when the turn produced no file. Sizes come
/// from the `output/` listing (0 when the file is no longer listed).
pub(super) fn files_frame(report: &PublishReport, listing: &[OutputFile]) -> Option<Value> {
    let size_of = |name: &str| listing.iter().find(|file| file.name == name).map_or(0, |file| file.size);
    let entry = |name: &str, drive: Option<&str>| {
        json!({
            "name": name,
            "path": format!("output/{name}"),
            "size": size_of(name),
            "mime": drive_bridge::content_type(name),
            "drive_path": drive,
        })
    };
    let mut files: Vec<Value> = report
        .published
        .iter()
        .map(|item| entry(&item.name, Some(item.drive_key.as_str())))
        .collect();
    files.extend(report.unsaved.iter().filter_map(|link| link_name(link)).map(|name| entry(name, None)));
    (!files.is_empty()).then(|| json!({"type": "files_produced", "files": files}))
}

#[cfg(test)]
mod tests {
    use super::super::{output_link, PublishedArtifact};
    use super::*;
    use uuid::Uuid;

    #[test]
    fn saved_and_unsaved_files_are_listed_with_their_drive_copy() {
        let session = Uuid::new_v4();
        let report = PublishReport {
            folder: "relatorio-1a2b3c4d".into(),
            published: vec![PublishedArtifact {
                name: "relatorio.pdf".into(),
                drive_key: "gbo/default.gbai/default.gbdrive/conversations/relatorio-1a2b3c4d/relatorio.pdf".into(),
                link: String::new(),
            }],
            unsaved: vec![output_link("dados.csv", session)],
        };
        let listing = vec![
            OutputFile { name: "relatorio.pdf".into(), size: 2048, modified: 1 },
            OutputFile { name: "dados.csv".into(), size: 10, modified: 1 },
        ];
        let frame = files_frame(&report, &listing).unwrap_or_default();
        assert_eq!(frame["type"], "files_produced");
        let files = frame["files"].as_array().cloned().unwrap_or_default();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0]["path"], "output/relatorio.pdf");
        assert_eq!(files[0]["size"], 2048);
        assert_eq!(files[0]["mime"], "application/pdf");
        assert!(files[0]["drive_path"].as_str().is_some_and(|p| p.ends_with("/relatorio.pdf")));
        assert_eq!(files[1]["name"], "dados.csv");
        assert!(files[1]["drive_path"].is_null());
    }

    #[test]
    fn a_turn_without_files_has_no_frame() {
        assert!(files_frame(&PublishReport::default(), &[]).is_none());
        assert_eq!(link_name("[a b.pdf](vmfile:x/output/a b.pdf)"), Some("a b.pdf"));
        assert_eq!(link_name("texto"), None);
    }
}
