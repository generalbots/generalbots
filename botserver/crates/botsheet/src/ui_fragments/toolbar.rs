use super::html_escape;
use axum::{response::Html, Json};

pub async fn handle_share_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("share");
    if action == "delete_named" {
        let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
        return Html(format!(
            r##"<div class="ss-toast" style="padding:12px;background:#7f1d1d;color:#fecaca;border-radius:6px;">Range "{name}" removido.</div>"##,
            name = html_escape(name)
        ));
    }
    Html(format!(
        r##"<form class="ss-form" id="share-form" hx-post="/api/sheet/share" hx-vals='{{"id":"{id}","emails":""}}' hx-target="#share-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Email do colaborador
<input type="email" name="email" placeholder="usuario@empresa.com" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
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
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_find_replace_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="ss-form" id="find-replace-form" hx-post="/api/sheet/load" hx-vals='{{"id":"{id}","action":"find_replace"}}' hx-target="#find-replace-results" hx-swap="innerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Localizar
<input type="text" name="find" placeholder="Texto a buscar" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Substituir por
<input type="text" name="replace" placeholder="Texto substituto" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<div id="find-replace-results"></div>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar</button>
</form>"##,
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_conditional_format_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="ss-form" id="cf-form" hx-post="/api/sheet/conditional-format" hx-vals='{{"id":"{id}"}}' hx-target="#conditional-format-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Range (ex: A1:D10)
<input type="text" name="range" placeholder="A1:D10" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Regra
<select name="rule_type" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="greater_than">Maior que</option>
<option value="less_than">Menor que</option>
<option value="equal_to">Igual a</option>
<option value="between">Entre</option>
<option value="text_contains">Contém texto</option>
<option value="duplicate">Valores duplicados</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Valor
<input type="text" name="value" placeholder="100" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Cor de fundo
<input type="color" name="bg_color" value="#3b82f6" style="width:100%;height:36px;background:#0f172a;border:1px solid #334155;border-radius:4px;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar Regra</button>
</form>"##,
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_data_validation_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="ss-form" id="dv-form" hx-post="/api/sheet/data-validation" hx-vals='{{"id":"{id}"}}' hx-target="#data-validation-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Célula ou range
<input type="text" name="range" placeholder="A1" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Tipo de validação
<select name="kind" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="list">Lista</option>
<option value="number">Número</option>
<option value="date">Data</option>
<option value="text_length">Comprimento do texto</option>
<option value="custom">Fórmula personalizada</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Valores (separados por vírgula para lista)
<input type="text" name="values" placeholder="Sim,Não,Talvez" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Mensagem de erro
<input type="text" name="error_message" value="Valor inválido" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar Validação</button>
</form>"##,
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_custom_format_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="ss-form" id="cf-custom-form" hx-post="/api/sheet/format" hx-vals='{{"id":"{id}","format_type":"custom"}}' hx-target="#custom-format-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Range
<input type="text" name="range" placeholder="A1:A100" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Formato personalizado (Excel pattern)
<input type="text" name="format" placeholder="#,##0.00_);[Red](#,##0.00)" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;font-family:monospace;" />
</label>
<div style="color:#94a3b8;font-size:11px;background:#0f172a;padding:8px;border-radius:4px;">
<strong>Exemplos:</strong><br/>
<code>0.00</code> — 1234.56 → 1,234.56<br/>
<code>0.00%</code> — 0.123 → 12.30%<br/>
<code>R$ #,##0.00</code> — 1234.5 → R$ 1,234.50<br/>
<code>dd/mm/yyyy</code> — 45292 → 15/01/2024
</div>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Aplicar Formato</button>
</form>"##,
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_insert_image_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="ss-form" id="img-form" hx-post="/api/sheet/format" hx-vals='{{"id":"{id}","format_type":"image"}}' hx-target="#insert-image-modal" hx-swap="outerHTML" enctype="multipart/form-data" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Arquivo de imagem
<input type="file" name="image" accept="image/*" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Ancorar na célula
<input type="text" name="anchor" placeholder="B5" value="B5" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Inserir Imagem</button>
</form>"##,
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_print_preview_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="ss-form" id="print-form" hx-post="/api/sheet/export" hx-vals='{{"id":"{id}","format":"pdf"}}' hx-target="#print-preview-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<div class="print-preview-frame" style="background:#0f172a;border:1px solid #334155;border-radius:4px;padding:24px;min-height:300px;">
<h3 style="color:#f8fafc;margin-top:0;">Pré-visualização de Impressão</h3>
<p style="color:#94a3b8;">A planilha será renderizada no formato paisagem A4 com margens de 2cm.</p>
</div>
<label style="color:#f8fafc;font-size:13px;">Orientação
<select name="orientation" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="portrait">Retrato</option>
<option value="landscape" selected>Paisagem</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Escala (%)
<input type="number" name="scale" value="100" min="10" max="200" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Gerar PDF</button>
</form>"##,
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_chart_form(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<form class="ss-form" id="chart-form" hx-post="/api/sheet/chart" hx-vals='{{"id":"{id}"}}' hx-target="#chart-modal" hx-swap="outerHTML" style="padding:16px;display:flex;flex-direction:column;gap:12px;">
<label style="color:#f8fafc;font-size:13px;">Tipo de gráfico
<select name="chart_type" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;">
<option value="bar">Barras</option>
<option value="line">Linhas</option>
<option value="pie">Pizza</option>
<option value="scatter">Dispersão</option>
<option value="area">Área</option>
<option value="column">Colunas</option>
</select>
</label>
<label style="color:#f8fafc;font-size:13px;">Range de dados
<input type="text" name="data_range" placeholder="A1:D10" required style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Título
<input type="text" name="title" placeholder="Vendas por Trimestre" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<label style="color:#f8fafc;font-size:13px;">Ancorar em
<input type="text" name="anchor" value="F2" style="width:100%;padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;margin-top:4px;" />
</label>
<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">Criar Gráfico</button>
</form>"##,
        id = html_escape(sheet_id)
    ))
}
