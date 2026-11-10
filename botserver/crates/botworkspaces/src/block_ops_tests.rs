//! #2069 — block operations are the write path of the page editor and had no
//! tests: position clamping, move semantics and nested lookup are pinned here.
use super::*;

fn text_block(text: &str) -> Block {
    create_paragraph(text, Uuid::nil())
}

fn texts(blocks: &[Block]) -> Vec<String> {
    blocks
        .iter()
        .map(|b| match &b.content {
            BlockContent::Text { text } => text
                .segments
                .first()
                .map(|s| s.text.clone())
                .unwrap_or_default(),
            _ => String::new(),
        })
        .collect()
}

fn ids(blocks: &[Block]) -> Vec<Uuid> {
    blocks.iter().map(|b| b.id).collect()
}

fn operation(kind: BlockOperationType) -> BlockOperation {
    BlockOperation {
        operation_type: kind,
        block_id: None,
        parent_id: None,
        position: None,
        block: None,
        properties: None,
        content: None,
    }
}

#[test]
fn insert_honours_the_requested_position() {
    let mut blocks = vec![text_block("a"), text_block("c")];
    let mut op = operation(BlockOperationType::Insert);
    op.block = Some(text_block("b"));
    op.position = Some(1);
    apply_block_operations(&mut blocks, vec![op]);

    assert_eq!(texts(&blocks), vec!["a", "b", "c"]);
}

#[test]
fn insert_without_a_position_appends() {
    let mut blocks = vec![text_block("a")];
    let mut op = operation(BlockOperationType::Insert);
    op.block = Some(text_block("b"));
    apply_block_operations(&mut blocks, vec![op]);

    assert_eq!(texts(&blocks), vec!["a", "b"]);
}

#[test]
fn insert_past_the_end_is_dropped_not_clamped() {
    // Documented contract: an out-of-range insert is ignored (unlike `Move`
    // and `Duplicate`, which clamp). The editor never sends one; a client that
    // does must not corrupt the tree.
    let mut blocks = vec![text_block("a")];
    let mut op = operation(BlockOperationType::Insert);
    op.block = Some(text_block("b"));
    op.position = Some(5);
    apply_block_operations(&mut blocks, vec![op]);

    assert_eq!(texts(&blocks), vec!["a"]);
}

#[test]
fn insert_without_a_block_is_a_no_op() {
    let mut blocks = vec![text_block("a")];
    let mut op = operation(BlockOperationType::Insert);
    op.position = Some(0);
    apply_block_operations(&mut blocks, vec![op]);
    assert_eq!(texts(&blocks), vec!["a"]);
}

#[test]
fn update_replaces_content_and_properties_and_stamps_the_time() {
    let mut blocks = vec![text_block("old")];
    let original = blocks[0].updated_at;

    let mut op = operation(BlockOperationType::Update);
    op.block_id = Some(blocks[0].id);
    op.content = Some(BlockContent::Text {
        text: RichText {
            segments: vec![TextSegment {
                text: "new".to_string(),
                annotations: TextAnnotations::default(),
                link: None,
                mention: None,
            }],
        },
    });
    op.properties = Some(BlockProperties {
        color: Some("red".to_string()),
        background_color: None,
        indent_level: 2,
        collapsed: true,
    });
    apply_block_operations(&mut blocks, vec![op]);

    assert_eq!(texts(&blocks), vec!["new"]);
    assert_eq!(blocks[0].properties.indent_level, 2);
    assert_eq!(blocks[0].properties.color.as_deref(), Some("red"));
    assert!(blocks[0].updated_at >= original);
}

#[test]
fn update_finds_a_nested_block_and_ignores_an_unknown_id() {
    let mut parent = text_block("parent");
    let child = text_block("child");
    let child_id = child.id;
    parent.children = vec![child];
    let mut blocks = vec![parent];

    let mut op = operation(BlockOperationType::Update);
    op.block_id = Some(child_id);
    op.properties = Some(BlockProperties {
        indent_level: 3,
        ..BlockProperties::default()
    });
    apply_block_operations(&mut blocks, vec![op.clone()]);

    assert_eq!(blocks[0].children[0].properties.indent_level, 3);

    op.block_id = Some(Uuid::new_v4());
    apply_block_operations(&mut blocks, vec![op]);
    assert_eq!(blocks[0].children[0].properties.indent_level, 3);
}

#[test]
fn delete_removes_top_level_and_nested_blocks() {
    let mut parent = text_block("parent");
    let child = text_block("child");
    let child_id = child.id;
    parent.children = vec![child];
    let top = text_block("top");
    let top_id = top.id;
    let mut blocks = vec![top, parent];

    let mut op = operation(BlockOperationType::Delete);
    op.block_id = Some(child_id);
    apply_block_operations(&mut blocks, vec![op]);

    assert_eq!(blocks.len(), 2);
    assert!(blocks[1].children.is_empty());

    let mut op = operation(BlockOperationType::Delete);
    op.block_id = Some(top_id);
    apply_block_operations(&mut blocks, vec![op]);
    assert_eq!(texts(&blocks), vec!["parent"]);
}

#[test]
fn move_reorders_by_removing_then_inserting() {
    let mut blocks = vec![text_block("a"), text_block("b"), text_block("c")];
    let a = blocks[0].id;

    let mut op = operation(BlockOperationType::Move);
    op.block_id = Some(a);
    op.position = Some(2);
    apply_block_operations(&mut blocks, vec![op]);

    assert_eq!(texts(&blocks), vec!["b", "c", "a"]);
}

#[test]
fn move_clamps_a_position_past_the_end_and_requires_one() {
    let mut blocks = vec![text_block("a"), text_block("b")];
    let a = blocks[0].id;

    let mut op = operation(BlockOperationType::Move);
    op.block_id = Some(a);
    op.position = Some(99);
    apply_block_operations(&mut blocks, vec![op.clone()]);
    assert_eq!(texts(&blocks), vec!["b", "a"]);

    // Without a position the block is left where it was.
    let mut op = operation(BlockOperationType::Move);
    op.block_id = Some(a);
    apply_block_operations(&mut blocks, vec![op]);
    assert_eq!(texts(&blocks), vec!["b", "a"]);
}

#[test]
fn duplicate_creates_an_independent_copy_at_the_requested_position() {
    let mut parent = text_block("parent");
    parent.children = vec![text_block("child")];
    let original_id = parent.id;
    let child_id = parent.children[0].id;
    let mut blocks = vec![text_block("a"), parent];

    let mut op = operation(BlockOperationType::Duplicate);
    op.block_id = Some(original_id);
    op.position = Some(0);
    apply_block_operations(&mut blocks, vec![op]);

    assert_eq!(texts(&blocks), vec!["parent", "a", "parent"]);
    let copy = &blocks[0];
    assert_ne!(copy.id, original_id);
    assert_eq!(copy.children.len(), 1);
    // The copy's child must be addressable on its own (#2069): a shared id
    // makes `remove_block` delete the original child instead of the copy's.
    assert_ne!(copy.children[0].id, child_id);
    assert_eq!(blocks[1].children[0].id, child_id);
}

#[test]
fn duplicate_of_a_missing_block_is_a_no_op() {
    let mut blocks = vec![text_block("a")];
    let mut op = operation(BlockOperationType::Duplicate);
    op.block_id = Some(Uuid::new_v4());
    apply_block_operations(&mut blocks, vec![op]);
    assert_eq!(blocks.len(), 1);
}

#[test]
fn operations_apply_in_order() {
    let mut blocks = vec![text_block("a")];

    let mut insert = operation(BlockOperationType::Insert);
    insert.block = Some(text_block("b"));

    let duplicate = {
        let mut op = operation(BlockOperationType::Duplicate);
        op.block_id = Some(blocks[0].id);
        op
    };

    let delete = {
        let mut op = operation(BlockOperationType::Delete);
        op.block_id = Some(blocks[0].id);
        op
    };

    apply_block_operations(&mut blocks, vec![insert, duplicate, delete]);

    // a → insert b at end → duplicate a at end → delete a.
    assert_eq!(texts(&blocks), vec!["b", "a"]);
    assert_eq!(ids(&blocks).len(), 2);
}

#[test]
fn find_and_remove_walk_the_whole_tree() {
    let mut root = text_block("root");
    let mut branch = text_block("branch");
    let leaf = text_block("leaf");
    let leaf_id = leaf.id;
    branch.children = vec![leaf];
    root.children = vec![branch];
    let mut blocks = vec![root];

    assert!(find_block(&blocks, leaf_id).is_some());
    assert!(find_block_mut(&mut blocks, leaf_id).is_some());
    assert!(find_block(&blocks, Uuid::new_v4()).is_none());

    let removed = remove_block(&mut blocks, leaf_id).expect("removed");
    assert_eq!(texts(std::slice::from_ref(&removed)), vec!["leaf"]);
    assert!(find_block(&blocks, leaf_id).is_none());
    assert!(remove_block(&mut blocks, leaf_id).is_none());
}
