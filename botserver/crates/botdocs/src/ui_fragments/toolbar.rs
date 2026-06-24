use super::html_escape;
use axum::{response::Html, Json};

pub async fn handle_share_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="dc-form" hx-post="/api/docs/share" hx-vals='{{"id":"{id}"}}' hx-target="#share-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Email
<input type="email" name="email" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Permissão
<select name="permission" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="view">Visualizar</option>
<option value="comment">Comentar</option>
<option value="edit" selected>Editar</option>
</select>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Compartilhar</button>
</form>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_ai_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("improve");
    let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="dc-form" hx-post="/api/docs/ai/{action}" hx-vals='{{"doc_id":"{id}"}}' hx-target="#ai-result" hx-swap="innerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Texto a processar
<textarea name="selected_text" rows="6" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;font-family:inherit;resize:vertical;">{text}</textarea>
</label>
<button type="submit" style="background:#6366f1;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Executar {action}</button>
</form>"##,
        id = html_escape(id),
        action = html_escape(action),
        text = html_escape(text)
    ))
}

pub async fn handle_find_replace_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="dc-form" hx-post="/api/docs/load" hx-vals='{{"id":"{id}","action":"find_replace"}}' hx-target="#find-replace-results" hx-swap="innerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Localizar
<input type="text" name="find" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Substituir por
<input type="text" name="replace" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<div id="find-replace-results"></div>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar</button>
</form>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_print_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="dc-form" hx-post="/api/docs/export/pdf" hx-vals='{{"id":"{id}"}}' hx-target="#print-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<div style="background:#0f172a;border:1px solid #334155;border-radius:4px;padding:24px;min-height:200px;text-align:center;color:#94a3b8;">
<p>Pré-visualização de impressão do documento</p>
<p style="font-size:11px;">O PDF será gerado em formato A4 com margens de 2cm.</p>
</div>
<label style="color:#f8fafc;font-size:13px;">Formato de saída
<select name="format" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="pdf">PDF</option>
<option value="docx">DOCX (Word)</option>
<option value="md">Markdown</option>
<option value="html">HTML</option>
<option value="txt">Texto puro</option>
</select>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Gerar arquivo</button>
</form>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_footnote_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let position = payload.get("position").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<form class="dc-form" hx-post="/api/docs/footnote" hx-vals='{{"doc_id":"{id}","position":"{pos}"}}' hx-target="#footnote-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Conteúdo da nota
<textarea name="content" rows="4" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;font-family:inherit;resize:vertical;">{content}</textarea>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Inserir Nota de Rodapé</button>
</form>"##,
        id = html_escape(id),
        pos = position,
        content = html_escape(content)
    ))
}

pub async fn handle_endnote_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="dc-form" hx-post="/api/docs/endnote" hx-vals='{{"doc_id":"{id}","position":"0"}}' hx-target="#endnote-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Conteúdo da nota final
<textarea name="content" rows="4" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;font-family:inherit;resize:vertical;">{content}</textarea>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Inserir Nota Final</button>
</form>"##,
        id = html_escape(id),
        content = html_escape(content)
    ))
}

pub async fn handle_track_changes_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let label = if enabled { "Desativar" } else { "Ativar" };
    Html(format!(
        r##"<form class="dc-form" hx-post="/api/docs/track-changes/enable" hx-vals='{{"id":"{id}","enabled":"{enabled}"}}' hx-target="#track-changes-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<p style="color:#cbd5e1;font-size:13px;">{label} o controle de alterações para registrar todas as edições feitas no documento.</p>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{label} Controle</button>
</form>"##,
        id = html_escape(id),
        enabled = !enabled,
        label = label
    ))
}

pub async fn handle_compare_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="dc-form" hx-post="/api/docs/compare" hx-vals='{{"id":"{id}"}}' hx-target="#compare-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Documento para comparar (ID)
<input type="text" name="compare_id" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" placeholder="UUID do segundo documento" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Comparar Documentos</button>
</form>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_template_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let template = payload.get("template").and_then(|v| v.as_str()).unwrap_or("blank");
    Html(format!(
        r##"<form class="dc-form" hx-post="/api/docs/template/{template}" hx-vals='{{"id":"{id}"}}' hx-target="#doc-content" hx-swap="innerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Modelo
<select name="template" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="blank">Em Branco</option>
<option value="meeting">Ata de Reunião</option>
<option value="report">Relatório</option>
<option value="letter">Carta</option>
</select>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar Modelo</button>
</form>"##,
        id = html_escape(id),
        template = html_escape(template)
    ))
}
