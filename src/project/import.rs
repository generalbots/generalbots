use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use uuid::Uuid;

use super::{
    DependencyType, Project, ProjectSettings, ProjectStatus, ProjectTask, Resource,
    ResourceAssignment, ResourceType, TaskDependency, TaskPriority, TaskStatus, TaskType,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportFormat {
    MsProjectXml,
    MsProjectMpp,
    Csv,
    Json,
    Jira,
    Asana,
    Trello,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub format: ImportFormat,
    pub organization_id: Uuid,
    pub owner_id: Uuid,
    pub import_resources: bool,
    pub import_assignments: bool,
    pub import_dependencies: bool,
    pub import_custom_fields: bool,
    pub map_users: HashMap<String, Uuid>,
    pub default_resource_rate: f64,
    pub preserve_task_ids: bool,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    Skip,
    Overwrite,
    CreateNew,
    Merge,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            format: ImportFormat::MsProjectXml,
            organization_id: Uuid::nil(),
            owner_id: Uuid::nil(),
            import_resources: true,
            import_assignments: true,
            import_dependencies: true,
            import_custom_fields: false,
            map_users: HashMap::new(),
            default_resource_rate: 50.0,
            preserve_task_ids: false,
            conflict_resolution: ConflictResolution::CreateNew,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub project: Project,
    pub tasks: Vec<ProjectTask>,
    pub resources: Vec<Resource>,
    pub assignments: Vec<ResourceAssignment>,
    pub warnings: Vec<ImportWarning>,
    pub errors: Vec<ImportError>,
    pub stats: ImportStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStats {
    pub tasks_imported: u32,
    pub tasks_skipped: u32,
    pub resources_imported: u32,
    pub dependencies_imported: u32,
    pub assignments_imported: u32,
    pub custom_fields_imported: u32,
    pub import_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWarning {
    pub code: String,
    pub message: String,
    pub source_element: Option<String>,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub code: String,
    pub message: String,
    pub source_element: Option<String>,
    pub fatal: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct MsProjectXml {
    #[serde(rename = "Name", default)]
    name: Option<String>,
    #[serde(rename = "Title", default)]
    title: Option<String>,
    #[serde(rename = "StartDate", default)]
    start_date: Option<String>,
    #[serde(rename = "FinishDate", default)]
    finish_date: Option<String>,
    #[serde(rename = "Tasks", default)]
    tasks: Option<MsProjectTasks>,
    #[serde(rename = "Resources", default)]
    resources: Option<MsProjectResources>,
    #[serde(rename = "Assignments", default)]
    assignments: Option<MsProjectAssignments>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct MsProjectTasks {
    #[serde(rename = "Task", default)]
    task: Vec<MsProjectTask>,
}

#[derive(Debug, Clone, Deserialize)]
struct MsProjectTask {
    #[serde(rename = "UID", default)]
    uid: i32,
    #[serde(rename = "Name", default)]
    name: Option<String>,
    #[serde(rename = "IsNull", default)]
    is_null: Option<bool>,
    #[serde(rename = "WBS", default)]
    wbs: Option<String>,
    #[serde(rename = "OutlineLevel", default)]
    outline_level: Option<i32>,
    #[serde(rename = "Priority", default)]
    priority: Option<i32>,
    #[serde(rename = "Start", default)]
    start: Option<String>,
    #[serde(rename = "Finish", default)]
    finish: Option<String>,
    #[serde(rename = "Duration", default)]
    duration: Option<String>,
    #[serde(rename = "Work", default)]
    work: Option<String>,
    #[serde(rename = "PercentComplete", default)]
    percent_complete: Option<i32>,
    #[serde(rename = "Cost", default)]
    cost: Option<f64>,
    #[serde(rename = "Milestone", default)]
    milestone: Option<bool>,
    #[serde(rename = "Summary", default)]
    summary: Option<bool>,
    #[serde(rename = "Critical", default)]
    critical: Option<bool>,
    #[serde(rename = "Notes", default)]
    notes: Option<String>,
    #[serde(rename = "PredecessorLink", default)]
    predecessor_links: Vec<MsPredecessorLink>,
}

#[derive(Debug, Clone, Deserialize)]
struct MsPredecessorLink {
    #[serde(rename = "PredecessorUID", default)]
    predecessor_uid: i32,
    #[serde(rename = "Type", default)]
    link_type: Option<i32>,
    #[serde(rename = "LinkLag", default)]
    link_lag: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct MsProjectResources {
    #[serde(rename = "Resource", default)]
    resource: Vec<MsProjectResource>,
}

#[derive(Debug, Clone, Deserialize)]
struct MsProjectResource {
    #[serde(rename = "UID", default)]
    uid: i32,
    #[serde(rename = "Name", default)]
    name: Option<String>,
    #[serde(rename = "Type", default)]
    resource_type: Option<i32>,
    #[serde(rename = "IsNull", default)]
    is_null: Option<bool>,
    #[serde(rename = "MaxUnits", default)]
    max_units: Option<f64>,
    #[serde(rename = "StandardRate", default)]
    standard_rate: Option<f64>,
    #[serde(rename = "OvertimeRate", default)]
    overtime_rate: Option<f64>,
    #[serde(rename = "CostPerUse", default)]
    cost_per_use: Option<f64>,
    #[serde(rename = "EmailAddress", default)]
    email_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct MsProjectAssignments {
    #[serde(rename = "Assignment", default)]
    assignment: Vec<MsProjectAssignment>,
}

#[derive(Debug, Clone, Deserialize)]
struct MsProjectAssignment {
    #[serde(rename = "UID", default)]
    uid: i32,
    #[serde(rename = "TaskUID", default)]
    task_uid: i32,
    #[serde(rename = "ResourceUID", default)]
    resource_uid: i32,
    #[serde(rename = "Units", default)]
    units: Option<f64>,
    #[serde(rename = "Work", default)]
    work: Option<String>,
    #[serde(rename = "Start", default)]
    start: Option<String>,
    #[serde(rename = "Finish", default)]
    finish: Option<String>,
    #[serde(rename = "Cost", default)]
    cost: Option<f64>,
}

pub struct ProjectImportService;

fn parse_ms_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .ok()
}

fn parse_ms_duration(duration: &Option<String>) -> Option<u32> {
    duration.as_ref().and_then(|d| {
        if d.starts_with("PT") {
            let hours_str = d.trim_start_matches("PT").trim_end_matches('H');
            hours_str.parse::<f64>().ok().map(|h| (h / 8.0).ceil() as u32)
        } else {
            Some(1)
        }
    })
}

fn parse_ms_work(work: &Option<String>) -> Option<f32> {
    work.as_ref().and_then(|w| {
        if w.starts_with("PT") {
            let hours_str = w.trim_start_matches("PT").trim_end_matches('H');
            hours_str.parse::<f32>().ok()
        } else {
            None
        }
    })
}

impl ProjectImportService {
    pub fn new() -> Self {
        Self
    }

    pub fn import<R: Read>(
        &self,
        reader: R,
        options: ImportOptions,
    ) -> Result<ImportResult, String> {
        let start_time = std::time::Instant::now();

        let result = match options.format {
            ImportFormat::MsProjectXml => self.import_ms_project_xml(reader, &options),
            ImportFormat::MsProjectMpp => self.import_ms_project_mpp(reader, &options),
            ImportFormat::Csv => self.import_csv(reader, &options),
            ImportFormat::Json => self.import_json(reader, &options),
            ImportFormat::Jira => self.import_generic_json(reader, &options, "Jira"),
            ImportFormat::Asana => self.import_generic_json(reader, &options, "Asana"),
            ImportFormat::Trello => self.import_generic_json(reader, &options, "Trello"),
        };

        result.map(|mut r| {
            r.stats.import_duration_ms = start_time.elapsed().as_millis() as u64;
            r
        })
    }

    fn import_generic_json<R: Read>(
        &self,
        mut reader: R,
        options: &ImportOptions,
        source_name: &str,
    ) -> Result<ImportResult, String> {
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|e| format!("Failed to read {source_name} content: {e}"))?;

        let project = Project {
            id: Uuid::new_v4(),
            organization_id: options.organization_id,
            name: format!("Imported {source_name} Project"),
            description: Some(format!("{source_name} import - manual task mapping may be required")),
            start_date: Utc::now().date_naive(),
            end_date: None,
            status: ProjectStatus::Planning,
            owner_id: options.owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: ProjectSettings::default(),
        };

        Ok(ImportResult {
            project,
            tasks: Vec::new(),
            resources: Vec::new(),
            assignments: Vec::new(),
            warnings: vec![ImportWarning {
                code: format!("{}_BASIC_IMPORT", source_name.to_uppercase()),
                message: format!("{source_name} import creates a basic project structure. Tasks may need manual adjustment."),
                source_element: None,
                suggested_action: Some("Review and adjust imported tasks as needed".to_string()),
            }],
            errors: Vec::new(),
            stats: ImportStats {
                tasks_imported: 0,
                tasks_skipped: 0,
                resources_imported: 0,
                dependencies_imported: 0,
                assignments_imported: 0,
                custom_fields_imported: 0,
                import_duration_ms: 0,
            },
        })
    }

    fn resolve_task_hierarchy(&self, tasks: &mut [ProjectTask]) {
        let mut parent_map: HashMap<u32, Uuid> = HashMap::new();

        for task in tasks.iter() {
            parent_map.insert(task.outline_level, task.id);
        }

        for task in tasks.iter_mut() {
            if task.outline_level > 1 {
                if let Some(parent_id) = parent_map.get(&(task.outline_level - 1)) {
                    task.parent_id = Some(*parent_id);
                }
            }
        }
    }

    fn import_ms_project_xml<R: Read>(
        &self,
        mut reader: R,
        options: &ImportOptions,
    ) -> Result<ImportResult, String> {
        let mut xml_content = String::new();
        reader
            .read_to_string(&mut xml_content)
            .map_err(|e| format!("Failed to read XML content: {e}"))?;

        let ms_project: MsProjectXml = quick_xml::de::from_str(&xml_content)
            .map_err(|e| format!("Failed to parse MS Project XML: {e}"))?;

        let mut warnings = Vec::new();
        let errors = Vec::new();
        let mut stats = ImportStats {
            tasks_imported: 0,
            tasks_skipped: 0,
            resources_imported: 0,
            dependencies_imported: 0,
            assignments_imported: 0,
            custom_fields_imported: 0,
            import_duration_ms: 0,
        };

        let project_name = ms_project
            .title
            .or(ms_project.name)
            .unwrap_or_else(|| "Imported Project".to_string());

        let start_date = ms_project
            .start_date
            .as_ref()
            .and_then(|s| parse_ms_date(s))
            .unwrap_or_else(|| Utc::now().date_naive());

        let end_date = ms_project
            .finish_date
            .as_ref()
            .and_then(|s| parse_ms_date(s));

        let project = Project {
            id: Uuid::new_v4(),
            organization_id: options.organization_id,
            name: project_name,
            description: None,
            start_date,
            end_date,
            status: ProjectStatus::Planning,
            owner_id: options.owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: ProjectSettings::default(),
        };

        let mut tasks = Vec::new();
        let mut task_uid_map: HashMap<i32, Uuid> = HashMap::new();

        if let Some(ms_tasks) = &ms_project.tasks {
            for ms_task in &ms_tasks.task {
                if ms_task.is_null.unwrap_or(false) {
                    continue;
                }

                if ms_task.name.is_none() || ms_task.name.as_ref().map(|n| n.is_empty()).unwrap_or(true) {
                    stats.tasks_skipped += 1;
                    continue;
                }

                let task_id = Uuid::new_v4();
                task_uid_map.insert(ms_task.uid, task_id);

                let task_start = ms_task
                    .start
                    .as_ref()
                    .and_then(|s| parse_ms_date(s))
                    .unwrap_or(start_date);

                let task_end = ms_task
                    .finish
                    .as_ref()
                    .and_then(|s| parse_ms_date(s))
                    .unwrap_or(task_start);

                let duration_days = parse_ms_duration(&ms_task.duration).unwrap_or(1);

                let priority = match ms_task.priority.unwrap_or(500) {
                    0..=200 => TaskPriority::Low,
                    201..=400 => TaskPriority::Normal,
                    401..=700 => TaskPriority::High,
                    _ => TaskPriority::Critical,
                };

                let task_type = if ms_task.milestone.unwrap_or(false) {
                    TaskType::Milestone
                } else if ms_task.summary.unwrap_or(false) {
                    TaskType::Summary
                } else {
                    TaskType::Task
                };

                let task = ProjectTask {
                    id: task_id,
                    project_id: project.id,
                    parent_id: None,
                    name: ms_task.name.clone().unwrap_or_default(),
                    description: None,
                    task_type,
                    start_date: task_start,
                    end_date: task_end,
                    duration_days,
                    percent_complete: ms_task.percent_complete.unwrap_or(0) as u8,
                    status: if ms_task.percent_complete.unwrap_or(0) >= 100 {
                        TaskStatus::Completed
                    } else if ms_task.percent_complete.unwrap_or(0) > 0 {
                        TaskStatus::InProgress
                    } else {
                        TaskStatus::NotStarted
                    },
                    priority,
                    assigned_to: Vec::new(),
                    dependencies: Vec::new(),
                    estimated_hours: parse_ms_work(&ms_task.work),
                    actual_hours: None,
                    cost: ms_task.cost,
                    notes: ms_task.notes.clone(),
                    wbs: ms_task.wbs.clone().unwrap_or_default(),
                    outline_level: ms_task.outline_level.unwrap_or(1) as u32,
                    is_milestone: ms_task.milestone.unwrap_or(false),
                    is_summary: ms_task.summary.unwrap_or(false),
                    is_critical: ms_task.critical.unwrap_or(false),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                tasks.push(task);
                stats.tasks_imported += 1;
            }
        }

        if options.import_dependencies {
            if let Some(ms_tasks) = &ms_project.tasks {
                for ms_task in &ms_tasks.task {
                    if let Some(task_id) = task_uid_map.get(&ms_task.uid) {
                        for pred_link in &ms_task.predecessor_links {
                            if let Some(pred_id) = task_uid_map.get(&pred_link.predecessor_uid) {
                                let dep_type = match pred_link.link_type.unwrap_or(1) {
                                    0 => DependencyType::FinishToFinish,
                                    1 => DependencyType::FinishToStart,
                                    2 => DependencyType::StartToFinish,
                                    3 => DependencyType::StartToStart,
                                    _ => DependencyType::FinishToStart,
                                };

                                let lag_days = pred_link.link_lag.unwrap_or(0) / 4800;

                                if let Some(task) = tasks.iter_mut().find(|t| t.id == *task_id) {
                                    task.dependencies.push(TaskDependency {
                                        predecessor_id: *pred_id,
                                        dependency_type: dep_type,
                                        lag_days,
                                    });
                                    stats.dependencies_imported += 1;
                                }
                            } else {
                                warnings.push(ImportWarning {
                                    code: "PRED_NOT_FOUND".to_string(),
                                    message: format!(
                                        "Predecessor task UID {} not found for task {}",
                                        pred_link.predecessor_uid, ms_task.uid
                                    ),
                                    source_element: Some(format!("Task UID {}", ms_task.uid)),
                                    suggested_action: Some("Dependency will be skipped".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }

        let mut resources = Vec::new();
        let mut resource_uid_map: HashMap<i32, Uuid> = HashMap::new();

        if options.import_resources {
            if let Some(ms_resources) = &ms_project.resources {
                for ms_resource in &ms_resources.resource {
                    if ms_resource.is_null.unwrap_or(false) {
                        continue;
                    }

                    if ms_resource.name.is_none()
                        || ms_resource.name.as_ref().map(|n| n.is_empty()).unwrap_or(true)
                    {
                        continue;
                    }

                    let resource_id = Uuid::new_v4();
                    resource_uid_map.insert(ms_resource.uid, resource_id);

                    let resource_type = match ms_resource.resource_type.unwrap_or(1) {
                        0 => ResourceType::Material,
                        1 => ResourceType::Work,
                        2 => ResourceType::Cost,
                        _ => ResourceType::Work,
                    };

                    let resource = Resource {
                        id: resource_id,
                        project_id: project.id,
                        user_id: options
                            .map_users
                            .get(ms_resource.name.as_ref().unwrap_or(&String::new()))
                            .copied(),
                        name: ms_resource.name.clone().unwrap_or_default(),
                        resource_type,
                        email: ms_resource.email_address.clone(),
                        max_units: ms_resource.max_units.unwrap_or(1.0) as f32,
                        standard_rate: Some(ms_resource.standard_rate.unwrap_or(options.default_resource_rate)),
                        overtime_rate: Some(ms_resource.overtime_rate.unwrap_or(0.0)),
                        cost_per_use: Some(ms_resource.cost_per_use.unwrap_or(0.0)),
                        calendar_id: None,
                        created_at: Utc::now(),
                    };

                    resources.push(resource);
                    stats.resources_imported += 1;
                }
            }
        }

        let mut assignments = Vec::new();

        if options.import_assignments {
            if let Some(ms_assignments) = &ms_project.assignments {
                for ms_assignment in &ms_assignments.assignment {
                    let task_id = task_uid_map.get(&ms_assignment.task_uid);
                    let resource_id = resource_uid_map.get(&ms_assignment.resource_uid);

                    match (task_id, resource_id) {
                        (Some(tid), Some(rid)) => {
                            let assignment_start = ms_assignment
                                .start
                                .as_ref()
                                .and_then(|s| parse_ms_date(s))
                                .unwrap_or(start_date);

                            let assignment_end = ms_assignment
                                .finish
                                .as_ref()
                                .and_then(|s| parse_ms_date(s))
                                .unwrap_or(assignment_start);

                            let work_hours = parse_ms_work(&ms_assignment.work).unwrap_or(0.0);

                            let assignment = ResourceAssignment {
                                id: Uuid::new_v4(),
                                task_id: *tid,
                                resource_id: *rid,
                                units: ms_assignment.units.unwrap_or(1.0) as f32,
                                work_hours,
                                start_date: assignment_start,
                                end_date: assignment_end,
                                cost: ms_assignment.cost.unwrap_or(0.0),
                            };

                            if let Some(task) = tasks.iter_mut().find(|t| t.id == *tid) {
                                if !task.assigned_to.contains(rid) {
                                    task.assigned_to.push(*rid);
                                }
                            }

                            assignments.push(assignment);
                            stats.assignments_imported += 1;
                        }
                        _ => {
                            warnings.push(ImportWarning {
                                code: "ASSIGNMENT_REF_MISSING".to_string(),
                                message: format!(
                                    "Assignment {} references missing task or resource",
                                    ms_assignment.uid
                                ),
                                source_element: Some(format!("Assignment UID {}", ms_assignment.uid)),
                                suggested_action: Some("Assignment will be skipped".to_string()),
                            });
                        }
                    }
                }
            }
        }

        self.resolve_task_hierarchy(&mut tasks);

        Ok(ImportResult {
            project,
            tasks,
            resources,
            assignments,
            warnings,
            errors,
            stats,
        })
    }

    fn import_ms_project_mpp<R: Read>(
        &self,
        _reader: R,
        options: &ImportOptions,
    ) -> Result<ImportResult, String> {
        let project = Project {
            id: Uuid::new_v4(),
            organization_id: options.organization_id,
            name: "Imported MPP Project".to_string(),
            description: Some("MPP format requires conversion. Please export as XML from MS Project.".to_string()),
            start_date: Utc::now().date_naive(),
            end_date: None,
            status: ProjectStatus::Planning,
            owner_id: options.owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: ProjectSettings::default(),
        };

        Ok(ImportResult {
            project,
            tasks: Vec::new(),
            resources: Vec::new(),
            assignments: Vec::new(),
            warnings: vec![ImportWarning {
                code: "MPP_NOT_SUPPORTED".to_string(),
                message: "Native MPP format is not fully supported. Please export as XML from MS Project for best results.".to_string(),
                source_element: None,
                suggested_action: Some("Use File > Save As > XML in MS Project".to_string()),
            }],
            errors: Vec::new(),
            stats: ImportStats {
                tasks_imported: 0,
                tasks_skipped: 0,
                resources_imported: 0,
                dependencies_imported: 0,
                assignments_imported: 0,
                custom_fields_imported: 0,
                import_duration_ms: 0,
            },
        })
    }

    fn import_csv<R: Read>(
        &self,
        mut reader: R,
        options: &ImportOptions,
    ) -> Result<ImportResult, String> {
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|e| format!("Failed to read CSV content: {e}"))?;

        let mut tasks = Vec::new();
        let mut stats = ImportStats {
            tasks_imported: 0,
            tasks_skipped: 0,
            resources_imported: 0,
            dependencies_imported: 0,
            assignments_imported: 0,
            custom_fields_imported: 0,
            import_duration_ms: 0,
        };

        let project = Project {
            id: Uuid::new_v4(),
            organization_id: options.organization_id,
            name: "Imported CSV Project".to_string(),
            description: None,
            start_date: Utc::now().date_naive(),
            end_date: None,
            status: ProjectStatus::Planning,
            owner_id: options.owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: ProjectSettings::default(),
        };

        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Ok(ImportResult {
                project,
                tasks: Vec::new(),
                resources: Vec::new(),
                assignments: Vec::new(),
                warnings: vec![ImportWarning {
                    code: "EMPTY_CSV".to_string(),
                    message: "CSV file is empty".to_string(),
                    source_element: None,
                    suggested_action: None,
                }],
                errors: Vec::new(),
                stats,
            });
        }

        let headers: Vec<&str> = lines[0].split(',').map(|s| s.trim()).collect();

        let name_idx = headers.iter().position(|h| h.eq_ignore_ascii_case("name") || h.eq_ignore_ascii_case("task"));
        let start_idx = headers.iter().position(|h| h.eq_ignore_ascii_case("start") || h.eq_ignore_ascii_case("start_date"));
        let end_idx = headers.iter().position(|h| h.eq_ignore_ascii_case("end") || h.eq_ignore_ascii_case("finish") || h.eq_ignore_ascii_case("end_date"));
        let duration_idx = headers.iter().position(|h| h.eq_ignore_ascii_case("duration"));

        for line in lines.iter().skip(1) {
            let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

            let name = name_idx
                .and_then(|i| fields.get(i))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Task {}", tasks.len() + 1));

            if name.is_empty() {
                stats.tasks_skipped += 1;
                continue;
            }

            let start_date = start_idx
                .and_then(|i| fields.get(i))
                .and_then(|s| parse_date_flexible(s))
                .unwrap_or_else(|| Utc::now().date_naive());

            let end_date = end_idx
                .and_then(|i| fields.get(i))
                .and_then(|s| parse_date_flexible(s))
                .unwrap_or(start_date);

            let duration_days = duration_idx
                .and_then(|i| fields.get(i))
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);

            let task = ProjectTask {
                id: Uuid::new_v4(),
                project_id: project.id,
                parent_id: None,
                name,
                description: None,
                task_type: TaskType::Task,
                start_date,
                end_date,
                duration_days,
                percent_complete: 0,
                status: TaskStatus::NotStarted,
                priority: TaskPriority::Normal,
                assigned_to: Vec::new(),
                dependencies: Vec::new(),
                estimated_hours: None,
                actual_hours: None,
                cost: None,
                notes: None,
                wbs: format!("{}", tasks.len() + 1),
                outline_level: 1,
                is_milestone: false,
                is_summary: false,
                is_critical: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            tasks.push(task);
            stats.tasks_imported += 1;
        }

        Ok(ImportResult {
            project,
            tasks,
            resources: Vec::new(),
            assignments: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            stats,
        })
    }

    fn import_json<R: Read>(
        &self,
        mut reader: R,
        options: &ImportOptions,
    ) -> Result<ImportResult, String> {
        let start = std::time::Instant::now();
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|e| format!("Failed to read JSON content: {e}"))?;

        #[derive(Deserialize)]
        struct JsonProject {
            name: String,
            description: Option<String>,
            start_date: Option<String>,
            tasks: Option<Vec<JsonTask>>,
        }

        #[derive(Deserialize)]
        struct JsonTask {
            name: String,
            start_date: Option<String>,
            end_date: Option<String>,
            duration: Option<u32>,
            progress: Option<u8>,
        }

        let json_project: JsonProject = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;

        let project = Project {
            id: Uuid::new_v4(),
            organization_id: options.organization_id,
            name: json_project.name,
            description: json_project.description,
            start_date: json_project
                .start_date
                .as_ref()
                .and_then(|s| parse_date_flexible(s))
                .unwrap_or_else(|| Utc::now().date_naive()),
            end_date: None,
            status: ProjectStatus::Planning,
            owner_id: options.owner_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: ProjectSettings::default(),
        };

        let mut tasks = Vec::new();
        let mut stats = ImportStats {
            tasks_imported: 0,
            tasks_skipped: 0,
            resources_imported: 0,
            dependencies_imported: 0,
            assignments_imported: 0,
            custom_fields_imported: 0,
            import_duration_ms: 0,
        };

        if let Some(json_tasks) = json_project.tasks {
            for (idx, json_task) in json_tasks.iter().enumerate() {
                let start_date = json_task
                    .start_date
                    .as_ref()
                    .and_then(|s| parse_date_flexible(s))
                    .unwrap_or_else(|| Utc::now().date_naive());

                let end_date = json_task
                    .end_date
                    .as_ref()
                    .and_then(|s| parse_date_flexible(s))
                    .unwrap_or(start_date);

                let task = ProjectTask {
                    id: Uuid::new_v4(),
                    project_id: project.id,
                    parent_id: None,
                    name: json_task.name.clone(),
                    description: None,
                    task_type: TaskType::Task,
                    start_date,
                    end_date,
                    duration_days: json_task.duration.unwrap_or(1),
                    percent_complete: json_task.progress.unwrap_or(0),
                    status: TaskStatus::NotStarted,
                    priority: TaskPriority::Normal,
                    assigned_to: Vec::new(),
                    dependencies: Vec::new(),
                    estimated_hours: None,
                    actual_hours: None,
                    cost: None,
                    notes: None,
                    wbs: format!("{}", idx + 1),
                    outline_level: 1,
                    is_milestone: false,
                    is_summary: false,
                    is_critical: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                tasks.push(task);
                stats.tasks_imported += 1;
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        stats.import_duration_ms = duration;

        Ok(ImportResult {
            project,
            tasks,
            resources: Vec::new(),
            assignments: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            stats,
        })
    }
}

fn parse_date_flexible(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y"))
        .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%d/%m/%Y"))
        .ok()
}
