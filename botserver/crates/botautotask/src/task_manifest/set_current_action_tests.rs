//! Offline tests for manifest plan construction and progress accounting
//! (issue #2072, AI-TASK-07). Pure structs: no DB, no clock injection needed.

use super::super::*;

fn section(id: &str, total_steps: u32, children: Vec<ManifestSection>) -> ManifestSection {
    ManifestSection {
        id: id.to_string(),
        name: id.to_string(),
        section_type: SectionType::Files,
        status: SectionStatus::Pending,
        current_step: 0,
        total_steps,
        global_step_start: 0,
        duration_seconds: None,
        started_at: None,
        completed_at: None,
        items: Vec::new(),
        item_groups: Vec::new(),
        children,
    }
}

fn item(id: &str) -> ManifestItem {
    ManifestItem {
        id: id.to_string(),
        name: id.to_string(),
        item_type: ItemType::File,
        status: ItemStatus::Pending,
        details: None,
        duration_seconds: None,
        started_at: None,
        completed_at: None,
        metadata: HashMap::new(),
    }
}

fn plan() -> TaskManifest {
    let mut manifest = TaskManifest::new("inventory", "track stock");
    manifest.add_section(section("setup", 3, vec![section("child", 2, Vec::new())]));
    manifest.add_section(section("build", 4, Vec::new()));
    manifest
}

#[test]
fn progress_percentage_is_zero_until_steps_complete() {
    let mut manifest = TaskManifest::new("empty", "nothing to do");
    assert_eq!(manifest.progress_percentage(), 0.0);

    manifest = plan();
    assert_eq!(manifest.total_steps, 9);
    assert_eq!(manifest.progress_percentage(), 0.0);

    manifest.update_section_status("build", SectionStatus::Completed);
    assert_eq!(manifest.completed_steps, 4);
    assert!((manifest.progress_percentage() - 44.444).abs() < 0.01);

    manifest.update_section_status("setup", SectionStatus::Completed);
    manifest.update_section_status("child", SectionStatus::Completed);
    assert_eq!(manifest.completed_steps, 9);
    assert!((manifest.progress_percentage() - 100.0).abs() < 1e-9);

    manifest.complete();
    assert_eq!(manifest.status, ManifestStatus::Completed);
    assert_eq!(manifest.completed_steps, manifest.total_steps);
}

#[test]
fn add_section_and_recalculate_keep_global_step_offsets() {
    let mut manifest = plan();
    assert_eq!(manifest.sections[0].global_step_start, 0);
    assert_eq!(manifest.sections[0].children[0].global_step_start, 0);
    assert_eq!(manifest.sections[1].global_step_start, 5);

    manifest.sections[1].total_steps = 10;
    manifest.recalculate_global_steps();
    assert_eq!(manifest.total_steps, 15);
    assert_eq!(manifest.sections[1].global_step_start, 5);
    assert_eq!(manifest.sections[0].global_step_start, 0);

    manifest.recalculate_global_steps();
    assert_eq!(manifest.total_steps, 15);
}

#[test]
fn unknown_section_and_item_ids_are_no_ops() {
    let mut manifest = plan();
    manifest.update_section_status("missing", SectionStatus::Completed);
    assert_eq!(manifest.completed_steps, 0);
    manifest.update_item_status("setup", "missing", ItemStatus::Completed);
    assert!(manifest.sections[0].items.is_empty());

    manifest.sections[0].items.push(item("start.bas"));
    manifest.update_item_status("setup", "start.bas", ItemStatus::Completed);
    assert_eq!(manifest.sections[0].items[0].status, ItemStatus::Completed);
    assert!(manifest.sections[0].items[0].completed_at.is_some());
}

#[test]
fn web_json_exposes_progress_action_and_decision_point() {
    let mut manifest = plan();
    manifest.set_current_action("writing tables.bas");
    manifest.set_decision_point(2, 5, "continue?");
    manifest.add_terminal_line("compiled", TerminalLineType::Success);
    manifest.update_section_status("build", SectionStatus::Completed);

    let json = manifest.to_web_json();
    assert_eq!(json["app_name"], "inventory");
    assert_eq!(json["status"]["current_action"], "writing tables.bas");
    assert_eq!(json["status"]["decision_point"]["step_current"], 2);
    assert_eq!(json["status"]["decision_point"]["step_total"], 5);
    assert_eq!(json["progress"]["current"], 4);
    assert_eq!(json["progress"]["total"], 9);
    assert_eq!(json["sections"].as_array().map(Vec::len), Some(2));
    assert_eq!(json["terminal"]["lines"][0]["type"], "success");
    assert_eq!(json["status"]["runtime_display"], "0 sec");
}

#[test]
fn task_md_lists_sections_and_items_with_checkboxes() {
    let mut manifest = plan();
    manifest.sections[0].items.push(item("start.bas"));
    manifest.update_section_status("setup", SectionStatus::Running);
    manifest.update_item_status("setup", "start.bas", ItemStatus::Completed);

    let md = manifest.to_task_md();
    assert!(md.starts_with("# TASK.md - inventory\n"));
    assert!(md.contains("- [>] setup (Step 0/9)"));
    assert!(md.contains("  - [x] start.bas"));
}
