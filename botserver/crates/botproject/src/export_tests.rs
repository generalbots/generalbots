//! Export surface: the three writers (MS Project XML, CSV, JSON) are pure
//! string builders, and the XML writer must stay importable by
//! [`crate::import`] — that round trip is the contract both sides share.
use super::*;

use crate::import::{ImportOptions, ProjectImportService};

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("fixture date")
}

fn project() -> Project {
    Project {
        id: Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("uuid"),
        organization_id: Uuid::nil(),
        name: "Migration & rollout".to_string(),
        description: Some("Cutover <planned>".to_string()),
        start_date: date(2026, 9, 1),
        end_date: Some(date(2026, 9, 30)),
        status: ProjectStatus::Active,
        owner_id: Uuid::nil(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        settings: ProjectSettings::default(),
    }
}

fn task(project_id: Uuid, name: &str, uid_seed: u8) -> ProjectTask {
    ProjectTask {
        id: Uuid::from_u128(u128::from(uid_seed)),
        project_id,
        parent_id: None,
        name: name.to_string(),
        description: None,
        task_type: TaskType::Task,
        start_date: date(2026, 9, 1),
        end_date: date(2026, 9, 3),
        duration_days: 2,
        percent_complete: 40,
        status: TaskStatus::InProgress,
        priority: TaskPriority::Normal,
        assigned_to: Vec::new(),
        dependencies: Vec::new(),
        estimated_hours: None,
        actual_hours: None,
        cost: None,
        notes: None,
        wbs: "1.1".to_string(),
        outline_level: 1,
        is_milestone: false,
        is_summary: false,
        is_critical: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn resource(project_id: Uuid) -> Resource {
    Resource {
        id: Uuid::from_u128(7),
        project_id,
        user_id: None,
        name: "Ana".to_string(),
        resource_type: ResourceType::Work,
        email: Some("ana@example.com".to_string()),
        max_units: 1.0,
        standard_rate: Some(60.0),
        overtime_rate: None,
        cost_per_use: None,
        calendar_id: None,
        created_at: Utc::now(),
    }
}

#[test]
fn escape_xml_covers_every_markup_character() {
    assert_eq!(
        escape_xml("a & b <c> \"d\" 'e'"),
        "a &amp; b &lt;c&gt; &quot;d&quot; &apos;e&apos;"
    );
}

#[test]
fn ms_date_uses_the_midnight_form_the_importer_accepts() {
    assert_eq!(ms_date(date(2026, 9, 1)), "2026-09-01T00:00:00");
}

#[test]
fn ms_duration_converts_days_at_eight_hours_each() {
    assert_eq!(ms_duration(0), "PT0H");
    assert_eq!(ms_duration(3), "PT24H");
}

#[test]
fn dependency_types_map_to_the_ms_project_codes() {
    assert_eq!(dependency_type_code(DependencyType::FinishToFinish), 0);
    assert_eq!(dependency_type_code(DependencyType::FinishToStart), 1);
    assert_eq!(dependency_type_code(DependencyType::StartToFinish), 2);
    assert_eq!(dependency_type_code(DependencyType::StartToStart), 3);
}

#[test]
fn csv_escape_quotes_only_when_needed() {
    assert_eq!(csv_escape("plain"), "plain");
    assert_eq!(csv_escape("a,b"), "\"a,b\"");
    assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    assert_eq!(csv_escape("two\nlines"), "\"two\nlines\"");
}

#[test]
fn csv_export_writes_the_header_and_escaped_rows() {
    let project = project();
    let csv = export_csv(&project, &[task(project.id, "Plan, phase 1", 2)]);

    let mut lines = csv.lines();
    assert_eq!(lines.next(), Some("name,start,end,duration,progress"));
    assert_eq!(
        lines.next(),
        Some("\"Plan, phase 1\",2026-09-01,2026-09-03,2,40")
    );
    assert_eq!(lines.next(), None);
}

#[test]
fn json_export_serializes_the_project_and_its_tasks() {
    let project = project();
    let json = export_json(&project, &[task(project.id, "Plan", 2)]);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");

    assert_eq!(parsed["name"], "Migration & rollout");
    assert_eq!(parsed["tasks"][0]["name"], "Plan");
    assert_eq!(parsed["tasks"][0]["duration"], 2);
    assert_eq!(parsed["tasks"][0]["progress"], 40);
}

#[test]
fn xml_export_escapes_names_and_marks_a_milestone() {
    let project = project();
    let mut milestone = task(project.id, "Go live", 3);
    milestone.is_milestone = true;
    milestone.outline_level = 0;

    let xml = export_ms_project_xml(&project, &[milestone], &[], &[]);

    assert!(xml.starts_with("<?xml version=\"1.0\""));
    assert!(xml.contains("<Name>Migration &amp; rollout</Name>"));
    assert!(xml.contains("<Notes>Cutover &lt;planned&gt;</Notes>"));
    assert!(xml.contains("<StartDate>2026-09-01T00:00:00</StartDate>"));
    assert!(xml.contains("<FinishDate>2026-09-30T00:00:00</FinishDate>"));
    assert!(xml.contains("<Milestone>1</Milestone>"));
    // `outline_level` is clamped so MS Project never sees level 0.
    assert!(xml.contains("<OutlineLevel>1</OutlineLevel>"));
    assert!(xml.contains("<Duration>PT16H</Duration>"));
}

#[test]
fn xml_export_links_dependencies_by_uid_and_scales_the_lag() {
    let project = project();
    let mut first = task(project.id, "Design", 2);
    let second_seed = 3u8;
    let mut second = task(project.id, "Build", second_seed);
    second.dependencies = vec![TaskDependency {
        predecessor_id: first.id,
        dependency_type: DependencyType::StartToStart,
        lag_days: 2,
    }];
    first.dependencies = Vec::new();

    let xml = export_ms_project_xml(&project, &[first, second], &[], &[]);

    assert!(xml.contains("<PredecessorUID>1</PredecessorUID>"));
    assert!(xml.contains("<Type>3</Type>"));
    assert!(xml.contains("<LinkLag>9600</LinkLag>"));
}

#[test]
fn xml_export_round_trips_through_the_importer() {
    let project = project();
    let mut design = task(project.id, "Design", 2);
    let build = task(project.id, "Build", 3);
    design.dependencies = Vec::new();
    let resource = resource(project.id);
    let assignment = ResourceAssignment {
        id: Uuid::from_u128(9),
        task_id: design.id,
        resource_id: resource.id,
        units: 1.0,
        work_hours: 16.0,
        start_date: date(2026, 9, 1),
        end_date: date(2026, 9, 3),
        cost: 960.0,
    };

    let xml = export_ms_project_xml(
        &project,
        &[design.clone(), build],
        &[resource],
        &[assignment],
    );
    let imported = ProjectImportService::new()
        .import(xml.as_bytes(), ImportOptions::default())
        .expect("the exporter's own output must import");

    assert_eq!(imported.project.name, "Migration & rollout");
    assert_eq!(imported.project.start_date, date(2026, 9, 1));
    assert_eq!(imported.tasks.len(), 2);
    assert_eq!(imported.stats.tasks_imported, 2);

    let design_again = imported
        .tasks
        .iter()
        .find(|task| task.name == "Design")
        .expect("design task");
    assert_eq!(design_again.duration_days, 2);
    assert_eq!(design_again.percent_complete, 40);
    assert_eq!(design_again.wbs, "1.1");

    assert_eq!(imported.resources.len(), 1);
    assert_eq!(imported.resources[0].name, "Ana");
    assert_eq!(imported.resources[0].resource_type, ResourceType::Work);
    assert_eq!(imported.assignments.len(), 1);
    assert_eq!(imported.assignments[0].work_hours, 16.0);
}
