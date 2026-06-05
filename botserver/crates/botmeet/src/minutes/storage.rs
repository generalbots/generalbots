use anyhow::Result;
use diesel::prelude::*;
use diesel::sql_query;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use log::{error, info};

use crate::minutes::types::*;

pub struct MinuteStorage;

impl MinuteStorage {
    pub async fn save_recording(conn: &mut PgConnection, recording: &MeetRecording) -> Result<()> {
        sql_query(
            "INSERT INTO meet_recordings (id, bot_id, meeting_id, title, recording_path, duration_seconds, file_size, language, status, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind::<diesel::sql_types::Uuid, _>(recording.id)
        .bind::<diesel::sql_types::Uuid, _>(recording.bot_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(recording.meeting_id)
        .bind::<diesel::sql_types::Text, _>(&recording.title)
        .bind::<diesel::sql_types::Text, _>(&recording.recording_path)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(recording.duration_seconds)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(recording.file_size)
        .bind::<diesel::sql_types::Text, _>(&recording.language)
        .bind::<diesel::sql_types::Text, _>(&recording.status.to_string())
        .bind::<diesel::sql_types::Timestamptz, _>(recording.created_at)
        .execute(conn)?;
        Ok(())
    }

    pub async fn get_recording(conn: &mut PgConnection, id: Uuid) -> Result<MeetRecording> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)] bot_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] meeting_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)] title: String,
            #[diesel(sql_type = diesel::sql_types::Text)] recording_path: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)] duration_seconds: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)] file_size: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Text)] language: String,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: DateTime<Utc>,
        }

        let row: Row = sql_query("SELECT * FROM meet_recordings WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .get_result(conn)?;

        Ok(MeetRecording {
            id: row.id, bot_id: row.bot_id, meeting_id: row.meeting_id,
            title: row.title, recording_path: row.recording_path,
            duration_seconds: row.duration_seconds, file_size: row.file_size,
            language: row.language,
            status: row.status.parse().unwrap_or(RecordingStatus::Recorded),
            created_at: row.created_at,
        })
    }

    pub async fn update_recording_status(conn: &mut PgConnection, id: Uuid, status: &RecordingStatus) -> Result<()> {
        sql_query("UPDATE meet_recordings SET status = $1 WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(&status.to_string())
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(conn)?;
        Ok(())
    }

    pub async fn list_recordings(conn: &mut PgConnection, bot_id: Uuid, limit: i64, offset: i64) -> Result<Vec<MeetRecording>> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)] bot_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] meeting_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)] title: String,
            #[diesel(sql_type = diesel::sql_types::Text)] recording_path: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)] duration_seconds: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)] file_size: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Text)] language: String,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: DateTime<Utc>,
        }

        let rows: Vec<Row> = sql_query("SELECT * FROM meet_recordings WHERE bot_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .get_results(conn)?;

        Ok(rows.into_iter().map(|r| MeetRecording {
            id: r.id, bot_id: r.bot_id, meeting_id: r.meeting_id,
            title: r.title, recording_path: r.recording_path,
            duration_seconds: r.duration_seconds, file_size: r.file_size,
            language: r.language,
            status: r.status.parse().unwrap_or(RecordingStatus::Recorded),
            created_at: r.created_at,
        }).collect())
    }

    pub async fn save_transcription(conn: &mut PgConnection, t: &Transcription) -> Result<()> {
        sql_query(
            "INSERT INTO meet_transcriptions (id, recording_id, full_text, segments, speakers, language, confidence, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind::<diesel::sql_types::Uuid, _>(t.id)
        .bind::<diesel::sql_types::Uuid, _>(t.recording_id)
        .bind::<diesel::sql_types::Text, _>(&t.full_text)
        .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&t.segments).unwrap_or_default())
        .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&t.speakers).unwrap_or_default())
        .bind::<diesel::sql_types::Text, _>(&t.language)
        .bind::<diesel::sql_types::Double, _>(t.confidence)
        .bind::<diesel::sql_types::Timestamptz, _>(t.created_at)
        .execute(conn)?;
        Ok(())
    }

    pub async fn get_transcription(conn: &mut PgConnection, recording_id: Uuid) -> Result<Transcription> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)] recording_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)] full_text: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] segments: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] speakers: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Text)] language: String,
            #[diesel(sql_type = diesel::sql_types::Double)] confidence: f64,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: DateTime<Utc>,
        }

        let row: Row = sql_query("SELECT * FROM meet_transcriptions WHERE recording_id = $1")
            .bind::<diesel::sql_types::Uuid, _>(recording_id)
            .get_result(conn)?;

        Ok(Transcription {
            id: row.id, recording_id: row.recording_id,
            full_text: row.full_text,
            segments: serde_json::from_value(row.segments).unwrap_or_default(),
            speakers: serde_json::from_value(row.speakers).unwrap_or_default(),
            language: row.language, confidence: row.confidence,
            created_at: row.created_at,
        })
    }

    pub async fn save_minute(conn: &mut PgConnection, m: &MeetingMinute) -> Result<()> {
        sql_query(
            "INSERT INTO meet_minutes (id, bot_id, recording_id, meeting_id, title, summary, key_points, action_items, decisions, attendees, duration_minutes, status, version, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"
        )
        .bind::<diesel::sql_types::Uuid, _>(m.id)
        .bind::<diesel::sql_types::Uuid, _>(m.bot_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(m.recording_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(m.meeting_id)
        .bind::<diesel::sql_types::Text, _>(&m.title)
        .bind::<diesel::sql_types::Text, _>(&m.summary)
        .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&m.key_points).unwrap_or_default())
        .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&m.action_items).unwrap_or_default())
        .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&m.decisions).unwrap_or_default())
        .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&m.attendees).unwrap_or_default())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(m.duration_minutes)
        .bind::<diesel::sql_types::Text, _>(&m.status.to_string())
        .bind::<diesel::sql_types::Integer, _>(m.version)
        .bind::<diesel::sql_types::Timestamptz, _>(m.created_at)
        .bind::<diesel::sql_types::Timestamptz, _>(m.updated_at)
        .execute(conn)?;
        Ok(())
    }

    pub async fn get_minute(conn: &mut PgConnection, id: Uuid) -> Result<MeetingMinute> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)] bot_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] recording_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] meeting_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)] title: String,
            #[diesel(sql_type = diesel::sql_types::Text)] summary: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] key_points: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] action_items: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] decisions: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] attendees: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)] duration_minutes: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
            #[diesel(sql_type = diesel::sql_types::Integer)] version: i32,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] updated_at: DateTime<Utc>,
        }

        let row: Row = sql_query("SELECT * FROM meet_minutes WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .get_result(conn)?;

        Ok(MeetingMinute {
            id: row.id, bot_id: row.bot_id, recording_id: row.recording_id,
            meeting_id: row.meeting_id, title: row.title, summary: row.summary,
            key_points: serde_json::from_value(row.key_points).unwrap_or_default(),
            action_items: serde_json::from_value(row.action_items).unwrap_or_default(),
            decisions: serde_json::from_value(row.decisions).unwrap_or_default(),
            attendees: serde_json::from_value(row.attendees).unwrap_or_default(),
            duration_minutes: row.duration_minutes,
            status: row.status.parse().unwrap_or(MinuteStatus::Draft),
            version: row.version, created_at: row.created_at, updated_at: row.updated_at,
        })
    }

    pub async fn list_minutes(conn: &mut PgConnection, bot_id: Uuid, limit: i64, offset: i64) -> Result<Vec<MeetingMinute>> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)] bot_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] recording_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] meeting_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)] title: String,
            #[diesel(sql_type = diesel::sql_types::Text)] summary: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] key_points: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] action_items: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] decisions: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] attendees: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)] duration_minutes: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
            #[diesel(sql_type = diesel::sql_types::Integer)] version: i32,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] updated_at: DateTime<Utc>,
        }

        let rows: Vec<Row> = sql_query("SELECT * FROM meet_minutes WHERE bot_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .get_results(conn)?;

        Ok(rows.into_iter().map(|r| MeetingMinute {
            id: r.id, bot_id: r.bot_id, recording_id: r.recording_id,
            meeting_id: r.meeting_id, title: r.title, summary: r.summary,
            key_points: serde_json::from_value(r.key_points).unwrap_or_default(),
            action_items: serde_json::from_value(r.action_items).unwrap_or_default(),
            decisions: serde_json::from_value(r.decisions).unwrap_or_default(),
            attendees: serde_json::from_value(r.attendees).unwrap_or_default(),
            duration_minutes: r.duration_minutes,
            status: r.status.parse().unwrap_or(MinuteStatus::Draft),
            version: r.version, created_at: r.created_at, updated_at: r.updated_at,
        }).collect())
    }

    pub async fn update_minute(conn: &mut PgConnection, id: Uuid, req: &UpdateMinutesRequest) -> Result<()> {
        if let Some(ref title) = req.title {
            sql_query("UPDATE meet_minutes SET title = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(title)
                .bind::<diesel::sql_types::Uuid, _>(id).execute(conn)?;
        }
        if let Some(ref summary) = req.summary {
            sql_query("UPDATE meet_minutes SET summary = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(summary)
                .bind::<diesel::sql_types::Uuid, _>(id).execute(conn)?;
        }
        if let Some(ref kp) = req.key_points {
            sql_query("UPDATE meet_minutes SET key_points = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(kp).unwrap_or_default())
                .bind::<diesel::sql_types::Uuid, _>(id).execute(conn)?;
        }
        if let Some(ref ai) = req.action_items {
            sql_query("UPDATE meet_minutes SET action_items = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(ai).unwrap_or_default())
                .bind::<diesel::sql_types::Uuid, _>(id).execute(conn)?;
        }
        if let Some(ref dec) = req.decisions {
            sql_query("UPDATE meet_minutes SET decisions = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(dec).unwrap_or_default())
                .bind::<diesel::sql_types::Uuid, _>(id).execute(conn)?;
        }
        if let Some(ref att) = req.attendees {
            sql_query("UPDATE meet_minutes SET attendees = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(att).unwrap_or_default())
                .bind::<diesel::sql_types::Uuid, _>(id).execute(conn)?;
        }
        sql_query("UPDATE meet_minutes SET updated_at = NOW(), version = version + 1 WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id).execute(conn)?;
        Ok(())
    }

    pub async fn finalize_minute(conn: &mut PgConnection, id: Uuid) -> Result<()> {
        sql_query("UPDATE meet_minutes SET status = 'final', updated_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id).execute(conn)?;
        Ok(())
    }

    pub async fn save_signature(conn: &mut PgConnection, sig: &MinuteSignature) -> Result<()> {
        sql_query(
            "INSERT INTO meet_minute_signatures (id, minute_id, user_id, signature_id, signed_hash, signed_at, ip_address)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind::<diesel::sql_types::Uuid, _>(sig.id)
        .bind::<diesel::sql_types::Uuid, _>(sig.minute_id)
        .bind::<diesel::sql_types::Uuid, _>(sig.user_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(sig.signature_id)
        .bind::<diesel::sql_types::Text, _>(&sig.signed_hash)
        .bind::<diesel::sql_types::Timestamptz, _>(sig.signed_at)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(sig.ip_address.as_ref())
        .execute(conn)?;
        Ok(())
    }

    pub async fn get_signatures(conn: &mut PgConnection, minute_id: Uuid) -> Result<Vec<MinuteSignature>> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)] minute_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)] user_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] signature_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)] signed_hash: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] signed_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] ip_address: Option<String>,
        }

        let rows: Vec<Row> = sql_query("SELECT * FROM meet_minute_signatures WHERE minute_id = $1 ORDER BY signed_at")
            .bind::<diesel::sql_types::Uuid, _>(minute_id)
            .get_results(conn)?;

        Ok(rows.into_iter().map(|r| MinuteSignature {
            id: r.id, minute_id: r.minute_id, user_id: r.user_id,
            signature_id: r.signature_id, signed_hash: r.signed_hash,
            signed_at: r.signed_at, ip_address: r.ip_address,
        }).collect())
    }
}
