/* =============================================================================
 * ui_fragments/timeclock.rs — clock in/out, overtime, justifications
 * =============================================================================*/
use super::*;
use axum::Json;
use crate::db;
use crate::timeclock;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/suite/timeclock/fragments/dashboard", get(dashboard))
        .route("/suite/timeclock/fragments/today", get(today))
        .route("/suite/timeclock/fragments/records", get(records))
        .route("/suite/timeclock/fragments/overtime", get(overtime))
        .route("/suite/timeclock/fragments/justifications", get(justifications))
        .route("/suite/timeclock/fragments/schedule", get(schedule))
        .route("/suite/timeclock/fragments/holidays", get(holidays))
        .route("/suite/timeclock/fragments/team", get(team))
        .route("/suite/timeclock/fragments/audit", get(audit))
        .route("/suite/timeclock/fragments/reports", get(reports))
        .route("/api/timeclock/forms/clock-in", post(clock_in))
        .route("/api/timeclock/forms/clock-out", post(clock_out))
        .route("/api/timeclock/forms/break-start", post(break_start))
        .route("/api/timeclock/forms/break-end", post(break_end))
        .route("/api/timeclock/forms/justification", post(submit_justification))
        .route("/api/timeclock/forms/overtime", post(submit_overtime))
        .route("/api/timeclock/forms/overtime/{id}/approve", post(approve_overtime))
}

async fn dashboard() -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(format!(
        r##"<section class="tc-dashboard" hx-get="/suite/timeclock/fragments/dashboard" hx-trigger="every 30s" hx-swap="outerHTML">
  <div class="tc-kpi-grid">
<article class="tc-kpi"><span class="tc-kpi-label">Horas hoje</span><span class="tc-kpi-value blue">7h 32m</span></article>
<article class="tc-kpi"><span class="tc-kpi-label">Saldo banco</span><span class="tc-kpi-value green">+12h 15m</span></article>
<article class="tc-kpi"><span class="tc-kpi-label">Férias disponíveis</span><span class="tc-kpi-value orange">22 dias</span></article>
<article class="tc-kpi"><span class="tc-kpi-label">Pendências</span><span class="tc-kpi-value red">2</span></article>
  </div>
  <div class="tc-quick-actions">
<button class="tc-btn tc-btn-success" hx-post="/api/timeclock/forms/clock-in" hx-swap="none">Registrar entrada</button>
<button class="tc-btn tc-btn-warn" hx-post="/api/timeclock/forms/break-start" hx-swap="none">Iniciar pausa</button>
<button class="tc-btn tc-btn-secondary" hx-post="/api/timeclock/forms/break-end" hx-swap="none">Encerrar pausa</button>
<button class="tc-btn tc-btn-danger" hx-post="/api/timeclock/forms/clock-out" hx-swap="none">Registrar saída</button>
  </div>
  <section class="tc-today-events" hx-get="/suite/timeclock/fragments/today" hx-trigger="load delay:200ms" hx-swap="innerHTML">
<h3>Eventos de hoje</h3>
<p class="tc-loading">Carregando…</p>
  </section>
</section>"##
    )))
}

async fn today() -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(r#"<ol class="tc-timeline">
<li class="tc-tl-in"><time>08:02</time> Entrada — <span class="tc-ok">no horário</span></li>
<li class="tc-tl-break"><time>12:15</time> Início pausa — almoço</li>
<li class="tc-tl-break-end"><time>13:08</time> Retorno — 53 min de pausa</li>
<li class="tc-tl-pending">… próximo: saída prevista às 17:30</li>
</ol>"#.to_string()))
}

async fn records() -> Result<Html<String>, (StatusCode, String)> {
    match timeclock::list_records().await {
        Ok(Json(items)) => Ok(Html(render_records(&items))),
        Err((_, e)) => Ok(Html(err_fragment(&format!("records: {e}")))),
    }
}

fn render_records(items: &[timeclock::TimeRecord]) -> String {
    if items.is_empty() {
        return r#"<div class="gb-empty">Nenhum registro no período.</div>"#.to_string();
    }
    let rows = items.iter().take(100).map(|r| {
        let in_t = r.clock_in.format("%H:%M");
        let out_t = r.clock_out.map(|t| t.format("%H:%M").to_string()).unwrap_or_else(|| "—".into());
        let cls = if r.status == "complete" { "ok" } else { "warn" };
        format!(
            r#"<tr>
<td>{date}</td><td>{in_t}</td><td>{out_t}</td><td>{hours}</td>
<td><span class="gb-badge {cls}">{status}</span></td>
</tr>"#,
            date = r.date,
            in_t = in_t,
            out_t = out_t,
            hours = htmx_escape(&r.hours_worked),
            status = htmx_escape(&r.status),
            cls = cls,
        )
    }).collect::<Vec<_>>().join("\n");
    format!(
        r##"<div class="gb-table-wrapper"><table class="gb-table">
<thead><tr><th>Data</th><th>Entrada</th><th>Saída</th><th>Horas</th><th>Status</th></tr></thead>
<tbody>{rows}</tbody>
<tfoot><tr><td colspan="5">{count} registro(s)</td></tr></tfoot>
</table></div>"##,
        rows = rows,
        count = items.len(),
    )
}

async fn overtime() -> Result<Html<String>, (StatusCode, String)> {
    match timeclock::list_overtime().await {
        Ok(Json(items)) => Ok(Html(render_overtime(&items))),
        Err((_, e)) => Ok(Html(err_fragment(&format!("overtime: {e}")))),
    }
}

fn render_overtime(items: &[timeclock::OvertimeRequest]) -> String {
    if items.is_empty() {
        return r#"<div class="gb-empty">Nenhuma hora extra solicitada.</div>"#.to_string();
    }
    let rows = items.iter().map(|o| {
        let cls = match o.status.as_str() {
            "approved" => "ok",
            "rejected" => "err",
            _ => "warn",
        };
        let actions = if o.status == "pending" {
            format!(
                r#"<button class="gb-btn gb-btn-success" hx-post="/api/timeclock/forms/overtime/{id}/approve" hx-swap="none" hx-trigger="click" hx-on::after-request="htmx.ajax('GET', '/suite/timeclock/fragments/overtime', '#tc-overtime-body')">Aprovar</button>"#,
                id = o.id,
            )
        } else { String::new() };
        format!(
            r#"<tr>
<td>{date}</td><td>{hours}h</td><td>{reason}</td>
<td><span class="gb-badge {cls}">{status}</span></td>
<td>{actions}</td>
</tr>"#,
            date = o.date,
            hours = htmx_escape(&o.hours),
            reason = htmx_escape(&o.reason),
            status = htmx_escape(&o.status),
            cls = cls,
            actions = actions,
        )
    }).collect::<Vec<_>>().join("\n");
    format!(
        r##"<div class="gb-table-wrapper"><table class="gb-table">
<thead><tr><th>Data</th><th>Horas</th><th>Motivo</th><th>Status</th><th>Ações</th></tr></thead>
<tbody>{rows}</tbody></table></div>"##,
        rows = rows,
    )
}

async fn justifications() -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(r##"<form hx-post="/api/timeclock/forms/justification" hx-swap="outerHTML" hx-target="#tc-just-result" class="gb-form-grid">
  <label>Data<input type="date" name="date" required></label>
  <label>Tipo
<select name="kind" required>
  <option value="atraso">Atraso</option>
  <option value="falta">Falta</option>
  <option value="saida_antecipada">Saída antecipada</option>
  <option value="esquecimento">Esquecimento de registro</option>
</select>
  </label>
  <label class="full">Justificativa<textarea name="reason" rows="3" required></textarea></label>
  <label class="full">Anexo (opcional)<input type="file" name="attachment" accept=".pdf,.png,.jpg"></label>
  <div class="gb-form-actions">
<button type="submit" class="gb-btn gb-btn-primary">Enviar justificativa</button>
  </div>
</form>
<div id="tc-just-result"></div>"##.to_string()))
}

async fn schedule() -> Result<Html<String>, (StatusCode, String)> {
    let days = ["Seg", "Ter", "Qua", "Qui", "Sex", "Sáb", "Dom"];
    let shifts = ["08:00–12:00 / 13:00–17:30"; 7];
    let grid = days.iter().zip(shifts.iter()).map(|(d, s)| {
        format!(
            r#"<article class="tc-schedule-day"><h4>{d}</h4><p>{s}</p><p class="tc-hours">8h 30m</p></article>"#,
            d = d, s = s,
        )
    }).collect::<String>();
    Ok(Html(format!(
        r##"<div class="tc-schedule-grid">{grid}</div>
<div class="tc-schedule-actions">
  <button class="gb-btn">Trocar turno</button>
  <button class="gb-btn">Solicitar folga</button>
  <button class="gb-btn gb-btn-primary">Salvar escala</button>
</div>"##,
        grid = grid,
    )))
}

async fn holidays(Query(q): Query<HashMap<String, String>>) -> Result<Html<String>, (StatusCode, String)> {
    let year = q.get("year").cloned().unwrap_or_else(|| Utc::now().format("%Y").to_string());
    Ok(Html(format!(
        r##"<div class="tc-holidays">
  <header class="gb-toolbar">
<select hx-get="/suite/timeclock/fragments/holidays" hx-trigger="change" hx-target="#tc-holidays-body" hx-swap="innerHTML" name="year">
  <option {sel_2024}>2024</option><option {sel_2025} selected>2025</option><option {sel_2026}>2026</option>
</select>
  </header>
  <div id="tc-holidays-body">
<ul class="tc-holiday-list">
  <li><time>01/01</time> Confraternização universal</li>
  <li><time>21/04</time> Tiradentes</li>
  <li><time>01/05</time> Dia do trabalho</li>
  <li><time>07/09</time> Independência</li>
  <li><time>12/10</time> Nossa Senhora Aparecida</li>
  <li><time>02/11</time> Finados</li>
  <li><time>15/11</time> Proclamação da República</li>
  <li><time>25/12</time> Natal</li>
</ul>
  </div>
</div>"##,
        sel_2024 = if year == "2024" { "selected" } else { "" },
        sel_2025 = if year == "2025" { "selected" } else { "" },
        sel_2026 = if year == "2026" { "selected" } else { "" },
    )))
}

async fn team() -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(r#"<div class="tc-team-grid">
<article class="tc-team-member"><span class="tc-team-status online"></span>Alice Silva — Eng. — entrada 08:01</article>
<article class="tc-team-member"><span class="tc-team-status online"></span>Bruno Costa — Eng. — entrada 08:15</article>
<article class="tc-team-member"><span class="tc-team-status break"></span>Carla Souza — Design — em pausa desde 12:10</article>
<article class="tc-team-member"><span class="tc-team-status offline"></span>Diego Lima — Vendas — ausente</article>
<article class="tc-team-member"><span class="tc-team-status online"></span>Elisa Rocha — RH — entrada 07:55</article>
</div>"#.to_string()))
}

async fn audit() -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(r#"<div class="gb-table-wrapper"><table class="gb-table">
<thead><tr><th>Data/hora</th><th>Operador</th><th>Ação</th><th>Alvo</th><th>IP</th><th>Resultado</th></tr></thead>
<tbody>
<tr><td>2025-05-22 14:32:11</td><td>alice@gb</td><td>Aprovou hora extra</td><td>overtime/123</td><td>10.x.x.x</td><td><span class="gb-badge ok">OK</span></td></tr>
<tr><td>2025-05-22 13:18:00</td><td>alice@gb</td><td>Editou registro</td><td>record/456</td><td>10.x.x.x</td><td><span class="gb-badge ok">OK</span></td></tr>
<tr><td>2025-05-22 11:05:42</td><td>system</td><td>Sync drive</td><td>drive/timeclock</td><td>—</td><td><span class="gb-badge ok">OK</span></td></tr>
</tbody></table></div>"#.to_string()))
}

async fn reports() -> Result<Html<String>, (StatusCode, String)> {
    match timeclock::get_reports().await {
        Ok(Json(items)) => Ok(Html(render_reports(&items))),
        Err((_, e)) => Ok(Html(err_fragment(&format!("reports: {e}")))),
    }
}

fn render_reports(items: &[timeclock::Report]) -> String {
    if items.is_empty() {
        return r#"<div class="gb-empty">Nenhum relatório gerado ainda.</div>"#.to_string();
    }
    let rows = items.iter().map(|r| {
        format!(
            r#"<tr><td>{period}</td><td>{total}h</td><td>{overtime}h</td><td>{employees}</td><td>{created}</td></tr>"#,
            period = htmx_escape(&r.period),
            total = htmx_escape(&r.total_hours),
            overtime = htmx_escape(&r.overtime_hours),
            employees = r.employees,
            created = r.created_at.format("%Y-%m-%d"),
        )
    }).collect::<String>();
    format!(
        r##"<div class="gb-table-wrapper"><table class="gb-table">
<thead><tr><th>Período</th><th>Horas totais</th><th>Horas extras</th><th>Colaboradores</th><th>Gerado em</th></tr></thead>
<tbody>{rows}</tbody></table></div>"##,
        rows = rows,
    )
}

#[derive(Deserialize)]
pub struct ClockForm {
    pub employee_id: Uuid,
    pub notes: Option<String>,
}

async fn clock_in(Form(f): Form<ClockForm>) -> Result<Html<String>, (StatusCode, String)> {
    let req = timeclock::NewClockEvent { employee_id: f.employee_id, kind: "in".into(), notes: f.notes };
    match timeclock::clock_in_out(Json(req)).await {
        Ok(_) => Ok(Html(r#"<span class="gb-ok">✓ Entrada registrada às <span id="tc-now"></span></span>"#.to_string())),
        Err((sc, e)) => Ok(Html(err_fragment(&format!("HTTP {sc}: {e}")))),
    }
}

async fn clock_out(Form(f): Form<ClockForm>) -> Result<Html<String>, (StatusCode, String)> {
    let req = timeclock::NewClockEvent { employee_id: f.employee_id, kind: "out".into(), notes: f.notes };
    match timeclock::clock_in_out(Json(req)).await {
        Ok(_) => Ok(Html(r#"<span class="gb-ok">✓ Saída registrada</span>"#.to_string())),
        Err((sc, e)) => Ok(Html(err_fragment(&format!("HTTP {sc}: {e}")))),
    }
}

async fn break_start(Form(f): Form<ClockForm>) -> Result<Html<String>, (StatusCode, String)> {
    let req = timeclock::NewClockEvent { employee_id: f.employee_id, kind: "break_start".into(), notes: f.notes };
    match timeclock::clock_in_out(Json(req)).await {
        Ok(_) => Ok(Html(r#"<span class="gb-ok">✓ Início de pausa</span>"#.to_string())),
        Err((sc, e)) => Ok(Html(err_fragment(&format!("HTTP {sc}: {e}")))),
    }
}

async fn break_end(Form(f): Form<ClockForm>) -> Result<Html<String>, (StatusCode, String)> {
    let req = timeclock::NewClockEvent { employee_id: f.employee_id, kind: "break_end".into(), notes: f.notes };
    match timeclock::clock_in_out(Json(req)).await {
        Ok(_) => Ok(Html(r#"<span class="gb-ok">✓ Pausa encerrada</span>"#.to_string())),
        Err((sc, e)) => Ok(Html(err_fragment(&format!("HTTP {sc}: {e}")))),
    }
}

#[derive(Deserialize)]
pub struct JustificationForm {
    pub date: String,
    pub kind: String,
    pub reason: String,
}

async fn submit_justification(Form(f): Form<JustificationForm>) -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(format!(
        r#"<div class="gb-form-result">
<strong>✓ Justificativa enviada</strong> — {kind} em {date}
<p class="gb-muted">Aguardando aprovação do gestor.</p>
</div>"#,
        kind = htmx_escape(&f.kind),
        date = htmx_escape(&f.date),
    )))
}

#[derive(Deserialize)]
pub struct OvertimeForm {
    pub employee_id: Uuid,
    pub date: String,
    pub hours: String,
    pub reason: String,
}

async fn submit_overtime(Form(_f): Form<OvertimeForm>) -> Result<Html<String>, (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool: {e}")))?;
    ensure_overtime_table(&mut conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let id = Uuid::new_v4();
    let hours_dec = Decimal::from_str_exact(&_f.hours)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid hours: {e}")))?;
    diesel::sql_query(
        "INSERT INTO timeclock_overtime (id, employee_id, date, hours, reason, status, created_at)
         VALUES ($1, $2, $3, $4, $5, 'pending', NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(_f.employee_id)
    .bind::<diesel::sql_types::Date, _>(chrono::NaiveDate::parse_from_str(&_f.date, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid date: {e}")))?)
    .bind::<diesel::sql_types::Numeric, _>(hours_dec)
    .bind::<diesel::sql_types::Text, _>(&_f.reason)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Html(r#"<div class="gb-form-result"><strong>✓ Hora extra solicitada</strong> — aguardando aprovação.</div>"#.to_string()))
}

async fn approve_overtime(Path(id): Path<String>) -> Result<Html<String>, (StatusCode, String)> {
    match timeclock::approve_overtime(Path(id)).await {
        Ok(_) => Ok(Html(r#"<span class="gb-ok">✓ Aprovado</span>"#.to_string())),
        Err((sc, e)) => Ok(Html(err_fragment(&format!("HTTP {sc}: {e}")))),
    }
}

fn ensure_overtime_table(conn: &mut diesel::PgConnection) -> Result<(), String> {
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS timeclock_overtime (
            id UUID PRIMARY KEY,
            employee_id UUID NOT NULL,
            date DATE NOT NULL,
            hours NUMERIC(8,2) NOT NULL DEFAULT 0,
            reason TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            approved_by UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(conn)
    .map_err(|e| format!("{e}"))?;
    Ok(())
}
