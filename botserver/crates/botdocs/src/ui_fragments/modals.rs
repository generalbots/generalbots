use super::html_escape;
use axum::{response::Html, Json};
use botuifragments::ui_fragments::{form_field, modal_body, modal_shell, modal_shell_width};

pub async fn handle_share_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("Documento");
    let escaped = html_escape(title);
    let body = modal_body(
        "dc",
        r#" style="padding:16px;display:flex;flex-direction:column;gap:10px;""#,
        &format!(
            r#"
<p style="margin:0;color:#cbd5e1;font-size:13px;">Share "{escaped}" with your team and choose who can view, comment or edit.</p>
<button type="button" onclick="this.closest('.dc-modal').remove(); if(window.openDocShare) openDocShare();" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Abrir compartilhamento</button>
"#
        ),
    );
    Html(modal_shell(
        "dc",
        "share-modal",
        &format!("Compartilhar \"{escaped}\""),
        &body,
    ))
}

pub async fn handle_ai_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = html_escape(payload.get("id").and_then(|v| v.as_str()).unwrap_or(""));
    let text = html_escape(payload.get("text").and_then(|v| v.as_str()).unwrap_or(""));
    let body = modal_body(
        "dc",
        "",
        &format!(
            r#"
<div style="display:flex;gap:4px;padding:8px;border-bottom:1px solid #334155;">
<button hx-post="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"summarize","text":"{text}"}}' hx-target=".dc-modal-body" hx-swap="innerHTML" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">📋 Resumir</button>
<button hx-post="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"expand","text":"{text}"}}' hx-target=".dc-modal-body" hx-swap="innerHTML" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">📝 Expandir</button>
<button hx-post="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"improve","text":"{text}"}}' hx-target=".dc-modal-body" hx-swap="innerHTML" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">✨ Melhorar</button>
<button hx-post="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"simplify","text":"{text}"}}' hx-target=".dc-modal-body" hx-swap="innerHTML" style="background:#1e293b;color:#cbd5e1;border:1px solid #334155;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">🔍 Simplificar</button>
</div>
<div hx-post="/suite/docs/forms/ai" hx-vals='{{"id":"{id}","action":"improve","text":"{text}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
"#
        ),
    );
    Html(modal_shell_width(
        "dc",
        "ai-modal",
        "✨ Assistente IA",
        560,
        &body,
    ))
}

pub async fn handle_find_replace_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = html_escape(payload.get("id").and_then(|v| v.as_str()).unwrap_or(""));
    let body = modal_body(
        "dc",
        &format!(
            r#" hx-post="/suite/docs/forms/find-replace" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML""#
        ),
        "",
    );
    Html(modal_shell(
        "dc",
        "find-replace-modal",
        "Localizar e Substituir",
        &body,
    ))
}

pub async fn handle_footnote_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = html_escape(payload.get("id").and_then(|v| v.as_str()).unwrap_or(""));
    let position = payload.get("position").and_then(|v| v.as_u64()).unwrap_or(0);
    let body = modal_body(
        "dc",
        &format!(
            r#" hx-post="/suite/docs/forms/footnote" hx-vals='{{"id":"{id}","position":"{position}"}}' hx-trigger="load" hx-swap="innerHTML""#
        ),
        "",
    );
    Html(modal_shell(
        "dc",
        "footnote-modal",
        "Inserir Nota de Rodapé",
        &body,
    ))
}

pub async fn handle_endnote_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = html_escape(payload.get("id").and_then(|v| v.as_str()).unwrap_or(""));
    let body = modal_body(
        "dc",
        &format!(
            r#" hx-post="/suite/docs/forms/endnote" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML""#
        ),
        "",
    );
    Html(modal_shell(
        "dc",
        "endnote-modal",
        "Inserir Nota Final",
        &body,
    ))
}

pub async fn handle_track_changes_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = html_escape(payload.get("id").and_then(|v| v.as_str()).unwrap_or(""));
    let enabled = payload.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let body = modal_body(
        "dc",
        &format!(
            r#" hx-post="/suite/docs/forms/track-changes" hx-vals='{{"id":"{id}","enabled":"{enabled}"}}' hx-trigger="load" hx-swap="innerHTML""#
        ),
        "",
    );
    Html(modal_shell(
        "dc",
        "track-changes-modal",
        "Controle de Alterações",
        &body,
    ))
}

pub async fn handle_compare_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = html_escape(payload.get("id").and_then(|v| v.as_str()).unwrap_or(""));
    let body = modal_body(
        "dc",
        &format!(
            r#" hx-post="/suite/docs/forms/compare" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML""#
        ),
        "",
    );
    Html(modal_shell(
        "dc",
        "compare-modal",
        "Comparar Documentos",
        &body,
    ))
}

pub async fn handle_template_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = html_escape(payload.get("id").and_then(|v| v.as_str()).unwrap_or(""));
    let body = modal_body(
        "dc",
        r#" style="padding:16px;display:grid;grid-template-columns:1fr 1fr;gap:12px;""#,
        &format!(
            r#"
<button onclick="GBDocsApplyTemplateByName('blank', '{id}')" style="padding:20px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>📄 Em Branco</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">Documento vazio pronto para escrever.</div>
</button>
<button onclick="GBDocsApplyTemplateByName('meeting', '{id}')" style="padding:20px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>📋 Ata de Reunião</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">Estrutura com pauta, decisões e ações.</div>
</button>
<button onclick="GBDocsApplyTemplateByName('report', '{id}')" style="padding:20px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>📊 Relatório</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">Capa, resumo executivo, métricas, conclusão.</div>
</button>
<button onclick="GBDocsApplyTemplateByName('letter', '{id}')" style="padding:20px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>✉️ Carta</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">Formato comercial com cabeçalho e assinatura.</div>
</button>
"#
        ),
    );
    Html(modal_shell_width(
        "dc",
        "template-modal",
        "Escolher Modelo",
        640,
        &body,
    ))
}

pub async fn handle_comment_reply_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = html_escape(payload.get("id").and_then(|v| v.as_str()).unwrap_or(""));
    let comment_id = html_escape(
        payload
            .get("comment_id")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    );
    let body = modal_body(
        "dc",
        r#" style="padding:16px;""#,
        &format!(
            r#"
<form hx-post="/api/docs/comment/reply" hx-vals='{{"doc_id":"{id}","comment_id":"{comment_id}"}}' hx-swap="none" hx-on::after-request="if(event.detail.successful) GBDocsCloseModal(this)" style="display:flex;flex-direction:column;gap:12px;">
<textarea name="content" required rows="4" placeholder="Sua resposta" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;font-family:inherit;resize:vertical;"></textarea>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Responder</button>
</form>
"#
        ),
    );
    Html(modal_shell(
        "dc",
        "comment-reply-modal",
        "Responder Comentário",
        &body,
    ))
}

pub async fn handle_style_modal(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = html_escape(payload.get("id").and_then(|v| v.as_str()).unwrap_or(""));
    let name = form_field(
        "Nome do estilo",
        r#"<input type="text" name="name" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />"#,
    );
    let kind = form_field(
        "Tipo",
        r#"<select name="style_type" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="paragraph">Parágrafo</option>
<option value="character">Caractere</option>
<option value="heading">Cabeçalho</option>
<option value="list">Lista</option>
</select>"#,
    );
    let based_on = form_field(
        "Baseado em",
        r#"<input type="text" name="based_on" value="Normal" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />"#,
    );
    let size = form_field(
        "Tamanho da fonte (pt)",
        r#"<input type="number" name="font_size" value="12" min="6" max="72" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />"#,
    );
    let body = modal_body(
        "dc",
        r#" style="padding:16px;""#,
        &format!(
            r#"
<form hx-post="/api/docs/style" hx-vals='{{"doc_id":"{id}"}}' hx-swap="none" hx-on::after-request="if(event.detail.successful) GBDocsCloseModal(this)" style="display:flex;flex-direction:column;gap:12px;">
{name}
{kind}
{based_on}
{size}
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Criar Estilo</button>
</form>
"#
        ),
    );
    Html(modal_shell("dc", "style-modal", "Criar Estilo", &body))
}
