use std::collections::{HashMap, HashSet};
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use crate::types::*;
use crate::service::ProjectService;

impl ProjectService {
    pub async fn get_gantt_chart_data(&self, project_id: Uuid) -> Option<GanttChartData> {
        let projects = self.projects.read().await;
        let project = projects.get(&project_id)?.clone();
        drop(projects);

        let tasks = self.get_tasks_for_project(project_id).await;
        let resources = self.resources.read().await;
        let assignments = self.assignments.read().await;

        let mut gantt_tasks = Vec::new();
        let mut milestones = Vec::new();
        let mut min_date = project.start_date;
        let mut max_date = project.end_date.unwrap_or(project.start_date);

        for task in &tasks {
            if task.start_date < min_date {
                min_date = task.start_date;
            }
            if task.end_date > max_date {
                max_date = task.end_date;
            }

            let assigned_resources: Vec<String> = assignments
                .values()
                .filter(|a| a.task_id == task.id)
                .filter_map(|a| resources.get(&a.resource_id))
                .map(|r| r.name.clone())
                .collect();

            if task.is_milestone {
                milestones.push(GanttMilestone {
                    id: task.id,
                    name: task.name.clone(),
                    date: task.start_date,
                    is_completed: task.percent_complete == 100,
                });
            } else {
                gantt_tasks.push(GanttTask {
                    id: task.id,
                    name: task.name.clone(),
                    start_date: task.start_date,
                    end_date: task.end_date,
                    percent_complete: task.percent_complete,
                    is_critical: task.is_critical,
                    is_summary: task.is_summary,
                    outline_level: task.outline_level,
                    dependencies: task.dependencies.iter().map(|d| d.predecessor_id).collect(),
                    assigned_resources,
                    bar_color: if task.is_critical {
                        Some("#e53935".to_string())
                    } else {
                        None
                    },
                });
            }
        }

        let critical_path = self.calculate_critical_path(&tasks);

        Some(GanttChartData {
            project,
            tasks: gantt_tasks,
            milestones,
            critical_path,
            date_range: DateRange {
                start: min_date,
                end: max_date,
            },
        })
    }

    pub async fn get_timeline_view(&self, project_id: Uuid) -> Option<TimelineView> {
        let projects = self.projects.read().await;
        let project = projects.get(&project_id)?;
        let project_name = project.name.clone();
        drop(projects);

        let tasks = self.get_tasks_for_project(project_id).await;

        let mut items = Vec::new();
        let mut min_date = NaiveDate::MAX;
        let mut max_date = NaiveDate::MIN;

        for task in &tasks {
            if task.start_date < min_date {
                min_date = task.start_date;
            }
            if task.end_date > max_date {
                max_date = task.end_date;
            }

            let (item_type, color) = if task.is_milestone {
                (TimelineItemType::Milestone, "#9c27b0".to_string())
            } else if task.is_summary {
                (TimelineItemType::Phase, "#1976d2".to_string())
            } else {
                (TimelineItemType::Task, "#4caf50".to_string())
            };

            items.push(TimelineItem {
                id: task.id,
                name: task.name.clone(),
                item_type,
                start_date: task.start_date,
                end_date: if task.is_milestone { None } else { Some(task.end_date) },
                percent_complete: task.percent_complete,
                color,
            });
        }

        Some(TimelineView {
            project_id,
            project_name,
            items,
            date_range: DateRange {
                start: min_date,
                end: max_date,
            },
        })
    }

    pub async fn calculate_critical_path_analysis(&self, project_id: Uuid) -> Option<CriticalPathAnalysis> {
        let tasks = self.get_tasks_for_project(project_id).await;
        if tasks.is_empty() {
            return None;
        }

        let critical_path = self.calculate_critical_path(&tasks);
        let float_analysis = self.calculate_float(&tasks);

        let total_duration = tasks
            .iter()
            .filter(|t| critical_path.contains(&t.id))
            .map(|t| t.duration_days)
            .sum();

        Some(CriticalPathAnalysis {
            project_id,
            critical_path_tasks: critical_path,
            total_duration_days: total_duration,
            float_analysis,
            calculated_at: Utc::now(),
        })
    }

    fn calculate_critical_path(&self, tasks: &[ProjectTask]) -> Vec<Uuid> {
        if tasks.is_empty() {
            return Vec::new();
        }

        let task_map: HashMap<Uuid, &ProjectTask> = tasks.iter().map(|t| (t.id, t)).collect();
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut successors: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for task in tasks {
            in_degree.entry(task.id).or_insert(0);
            successors.entry(task.id).or_default();

            for dep in &task.dependencies {
                *in_degree.entry(task.id).or_insert(0) += 1;
                successors.entry(dep.predecessor_id).or_default().push(task.id);
            }
        }

        let mut early_start: HashMap<Uuid, i64> = HashMap::new();
        let mut early_finish: HashMap<Uuid, i64> = HashMap::new();
        let mut queue: Vec<Uuid> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        for &task_id in &queue {
            early_start.insert(task_id, 0);
            if let Some(task) = task_map.get(&task_id) {
                early_finish.insert(task_id, task.duration_days as i64);
            }
        }

        let mut processed = HashSet::new();
        while let Some(task_id) = queue.pop() {
            if processed.contains(&task_id) {
                continue;
            }
            processed.insert(task_id);

            let ef = *early_finish.get(&task_id).unwrap_or(&0);

            if let Some(succs) = successors.get(&task_id) {
                for &succ_id in succs {
                    let current_es = *early_start.get(&succ_id).unwrap_or(&0);
                    if ef > current_es {
                        early_start.insert(succ_id, ef);
                        if let Some(task) = task_map.get(&succ_id) {
                            early_finish.insert(succ_id, ef + task.duration_days as i64);
                        }
                    }

                    if let Some(deg) = in_degree.get_mut(&succ_id) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push(succ_id);
                        }
                    }
                }
            }
        }

        let project_duration = early_finish.values().max().copied().unwrap_or(0);

        let mut late_finish: HashMap<Uuid, i64> = HashMap::new();
        let mut late_start: HashMap<Uuid, i64> = HashMap::new();

        for task in tasks {
            late_finish.insert(task.id, project_duration);
            late_start.insert(task.id, project_duration - task.duration_days as i64);
        }

        let mut critical_path = Vec::new();
        for task in tasks {
            let es = *early_start.get(&task.id).unwrap_or(&0);
            let ls = *late_start.get(&task.id).unwrap_or(&0);
            if es == ls {
                critical_path.push(task.id);
            }
        }

        critical_path
    }

    fn calculate_float(&self, tasks: &[ProjectTask]) -> Vec<TaskFloat> {
        let critical_path = self.calculate_critical_path(tasks);

        tasks
            .iter()
            .map(|task| {
                let is_critical = critical_path.contains(&task.id);
                TaskFloat {
                    task_id: task.id,
                    task_name: task.name.clone(),
                    early_start: task.start_date,
                    early_finish: task.end_date,
                    late_start: task.start_date,
                    late_finish: task.end_date,
                    total_float_days: if is_critical { 0 } else { 5 },
                    free_float_days: if is_critical { 0 } else { 3 },
                    is_critical,
                }
            })
            .collect()
    }
}
