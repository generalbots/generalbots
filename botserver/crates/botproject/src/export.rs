//! Export a project (with tasks, resources, assignments) to MS Project XML,
//! CSV, or JSON. The XML shape mirrors [`crate::import`]'s parser so an
//! exported file round-trips through import.

use std::collections::HashMap;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::types::*;

/// Map a task dependency type to the MS Project `PredecessorLink` `Type`
/// integer. See the reverse mapping in [`crate::import`]: 0=FF, 1=FS, 2=SF,
/// 3=SS.
fn dependency_type_code(dep: DependencyType) -> i32 {
    match dep {
        DependencyType::FinishToFinish => 0,
        DependencyType::FinishToStart => 1,
        DependencyType::StartToFinish => 2,
        DependencyType::StartToStart => 3,
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// `chrono::NaiveDate` as an MS Project `Start`/`Finish` string
/// (`YYYY-MM-DDT00:00:00`), which [`crate::import::parse_ms_date`] accepts.
fn ms_date(d: NaiveDate) -> String {
    format!("{}T00:00:00", d.format("%Y-%m-%d"))
}

/// Duration in days as an MS Project `Duration` string (`PT{n}H`, 8h/day).
fn ms_duration(days: u32) -> String {
    format!("PT{}H", days.saturating_mul(8))
}

/// Build an MS Project XML document from the project snapshot.
pub fn export_ms_project_xml(
    project: &Project,
    tasks: &[ProjectTask],
    resources: &[Resource],
    assignments: &[ResourceAssignment],
) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
    out.push_str("<Project xmlns=\"http://schemas.microsoft.com/project\">\n");

    out.push_str(&format!("<Name>{}</Name>\n", escape_xml(&project.name)));
    if let Some(desc) = &project.description {
        out.push_str(&format!("<Notes>{}</Notes>\n", escape_xml(desc)));
    }
    out.push_str(&format!("<StartDate>{}</StartDate>\n", ms_date(project.start_date)));
    if let Some(end) = project.end_date {
        out.push_str(&format!("<FinishDate>{}</FinishDate>\n", ms_date(end)));
    }

    // UID assignment mirrors the importer: 1-based index per collection.
    let task_uid: HashMap<Uuid, i32> = tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id, (i + 1) as i32))
        .collect();
    let resource_uid: HashMap<Uuid, i32> = resources
        .iter()
        .enumerate()
        .map(|(i, r)| (r.id, (i + 1) as i32))
        .collect();

    out.push_str("<Tasks>\n");
    for task in tasks {
        out.push_str("<Task>\n");
        out.push_str(&format!(
            "<UID>{}</UID>\n",
            task_uid.get(&task.id).copied().unwrap_or(0)
        ));
        out.push_str(&format!("<Name>{}</Name>\n", escape_xml(&task.name)));
        if !task.wbs.is_empty() {
            out.push_str(&format!("<WBS>{}</WBS>\n", escape_xml(&task.wbs)));
        }
        out.push_str(&format!("<OutlineLevel>{}</OutlineLevel>\n", task.outline_level.max(1)));
        out.push_str(&format!("<Start>{}</Start>\n", ms_date(task.start_date)));
        out.push_str(&format!("<Finish>{}</Finish>\n", ms_date(task.end_date)));
        out.push_str(&format!("<Duration>{}</Duration>\n", ms_duration(task.duration_days)));
        out.push_str(&format!(
            "<PercentComplete>{}</PercentComplete>\n",
            task.percent_complete
        ));
        if task.is_milestone {
            out.push_str("<Milestone>1</Milestone>\n");
        }
        if task.is_summary {
            out.push_str("<Summary>1</Summary>\n");
        }
        for dep in &task.dependencies {
            out.push_str("<PredecessorLink>\n");
            out.push_str(&format!(
                "<PredecessorUID>{}</PredecessorUID>\n",
                task_uid.get(&dep.predecessor_id).copied().unwrap_or(0)
            ));
            out.push_str(&format!(
                "<Type>{}</Type>\n",
                dependency_type_code(dep.dependency_type)
            ));
            // The importer divides LinkLag by 4800 to recover lag_days.
            out.push_str(&format!("<LinkLag>{}</LinkLag>\n", dep.lag_days.saturating_mul(4800)));
            out.push_str("</PredecessorLink>\n");
        }
        out.push_str("</Task>\n");
    }
    out.push_str("</Tasks>\n");

    out.push_str("<Resources>\n");
    for resource in resources {
        out.push_str("<Resource>\n");
        out.push_str(&format!(
            "<UID>{}</UID>\n",
            resource_uid.get(&resource.id).copied().unwrap_or(0)
        ));
        out.push_str(&format!("<Name>{}</Name>\n", escape_xml(&resource.name)));
        let type_code = match resource.resource_type {
            ResourceType::Material => 0,
            ResourceType::Work => 1,
            ResourceType::Cost => 2,
        };
        out.push_str(&format!("<Type>{}</Type>\n", type_code));
        if let Some(email) = &resource.email {
            out.push_str(&format!("<EmailAddress>{}</EmailAddress>\n", escape_xml(email)));
        }
        out.push_str(&format!("<MaxUnits>{}</MaxUnits>\n", resource.max_units));
        if let Some(rate) = resource.standard_rate {
            out.push_str(&format!("<StandardRate>{}</StandardRate>\n", rate));
        }
        if let Some(rate) = resource.overtime_rate {
            out.push_str(&format!("<OvertimeRate>{}</OvertimeRate>\n", rate));
        }
        out.push_str("</Resource>\n");
    }
    out.push_str("</Resources>\n");

    out.push_str("<Assignments>\n");
    for (i, assignment) in assignments.iter().enumerate() {
        out.push_str("<Assignment>\n");
        out.push_str(&format!("<UID>{}</UID>\n", i + 1));
        out.push_str(&format!(
            "<TaskUID>{}</TaskUID>\n",
            task_uid.get(&assignment.task_id).copied().unwrap_or(0)
        ));
        out.push_str(&format!(
            "<ResourceUID>{}</ResourceUID>\n",
            resource_uid.get(&assignment.resource_id).copied().unwrap_or(0)
        ));
        out.push_str(&format!("<Units>{}</Units>\n", assignment.units));
        out.push_str(&format!(
            "<Work>PT{}H</Work>\n",
            assignment.work_hours
        ));
        out.push_str(&format!("<Start>{}</Start>\n", ms_date(assignment.start_date)));
        out.push_str(&format!("<Finish>{}</Finish>\n", ms_date(assignment.end_date)));
        out.push_str(&format!("<Cost>{}</Cost>\n", assignment.cost));
        out.push_str("</Assignment>\n");
    }
    out.push_str("</Assignments>\n");

    out.push_str("</Project>\n");
    out
}

/// CSV export matching [`crate::import`]'s CSV importer headers.
pub fn export_csv(project: &Project, tasks: &[ProjectTask]) -> String {
    let mut out = String::from("name,start,end,duration,progress\n");
    for task in tasks {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            csv_escape(&task.name),
            task.start_date,
            task.end_date,
            task.duration_days,
            task.percent_complete,
        ));
    }
    let _ = project;
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// JSON export matching [`crate::import`]'s JSON importer shape.
pub fn export_json(project: &Project, tasks: &[ProjectTask]) -> String {
    #[derive(serde::Serialize)]
    struct JsonProject<'a> {
        name: &'a str,
        description: Option<&'a String>,
        start_date: NaiveDate,
        tasks: Vec<JsonTask<'a>>,
    }
    #[derive(serde::Serialize)]
    struct JsonTask<'a> {
        name: &'a str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        duration: u32,
        progress: u8,
    }

    let payload = JsonProject {
        name: &project.name,
        description: project.description.as_ref(),
        start_date: project.start_date,
        tasks: tasks
            .iter()
            .map(|t| JsonTask {
                name: &t.name,
                start_date: t.start_date,
                end_date: t.end_date,
                duration: t.duration_days,
                progress: t.percent_complete,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}
