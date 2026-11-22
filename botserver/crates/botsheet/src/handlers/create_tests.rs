    use super::*;

    #[test]
    fn created_spreadsheet_carries_an_id_and_the_rows() {
        let rows = json!([["Region", "Total"], ["North", 1200], ["South", 980],]);
        let sheet = build_spreadsheet("owner-1", Some("  Budget  "), Some(&rows))
            .expect("rows are a valid grid");
        assert!(!sheet.id.is_empty(), "the created artifact needs an id");
        assert_eq!(sheet.name, "Budget");
        let data = &sheet.worksheets[0].data;
        assert_eq!(
            data.get("0,0").and_then(|cell| cell.value.as_deref()),
            Some("Region")
        );
        assert_eq!(
            data.get("1,1").and_then(|cell| cell.value.as_deref()),
            Some("1200")
        );
    }

    #[test]
    fn a_flat_array_is_one_row() {
        let rows = json!(["Name", "Amount"]);
        let sheet = build_spreadsheet("owner-1", None, Some(&rows)).expect("valid row");
        assert_eq!(
            sheet.worksheets[0]
                .data
                .get("0,1")
                .and_then(|cell| cell.value.as_deref()),
            Some("Amount")
        );
    }

    #[test]
    fn rows_that_are_not_an_array_are_rejected() {
        let rows = json!({ "not": "a grid" });
        assert!(build_spreadsheet("owner-1", None, Some(&rows)).is_err());
    }

    #[test]
    fn a_cell_block_writes_from_the_range_start() {
        let grid = row_values(&json!([["a", "b"], ["c", "d"]])).expect("valid grid");
        assert_eq!(grid, vec![vec!["a", "b"], vec!["c", "d"]]);
        let ((row, col), _) =
            botsheet_core::formulas::refs::parse_range_or_single("B2:C3").expect("valid range");
        assert_eq!((row, col), (1, 1));
        let ((row, col), _) =
            botsheet_core::formulas::refs::parse_range_or_single("A1").expect("single cell");
        assert_eq!((row, col), (0, 0));
    }

    #[test]
    fn writing_a_value_keeps_the_existing_presentation() {
        let mut cell = cell_from_text("Cinza", &[], 0);
        let existing = CellData {
            value: Some("old".to_string()),
            typed: None,
            formula: None,
            style: Some(botsheet_core::types::CellStyle {
                background: Some("#fde68a".to_string()),
                font_weight: Some("bold".to_string()),
                ..Default::default()
            }),
            format: Some("0.00".to_string()),
            note: Some("keep me".to_string()),
            locked: Some(true),
            has_comment: Some(true),
            array_formula_id: None,
        };

        keep_presentation(&mut cell, Some(&existing));

        assert_eq!(cell.value.as_deref(), Some("Cinza"));
        let style = cell.style.as_ref().expect("style survives the write");
        assert_eq!(style.background.as_deref(), Some("#fde68a"));
        assert_eq!(style.font_weight.as_deref(), Some("bold"));
        assert_eq!(cell.format.as_deref(), Some("0.00"));
        assert_eq!(cell.note.as_deref(), Some("keep me"));
        assert_eq!(cell.locked, Some(true));
    }

