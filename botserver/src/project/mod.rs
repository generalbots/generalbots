use axum::Router;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub use botproject::{
    Project, ProjectStatus, ProjectSettings, Weekday, ProjectTask, TaskType, TaskStatus,
    TaskPriority, TaskDependency, DependencyType, Resource, ResourceType, ResourceAssignment,
    GanttChartData, GanttTask, GanttMilestone, DateRange, TimelineView, TimelineItem,
    TimelineItemType, CriticalPathAnalysis, TaskFloat,
    ProjectService, CreateProjectRequest, CreateTaskRequest, UpdateProgressRequest,
    AddDependencyRequest,
};

pub mod import {
    pub use botproject::import::*;
}

pub fn configure() -> Router<()> {
    let service = Arc::new(botproject::ProjectService::new());
    botproject::configure(Arc::clone(&service)).with_state(service)
}
