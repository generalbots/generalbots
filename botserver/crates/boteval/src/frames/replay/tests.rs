    use serde_json::json;

    use super::apply_patch;

use serde_json::json;

use super::apply_patch;

    #[test]
    fn patch_add_and_replace() {
        let mut document = json!({ "a": 1, "list": [1, 2] });
        apply_patch(
            &mut document,
            &[
                json!({ "op": "replace", "path": "/a", "value": 2 }),
                json!({ "op": "add", "path": "/list/-", "value": 3 }),
            ],
        )
        .expect("valid patch");
        assert_eq!(document, json!({ "a": 2, "list": [1, 2, 3] }));
    }

    #[test]
    fn patch_rejects_unknown_op() {
        let mut document = json!({});
        assert!(apply_patch(
            &mut document,
            &[json!({ "op": "move", "path": "/a" })]
        )
        .is_err());
    }
