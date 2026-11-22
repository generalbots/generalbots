use super::*;

pub(super) fn span_finish_from(result: &StepResult, vm_seconds: i32) -> SpanFinish {
    match result {
        Ok((output, ti, to)) => SpanFinish {
            ok: true,
            output: Some(output.clone()),
            error: None,
            tokens_in: *ti,
            tokens_out: *to,
            vm_seconds,
        },
        Err(e) => SpanFinish {
            ok: false,
            output: None,
            error: Some(e.clone()),
            tokens_in: 0,
            tokens_out: 0,
            vm_seconds,
        },
    }
}

pub(super) fn insert_span(
    conn: &mut diesel::PgConnection,
    run_id: Uuid,
    parent_id: Option<Uuid>,
    kind: &str,
    name: &str,
    input_ref: &str,
) -> Option<Uuid> {
    let span_id = Uuid::new_v4();
    let new_span = NewAgentSpan {
        id: span_id,
        run_id,
        parent_id,
        kind: kind.to_string(),
        name: name.to_string(),
        input_ref: Some(input_ref.to_string()),
        status: "running".to_string(),
        started_at: Some(chrono::Utc::now()),
    };
    match diesel::insert_into(agent_spans::dsl::agent_spans)
        .values(&new_span)
        .execute(conn)
    {
        Ok(_) => Some(span_id),
        Err(e) => {
            tracing::error!("automation run {run_id}: span insert failed: {e}");
            None
        }
    }
}

pub(super) fn finish_span(conn: &mut diesel::PgConnection, span_id: Uuid, finish: SpanFinish) {
    let preview = finish
        .output
        .map(|o| o.chars().take(OUTPUT_PREVIEW_CHARS).collect::<String>());
    if let Err(e) = diesel::update(agent_spans::dsl::agent_spans.find(span_id))
        .set((
            agent_spans::dsl::status.eq(if finish.ok { "ok" } else { "failed" }),
            agent_spans::dsl::output_ref.eq(preview),
            agent_spans::dsl::error.eq(finish.error),
            agent_spans::dsl::tokens_in.eq(Some(finish.tokens_in)),
            agent_spans::dsl::tokens_out.eq(Some(finish.tokens_out)),
            agent_spans::dsl::vm_seconds.eq(Some(finish.vm_seconds)),
            agent_spans::dsl::finished_at.eq(diesel::dsl::now),
        ))
        .execute(conn)
    {
        tracing::error!("automation span {span_id}: finish failed: {e}");
    }
}
