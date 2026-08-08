use std::sync::Arc;

use diesel::prelude::*;
use uuid::Uuid;

use crate::state::TasksState;
use crate::types::{NewTask, Task};

pub struct TaskEngine {
    state: Arc<TasksState>,
}

impl TaskEngine {
    pub fn new(state: Arc<TasksState>) -> Self {
        Self { state }
    }

    pub fn create_task(
        &self,
        task_branch_id: Uuid,
        task_title: &str,
        task_description: Option<&str>,
    ) -> Result<Task, String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        let new_task = NewTask {
            id: Uuid::new_v4(),
            branch_id: task_branch_id,
            title: task_title.to_string(),
            description: task_description.map(|s| s.to_string()),
            status: Some("pending".to_string()),
            priority: None,
            assignee_id: None,
            due_date: None,
            completed_at: None,
            parent_id: None,
        };

        diesel::insert_into(tasks)
            .values(&new_task)
            .execute(&mut conn)
            .map_err(|e| format!("Insert error: {}", e))?;

        tasks
            .find(new_task.id)
            .first::<Task>(&mut conn)
            .map_err(|e| format!("Fetch error: {}", e))
    }

    pub fn list_tasks(&self, filter_branch_id: Option<Uuid>) -> Result<Vec<Task>, String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        let query = tasks.into_boxed();
        match filter_branch_id {
            Some(bid) => query
                .filter(branch_id.eq(bid))
                .load::<Task>(&mut conn)
                .map_err(|e| format!("Query error: {}", e)),
            None => query
                .load::<Task>(&mut conn)
                .map_err(|e| format!("Query error: {}", e)),
        }
    }

    pub fn get_task(&self, task_id: Uuid, task_branch_id: Uuid) -> Result<Task, String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        tasks
            .filter(id.eq(task_id).and(branch_id.eq(task_branch_id)))
            .first::<Task>(&mut conn)
            .map_err(|e| format!("Query error: {}", e))
    }

    pub fn update_task_status(&self, task_id: Uuid, task_branch_id: Uuid, new_status: &str) -> Result<(), String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        diesel::update(tasks.filter(id.eq(task_id).and(branch_id.eq(task_branch_id))))
            .set(status.eq(Some(new_status.to_string())))
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {}", e))?;

        Ok(())
    }

    pub fn delete_task(&self, task_id: Uuid, task_branch_id: Uuid) -> Result<(), String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        diesel::delete(tasks.filter(id.eq(task_id).and(branch_id.eq(task_branch_id))))
            .execute(&mut conn)
            .map_err(|e| format!("Delete error: {}", e))?;

        Ok(())
    }
}
