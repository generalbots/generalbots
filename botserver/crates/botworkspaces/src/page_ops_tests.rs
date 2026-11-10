//! #2069 — page listing (sort/filter), statistics, duplication and templates.
//! All pure: no pool, no clock injection needed.
use super::*;

fn page(title: &str, blocks: Vec<Block>, children: Vec<Uuid>) -> Page {
    let now = Utc::now();
    Page {
        id: Uuid::new_v4(),
        workspace_id: Uuid::nil(),
        parent_id: None,
        title: title.to_string(),
        icon: None,
        cover_image: None,
        blocks,
        children,
        properties: HashMap::new(),
        permissions: PagePermissions::default(),
        is_template: false,
        template_id: None,
        created_at: now,
        updated_at: now,
        created_by: Uuid::nil(),
        last_edited_by: Uuid::nil(),
    }
}

fn summary(title: &str, created: i64, updated: i64) -> PageSummary {
    let base = Utc::now();
    PageSummary {
        id: Uuid::new_v4(),
        title: title.to_string(),
        icon: None,
        parent_id: None,
        has_children: false,
        created_at: base + chrono::Duration::seconds(created),
        updated_at: base + chrono::Duration::seconds(updated),
        created_by: Uuid::nil(),
        last_edited_by: Uuid::nil(),
    }
}

fn titles(pages: &[PageSummary]) -> Vec<String> {
    pages.iter().map(|p| p.title.clone()).collect()
}

#[test]
fn sorting_by_title_is_case_insensitive() {
    let mut pages = vec![summary("beta", 0, 0), summary("Alpha", 0, 0), summary("gamma", 0, 0)];
    sort_pages_by_title(&mut pages);
    assert_eq!(titles(&pages), vec!["Alpha", "beta", "gamma"]);
}

#[test]
fn sorting_by_updated_and_created_is_newest_first() {
    let mut pages = vec![summary("old", 0, 0), summary("new", 1, 2), summary("mid", 2, 1)];

    sort_pages_by_updated(&mut pages);
    assert_eq!(titles(&pages), vec!["new", "mid", "old"]);

    sort_pages_by_created(&mut pages);
    assert_eq!(titles(&pages), vec!["mid", "new", "old"]);
}

#[test]
fn date_filter_bounds_are_inclusive_and_optional() {
    let base = Utc::now();
    let pages = vec![
        summary("before", 0, -10),
        summary("inside", 0, 0),
        summary("after", 0, 10),
    ];

    let start = base - chrono::Duration::seconds(1);
    let end = base + chrono::Duration::seconds(1);
    let inside = filter_pages_by_date_range(pages.clone(), Some(start), Some(end));
    assert_eq!(titles(&inside), vec!["inside"]);

    let from_start = filter_pages_by_date_range(pages.clone(), Some(start), None);
    assert_eq!(titles(&from_start), vec!["inside", "after"]);

    let until_end = filter_pages_by_date_range(pages.clone(), None, Some(end));
    assert_eq!(titles(&until_end), vec!["before", "inside"]);

    let all = filter_pages_by_date_range(pages, None, None);
    assert_eq!(all.len(), 3);
}

#[test]
fn page_stats_count_nested_blocks_words_and_flags() {
    let mut paragraph = create_paragraph("hello brave new world", Uuid::nil());
    paragraph.children = vec![create_paragraph("nested two words", Uuid::nil())];

    let blocks = vec![
        paragraph,
        create_image("https://example.com/a.png", None, Uuid::nil()),
        create_table(2, 2, Uuid::nil()),
        create_code("fn main() {}", "rust", Uuid::nil()),
    ];
    let mut target = page("stats", blocks, vec![Uuid::new_v4(), Uuid::new_v4()]);

    let stats = calculate_page_stats(&target, 3);

    assert_eq!(stats.total_blocks, 5);
    assert_eq!(stats.total_words, 7);
    assert_eq!(stats.total_characters, "hello brave new world".len() + "nested two words".len());
    assert!(stats.has_images);
    assert!(stats.has_tables);
    assert!(stats.has_code);
    assert_eq!(stats.child_count, 2);
    assert_eq!(stats.comment_count, 3);

    target.blocks.clear();
    let empty = calculate_page_stats(&target, 0);
    assert_eq!(empty.total_blocks, 0);
    assert_eq!(empty.total_words, 0);
    assert!(!empty.has_images);
}

#[test]
fn cloning_blocks_gives_every_level_a_new_identity() {
    let mut parent = create_paragraph("parent", Uuid::nil());
    let child = create_paragraph("child", Uuid::nil());
    let original_ids = vec![parent.id, child.id];
    parent.children = vec![child];
    let author = Uuid::new_v4();

    let cloned = clone_blocks_with_new_ids(std::slice::from_ref(&parent), author);

    assert_eq!(cloned.len(), 1);
    assert!(!original_ids.contains(&cloned[0].id));
    assert!(!original_ids.contains(&cloned[0].children[0].id));
    assert_eq!(cloned[0].created_by, author);
    assert_eq!(cloned[0].children[0].created_by, author);
    // Content survives the copy.
    assert_eq!(cloned[0].children.len(), 1);
}

#[test]
fn duplicating_a_page_renames_it_and_reparents_the_copy_of_its_children() {
    let parent = page("Parent", vec![create_paragraph("body", Uuid::nil())], vec![]);
    let child = page("Child", vec![], vec![]);
    let parent = Page {
        children: vec![child.id],
        ..parent
    };

    let mut pages = HashMap::new();
    pages.insert(parent.id, parent.clone());
    pages.insert(child.id, child.clone());

    let copies = duplicate_page(&parent, None, None, None, Uuid::new_v4(), &pages, true);
    assert_eq!(copies.len(), 2);
    assert_eq!(copies[0].title, "Parent (Copy)");
    assert_ne!(copies[0].id, parent.id);
    assert_eq!(copies[1].title, "Child (Copy)");
    assert_eq!(copies[1].parent_id, Some(copies[0].id));
    // The copy is a page, never a template.
    assert!(!copies[0].is_template);
    // Children of the copy are not listed (they are duplicated as siblings).
    assert!(copies[0].children.is_empty());

    let shallow = duplicate_page(&parent, None, None, None, Uuid::nil(), &pages, false);
    assert_eq!(shallow.len(), 1);
}

#[test]
fn duplicating_a_page_honours_explicit_overrides() {
    let source = page("Original", vec![], vec![]);
    let workspace = Uuid::new_v4();
    let new_parent = Uuid::new_v4();
    let pages = HashMap::new();

    let copies = duplicate_page(
        &source,
        Some(new_parent),
        Some(workspace),
        Some("Renamed".to_string()),
        Uuid::nil(),
        &pages,
        false,
    );

    assert_eq!(copies[0].title, "Renamed");
    assert_eq!(copies[0].parent_id, Some(new_parent));
    assert_eq!(copies[0].workspace_id, workspace);
    assert!(copies[0].children.is_empty());
}

#[test]
fn applying_a_template_copies_blocks_with_new_ids() {
    let template = PageTemplate {
        id: Uuid::new_v4(),
        name: "Meeting Notes".to_string(),
        description: String::new(),
        icon: None,
        cover_image: None,
        blocks: vec![create_heading1("Agenda", Uuid::nil())],
        properties: HashMap::new(),
        category: TemplateCategory::Meeting,
        tags: vec![],
        is_system: true,
        organization_id: None,
        workspace_id: None,
        created_by: Uuid::nil(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        use_count: 0,
    };
    let workspace = Uuid::new_v4();

    let created = apply_template_to_page(&template, workspace, None, None, Uuid::nil());
    assert_eq!(created.title, "Meeting Notes");
    assert_eq!(created.template_id, Some(template.id));
    assert_eq!(created.workspace_id, workspace);
    assert_eq!(created.blocks.len(), 1);
    assert_ne!(created.blocks[0].id, template.blocks[0].id);

    let renamed = apply_template_to_page(&template, workspace, None, Some("Week 1".into()), Uuid::nil());
    assert_eq!(renamed.title, "Week 1");
}

#[test]
fn system_templates_are_stable_and_unique() {
    let templates = get_system_templates();
    assert_eq!(templates.len(), 3);
    assert!(templates.iter().all(|t| t.is_system));
    assert!(templates.iter().all(|t| t.created_by == Uuid::nil()));

    let ids: std::collections::HashSet<Uuid> = templates.iter().map(|t| t.id).collect();
    assert_eq!(ids.len(), templates.len(), "template ids must be unique");
}
