use super::*;

pub async fn handle_ai_panel(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let doc_id = payload
        .get("document_id")
        .or_else(|| payload.get("id"))
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("improve");
    let prompt = payload.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="dc-panel" id="ai-panel" style="padding:16px;background:#1e1b4b;border:1px solid #4338ca;border-radius:8px;">
<div style="display:flex;align-items:center;gap:8px;margin-bottom:12px;">
<span style="font-size:24px;">✨</span>
<h3 style="margin:0;color:#c7d2fe;font-size:16px;">Assistente IA — {action_label}</h3>
</div>
<form hx-post="/api/docs/ai/custom" hx-vals='{{"document_id":"{id}","action":"{action}","prompt":"{action}"}}' hx-target="#ai-result-panel" hx-swap="innerHTML" style="display:flex;flex-direction:column;gap:8px;">
<textarea name="selected_text" placeholder="Cole o texto a processar aqui" rows="5" style="padding:8px;background:#0f172a;border:1px solid #4338ca;border-radius:4px;color:#f8fafc;font-family:inherit;resize:vertical;">{prompt}</textarea>
<button type="submit" style="background:#6366f1;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{action_label}</button>
</form>
<div id="ai-result-panel" style="margin-top:12px;padding:8px;background:#0f172a;border-radius:4px;min-height:80px;color:#cbd5e1;font-size:13px;"></div>
</div>"##,
        id = html_escape(doc_id),
        action = html_escape(action),
        action_label = match action {
            "summarize" => "Resumir",
            "expand" => "Expandir",
            "improve" => "Melhorar",
            "simplify" => "Simplificar",
            "translate" => "Traduzir",
            "custom" => "Comando personalizado",
            _ => "Processar",
        },
        prompt = html_escape(prompt)
    ))
}
