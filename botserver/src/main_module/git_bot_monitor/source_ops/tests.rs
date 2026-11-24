use super::{flatten, tables::split_table_blocks};

#[test]
fn flatten_strips_generated_subdirectories() {
    assert_eq!(
        flatten("tools/classify_media.bas").unwrap_or(Default::default()),
        "classify_media.bas"
    );
    assert_eq!(
        flatten("classify_media.bas").unwrap_or(Default::default()),
        "classify_media.bas"
    );
}

#[test]
fn split_table_blocks_reads_named_blocks() {
    let src = "BEGIN TABLE orders\n    id UUID PRIMARY KEY\nEND TABLE\n";
    let blocks = split_table_blocks(src);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].0, "orders");
    assert!(blocks[0].1.contains("END TABLE"));
}

#[test]
fn split_table_blocks_ignores_prose() {
    assert!(split_table_blocks("' just a comment\n").is_empty());
}
