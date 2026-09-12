pub mod db;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod storage;

pub use routes::configure;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    /// The shipped media-filing template that turns an inbound Telegram image
    /// or document into a filed item (#1328, #1332).
    fn media_filing_dialog_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("bottemplates/bots/media-filing/media-filing.gbai/media-filing.gbdialog")
    }

    #[test]
    fn media_filing_template_ships_the_classification_tool() {
        let dir = media_filing_dialog_dir();
        assert!(
            dir.join("classify_media.bas").is_file(),
            "missing classify_media.bas in {}",
            dir.display()
        );
        assert!(
            dir.join("classify_media.mcp.json").is_file(),
            "missing classify_media.mcp.json in {}",
            dir.display()
        );
    }

    #[test]
    fn media_filing_taxonomy_is_closed_and_perception_degrades() {
        let script = std::fs::read_to_string(media_filing_dialog_dir().join("classify_media.bas"))
            .expect("classify_media.bas should be readable");

        // The closed taxonomy is what keeps a model answer from creating an
        // arbitrary folder, so every entry must stay in the script.
        for category in [
            "invoice",
            "receipt",
            "contract",
            "identity",
            "report",
            "unsorted",
        ] {
            assert!(
                script.contains(category),
                "closed taxonomy entry '{category}' is missing"
            );
        }

        // Perception must degrade to the caption instead of ending the task
        // before the item is filed (the vision service may be unavailable).
        assert!(
            script.contains("ON ERROR RESUME NEXT"),
            "perception errors are not trapped"
        );
        assert!(
            script.contains("perception = \"caption\""),
            "no caption fallback for an unavailable perception step"
        );

        // Filing stays outside the error-trapping region and defaults to the
        // neutral category.
        assert!(script.contains("MOVE path, destination"), "filing step missing");
        assert!(script.contains("ON ERROR GOTO 0"), "error trapping is never disabled");
        assert!(
            script.contains("category = \"unsorted\""),
            "classification must default to 'unsorted'"
        );
    }
}
