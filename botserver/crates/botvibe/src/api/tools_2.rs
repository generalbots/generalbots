//! `api::tools_2` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) async fn list_capabilities_for_use_case(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(use_case): Path<String>,
) -> impl IntoResponse {
    let uc = parse_use_case(&use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let tools = api.tool_executor.registry().list_tools().await;
    let capabilities = crate::capability_registry::build_capabilities(&tools);
    let filtered = crate::capability_registry::capabilities_for(&capabilities, uc);
    Json(CapabilitiesResponse {
        success: true,
        capabilities: filtered,
        error: None,
    })
}

pub(crate) async fn list_tools_for_use_case(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(use_case): Path<String>,
) -> impl IntoResponse {
    let uc = parse_use_case(&use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let tools = api.tool_executor.registry().list_tools_for_use_case(uc).await;
    Json(ListToolsResponse { tools })
}

pub(crate) async fn execute_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let runs = api.runs.read().await;
    let run = match runs.get(&run_id) {
        Some(r) => r,
        None => {
            return Json(ActionResponse {
                success: false,
                message: None,
                error: Some("Run not found".to_string()),
            });
        }
    };

    let use_case = run.use_case;
    let tool_calls = run.tool_calls.clone();
    drop(runs);
    let state_clone = api.state.clone();

    for tool_call in &tool_calls {
        if !tool_call.approved && tool_call.requires_approval {
            return Json(ActionResponse {
                success: false,
                message: Some("Approval required".to_string()),
                error: None,
            });
        }

        let mut owned = tool_call.clone();
        let result = api
            .tool_executor
            .execute(&mut owned, use_case, state_clone.as_ref())
            .await;

        match result {
            Ok(_) => info!("Tool executed successfully"),
            Err(e) => {
                return Json(ActionResponse {
                    success: false,
                    message: None,
                    error: Some(format!("Execution error: {e}")),
                });
            }
        }
    }

    Json(ActionResponse {
        success: true,
        message: Some("Run executed".to_string()),
        error: None,
    })
}
