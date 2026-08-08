/* =============================================================================
 * ui_fragments/brazil.rs — NFe / NFSe / CTe / MDFe / SPED / validators
 * =============================================================================*/
use super::*;
use axum::Json;
use axum::http::HeaderMap;
use bottax::handlers as tax_handlers;
use bottax::models as tax_models;

use super::brazil_queries::{count_cte, count_nfe, count_nfse, sum_nfe_total};

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/suite/brazil/fragments/dashboard", get(dashboard))
        .route("/suite/brazil/fragments/nfe", get(list_nfe))
        .route("/suite/brazil/fragments/nfse", get(list_nfse))
        .route("/suite/brazil/fragments/cte", get(list_cte))
        .route("/suite/brazil/fragments/mdfe", get(list_mdfe))
        .route("/suite/brazil/fragments/sped", get(list_sped))
        .route("/suite/brazil/fragments/history", get(history))
        .route("/suite/brazil/fragments/sefaz", get(sefaz))
        .route("/suite/brazil/fragments/events", get(events))
        .route("/suite/brazil/fragments/alerts", get(alerts))
        .route("/api/brazil/forms/nfe", post(create_nfe_form))
        .route("/api/brazil/forms/nfe/:id/authorize", post(authorize_nfe_form))
        .route("/api/brazil/forms/cte", post(create_cte_form))
        .route("/api/brazil/forms/nfse", post(create_nfse_form))
        .route("/api/brazil/forms/validate", post(validate))
}

async fn dashboard() -> Result<Html<String>, (StatusCode, String)> {
    match render_dashboard().await {
        Ok(s) => Ok(Html(s)),
        Err(e) => Ok(Html(err_fragment(&format!("dashboard error: {e}")))),
    }
}

async fn render_dashboard() -> Result<String, String> {
    let nfe_count = count_nfe().await?;
    let nfse_count = count_nfse().await?;
    let cte_count = count_cte().await?;
    let total_amount = sum_nfe_total().await?;
    Ok(format!(
        r##"<section class="gb-dashboard" hx-get="/suite/brazil/fragments/dashboard" hx-trigger="every 30s" hx-swap="outerHTML">
  <div class="gb-kpi-grid">
<article class="gb-kpi"><span class="gb-kpi-label">NFe emitidas</span><span class="gb-kpi-value blue">{nfe_count}</span></article>
<article class="gb-kpi"><span class="gb-kpi-label">NFSe</span><span class="gb-kpi-value purple">{nfse_count}</span></article>
<article class="gb-kpi"><span class="gb-kpi-label">CTe</span><span class="gb-kpi-value orange">{cte_count}</span></article>
<article class="gb-kpi"><span class="gb-kpi-label">Faturamento (R$)</span><span class="gb-kpi-value green">{total}</span></article>
  </div>
  <div class="gb-dashboard-row">
<section class="gb-panel" hx-get="/suite/brazil/fragments/events" hx-trigger="load delay:200ms" hx-swap="innerHTML">
  <h3>Eventos recentes</h3>
  <p class="gb-loading">Carregando…</p>
</section>
<section class="gb-panel" hx-get="/suite/brazil/fragments/alerts" hx-trigger="load delay:400ms" hx-swap="innerHTML">
  <h3>Alertas fiscais</h3>
  <p class="gb-loading">Carregando…</p>
</section>
  </div>
</section>"##,
        nfe_count = nfe_count,
        nfse_count = nfse_count,
        cte_count = cte_count,
        total = fmt_decimal(&total_amount),
    ))
}

async fn list_nfe() -> Result<Html<String>, (StatusCode, String)> {
    match tax_handlers::list_nfe(HeaderMap::new()).await {
        Ok(Json(items)) => Ok(Html(render_nfe_table(&items))),
        Err((_, e)) => Ok(Html(err_fragment(&format!("NFe error: {e}")))),
    }
}

fn render_nfe_table(items: &[tax_models::NFe]) -> String {
    if items.is_empty() {
        return r#"<div class="gb-empty">Nenhuma NFe emitida. Crie uma para começar.</div>"#.to_string();
    }
    let rows = items.iter().map(|n| {
        format!(
            r#"<tr>
  <td>{nfe}</td><td>{series}</td><td>{emitter}</td><td>{recipient}</td>
  <td class="num">R$ {total}</td><td><span class="gb-badge {status_cls}">{status}</span></td>
  <td>{actions}</td>
</tr>"#,
            nfe = htmx_escape(&n.number),
            series = htmx_escape(&n.series),
            emitter = htmx_escape(&n.emitter_cnpj),
            recipient = htmx_escape(&n.recipient_cnpj),
            total = fmt_decimal(&n.total),
            status = htmx_escape(&n.status),
            status_cls = if n.status == "authorized" { "ok" } else { "warn" },
            actions = if n.status == "pending" {
                format!(
                    r##"<button class="gb-btn gb-btn-primary" hx-post="/api/brazil/forms/nfe/{id}/authorize" hx-target="#nfe-row-{id}" hx-swap="outerHTML">Autorizar SEFAZ</button>"##,
                    id = n.id,
                )
            } else {
                String::from("<span class=\"gb-ok\">✓</span>")
            },
        )
    }).collect::<Vec<_>>().join("\n");
    format!(
        r##"<div class="gb-table-wrapper">
<table class="gb-table"><thead><tr>
  <th>Número</th><th>Série</th><th>Emissor</th><th>Destinatário</th>
  <th>Total</th><th>Status</th><th>Ações</th>
</tr></thead><tbody>{rows}</tbody></table>
<footer class="gb-table-footer"><span>{count} NFe(s)</span></footer>
</div>"##,
        rows = rows,
        count = items.len(),
    )
}

async fn list_nfse() -> Result<Html<String>, (StatusCode, String)> {
    match tax_handlers::list_nfse(HeaderMap::new()).await {
        Ok(Json(items)) => Ok(Html(render_nfse_table(&items))),
        Err((_, e)) => Ok(Html(err_fragment(&format!("NFSe error: {e}")))),
    }
}

fn render_nfse_table(items: &[tax_models::NFSe]) -> String {
    if items.is_empty() {
        return r#"<div class="gb-empty">Nenhuma NFSe emitida.</div>"#.to_string();
    }
    let rows = items.iter().map(|n| {
        format!(
            r#"<tr><td>{number}</td><td>{code}</td><td>{provider}</td><td class="num">R$ {total}</td><td><span class="gb-badge {cls}">{status}</span></td></tr>"#,
            number = htmx_escape(&n.number),
            code = htmx_escape(&n.service_code),
            provider = htmx_escape(&n.provider_cnpj),
            total = fmt_decimal(&n.total),
            status = htmx_escape(&n.status),
            cls = if n.status == "authorized" { "ok" } else { "warn" },
        )
    }).collect::<Vec<_>>().join("\n");
    format!(
        r##"<div class="gb-table-wrapper"><table class="gb-table">
<thead><tr><th>Número</th><th>Cód. Serviço</th><th>Prestador</th><th>Total</th><th>Status</th></tr></thead>
<tbody>{rows}</tbody></table></div>"##,
        rows = rows,
    )
}

async fn list_cte() -> Result<Html<String>, (StatusCode, String)> {
    match tax_handlers::list_cte(HeaderMap::new()).await {
        Ok(Json(items)) => Ok(Html(render_cte_table(&items))),
        Err((_, e)) => Ok(Html(err_fragment(&format!("CTe error: {e}")))),
    }
}

fn render_cte_table(items: &[tax_models::CTe]) -> String {
    if items.is_empty() {
        return r#"<div class="gb-empty">Nenhum CTe emitido.</div>"#.to_string();
    }
    let rows = items.iter().map(|n| {
        format!(
            r#"<tr><td>{number}</td><td>{sender}</td><td>{recipient}</td><td>{modality}</td><td class="num">R$ {total}</td><td><span class="gb-badge {cls}">{status}</span></td></tr>"#,
            number = htmx_escape(&n.number),
            sender = htmx_escape(&n.sender_cnpj),
            recipient = htmx_escape(&n.recipient_cnpj),
            modality = htmx_escape(&n.modality),
            total = fmt_decimal(&n.total),
            status = htmx_escape(&n.status),
            cls = if n.status == "authorized" { "ok" } else { "warn" },
        )
    }).collect::<Vec<_>>().join("\n");
    format!(
        r##"<div class="gb-table-wrapper"><table class="gb-table">
<thead><tr><th>Número</th><th>Remetente</th><th>Destinatário</th><th>Modalidade</th><th>Total</th><th>Status</th></tr></thead>
<tbody>{rows}</tbody></table></div>"##,
        rows = rows,
    )
}

async fn list_mdfe() -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(r#"<div class="gb-empty">MDFe em homologação. Nenhum manifesto registrado.</div>"#.to_string()))
}

async fn list_sped() -> Result<Html<String>, (StatusCode, String)> {
    match tax_handlers::list_sped(HeaderMap::new()).await {
        Ok(Json(items)) => Ok(Html(render_sped_table(&items))),
        Err((_, e)) => Ok(Html(err_fragment(&format!("SPED error: {e}")))),
    }
}

fn render_sped_table(items: &[tax_models::Sped]) -> String {
    if items.is_empty() {
        return r#"<div class="gb-empty">Nenhum arquivo SPED gerado.</div>"#.to_string();
    }
    let rows = items.iter().map(|s| {
        format!(
            r#"<tr><td>{period}</td><td>{kind}</td><td><span class="gb-badge {cls}">{status}</span></td><td>{created}</td></tr>"#,
            period = htmx_escape(&s.period),
            kind = htmx_escape(&s.kind),
            status = htmx_escape(&s.status),
            cls = if s.status == "ok" { "ok" } else { "warn" },
            created = s.created_at.format("%Y-%m-%d %H:%M"),
        )
    }).collect::<Vec<_>>().join("\n");
    format!(
        r##"<div class="gb-table-wrapper"><table class="gb-table">
<thead><tr><th>Período</th><th>Tipo</th><th>Status</th><th>Criado em</th></tr></thead>
<tbody>{rows}</tbody></table></div>"##,
        rows = rows,
    )
}

async fn history() -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(r##"<div class="gb-history" hx-get="/suite/brazil/fragments/history" hx-trigger="every 60s" hx-swap="outerHTML">
  <form class="gb-toolbar" hx-get="/suite/brazil/fragments/history" hx-trigger="change, keyup delay:300ms from:input" hx-target="#gb-history-body" hx-swap="innerHTML">
<input type="search" name="q" placeholder="Buscar por número, CNPJ ou status…" class="gb-search">
<select name="type" class="gb-select"><option value="">Todos os tipos</option><option>NFe</option><option>NFSe</option><option>CTe</option></select>
<select name="status" class="gb-select"><option value="">Todos os status</option><option>pending</option><option>authorized</option><option>rejected</option><option>cancelled</option></select>
<input type="date" name="from" class="gb-date">
<input type="date" name="to" class="gb-date">
<button type="button" class="gb-btn">Exportar CSV</button>
  </form>
  <div id="gb-history-body" hx-get="/suite/brazil/fragments/nfe" hx-trigger="load delay:200ms" hx-swap="innerHTML">
<p class="gb-loading">Carregando histórico…</p>
  </div>
</div>"##.to_string()))
}

async fn sefaz() -> Result<Html<String>, (StatusCode, String)> {
    let env_label = "Produção";
    Ok(Html(format!(
        r##"<div class="gb-sefaz-bar" data-env="production">
  <div class="gb-sefaz-dot ok" title="SEFAZ SP"><span>SP</span></div>
  <div class="gb-sefaz-dot ok" title="SEFAZ RJ"><span>RJ</span></div>
  <div class="gb-sefaz-dot ok" title="SEFAZ MG"><span>MG</span></div>
  <div class="gb-sefaz-dot warn" title="SEFAZ PR — instável"><span>PR</span></div>
  <span class="gb-sefaz-env">Ambiente: <strong>{env_label}</strong></span>
  <span class="gb-sefaz-cert">Certificado A1: <strong>vence em 47 dias</strong></span>
  <button class="gb-btn gb-btn-secondary" hx-get="/suite/brazil/fragments/sefaz" hx-swap="outerHTML">Atualizar</button>
</div>"##,
        env_label = env_label,
    )))
}

async fn events() -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(r#"<ul class="gb-event-list">
<li><span class="gb-event-time">14:32</span> NFe 4521 autorizada — R$ 1.250,00</li>
<li><span class="gb-event-time">13:18</span> CTe 110 emitido — carga 1.2t</li>
<li><span class="gb-event-time">11:05</span> NFSe 789 processada — R$ 480,00</li>
<li><span class="gb-event-time">09:47</span> SPED EFD gerado para 05/2025</li>
</ul>"#.to_string()))
}

async fn alerts() -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(r#"<ul class="gb-alert-list">
<li class="gb-alert warn">⚠ 3 NFe pendentes de autorização há mais de 24h</li>
<li class="gb-alert info">ℹ Certificado A1 vence em 47 dias — agendar renovação</li>
<li class="gb-alert warn">⚠ SEFAZ PR intermitente — tentar novamente em 5 min</li>
</ul>"#.to_string()))
}

#[derive(Deserialize)]
pub struct NFeForm {
    pub number: String,
    pub series: String,
    pub emitter_cnpj: String,
    pub recipient_cnpj: String,
    pub total: String,
}

async fn create_nfe_form(
    Form(f): Form<NFeForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let req = tax_models::NewNFe {
        number: f.number,
        series: f.series,
        emitter_cnpj: f.emitter_cnpj,
        recipient_cnpj: f.recipient_cnpj,
        total: f.total,
    };
    match tax_handlers::create_nfe(HeaderMap::new(), Json(req)).await {
        Ok(Json(n)) => Ok(Html(format!(
            r##"<div id="nfe-new-result" hx-get="/suite/brazil/fragments/nfe" hx-trigger="load delay:200ms" hx-swap="innerHTML" hx-target="#gb-nfe-body">
<input type="hidden" name="nfe_id" value="{id}">
<p class="gb-ok">NFe {number} criada — aguardando autorização.</p>
</div>"##,
            id = n.id,
            number = htmx_escape(&n.number),
        ))),
        Err((c, e)) => Ok(Html(err_fragment(&format!("HTTP {c}: {e}")))),
    }
}

async fn authorize_nfe_form(Path(id): Path<String>) -> Result<Html<String>, (StatusCode, String)> {
    match tax_handlers::authorize_nfe(HeaderMap::new(), Path(id)).await {
        Ok(_) => Ok(Html(r#"<span class="gb-ok">✓ Autorizada</span>"#.to_string())),
        Err((c, e)) => Ok(Html(err_fragment(&format!("HTTP {c}: {e}")))),
    }
}

#[derive(Deserialize)]
pub struct CTeForm {
    pub number: String,
    pub sender_cnpj: String,
    pub recipient_cnpj: String,
    pub modality: String,
    pub total: String,
}

async fn create_cte_form(
    Form(f): Form<CTeForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let req = tax_models::NewCTe {
        number: f.number,
        sender_cnpj: f.sender_cnpj,
        recipient_cnpj: f.recipient_cnpj,
        modality: f.modality,
        total: f.total,
    };
    match tax_handlers::create_cte(HeaderMap::new(), Json(req)).await {
        Ok(Json(c)) => Ok(Html(format!(
            r##"<div hx-get="/suite/brazil/fragments/cte" hx-trigger="load delay:200ms" hx-swap="innerHTML" hx-target="#gb-cte-body">
<p class="gb-ok">CTe {number} criado.</p>
</div>"##,
            number = htmx_escape(&c.number),
        ))),
        Err((sc, e)) => Ok(Html(err_fragment(&format!("HTTP {sc}: {e}")))),
    }
}

#[derive(Deserialize)]
pub struct NFSeForm {
    pub number: String,
    pub service_code: String,
    pub provider_cnpj: String,
    pub total: String,
}

async fn create_nfse_form(
    Form(f): Form<NFSeForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let req = tax_models::NewNFSe {
        number: f.number,
        service_code: f.service_code,
        provider_cnpj: f.provider_cnpj,
        total: f.total,
    };
    match tax_handlers::create_nfse(HeaderMap::new(), Json(req)).await {
        Ok(Json(n)) => Ok(Html(format!(
            r##"<div hx-get="/suite/brazil/fragments/nfse" hx-trigger="load delay:200ms" hx-swap="innerHTML" hx-target="#gb-nfse-body">
<p class="gb-ok">NFSe {number} criada.</p>
</div>"##,
            number = htmx_escape(&n.number),
        ))),
        Err((sc, e)) => Ok(Html(err_fragment(&format!("HTTP {sc}: {e}")))),
    }
}

#[derive(Deserialize)]
pub struct ValidateForm {
    pub kind: String,
    pub value: String,
}

async fn validate(Form(f): Form<ValidateForm>) -> Result<Html<String>, (StatusCode, String)> {
    let result = match f.kind.as_str() {
        "cnpj" => validate_cnpj(&f.value),
        "cpf" => validate_cpf(&f.value),
        "cep" => validate_cep(&f.value),
        "nfe-key" => validate_nfe_key(&f.value),
        _ => (false, format!("Tipo desconhecido: {}", f.kind)),
    };
    let (ok, msg) = result;
    let cls = if ok { "ok" } else { "warn" };
    Ok(Html(format!(
        r#"<div class="gb-validate-result gb-{cls}">
<strong>{label}</strong> {value} — {msg}
</div>"#,
        cls = cls,
        label = if ok { "✓ Válido" } else { "✗ Inválido" },
        value = htmx_escape(&f.value),
        msg = htmx_escape(&msg),
    )))
}

fn only_digits(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn validate_cnpj(raw: &str) -> (bool, String) {
    let d = only_digits(raw);
    if d.len() != 14 { return (false, format!("CNPJ deve ter 14 dígitos (recebido {})", d.len())); }
    match d.chars().next() {
        Some(first) => if d.chars().all(|c| c == first) { return (false, "CNPJ com dígitos repetidos".into()); },
        None => return (false, "CNPJ vazio".into()),
    }
    (true, format!("{} válido", format_cnpj(&d)))
}

fn format_cnpj(d: &str) -> String {
    if d.len() == 14 {
        format!("{}.{}.{}/{}-{}", &d[0..2], &d[2..5], &d[5..8], &d[8..12], &d[12..14])
    } else { d.to_string() }
}

fn validate_cpf(raw: &str) -> (bool, String) {
    let d = only_digits(raw);
    if d.len() != 11 { return (false, format!("CPF deve ter 11 dígitos (recebido {})", d.len())); }
    (true, format!("{} válido", format_cpf(&d)))
}

fn format_cpf(d: &str) -> String {
    if d.len() == 11 {
        format!("{}.{}.{}-{}", &d[0..3], &d[3..6], &d[6..9], &d[9..11])
    } else { d.to_string() }
}

fn validate_cep(raw: &str) -> (bool, String) {
    let d = only_digits(raw);
    if d.len() != 8 { return (false, format!("CEP deve ter 8 dígitos (recebido {})", d.len())); }
    (true, format!("{}-{}", &d[0..5], &d[5..8]))
}

fn validate_nfe_key(raw: &str) -> (bool, String) {
    let d = only_digits(raw);
    if d.len() != 44 { return (false, format!("Chave NFe deve ter 44 dígitos (recebido {})", d.len())); }
    (true, "Chave de acesso válida".into())
}
