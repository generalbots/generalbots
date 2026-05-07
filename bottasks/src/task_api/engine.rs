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
        task_bot_id: Uuid,
        task_title: &str,
        task_description: Option<&str>,
        task_user_id: Option<Uuid>,
    ) -> Result<Task, String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        let new_task = NewTask {
            id: Uuid::new_v4(),
            bot_id: task_bot_id,
            title: task_title.to_string(),
            description: task_description.map(|s| s.to_string()),
            status: "pending".to_string(),
            user_id: task_user_id,
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

    pub fn list_tasks(&self, filter_bot_id: Option<Uuid>) -> Result<Vec<Task>, String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        let query = tasks.into_boxed();
        match filter_bot_id {
            Some(bid) => query
                .filter(bot_id.eq(bid))
                .load::<Task>(&mut conn)
                .map_err(|e| format!("Query error: {}", e)),
            None => query
                .load::<Task>(&mut conn)
                .map_err(|e| format!("Query error: {}", e)),
        }
    }

    pub fn get_task(&self, task_id: Uuid) -> Result<Task, String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        tasks
            .find(task_id)
            .first::<Task>(&mut conn)
            .map_err(|e| format!("Query error: {}", e))
    }

    pub fn update_task_status(&self, task_id: Uuid, new_status: &str) -> Result<(), String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        diesel::update(tasks.find(task_id))
            .set(status.eq(new_status))
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {}", e))?;

        Ok(())
    }

    pub fn delete_task(&self, task_id: Uuid) -> Result<(), String> {
        use crate::schema::tasks::dsl::*;

        let mut conn = self
            .state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;

        diesel::delete(tasks.find(task_id))
            .execute(&mut conn)
            .map_err(|e| format!("Delete error: {}", e))?;

        Ok(())
    }

    pub fn get_bot_id_for_task(&self, task_id: Uuid) -> Result<Uuid, String> {
        let task = self.get_task(task_id)?;
        Ok(task.bot_id)
    }
}
