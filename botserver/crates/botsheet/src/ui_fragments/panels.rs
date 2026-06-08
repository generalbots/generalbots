use super::{empty_fragment, html_escape};
use axum::{response::Html, Json};
use botsheet_core::types::{ArrayFormula, CellComment, ConditionalFormatRule, ChartConfig, ExternalLink, NamedRange, Spreadsheet, ValidationRule};

pub async fn handle_named_ranges_panel(Json(sheet): Json<Spreadsheet>) -> Html<String> {
    match sheet.named_ranges {
        Some(ref ranges) if !ranges.is_empty() => {
            let mut html = String::from(
                r##"<div class="ss-panel" id="named-ranges-panel">
<div class="ss-panel-header" style="display:flex;justify-content:space-between;align-items:center;padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Ranges Nomeados</h3>
<button class="btn-icon" hx-get="/suite/sheet/modals/conditional-format" hx-target="#modal-container" hx-swap="innerHTML" title="Adicionar Range" style="background:#3b82f6;color:white;border:none;padding:6px 12px;border-radius:4px;cursor:pointer;">+ Novo</button>
</div>
<div class="ss-panel-body" style="padding:8px;">"##
            );
            for r in ranges {
                html.push_str(&render_named_range(r));
            }
            html.push_str("</div></div>");
            Html(html)
        }
        _ => Html(empty_fragment("Nenhum range nomeado definido. Crie ranges nomeados para facilitar referências em fórmulas.")),
    }
}

fn render_named_range(r: &NamedRange) -> String {
    let ws_label = r.worksheet_index.map(|i| format!("Planilha {}", i + 1)).unwrap_or_else(|| "Global".to_string());
    let ref_str = format!("{}:{}{}", col_name(r.start_col), r.start_row + 1, col_name(r.end_col)) + &format!("{}", r.end_row + 1);
    let comment = r.comment.as_deref().unwrap_or("");
    format!(
        r##"<div class="ss-range-item" style="padding:8px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<div>
<div style="font-weight:600;color:#f8fafc;">{name}</div>
<div style="font-size:12px;color:#94a3b8;">{ws} • {range_ref}</div>
{comment_html}
</div>
<button class="btn-icon" hx-post="/suite/sheet/forms/find-replace" hx-vals='{{"action":"delete_named","name":"{name}"}}' hx-target="#named-ranges-panel" hx-swap="outerHTML" style="background:#dc2626;color:white;border:none;padding:4px 8px;border-radius:4px;cursor:pointer;font-size:12px;">×</button>
</div>"##,
        name = html_escape(&r.name),
        ws = html_escape(&ws_label),
        range_ref = html_escape(&ref_str),
        comment_html = if comment.is_empty() { String::new() } else { format!(r##"<div style="font-size:11px;color:#64748b;margin-top:2px;font-style:italic;">{}</div>"##, html_escape(comment)) }
    )
}

fn col_name(idx: u32) -> String {
    let mut n = idx + 1;
    let mut s = String::new();
    while n > 0 {
        let r = (n - 1) % 26;
        s = format!("{}{}", (b'A' + r as u8) as char, s);
        n = (n - 1) / 26;
    }
    s
}

pub async fn handle_charts_panel(Json(sheet): Json<Spreadsheet>) -> Html<String> {
    let chart_data: Vec<(usize, &ChartConfig)> = sheet.worksheets.iter()
        .enumerate()
        .flat_map(|(i, w)| {
            w.charts.as_ref().into_iter().flatten().enumerate()
                .map(move |(j, ch)| (i * 1000 + j, ch))
        })
        .collect();
    let charts_total = chart_data.len();
    if charts_total == 0 {
        return Html(empty_fragment("Nenhum gráfico criado. Use o botão Inserir Gráfico para adicionar."));
    }
    let mut html = String::from(
        r##"<div class="ss-panel" id="charts-panel">
<div class="ss-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Gráficos</h3>
</div>
<div class="ss-panel-body" style="padding:8px;display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:12px;">"##
    );
    for (_idx, c) in chart_data {
        let chart_type = c.chart_type.clone();
        let title = c.title.clone();
        let series_count = c.datasets.len();
        html.push_str(&format!(
            r##"<div class="ss-chart-card" style="padding:12px;background:#0f172a;border:1px solid #334155;border-radius:6px;">
<div style="font-weight:600;color:#f8fafc;font-size:14px;">{title}</div>
<div style="font-size:12px;color:#94a3b8;margin-top:4px;">Tipo: {chart_type} • {series_count} série(s)</div>
<div style="margin-top:8px;height:60px;background:linear-gradient(135deg,#3b82f620,#10b98120);border-radius:4px;"></div>
</div>"##,
            title = html_escape(&title),
            chart_type = html_escape(&chart_type),
            series_count = series_count
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_validations_panel(Json(sheet): Json<Spreadsheet>) -> Html<String> {
    let validations: Vec<(&str, &ValidationRule)> = sheet.worksheets.iter()
        .flat_map(|w| w.validations.as_ref().into_iter().flatten())
        .map(|(k, v)| (k.as_str(), v))
        .collect();
    if validations.is_empty() {
        return Html(empty_fragment("Nenhuma validação de dados configurada."));
    }
    let mut html = String::from(
        r##"<div class="ss-panel" id="validations-panel">
<div class="ss-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Validações de Dados</h3>
</div>
<div class="ss-panel-body" style="padding:8px;">"##
    );
    for (cell_ref, v) in validations {
        html.push_str(&format!(
            r##"<div class="ss-validation-item" style="padding:8px;border-bottom:1px solid #334155;">
<div style="font-weight:600;color:#f8fafc;">{cell}</div>
<div style="font-size:12px;color:#94a3b8;">Tipo: {kind}</div>
</div>"##,
            cell = html_escape(cell_ref),
            kind = html_escape(&v.validation_type)
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_conditional_formats_panel(Json(sheet): Json<Spreadsheet>) -> Html<String> {
    let rules: Vec<&ConditionalFormatRule> = sheet.worksheets.iter()
        .filter_map(|w| w.conditional_formats.as_ref())
        .flatten()
        .collect();
    if rules.is_empty() {
        return Html(empty_fragment("Nenhuma regra de formatação condicional aplicada."));
    }
    let mut html = String::from(
        r##"<div class="ss-panel" id="cf-panel">
<div class="ss-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Formatação Condicional</h3>
</div>
<div class="ss-panel-body" style="padding:8px;">"##
    );
    for r in rules {
        let preview = r.style.background.clone().unwrap_or_else(|| "#3b82f6".to_string());
        let range = format!("{}{}:{}{}", col_name(r.start_col), r.start_row + 1, col_name(r.end_col), r.end_row + 1);
        html.push_str(&format!(
            r##"<div class="ss-cf-item" style="padding:8px;border-bottom:1px solid #334155;display:flex;align-items:center;gap:8px;">
<div style="width:24px;height:24px;background:{preview};border-radius:4px;border:1px solid #334155;"></div>
<div>
<div style="font-weight:600;color:#f8fafc;font-size:13px;">{rule_type}</div>
<div style="font-size:12px;color:#94a3b8;">{range}</div>
</div>
</div>"##,
            preview = html_escape(&preview),
            rule_type = html_escape(&r.rule_type),
            range = html_escape(&range)
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_external_links_panel(Json(sheet): Json<Spreadsheet>) -> Html<String> {
    let links: Vec<&ExternalLink> = sheet.external_links.as_deref().unwrap_or(&[]).iter().collect();
    if links.is_empty() {
        return Html(empty_fragment("Nenhum link externo configurado."));
    }
    let mut html = String::from(
        r##"<div class="ss-panel" id="external-links-panel">
<div class="ss-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Links Externos</h3>
</div>
<div class="ss-panel-body" style="padding:8px;">"##
    );
    for l in links {
        html.push_str(&format!(
            r##"<div class="ss-link-item" style="padding:8px;border-bottom:1px solid #334155;">
<div style="font-weight:600;color:#f8fafc;">{source}</div>
<div style="font-size:12px;color:#94a3b8;">Tipo: {link_type} • Status: {status}</div>
</div>"##,
            source = html_escape(&l.source_path),
            link_type = html_escape(&l.link_type),
            status = html_escape(&l.status)
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_comments_panel(Json(sheet): Json<Spreadsheet>) -> Html<String> {
    let comments: Vec<(&str, &CellComment)> = sheet.worksheets.iter()
        .flat_map(|w| w.comments.as_ref().into_iter().flatten())
        .map(|(k, v)| (k.as_str(), v))
        .collect();
    if comments.is_empty() {
        return Html(empty_fragment("Nenhum comentário adicionado."));
    }
    let mut html = String::from(
        r##"<div class="ss-panel" id="comments-panel">
<div class="ss-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Comentários</h3>
</div>
<div class="ss-panel-body" style="padding:8px;max-height:400px;overflow-y:auto;">"##
    );
    for (cell, c) in comments {
        let resolved = if c.resolved { r##"<span style="color:#10b981;font-size:11px;background:#064e3b;padding:2px 6px;border-radius:3px;">Resolvido</span>"## } else { "" };
        html.push_str(&format!(
            r##"<div class="ss-comment-item" style="padding:8px;border-bottom:1px solid #334155;border-left:3px solid #f59e0b;background:#1e293b50;margin-bottom:4px;">
<div style="display:flex;justify-content:space-between;align-items:center;">
<div style="font-weight:600;color:#f8fafc;font-size:13px;">{cell} — {author}</div>
{resolved}
</div>
<div style="font-size:12px;color:#cbd5e1;margin-top:4px;">{content}</div>
<div style="font-size:10px;color:#64748b;margin-top:4px;">{created}</div>
</div>"##,
            cell = html_escape(cell),
            author = html_escape(&c.author_name),
            content = html_escape(&c.content),
            created = c.created_at.format("%d/%m/%Y %H:%M"),
            resolved = resolved
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_protection_panel(Json(sheet): Json<Spreadsheet>) -> Html<String> {
    let mut html = String::from(
        r##"<div class="ss-panel" id="protection-panel">
<div class="ss-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Proteção da Planilha</h3>
</div>
<div class="ss-panel-body" style="padding:12px;">"##
    );
    for (i, w) in sheet.worksheets.iter().enumerate() {
        if let Some(p) = &w.protection {
            let status = if p.protected { r##"<span style="color:#10b981;">🔒 Protegida</span>"## } else { r##"<span style="color:#94a3b8;">🔓 Desprotegida</span>"## };
            html.push_str(&format!(
                r##"<div class="ss-prot-item" style="padding:8px;margin-bottom:8px;background:#0f172a;border:1px solid #334155;border-radius:6px;">
<div style="font-weight:600;color:#f8fafc;">Planilha {idx} {status}</div>
<div style="font-size:12px;color:#94a3b8;margin-top:4px;">Células bloqueadas: {locked} • {perms}</div>
</div>"##,
                idx = i + 1,
                status = status,
                locked = p.locked_cells.len(),
                perms = if p.allow_format_cells { "Formatação permitida" } else { "Formatação bloqueada" }
            ));
        }
    }
    if sheet.worksheets.iter().all(|w| w.protection.is_none()) {
        html.push_str(&empty_fragment("Nenhuma proteção configurada."));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_array_formulas_panel(Json(sheet): Json<Spreadsheet>) -> Html<String> {
    let formulas: Vec<&ArrayFormula> = sheet.worksheets.iter()
        .filter_map(|w| w.array_formulas.as_ref())
        .flatten()
        .collect();
    if formulas.is_empty() {
        return Html(empty_fragment("Nenhuma fórmula de matriz (array) definida."));
    }
    let mut html = String::from(
        r##"<div class="ss-panel" id="array-formulas-panel">
<div class="ss-panel-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:16px;">Fórmulas de Matriz</h3>
</div>
<div class="ss-panel-body" style="padding:8px;">"##
    );
    for f in formulas {
        let dynamic = if f.is_dynamic { r##"<span style="color:#10b981;font-size:10px;background:#064e3b;padding:1px 4px;border-radius:2px;">Dinâmica</span>"## } else { "" };
        html.push_str(&format!(
            r##"<div class="ss-array-item" style="padding:8px;border-bottom:1px solid #334155;">
<div style="font-weight:600;color:#f8fafc;font-family:monospace;font-size:12px;">{formula} {dynamic}</div>
<div style="font-size:11px;color:#94a3b8;margin-top:2px;">Range: {r1}:{c1} → {r2}:{c2}</div>
</div>"##,
            formula = html_escape(&f.formula),
            dynamic = dynamic,
            r1 = f.start_row + 1, c1 = col_name(f.start_col),
            r2 = f.end_row + 1, c2 = col_name(f.end_col)
        ));
    }
    html.push_str("</div></div>");
    Html(html)
}
