use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

use crate::types::*;

pub struct ProjectService {
    pub(crate) projects: Arc<RwLock<HashMap<Uuid, Project>>>,
    pub(crate) tasks: Arc<RwLock<HashMap<Uuid, ProjectTask>>>,
    pub(crate) resources: Arc<RwLock<HashMap<Uuid, Resource>>>,
    pub(crate) assignments: Arc<RwLock<HashMap<Uuid, ResourceAssignment>>>,
}

impl ProjectService {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_project(&self, project: Project) -> Project {
        let mut projects = self.projects.write().await;
        projects.insert(project.id, project.clone());
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
        let mut projects = self.projects.write().await;
        if projects.contains_key(&project.id) {
            projects.insert(project.id, project.clone());
            Some(project)
        } else {
            None
        }
    }

    pub async fn delete_project(&self, project_id: Uuid) -> bool {
        let mut projects = self.projects.write().await;
        let mut tasks = self.tasks.write().await;
        let mut resources = self.resources.write().await;

        tasks.retain(|_, t| t.project_id != project_id);
        resources.retain(|_, r| r.project_id != project_id);
        projects.remove(&project_id).is_some()
    }

    pub async fn create_task(&self, task: ProjectTask) -> ProjectTask {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id, task.clone());
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
        let mut tasks = self.tasks.write().await;
        if tasks.contains_key(&task.id) {
            tasks.insert(task.id, task.clone());
            Some(task)
        } else {
            None
        }
    }

    pub async fn delete_task(&self, task_id: Uuid) -> bool {
        let mut tasks = self.tasks.write().await;
        let mut assignments = self.assignments.write().await;

        assignments.retain(|_, a| a.task_id != task_id);
        tasks.remove(&task_id).is_some()
    }

    pub async fn add_dependency(
        &self,
        task_id: Uuid,
        predecessor_id: Uuid,
        dependency_type: DependencyType,
        lag_days: i32,
    ) -> Option<ProjectTask> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            let dependency = TaskDependency {
                predecessor_id,
                dependency_type,
                lag_days,
            };
            task.dependencies.push(dependency);
            task.updated_at = Utc::now();
            return Some(task.clone());
        }
        None
    }

    pub async fn remove_dependency(&self, task_id: Uuid, predecessor_id: Uuid) -> Option<ProjectTask> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.dependencies.retain(|d| d.predecessor_id != predecessor_id);
            task.updated_at = Utc::now();
            return Some(task.clone());
        }
        None
    }

    pub async fn create_resource(&self, resource: Resource) -> Resource {
        let mut resources = self.resources.write().await;
        resources.insert(resource.id, resource.clone());
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
        let mut resources = self.resources.write().await;
        let mut assignments = self.assignments.write().await;
        assignments.retain(|_, a| a.resource_id != resource_id);
        resources.remove(&resource_id).is_some()
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
        let tasks = self.tasks.read().await;
        let resources = self.resources.read().await;

        let task = tasks.get(&task_id)?;
        let resource = resources.get(&resource_id)?;

        let cost = work_hours * resource.standard_rate.unwrap_or(0.0) as f32;

        let assignment = ResourceAssignment {
            id: Uuid::new_v4(),
            task_id,
            resource_id,
            units,
            work_hours,
            start_date: task.start_date,
            end_date: task.end_date,
            cost: cost as f64,
        };

        drop(tasks);
        drop(resources);

        let mut assignments = self.assignments.write().await;
        assignments.insert(assignment.id, assignment.clone());
        Some(assignment)
    }

    pub async fn update_task_progress(&self, task_id: Uuid, percent_complete: u8) -> Option<ProjectTask> {
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
            return Some(task.clone());
        }
        None
    }
}

impl Default for ProjectService {
    fn default() -> Self {
        Self::new()
    }
}
