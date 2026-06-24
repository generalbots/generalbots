use super::html_escape;
use axum::{response::Html, Json};

pub async fn handle_share_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("Documento");
    Html(format!(
        r##"<div class="dc-modal" id="share-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Compartilhar "{title}"</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body" hx-get="/suite/docs/forms/share" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        title = html_escape(title)
    ))
}

pub async fn handle_ai_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="dc-modal" id="ai-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:560px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">✨ Assistente IA</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body">
<div style="display:flex;gap:4px;padding:8px;border-bottom:1px solid #334155;">
<button hx-get="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"summarize","text":"{text}"}}' hx-target=".dc-modal-body" hx-swap="innerHTML" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">📋 Resumir</button>
<button hx-get="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"expand","text":"{text}"}}' hx-target=".dc-modal-body" hx-swap="innerHTML" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">📝 Expandir</button>
<button hx-get="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"improve","text":"{text}"}}' hx-target=".dc-modal-body" hx-swap="innerHTML" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">✨ Melhorar</button>
<button hx-get="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"simplify","text":"{text}"}}' hx-target=".dc-modal-body" hx-swap="innerHTML" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">🔍 Simplificar</button>
</div>
<div hx-get="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"improve","text":"{text}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>
</div>"##,
        id = html_escape(id),
        text = html_escape(text)
    ))
}

pub async fn handle_find_replace_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="dc-modal" id="find-replace-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Localizar e Substituir</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body" hx-get="/suite/docs/forms/find-replace" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_footnote_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let position = payload.get("position").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<div class="dc-modal" id="footnote-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Inserir Nota de Rodapé</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body" hx-get="/suite/docs/forms/footnote" hx-vals='{{"id":"{id}","position":"{pos}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        pos = position
    ))
}

pub async fn handle_endnote_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="dc-modal" id="endnote-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Inserir Nota Final</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body" hx-get="/suite/docs/forms/endnote" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_track_changes_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    Html(format!(
        r##"<div class="dc-modal" id="track-changes-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Controle de Alterações</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body" hx-get="/suite/docs/forms/track-changes" hx-vals='{{"id":"{id}","enabled":"{enabled}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id),
        enabled = enabled
    ))
}

pub async fn handle_compare_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="dc-modal" id="compare-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Comparar Documentos</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body" hx-get="/suite/docs/forms/compare" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_template_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="dc-modal" id="template-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:640px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Escolher Modelo</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body" style="padding:16px;display:grid;grid-template-columns:1fr 1fr;gap:12px;">
<button hx-post="/api/docs/template/blank" hx-vals='{{"id":"{id}"}}' hx-target="#doc-content" hx-swap="innerHTML" style="padding:20px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>📄 Em Branco</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">Documento vazio pronto para escrever.</div>
</button>
<button hx-post="/api/docs/template/meeting" hx-vals='{{"id":"{id}"}}' hx-target="#doc-content" hx-swap="innerHTML" style="padding:20px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>📋 Ata de Reunião</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">Estrutura com pauta, decisões e ações.</div>
</button>
<button hx-post="/api/docs/template/report" hx-vals='{{"id":"{id}"}}' hx-target="#doc-content" hx-swap="innerHTML" style="padding:20px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>📊 Relatório</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">Capa, resumo executivo, métricas, conclusão.</div>
</button>
<button hx-post="/api/docs/template/letter" hx-vals='{{"id":"{id}"}}' hx-target="#doc-content" hx-swap="innerHTML" style="padding:20px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>✉️ Carta</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">Formato comercial com cabeçalho e assinatura.</div>
</button>
</div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_comment_reply_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let comment_id = payload.get("comment_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="dc-modal" id="comment-reply-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Responder Comentário</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body" style="padding:16px;">
<form hx-post="/api/docs/comment/reply" hx-vals='{{"doc_id":"{id}","comment_id":"{cid}"}}' hx-target="#comments-panel" hx-swap="outerHTML" style="display:flex;flex-direction:column;gap:12px;">
<textarea name="content" required rows="4" placeholder="Sua resposta" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;font-family:inherit;resize:vertical;"></textarea>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Responder</button>
</form>
</div>
</div>
</div>"##,
        id = html_escape(id),
        cid = html_escape(comment_id)
    ))
}

pub async fn handle_style_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="dc-modal" id="style-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="dc-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="dc-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">Criar Estilo</h3>
<button onclick="this.closest('.dc-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="dc-modal-body" style="padding:16px;">
<form hx-post="/api/docs/style" hx-vals='{{"doc_id":"{id}"}}' hx-target="#styles-panel" hx-swap="outerHTML" style="display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Nome do estilo
<input type="text" name="name" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Tipo
<select name="style_type" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="paragraph">Parágrafo</option>
<option value="character">Caractere</option>
<option value="heading">Cabeçalho</option>
<option value="list">Lista</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Baseado em
<input type="text" name="based_on" value="Normal" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Tamanho da fonte (pt)
<input type="number" name="font_size" value="12" min="6" max="72" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Criar Estilo</button>
</form>
</div>
</div>
</div>"##,
        id = html_escape(id)
    ))
}
