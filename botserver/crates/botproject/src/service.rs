use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::types::*;

/// Postgres pool shared by every persistence operation. Kept as a concrete
/// diesel type (not botcore's alias) so this leaf crate stays decoupled.
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Full tenant snapshot: one org's projects and everything hanging off them.
/// Serialized to a single JSONB row keyed by org_id in `project_snapshots`.
#[derive(Serialize, Deserialize, Default)]
struct OrgSnapshot {
    projects: Vec<Project>,
    tasks: Vec<ProjectTask>,
    resources: Vec<Resource>,
    assignments: Vec<ResourceAssignment>,
}

pub struct ProjectService {
    pub(crate) projects: Arc<RwLock<HashMap<Uuid, Project>>>,
    pub(crate) tasks: Arc<RwLock<HashMap<Uuid, ProjectTask>>>,
    pub(crate) resources: Arc<RwLock<HashMap<Uuid, Resource>>>,
    pub(crate) assignments: Arc<RwLock<HashMap<Uuid, ResourceAssignment>>>,
    pool: Option<DbPool>,
}

impl ProjectService {
    /// In-memory only (used where no pool is available, e.g. the AppState
    /// placeholder). Prefer [`ProjectService::with_pool`] in the router.
    pub fn new() -> Self {
        Self::with_optional_pool(None)
    }

    /// DB-backed: state is written through to `project_snapshots` on every
    /// mutation and loaded back on [`ProjectService::load_from_db`].
    pub fn with_pool(pool: DbPool) -> Self {
        Self::with_optional_pool(Some(pool))
    }

    fn with_optional_pool(pool: Option<DbPool>) -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
            pool,
        }
    }

    /// Hydrate the in-memory store from any previously persisted snapshots.
    /// Missing rows / parse failures are logged and skipped, never fatal.
    pub async fn load_from_db(&self) {
        let Some(pool) = self.pool.clone() else { return; };

        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            payload: serde_json::Value,
        }

        let loaded = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            diesel::sql_query("SELECT payload FROM project_snapshots")
                .load::<Row>(&mut conn)
                .map_err(|e| e.to_string())
        })
        .await;

        let rows = match loaded {
            Ok(Ok(rows)) => rows,
            Ok(Err(e)) => {
                warn!("project load failed: {e}");
                return;
            }
            Err(e) => {
                warn!("project load join failed: {e}");
                return;
            }
        };

        for row in rows {
            let snap: OrgSnapshot = match serde_json::from_value(row.payload) {
                Ok(s) => s,
                Err(e) => {
                    warn!("project snapshot parse failed: {e}");
                    continue;
                }
            };
            {
                let mut projects = self.projects.write().await;
                for p in snap.projects {
                    projects.entry(p.id).or_insert(p);
                }
            }
            {
                let mut tasks = self.tasks.write().await;
                for t in snap.tasks {
                    tasks.entry(t.id).or_insert(t);
                }
            }
            {
                let mut resources = self.resources.write().await;
                for r in snap.resources {
                    resources.entry(r.id).or_insert(r);
                }
            }
            {
                let mut assignments = self.assignments.write().await;
                for a in snap.assignments {
                    assignments.entry(a.id).or_insert(a);
                }
            }
        }
    }

    /// Persist the whole in-memory store as one JSONB row per organization.
    /// Called after every mutation; failures are logged, never fatal (the
    /// in-memory store stays authoritative for the live UI).
    pub async fn persist_all(&self) {
        let Some(pool) = self.pool.clone() else { return; };

        let grouped: HashMap<Uuid, OrgSnapshot> = {
            let projects = self.projects.read().await;
            let tasks = self.tasks.read().await;
            let resources = self.resources.read().await;
            let assignments = self.assignments.read().await;

            let mut by_org: HashMap<Uuid, OrgSnapshot> = HashMap::new();
            let mut project_org: HashMap<Uuid, Uuid> = HashMap::new();
            for p in projects.values() {
                project_org.insert(p.id, p.organization_id);
                by_org
                    .entry(p.organization_id)
                    .or_default()
                    .projects
                    .push(p.clone());
            }
            let mut task_org: HashMap<Uuid, Uuid> = HashMap::new();
            for t in tasks.values() {
                if let Some(org) = project_org.get(&t.project_id) {
                    task_org.insert(t.id, *org);
                    by_org.entry(*org).or_default().tasks.push(t.clone());
                }
            }
            for r in resources.values() {
                if let Some(org) = project_org.get(&r.project_id) {
                    by_org.entry(*org).or_default().resources.push(r.clone());
                }
            }
            for a in assignments.values() {
                if let Some(org) = task_org.get(&a.task_id) {
                    by_org.entry(*org).or_default().assignments.push(a.clone());
                }
            }
            by_org
        };

        let rows: Vec<(Uuid, serde_json::Value)> = grouped
            .into_iter()
            .filter_map(|(org, snap)| serde_json::to_value(&snap).ok().map(|v| (org, v)))
            .collect();

        let res = tokio::task::spawn_blocking(move || {
            let mut conn = match pool.get() {
                Ok(c) => c,
                Err(e) => return Err(e.to_string()),
            };
            for (org, payload) in rows {
                diesel::sql_query(
                    "INSERT INTO project_snapshots (org_id, payload, updated_at) \
                     VALUES ($1, $2, NOW()) \
                     ON CONFLICT (org_id) DO UPDATE SET payload = EXCLUDED.payload, updated_at = NOW()",
                )
                .bind::<diesel::sql_types::Uuid, _>(org)
                .bind::<diesel::sql_types::Jsonb, _>(payload)
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;
            }
            Ok(())
        })
        .await;

        if let Err(e) = res {
            warn!("project persist_all failed: {e}");
        }
    }

    pub async fn create_project(&self, project: Project) -> Project {
        {
            let mut projects = self.projects.write().await;
            projects.insert(project.id, project.clone());
        }
        self.persist_all().await;
        project
    }

    pub async fn get_project(&self, project_id: Uuid) -> Option<Project> {
        let projects = self.projects.read().await;
        projects.get(&project_id).cloned()
    }

    pub async fn get_projects_for_organization(&self, org_id: Uuid) -> Vec<Project> {
        let projects = self.projects.read().await;
        projects
            .values()
            .filter(|p| p.organization_id == org_id)
            .cloned()
            .collect()
    }

    pub async fn get_all_projects(&self) -> Vec<Project> {
        let projects = self.projects.read().await;
        projects.values().cloned().collect()
    }

    pub async fn update_project(&self, project: Project) -> Option<Project> {
        let updated = {
            let mut projects = self.projects.write().await;
            if projects.contains_key(&project.id) {
                projects.insert(project.id, project.clone());
                Some(project)
            } else {
                None
            }
        };
        self.persist_all().await;
        updated
    }

    pub async fn delete_project(&self, project_id: Uuid) -> bool {
        let removed = {
            let mut projects = self.projects.write().await;
            let mut tasks = self.tasks.write().await;
            let mut resources = self.resources.write().await;

            tasks.retain(|_, t| t.project_id != project_id);
            resources.retain(|_, r| r.project_id != project_id);
            projects.remove(&project_id).is_some()
        };
        self.persist_all().await;
        removed
    }

    pub async fn create_task(&self, task: ProjectTask) -> ProjectTask {
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task.id, task.clone());
        }
        self.persist_all().await;
        task
    }

    pub async fn get_task(&self, task_id: Uuid) -> Option<ProjectTask> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).cloned()
    }

    pub async fn get_tasks_for_project(&self, project_id: Uuid) -> Vec<ProjectTask> {
        let tasks = self.tasks.read().await;
        let mut project_tasks: Vec<ProjectTask> = tasks
            .values()
            .filter(|t| t.project_id == project_id)
            .cloned()
            .collect();
        project_tasks.sort_by(|a, b| a.wbs.cmp(&b.wbs));
        project_tasks
    }

    pub async fn update_task(&self, task: ProjectTask) -> Option<ProjectTask> {
        let updated = {
            let mut tasks = self.tasks.write().await;
            if tasks.contains_key(&task.id) {
                tasks.insert(task.id, task.clone());
                Some(task)
            } else {
                None
            }
        };
        self.persist_all().await;
        updated
    }

    pub async fn delete_task(&self, task_id: Uuid) -> bool {
        let removed = {
            let mut tasks = self.tasks.write().await;
            let mut assignments = self.assignments.write().await;

            assignments.retain(|_, a| a.task_id != task_id);
            tasks.remove(&task_id).is_some()
        };
        self.persist_all().await;
        removed
    }

    pub async fn add_dependency(
        &self,
        task_id: Uuid,
        predecessor_id: Uuid,
        dependency_type: DependencyType,
        lag_days: i32,
    ) -> Option<ProjectTask> {
        let result = {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                let dependency = TaskDependency {
                    predecessor_id,
                    dependency_type,
                    lag_days,
                };
                task.dependencies.push(dependency);
                task.updated_at = Utc::now();
                Some(task.clone())
            } else {
                None
            }
        };
        self.persist_all().await;
        result
    }

    pub async fn remove_dependency(&self, task_id: Uuid, predecessor_id: Uuid) -> Option<ProjectTask> {
        let result = {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.dependencies.retain(|d| d.predecessor_id != predecessor_id);
                task.updated_at = Utc::now();
                Some(task.clone())
            } else {
                None
            }
        };
        self.persist_all().await;
        result
    }

    pub async fn create_resource(&self, resource: Resource) -> Resource {
        {
            let mut resources = self.resources.write().await;
            resources.insert(resource.id, resource.clone());
        }
        self.persist_all().await;
        resource
    }

    pub async fn get_resources_for_project(&self, project_id: Uuid) -> Vec<Resource> {
        let resources = self.resources.read().await;
        resources
            .values()
            .filter(|r| r.project_id == project_id)
            .cloned()
            .collect()
    }

    pub async fn delete_resource(&self, resource_id: Uuid) -> bool {
        let removed = {
            let mut resources = self.resources.write().await;
            let mut assignments = self.assignments.write().await;
            assignments.retain(|_, a| a.resource_id != resource_id);
            resources.remove(&resource_id).is_some()
        };
        self.persist_all().await;
        removed
    }

    pub async fn get_assignments_for_task(&self, task_id: Uuid) -> Vec<ResourceAssignment> {
        let assignments = self.assignments.read().await;
        assignments
            .values()
            .filter(|a| a.task_id == task_id)
            .cloned()
            .collect()
    }

    pub async fn assign_resource(
        &self,
        task_id: Uuid,
        resource_id: Uuid,
        units: f32,
        work_hours: f32,
    ) -> Option<ResourceAssignment> {
        let assignment = {
            let tasks = self.tasks.read().await;
            let resources = self.resources.read().await;

            let task = tasks.get(&task_id)?;
            let resource = resources.get(&resource_id)?;

            let cost = work_hours * resource.standard_rate.unwrap_or(0.0) as f32;

            ResourceAssignment {
                id: Uuid::new_v4(),
                task_id,
                resource_id,
                units,
                work_hours,
                start_date: task.start_date,
                end_date: task.end_date,
                cost: cost as f64,
            }
        };

        {
            let mut assignments = self.assignments.write().await;
            assignments.insert(assignment.id, assignment.clone());
        }
        self.persist_all().await;
        Some(assignment)
    }

    pub async fn update_task_progress(&self, task_id: Uuid, percent_complete: u8) -> Option<ProjectTask> {
        let result = {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.percent_complete = percent_complete.min(100);
                task.status = if percent_complete == 0 {
                    TaskStatus::NotStarted
                } else if percent_complete == 100 {
                    TaskStatus::Completed
                } else {
                    TaskStatus::InProgress
                };
                task.updated_at = Utc::now();
                Some(task.clone())
            } else {
                None
            }
        };
        self.persist_all().await;
        result
    }
}

impl Default for ProjectService {
    fn default() -> Self {
        Self::new()
    }
}
