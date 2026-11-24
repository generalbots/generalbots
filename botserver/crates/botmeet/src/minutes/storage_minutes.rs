use super::{from_json_or_default, to_json_value, MinuteStorage};
use crate::minutes::types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use uuid::Uuid;

impl MinuteStorage {
    pub fn save_minute(conn: &mut PgConnection, m: &MeetingMinute) -> Result<()> {
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
        .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(&m.key_points))
        .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(&m.action_items))
        .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(&m.decisions))
        .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(&m.attendees))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(m.duration_minutes)
        .bind::<diesel::sql_types::Text, _>(&m.status.to_string())
        .bind::<diesel::sql_types::Integer, _>(m.version)
        .bind::<diesel::sql_types::Timestamptz, _>(m.created_at)
        .bind::<diesel::sql_types::Timestamptz, _>(m.updated_at)
        .execute(conn)?;
        Ok(())
    }

    pub fn get_minute(conn: &mut PgConnection, id: Uuid) -> Result<MeetingMinute> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            bot_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            recording_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            meeting_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            summary: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            key_points: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            action_items: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            decisions: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            attendees: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
            duration_minutes: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            version: i32,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: DateTime<Utc>,
        }

        let row: Row = sql_query("SELECT * FROM meet_minutes WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .get_result(conn)?;

        Ok(MeetingMinute {
            id: row.id,
            bot_id: row.bot_id,
            recording_id: row.recording_id,
            meeting_id: row.meeting_id,
            title: row.title,
            summary: row.summary,
            key_points: from_json_or_default(row.key_points),
            action_items: from_json_or_default(row.action_items),
            decisions: from_json_or_default(row.decisions),
            attendees: from_json_or_default(row.attendees),
            duration_minutes: row.duration_minutes,
            status: row.status.parse().unwrap_or(MinuteStatus::Draft),
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub fn list_minutes(
        conn: &mut PgConnection,
        bot_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MeetingMinute>> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            bot_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            recording_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            meeting_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            summary: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            key_points: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            action_items: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            decisions: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            attendees: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
            duration_minutes: Option<i32>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            status: String,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            version: i32,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: DateTime<Utc>,
        }

        let rows: Vec<Row> = sql_query("SELECT * FROM meet_minutes WHERE bot_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .get_results(conn)?;

        Ok(rows
            .into_iter()
            .map(|r| MeetingMinute {
                id: r.id,
                bot_id: r.bot_id,
                recording_id: r.recording_id,
                meeting_id: r.meeting_id,
                title: r.title,
                summary: r.summary,
                key_points: from_json_or_default(r.key_points),
                action_items: from_json_or_default(r.action_items),
                decisions: from_json_or_default(r.decisions),
                attendees: from_json_or_default(r.attendees),
                duration_minutes: r.duration_minutes,
                status: r.status.parse().unwrap_or(MinuteStatus::Draft),
                version: r.version,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub fn update_minute(
        conn: &mut PgConnection,
        id: Uuid,
        req: &UpdateMinutesRequest,
    ) -> Result<()> {
        if let Some(ref title) = req.title {
            sql_query("UPDATE meet_minutes SET title = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(title)
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)?;
        }
        if let Some(ref summary) = req.summary {
            sql_query("UPDATE meet_minutes SET summary = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(summary)
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)?;
        }
        if let Some(ref kp) = req.key_points {
            sql_query("UPDATE meet_minutes SET key_points = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(kp))
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)?;
        }
        if let Some(ref ai) = req.action_items {
            sql_query("UPDATE meet_minutes SET action_items = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(ai))
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)?;
        }
        if let Some(ref dec) = req.decisions {
            sql_query("UPDATE meet_minutes SET decisions = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(dec))
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)?;
        }
        if let Some(ref att) = req.attendees {
            sql_query("UPDATE meet_minutes SET attendees = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Jsonb, _>(&to_json_value(att))
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)?;
        }
        sql_query(
            "UPDATE meet_minutes SET updated_at = NOW(), version = version + 1 WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(conn)?;
        Ok(())
    }

    pub fn finalize_minute(conn: &mut PgConnection, id: Uuid) -> Result<()> {
        Self::set_minute_status(conn, id, &MinuteStatus::Final)
    }

    pub fn set_minute_status(
        conn: &mut PgConnection,
        id: Uuid,
        status: &MinuteStatus,
    ) -> Result<()> {
        sql_query("UPDATE meet_minutes SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(&status.to_string())
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(conn)?;
        Ok(())
    }

    pub fn save_signature(conn: &mut PgConnection, sig: &MinuteSignature) -> Result<()> {
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

    pub fn get_signatures(
        conn: &mut PgConnection,
        minute_id: Uuid,
    ) -> Result<Vec<MinuteSignature>> {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            minute_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            user_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            signature_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            signed_hash: String,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            signed_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            ip_address: Option<String>,
        }

        let rows: Vec<Row> = sql_query(
            "SELECT * FROM meet_minute_signatures WHERE minute_id = $1 ORDER BY signed_at",
        )
        .bind::<diesel::sql_types::Uuid, _>(minute_id)
        .get_results(conn)?;

        Ok(rows
            .into_iter()
            .map(|r| MinuteSignature {
                id: r.id,
                minute_id: r.minute_id,
                user_id: r.user_id,
                signature_id: r.signature_id,
                signed_hash: r.signed_hash,
                signed_at: r.signed_at,
                ip_address: r.ip_address,
            })
            .collect())
    }
}
