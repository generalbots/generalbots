use std::sync::Arc;

use log::{error, info};
use uuid::Uuid;

use crate::state::TasksState;
use crate::types::TaskManifest;

pub async fn execute_task(state: &Arc<TasksState>, task_id: Uuid, manifest: &mut TaskManifest) -> Result<(), String> {
    manifest.status = "running".to_string();
    manifest.current_status.title = "Executing task".to_string();
    manifest.add_terminal_output("Task execution started", Some("info"));

    let item_names: Vec<String> = manifest.items.iter().map(|i| i.name.clone()).collect();
    let step_names: Vec<Vec<String>> = manifest.items.iter().map(|i| i.steps.iter().map(|s| s.name.clone()).collect()).collect();

    for (item_idx, item_name) in item_names.iter().enumerate() {
        info!("Processing item: {}", item_name);

        for step_name in &step_names[item_idx] {
            manifest.current_status.title = format!("Running: {} - {}", item_name, step_name);
            manifest.add_terminal_output(
                &format!("Executing step: {}", step_name),
                Some("info"),
            );

            match (state.run_command)("echo", &[&format!("step: {}", step_name)]) {
                Ok(output) => {
                    manifest.complete_step(item_name, step_name);
                    manifest.add_terminal_output(&output, Some("output"));
                    info!("Step {} completed", step_name);
                }
                Err(e) => {
                    manifest.fail_step(item_name, step_name, &e);
                    manifest.add_terminal_output(&format!("Error: {}", e), Some("error"));
                    error!("Step {} failed: {}", step_name, e);
                    return Err(format!("Step {} failed: {}", step_name, e));
                }
            }

            if let Ok(tokens_str) = (state.get_config)("default_tokens") {
                if let Ok(tokens) = tokens_str.parse::<u64>() {
                    manifest.add_tokens(tokens, tokens as f64 * 0.00001);
                }
            }
        }
    }

    manifest.status = "completed".to_string();
    manifest.current_status.title = "Task completed".to_string();
    manifest.add_terminal_output("Task execution completed", Some("info"));
    info!("Task {} completed successfully", task_id);

    Ok(())
}

pub async fn cancel_task(_state: &Arc<TasksState>, task_id: Uuid, manifest: &mut TaskManifest) -> Result<(), String> {
    manifest.status = "cancelled".to_string();
    manifest.current_status.title = "Task cancelled".to_string();
    manifest.add_terminal_output("Task execution cancelled by user", Some("warning"));
    info!("Task {} cancelled", task_id);
    Ok(())
}

pub async fn execute_step_with_llm(
    state: &Arc<TasksState>,
    prompt: &str,
    manifest: &mut TaskManifest,
) -> Result<String, String> {
    let system = "You are a helpful assistant executing a task step. Provide a concise response.";
    let result = (state.call_llm)(system, prompt).await.map_err(|e| {
        manifest.add_terminal_output(&format!("LLM error: {}", e), Some("error"));
        e
    })?;

    let truncated: String = result.chars().take(200).collect();
    manifest.add_terminal_output(&format!("LLM response: {}", truncated), Some("output"));
    manifest.add_tokens(100, 0.001);

    Ok(result)
}
