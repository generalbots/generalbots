use crate::state::PaperState;
use std::sync::Arc;

pub async fn call_llm(
    state: &Arc<PaperState>,
    system_prompt: &str,
    user_content: &str,
) -> Result<String, String> {
    (state.call_llm)(system_prompt, user_content).await
}
