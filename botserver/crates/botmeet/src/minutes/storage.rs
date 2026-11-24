use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use uuid::Uuid;

use crate::minutes::types::*;

pub(super) fn to_json_value<T: serde::Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or_else(|e| {
        log::warn!("Failed to serialize minutes field: {e}");
        serde_json::Value::Null
    })
}

pub(super) fn from_json_or_default<T>(value: serde_json::Value) -> T
where
    T: serde::de::DeserializeOwned + Default,
{
    serde_json::from_value(value).unwrap_or_else(|e| {
        log::warn!("Failed to deserialize minutes field: {e}");
        T::default()
    })
}

pub struct MinuteStorage;

impl MinuteStorage {
    pub fn save_recording(conn: &mut PgConnection, recording: &MeetRecording) -> Result<()> {
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

    pub fn get_recording(conn: &mut PgConnection, id: Uuid) -> Result<MeetRecording> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            bot_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            meeting_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            recording_path: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
            duration_seconds: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            file_size: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            language: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let row: Row = sql_query("SELECT * FROM meet_recordings WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .get_result(conn)?;

        Ok(MeetRecording {
            id: row.id,
            bot_id: row.bot_id,
            meeting_id: row.meeting_id,
            title: row.title,
            recording_path: row.recording_path,
            duration_seconds: row.duration_seconds,
            file_size: row.file_size,
            language: row.language,
            status: row.status.parse().unwrap_or(RecordingStatus::Recorded),
            created_at: row.created_at,
        })
    }

    pub fn update_recording_status(
        conn: &mut PgConnection,
        id: Uuid,
        status: &RecordingStatus,
    ) -> Result<()> {
        sql_query("UPDATE meet_recordings SET status = $1 WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(&status.to_string())
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(conn)?;
        Ok(())
    }

    pub fn list_recordings(
        conn: &mut PgConnection,
        bot_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MeetRecording>> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            bot_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            meeting_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            recording_path: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
            duration_seconds: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            file_size: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            language: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let rows: Vec<Row> = sql_query("SELECT * FROM meet_recordings WHERE bot_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .get_results(conn)?;

        Ok(rows
            .into_iter()
            .map(|r| MeetRecording {
                id: r.id,
                bot_id: r.bot_id,
                meeting_id: r.meeting_id,
                title: r.title,
                recording_path: r.recording_path,
                duration_seconds: r.duration_seconds,
                file_size: r.file_size,
                language: r.language,
                status: r.status.parse().unwrap_or(RecordingStatus::Recorded),
                created_at: r.created_at,
            })
            .collect())
    }

    pub fn save_transcription(conn: &mut PgConnection, t: &Transcription) -> Result<()> {
        sql_query(
            "INSERT INTO meet_transcriptions (id, recording_id, full_text, segments, speakers, language, confidence, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind::<diesel::sql_types::Uuid, _>(t.id)
        .bind::<diesel::sql_types::Uuid, _>(t.recording_id)
        .bind::<diesel::sql_types::Text, _>(&t.full_text)
        .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(&t.segments))
        .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(&t.speakers))
        .bind::<diesel::sql_types::Text, _>(&t.language)
        .bind::<diesel::sql_types::Double, _>(t.confidence)
        .bind::<diesel::sql_types::Timestamptz, _>(t.created_at)
        .execute(conn)?;
        Ok(())
    }

    pub fn get_transcription(conn: &mut PgConnection, recording_id: Uuid) -> Result<Transcription> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            recording_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            full_text: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            segments: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            speakers: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Text)]
            language: String,
            #[diesel(sql_type = diesel::sql_types::Double)]
            confidence: f64,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
        }

        let row: Row = sql_query("SELECT * FROM meet_transcriptions WHERE recording_id = $1")
            .bind::<diesel::sql_types::Uuid, _>(recording_id)
            .get_result(conn)?;

        Ok(Transcription {
            id: row.id,
            recording_id: row.recording_id,
            full_text: row.full_text,
            segments: from_json_or_default(row.segments),
            speakers: from_json_or_default(row.speakers),
            language: row.language,
            confidence: row.confidence,
            created_at: row.created_at,
        })
    }
}

#[path = "storage_minutes.rs"]
mod storage_minutes;
