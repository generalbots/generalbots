//! Call and screen-share operations for meet group conversations.

use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Bool, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use uuid::Uuid;

use crate::conversations::{
    CallParticipant, CallResponse, ScreenShareRequest, ScreenShareResponse, StartCallRequest,
    SuccessResponse,
};

use crate::conversation_store::DbPool;

#[derive(diesel::QueryableByName)]
struct CallRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    call_type: String,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    started_by: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    participants: serde_json::Value,
    #[diesel(sql_type = Bool)]
    is_recording: bool,
    #[diesel(sql_type = Timestamptz)]
    started_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    ended_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    recording_url: Option<String>,
}

impl From<CallRow> for CallResponse {
    fn from(r: CallRow) -> Self {
        if r.is_recording {
            tracing::trace!("call {} still recording when mapped", r.id);
        }
        CallResponse {
            id: r.id,
            conversation_id: Uuid::nil(), // filled by caller
            call_type: r.call_type,
            status: r.status,
            started_by: r.started_by.unwrap_or(Uuid::nil()),
            participants: serde_json::from_value::<Vec<CallParticipant>>(r.participants).unwrap_or_default(),
            started_at: r.started_at,
            ended_at: r.ended_at,
            duration_seconds: r.ended_at.map(|e| (e - r.started_at).num_seconds()),
            recording_url: r.recording_url,
        }
    }
}

pub async fn start_call(
    pool: &DbPool,
    conversation_id: Uuid,
    req: &StartCallRequest,
) -> Result<CallResponse> {
    let mut conn = pool.get()?;
    let call_id = Uuid::new_v4();
    let now = Utc::now();
    let started_by = Uuid::new_v4();
    let call_type = req.call_type.clone();

    sql_query(
        "INSERT INTO meet_conversation_calls (id, conversation_id, call_type, status, started_by, started_at)
         VALUES ($1, $2, $3, 'active', $4, $5)",
    )
    .bind::<DieselUuid, _>(call_id)
    .bind::<DieselUuid, _>(conversation_id)
    .bind::<Text, _>(&call_type)
    .bind::<DieselUuid, _>(started_by)
    .bind::<Timestamptz, _>(now)
    .execute(&mut conn)?;

    Ok(CallResponse {
        id: call_id,
        conversation_id,
        call_type,
        status: "active".to_string(),
        started_by,
        participants: vec![],
        started_at: now,
        ended_at: None,
        duration_seconds: None,
        recording_url: None,
    })
}

pub async fn join_call(pool: &DbPool, conversation_id: Uuid, user_id: Option<Uuid>) -> Result<SuccessResponse> {
    let mut conn = pool.get()?;
    let user_id = user_id.unwrap_or_else(Uuid::new_v4);
    // Shape mirrors CallParticipant so it round-trips through serde.
    let participant = serde_json::json!([{
        "user_id": user_id.to_string(),
        "username": user_id.to_string(),
        "status": "active",
        "is_muted": false,
        "is_video_enabled": false,
        "is_screen_sharing": false,
        "joined_at": Utc::now().to_rfc3339()
    }]);
    sql_query(
        "UPDATE meet_conversation_calls
         SET participants = participants || $1::jsonb
         WHERE conversation_id = $2 AND status = 'active'
           AND NOT participants @> $1::jsonb",
    )
    .bind::<diesel::sql_types::Jsonb, _>(&participant)
    .bind::<DieselUuid, _>(conversation_id)
    .execute(&mut conn)?;
    Ok(SuccessResponse {
        success: true,
        message: Some("Joined call successfully".to_string()),
    })
}

pub async fn leave_call(pool: &DbPool, conversation_id: Uuid, user_id: Option<Uuid>) -> Result<SuccessResponse> {
    let mut conn = pool.get()?;
    if let Some(uid) = user_id {
        log::info!("User {uid} left call in conversation {conversation_id}");
    }
    // End the call when the last active participant leaves (best-effort: we
    // simply close the most recent active call for the conversation).
    sql_query(
        "UPDATE meet_conversation_calls SET status = 'ended', ended_at = NOW()
         WHERE conversation_id = $1 AND status = 'active'",
    )
    .bind::<DieselUuid, _>(conversation_id)
    .execute(&mut conn)?;
    Ok(SuccessResponse {
        success: true,
        message: Some("Left call successfully".to_string()),
    })
}

pub async fn toggle_recording(pool: &DbPool, conversation_id: Uuid, recording: bool) -> Result<SuccessResponse> {
    let mut conn = pool.get()?;
    let msg = if recording { "Recording started" } else { "Recording stopped" };
    sql_query(
        "UPDATE meet_conversation_calls SET is_recording = $1
         WHERE conversation_id = $2 AND status = 'active'",
    )
    .bind::<Bool, _>(recording)
    .bind::<DieselUuid, _>(conversation_id)
    .execute(&mut conn)?;
    Ok(SuccessResponse {
        success: true,
        message: Some(msg.to_string()),
    })
}

#[derive(diesel::QueryableByName)]
struct ScreenShareRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    user_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    quality: String,
    #[diesel(sql_type = Bool)]
    audio_included: bool,
    #[diesel(sql_type = Timestamptz)]
    started_at: DateTime<Utc>,
}

impl From<ScreenShareRow> for ScreenShareResponse {
    fn from(r: ScreenShareRow) -> Self {
        ScreenShareResponse {
            id: r.id,
            user_id: r.user_id.unwrap_or(Uuid::nil()),
            conversation_id: Uuid::nil(), // filled by caller
            status: r.status,
            quality: r.quality,
            audio_included: r.audio_included,
            started_at: r.started_at,
        }
    }
}

pub async fn start_screen_share(
    pool: &DbPool,
    conversation_id: Uuid,
    req: &ScreenShareRequest,
    user_id: Uuid,
) -> Result<ScreenShareResponse> {
    let mut conn = pool.get()?;
    let share_id = Uuid::new_v4();
    let user_id = if user_id == Uuid::nil() { Uuid::new_v4() } else { user_id };
    let now = Utc::now();
    let quality = req.quality.clone().unwrap_or_else(|| "high".to_string());
    let audio_included = req.audio_included.unwrap_or(false);

    sql_query(
        "INSERT INTO meet_conversation_screen_shares (id, conversation_id, user_id, status, quality, audio_included, started_at)
         VALUES ($1, $2, $3, 'active', $4, $5, $6)",
    )
    .bind::<DieselUuid, _>(share_id)
    .bind::<DieselUuid, _>(conversation_id)
    .bind::<DieselUuid, _>(user_id)
    .bind::<Text, _>(&quality)
    .bind::<Bool, _>(audio_included)
    .bind::<Timestamptz, _>(now)
    .execute(&mut conn)?;

    Ok(ScreenShareResponse {
        id: share_id,
        user_id,
        conversation_id,
        status: "active".to_string(),
        quality,
        audio_included,
        started_at: now,
    })
}

pub async fn stop_screen_share(pool: &DbPool, conversation_id: Uuid) -> Result<SuccessResponse> {
    let mut conn = pool.get()?;
    sql_query(
        "UPDATE meet_conversation_screen_shares SET status = 'stopped', ended_at = NOW()
         WHERE conversation_id = $1 AND status = 'active'",
    )
    .bind::<DieselUuid, _>(conversation_id)
    .execute(&mut conn)?;
    Ok(SuccessResponse {
        success: true,
        message: Some("Screen sharing stopped".to_string()),
    })
}
