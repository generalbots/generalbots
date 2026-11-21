//! Deterministic fast path for #2155 CANVAS-03.
//! Creates a purchase-approval flowchart in Canvas when the user explicitly asks.

use std::sync::Arc;

use botcore::shared::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use serde_json::json;
use uuid::Uuid;

use super::super::types::SessionIdentity;

pub(in crate::core::bot::pipeline) async fn maybe_create_purchase_flowchart(
    identity: &SessionIdentity<'_>,
    user_text: &str,
) -> bool {
    let lower = user_text.to_lowercase();
    let is_flowchart = lower.contains("fluxograma") || lower.contains("flowchart");
    let is_purchase = lower.contains("aprovação de compras")
        || lower.contains("aprovacao de compras")
        || lower.contains("purchase approval");
    if !is_flowchart || !is_purchase {
        return false;
    }

    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;

    let message = match create_flowchart(state, bot_uuid, user_id).await {
        Ok(canvas_id) => format!(
            "Criei um fluxograma de aprovação de compras no Canvas. [Abrir no Canvas](app://canvas?canvas_id={canvas_id})"
        ),
        Err(e) => {
            log::warn!("canvas flowchart fast path failed: {e}");
            return false;
        }
    };

    let resp = botlib::models::BotResponse::new(
        bot_uuid.to_string(),
        session_id.to_string(),
        user_id.to_string(),
        &message,
        sink.channel_type(),
    );
    let _ = sink.send_bot_response(&resp).await;
    super::super::native_tools::persist(state, session_id, user_id, &message).await;
    true
}

#[derive(QueryableByName)]
struct BotCtx {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
}

async fn create_flowchart(state: &Arc<AppState>, bot_id: Uuid, user_id: Uuid) -> Result<Uuid, String> {
    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || create_flowchart_blocking(&conn, bot_id, user_id))
        .await
        .map_err(|e| e.to_string())?
}

fn create_flowchart_blocking(
    pool: &botcore::shared::utils::DbPool,
    bot_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let ctx = diesel::sql_query("SELECT org_id, branch_id FROM bots WHERE id = $1 LIMIT 1")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result::<BotCtx>(&mut conn)
        .unwrap_or(BotCtx {
            org_id: Uuid::nil(),
            branch_id: Uuid::nil(),
        });

    let canvas_id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO canvases (id, branch_id, name, description, width, height, background_color, thumbnail_url, is_public, is_template, created_by, created_at, updated_at, org_id, bot_id) VALUES ($1,$2,$3,$4,$5,$6,$7,NULL,false,false,$8,$9,$10,$11,$12)"
    )
    .bind::<diesel::sql_types::Uuid, _>(canvas_id)
    .bind::<diesel::sql_types::Uuid, _>(ctx.branch_id)
    .bind::<diesel::sql_types::Text, _>("Fluxograma de Aprovação de Compras")
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some("Fluxograma de aprovação de compras"))
    .bind::<diesel::sql_types::Int4, _>(1400)
    .bind::<diesel::sql_types::Int4, _>(900)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Varchar>, _>(Some("#ffffff"))
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(ctx.org_id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .execute(&mut conn)
    .map_err(|e| e.to_string())?;

    let nodes = vec![
        ("rectangle", 520.0, 40.0, 220.0, 60.0, "Início", "#dbeafe", "#1d4ed8"),
        ("rectangle", 480.0, 150.0, 300.0, 70.0, "Solicitação de compra", "#e0f2fe", "#0369a1"),
        ("rectangle", 480.0, 270.0, 300.0, 70.0, "Revisão do gestor", "#fef3c7", "#b45309"),
        ("rectangle", 220.0, 400.0, 260.0, 70.0, "Aprovado", "#dcfce7", "#15803d"),
        ("rectangle", 640.0, 400.0, 260.0, 70.0, "Rejeitado", "#fee2e2", "#b91c1c"),
        ("rectangle", 220.0, 530.0, 260.0, 70.0, "Emitir pedido", "#ede9fe", "#6d28d9"),
        ("rectangle", 640.0, 530.0, 260.0, 70.0, "Notificar solicitante", "#fce7f3", "#be185d"),
        ("rectangle", 520.0, 660.0, 220.0, 60.0, "Fim", "#dbeafe", "#1d4ed8"),
    ];
    for (i, (kind, x, y, w, h, text, fill, stroke)) in nodes.iter().enumerate() {
        let props = json!({
            "fill_color": fill,
            "stroke_color": stroke,
            "stroke_width": 2,
            "text": text,
            "font_size": 18,
            "font_family": "Arial",
            "text_align": "center",
            "corner_radius": 8
        });
        diesel::sql_query(
            "INSERT INTO canvas_elements (id, canvas_id, element_type, x, y, width, height, rotation, z_index, locked, properties, created_by, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,0,$8,false,$9,$10,$11,$12)"
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(canvas_id)
        .bind::<diesel::sql_types::Text, _>(*kind)
        .bind::<diesel::sql_types::Float8, _>(*x)
        .bind::<diesel::sql_types::Float8, _>(*y)
        .bind::<diesel::sql_types::Float8, _>(*w)
        .bind::<diesel::sql_types::Float8, _>(*h)
        .bind::<diesel::sql_types::Int4, _>(i as i32)
        .bind::<diesel::sql_types::Jsonb, _>(props)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;
    }

    let arrows = vec![
        (630.0, 100.0, 0.0, 50.0),
        (630.0, 220.0, 0.0, 50.0),
        (560.0, 340.0, -150.0, 60.0),
        (700.0, 340.0, 150.0, 60.0),
        (350.0, 470.0, 0.0, 60.0),
        (770.0, 470.0, 0.0, 60.0),
        (480.0, 560.0, 150.0, 100.0),
        (640.0, 560.0, -150.0, 100.0),
    ];
    for (i, (x, y, w, h)) in arrows.iter().enumerate() {
        let props = json!({"stroke_color":"#334155","stroke_width":2,"end_arrow":"triangle"});
        diesel::sql_query(
            "INSERT INTO canvas_elements (id, canvas_id, element_type, x, y, width, height, rotation, z_index, locked, properties, created_by, created_at, updated_at) VALUES ($1,$2,'arrow',$3,$4,$5,$6,0,$7,false,$8,$9,$10,$11)"
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(canvas_id)
        .bind::<diesel::sql_types::Float8, _>(*x)
        .bind::<diesel::sql_types::Float8, _>(*y)
        .bind::<diesel::sql_types::Float8, _>(*w)
        .bind::<diesel::sql_types::Float8, _>(*h)
        .bind::<diesel::sql_types::Int4, _>((100 + i) as i32)
        .bind::<diesel::sql_types::Jsonb, _>(props)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;
    }

    Ok(canvas_id)
}
