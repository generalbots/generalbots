use super::html_escape;
use axum::{response::Html, Json};

pub async fn handle_share_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/cursor" hx-vals='{{"presentation_id":"{id}","action":"share"}}' hx-target="#share-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Email
<input type="email" name="email" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Permissão
<select name="permission" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="view">Visualizar</option>
<option value="comment">Comentar</option>
<option value="edit" selected>Editar</option>
<option value="present">Apresentar</option>
</select>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Compartilhar</button>
</form>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_ai_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let prompt = payload.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/ai" hx-vals='{{"presentation_id":"{id}","prompt":"{prompt}"}}' hx-target="#ai-result" hx-swap="innerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">O que você quer que a IA crie?
<textarea name="prompt" rows="4" placeholder="Ex: Crie 3 slides sobre os benefícios de IA em empresas" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #4338ca;border-radius:4px;color:#f8fafc;margin-top:4px;font-family:inherit;resize:vertical;">{prompt}</textarea>
</label>
<button type="submit" style="background:#6366f1;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">✨ Gerar Slides</button>
</form>"##,
        id = html_escape(id),
        prompt = html_escape(prompt)
    ))
}

pub async fn handle_new_slide_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/slide/add" hx-vals='{{"presentation_id":"{id}"}}' hx-target="#slides-content" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Layout
<select name="layout" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="title">Slide de Título</option>
<option value="content">Conteúdo</option>
<option value="two_column">Duas Colunas</option>
<option value="section">Cabeçalho de Seção</option>
<option value="image">Imagem em Destaque</option>
<option value="blank">Em Branco</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Posição (índice)
<input type="number" name="position" value="0" min="0" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Adicionar Slide</button>
</form>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_insert_element_form(Json(_payload): Json<serde_json::Value>) -> Html<String> {
    Html(r##"<div class="sl-form" style="padding:16px;display:grid;grid-template-columns:1fr 1fr;gap:8px;">
<button hx-post="/suite/slides/modals/add-text" hx-target="#modal-container" hx-swap="innerHTML" style="padding:16px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:center;">📝 Texto</button>
<button hx-post="/suite/slides/modals/add-image" hx-target="#modal-container" hx-swap="innerHTML" style="padding:16px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:center;">🖼️ Imagem</button>
<button hx-post="/suite/slides/modals/add-shape" hx-target="#modal-container" hx-swap="innerHTML" style="padding:16px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:center;">🔷 Forma</button>
<button hx-post="/suite/slides/modals/add-chart" hx-target="#modal-container" hx-swap="innerHTML" style="padding:16px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:center;">📊 Gráfico</button>
<button hx-post="/suite/slides/modals/add-table" hx-target="#modal-container" hx-swap="innerHTML" style="padding:16px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:center;">📋 Tabela</button>
<button onclick="alert('Use a opção Vídeo do menu Inserir')" style="padding:16px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:center;">🎬 Vídeo</button>
</div>"##.to_string())
}

pub async fn handle_add_text_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/element/add" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}","element_type":"text"}}' hx-target="#slide-canvas" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Texto
<textarea name="text" rows="3" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;font-family:inherit;resize:vertical;"></textarea>
</label>
<div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
<label style="color:#f8fafc;font-size:13px;">Tamanho fonte
<input type="number" name="font_size" value="18" min="8" max="120" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Cor
<input type="color" name="color" value="#f8fafc" style="width:100%;height:36px;background:#0f172a;border:1px solid #334155;border-radius:4px;margin-top:4px;" />
</label>
</div>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Adicionar Texto</button>
</form>"##,
        id = html_escape(id),
        idx = idx
    ))
}

pub async fn handle_add_image_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/media" hx-vals='{{"presentation_id":"{id}"}}' hx-target="#slide-canvas" hx-swap="outerHTML" enctype="multipart/form-data" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Arquivo de imagem
<input type="file" name="image" accept="image/*" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">URL externa (alternativa)
<input type="url" name="url" placeholder="https://exemplo.com/imagem.jpg" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Inserir Imagem</button>
</form>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_add_shape_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/element/add" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}","element_type":"shape"}}' hx-target="#slide-canvas" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Forma
<select name="shape_type" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="rectangle">Retângulo</option>
<option value="circle">Círculo</option>
<option value="triangle">Triângulo</option>
<option value="star">Estrela</option>
<option value="arrow">Seta</option>
<option value="line">Linha</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Cor de preenchimento
<input type="color" name="fill" value="#3b82f6" style="width:100%;height:36px;background:#0f172a;border:1px solid #334155;border-radius:4px;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Adicionar Forma</button>
</form>"##,
        id = html_escape(id),
        idx = idx
    ))
}

pub async fn handle_add_chart_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/element/add" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}","element_type":"chart"}}' hx-target="#slide-canvas" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Tipo
<select name="chart_type" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="bar">Barras</option>
<option value="line">Linhas</option>
<option value="pie">Pizza</option>
<option value="doughnut">Rosca</option>
<option value="area">Área</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Título
<input type="text" name="chart_title" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Dados (JSON: {{labels: [], datasets: [{{label, data, color}}]}})
<textarea name="data" rows="3" placeholder='{{"labels":["A","B","C"],"datasets":[{{"label":"X","data":[1,2,3],"color":"#3b82f6"}}]}}' style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#cbd5e1;margin-top:4px;font-family:monospace;font-size:11px;resize:vertical;">{{}}</textarea>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Adicionar Gráfico</button>
</form>"##,
        id = html_escape(id),
        idx = idx
    ))
}

pub async fn handle_add_table_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/element/add" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}","element_type":"table"}}' hx-target="#slide-canvas" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
<label style="color:#f8fafc;font-size:13px;">Linhas
<input type="number" name="rows" value="3" min="1" max="20" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Colunas
<input type="number" name="cols" value="3" min="1" max="10" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
</div>
<label style="color:#f8fafc;font-size:13px;">Cabeçalhos (separados por vírgula)
<input type="text" name="headers" placeholder="Col A, Col B, Col C" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Adicionar Tabela</button>
</form>"##,
        id = html_escape(id),
        idx = idx
    ))
}

pub async fn handle_notes_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    let notes = payload.get("notes").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/slide/notes" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}"}}' hx-target="#notes-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Notas do apresentador (não visíveis ao público)
<textarea name="notes" rows="6" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;font-family:inherit;resize:vertical;">{notes}</textarea>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Salvar Notas</button>
</form>"##,
        id = html_escape(id),
        idx = idx,
        notes = html_escape(notes)
    ))
}

pub async fn handle_background_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/element/update" hx-vals='{{"presentation_id":"{id}","target":"background"}}' hx-target="#background-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Tipo de fundo
<select name="bg_type" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="solid">Cor sólida</option>
<option value="gradient">Gradiente</option>
<option value="image">Imagem</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Cor primária
<input type="color" name="color" value="#0f172a" style="width:100%;height:36px;background:#0f172a;border:1px solid #334155;border-radius:4px;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Imagem de fundo (URL)
<input type="url" name="image_url" placeholder="https://..." style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar Fundo</button>
</form>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_transition_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let idx = payload.get("slide_index").and_then(|v| v.as_u64()).unwrap_or(0);
    let ttype = payload.get("transition_type").and_then(|v| v.as_str()).unwrap_or("none");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/transition" hx-vals='{{"presentation_id":"{id}","slide_index":"{idx}","transition_type":"{ttype}"}}' hx-target="#transition-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Duração (ms)
<input type="number" name="duration" value="500" min="100" max="5000" step="100" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Direção
<select name="direction" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="left">Esquerda</option>
<option value="right">Direita</option>
<option value="up">Cima</option>
<option value="down">Baixo</option>
<option value="in">Para dentro</option>
<option value="out">Para fora</option>
</select>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar Transição</button>
</form>"##,
        id = html_escape(id),
        idx = idx,
        ttype = html_escape(ttype)
    ))
}

pub async fn handle_animation_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    let atype = payload.get("type").and_then(|v| v.as_str()).unwrap_or("fade_in");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/element/update" hx-vals='{{"presentation_id":"{id}","action":"add_animation","animation_type":"{atype}"}}' hx-target="#animation-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
<label style="color:#f8fafc;font-size:13px;">Duração (s)
<input type="number" name="duration" value="0.5" min="0.1" max="10" step="0.1" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Atraso (s)
<input type="number" name="delay" value="0" min="0" max="10" step="0.1" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
</div>
<label style="color:#f8fafc;font-size:13px;">Gatilho
<select name="trigger" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="on_click">Ao clicar</option>
<option value="on_load">Ao aparecer</option>
<option value="with_previous">Com anterior</option>
<option value="after_previous">Após anterior</option>
</select>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar Animação</button>
</form>"##,
        id = html_escape(id),
        atype = html_escape(atype)
    ))
}

pub async fn handle_export_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/export" hx-vals='{{"presentation_id":"{id}"}}' hx-target="#export-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Formato
<select name="format" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="pptx">PPTX (PowerPoint)</option>
<option value="pdf">PDF</option>
<option value="html">HTML</option>
<option value="odp">ODP (LibreOffice)</option>
<option value="png">Imagens PNG (uma por slide)</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Qualidade
<select name="quality" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="standard">Padrão</option>
<option value="high">Alta (1920×1080)</option>
<option value="print">Impressão (300 DPI)</option>
</select>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Exportar</button>
</form>"##,
        id = html_escape(id)
    ))
}

pub async fn handle_master_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let id = payload.get("presentation_id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="sl-form" hx-post="/api/slides/theme" hx-vals='{{"presentation_id":"{id}","action":"set_master"}}' hx-target="#master-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Slide mestre
<select name="master_id" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="default">Padrão</option>
<option value="title_only">Apenas Título</option>
<option value="blank">Em Branco</option>
<option value="section">Cabeçalho de Seção</option>
<option value="comparison">Comparação</option>
</select>
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar Mestre</button>
</form>"##,
        id = html_escape(id)
    ))
}
