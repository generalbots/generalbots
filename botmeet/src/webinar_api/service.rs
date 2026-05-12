use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Integer, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::constants::{QA_QUESTION_MAX_LENGTH, MAX_RAISED_HANDS_VISIBLE};
use super::error::WebinarError;
use super::types::{
    CreateWebinarRequest, PanelistInvite, ParticipantRole, ParticipantStatus, QAQuestion,
    QuestionStatus, RegisterRequest, RegistrationStatus, Webinar, WebinarParticipant,
    WebinarRegistration, WebinarSettings, WebinarStatus, WebinarEventType,
};

#[derive(QueryableByName)]
struct WebinarRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    organization_id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    meeting_id: Uuid,
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = Nullable<Text>)]
    description: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    scheduled_start: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    scheduled_end: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    actual_start: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    actual_end: Option<DateTime<Utc>>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    settings_json: String,
    #[diesel(sql_type = Bool)]
    registration_required: bool,
    #[diesel(sql_type = Nullable<Text>)]
    registration_url: Option<String>,
    #[diesel(sql_type = DieselUuid)]
    host_id: Uuid,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    updated_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct ParticipantRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    webinar_id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    user_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Nullable<Text>)]
    email: Option<String>,
    #[diesel(sql_type = Text)]
    role: String,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Bool)]
    hand_raised: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    hand_raised_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Bool)]
    is_speaking: bool,
    #[diesel(sql_type = Bool)]
    video_enabled: bool,
    #[diesel(sql_type = Bool)]
    audio_enabled: bool,
    #[diesel(sql_type = Bool)]
    screen_sharing: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    joined_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    left_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    registration_data: Option<String>,
}

#[derive(QueryableByName)]
struct QuestionRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    webinar_id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    asker_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    asker_name: String,
    #[diesel(sql_type = Bool)]
    is_anonymous: bool,
    #[diesel(sql_type = Text)]
    question: String,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Integer)]
    upvotes: i32,
    #[diesel(sql_type = Nullable<Text>)]
    upvoted_by: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    answer: Option<String>,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    answered_by: Option<Uuid>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    answered_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Bool)]
    is_pinned: bool,
    #[diesel(sql_type = Bool)]
    is_highlighted: bool,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

pub struct WebinarService {
    pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    event_sender: broadcast::Sender<super::types::WebinarEvent>,
}

impl WebinarService {
    pub fn new(
        pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        Self { pool, event_sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<super::types::WebinarEvent> {
        self.event_sender.subscribe()
    }

    pub async fn create_webinar(
        &self,
        organization_id: Uuid,
        host_id: Uuid,
        request: CreateWebinarRequest,
    ) -> Result<Webinar, WebinarError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            WebinarError::DatabaseConnection
        })?;

        let id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let settings = request.settings.unwrap_or_default();
        let settings_json = serde_json::to_string(&settings).unwrap_or_else(|_| "{}".to_string());

        let registration_url = if request.registration_required {
            Some(format!("/webinar/{}/register", id))
        } else {
            None
        };

        let sql = r#"
            INSERT INTO webinars (
                id, organization_id, meeting_id, title, description,
                scheduled_start, scheduled_end, status, settings_json,
                registration_required, registration_url, host_id,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 'scheduled', $8, $9, $10, $11, NOW(), NOW()
            )
        "#;

        diesel::sql_query(sql)
            .bind::<DieselUuid, _>(id)
            .bind::<DieselUuid, _>(organization_id)
            .bind::<DieselUuid, _>(meeting_id)
            .bind::<Text, _>(&request.title)
            .bind::<Nullable<Text>, _>(request.description.as_deref())
            .bind::<Timestamptz, _>(request.scheduled_start)
            .bind::<Nullable<Timestamptz>, _>(request.scheduled_end)
            .bind::<Text, _>(&settings_json)
            .bind::<Bool, _>(request.registration_required)
            .bind::<Nullable<Text>, _>(registration_url.as_deref())
            .bind::<DieselUuid, _>(host_id)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to create webinar: {e}");
                WebinarError::CreateFailed
            })?;

        self.add_participant_internal(
            &mut conn,
            id,
            Some(host_id),
            "Host".to_string(),
            None,
            ParticipantRole::Host,
        )?;

        if let Some(panelists) = request.panelists {
            for panelist in panelists {
                self.add_participant_internal(
                    &mut conn,
                    id,
                    None,
                    panelist.name,
                    Some(panelist.email),
                    panelist.role,
                )?;
            }
        }

        info!("Created webinar {} for org {}", id, organization_id);

        self.get_webinar(id).await
    }

    pub async fn get_webinar(&self, webinar_id: Uuid) -> Result<Webinar, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, organization_id, meeting_id, title, description,
                   scheduled_start, scheduled_end, actual_start, actual_end,
                   status, settings_json, registration_required, registration_url,
                   host_id, created_at, updated_at
            FROM webinars WHERE id = $1
        "#;

        let rows: Vec<WebinarRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(webinar_id)
            .load(&mut conn)
            .map_err(|e| {
                error!("Failed to get webinar: {e}");
                WebinarError::DatabaseConnection
            })?;

        let row = rows.into_iter().next().ok_or(WebinarError::NotFound)?;
        Ok(self.row_to_webinar(row))
    }

    pub async fn start_webinar(&self, webinar_id: Uuid, host_id: Uuid) -> Result<Webinar, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if webinar.host_id != host_id {
            return Err(WebinarError::NotAuthorized);
        }

        if webinar.status != WebinarStatus::Scheduled && webinar.status != WebinarStatus::Paused {
            return Err(WebinarError::InvalidState("Webinar cannot be started".to_string()));
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinars SET status = 'live', actual_start = COALESCE(actual_start, NOW()), updated_at = NOW() WHERE id = $1"
        )
        .bind::<DieselUuid, _>(webinar_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to start webinar: {e}");
            WebinarError::UpdateFailed
        })?;

        self.broadcast_event(WebinarEventType::WebinarStarted, webinar_id, serde_json::json!({}));

        info!("Started webinar {}", webinar_id);
        self.get_webinar(webinar_id).await
    }

    pub async fn end_webinar(&self, webinar_id: Uuid, host_id: Uuid) -> Result<Webinar, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if webinar.host_id != host_id {
            return Err(WebinarError::NotAuthorized);
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinars SET status = 'ended', actual_end = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind::<DieselUuid, _>(webinar_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to end webinar: {e}");
            WebinarError::UpdateFailed
        })?;

        self.broadcast_event(WebinarEventType::WebinarEnded, webinar_id, serde_json::json!({}));

        info!("Ended webinar {}", webinar_id);
        self.get_webinar(webinar_id).await
    }

    pub async fn register_attendee(
        &self,
        webinar_id: Uuid,
        request: RegisterRequest,
    ) -> Result<WebinarRegistration, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if !webinar.registration_required {
            return Err(WebinarError::RegistrationNotRequired);
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let existing: Vec<CountRow> = diesel::sql_query(
            "SELECT COUNT(*) as count FROM webinar_registrations WHERE webinar_id = $1 AND email = $2"
        )
        .bind::<DieselUuid, _>(webinar_id)
        .bind::<Text, _>(&request.email)
        .load(&mut conn)
        .unwrap_or_default();

        if existing.first().map(|r| r.count > 0).unwrap_or(false) {
            return Err(WebinarError::AlreadyRegistered);
        }

        let id = Uuid::new_v4();
        let join_link = format!("/webinar/{}/join?token={}", webinar_id, Uuid::new_v4());
        let custom_fields = request.custom_fields.clone().unwrap_or_default();
        let custom_fields_json = serde_json::to_string(&custom_fields)
            .unwrap_or_else(|_| "{}".to_string());

        let sql = r#"
            INSERT INTO webinar_registrations (
                id, webinar_id, email, name, custom_fields, status, join_link,
                registered_at, confirmed_at
            ) VALUES ($1, $2, $3, $4, $5, 'confirmed', $6, NOW(), NOW())
        "#;

        diesel::sql_query(sql)
            .bind::<DieselUuid, _>(id)
            .bind::<DieselUuid, _>(webinar_id)
            .bind::<Text, _>(&request.email)
            .bind::<Text, _>(&request.name)
            .bind::<Text, _>(&custom_fields_json)
            .bind::<Text, _>(&join_link)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to register: {e}");
                WebinarError::RegistrationFailed
            })?;

        self.add_participant_internal(
            &mut conn,
            webinar_id,
            None,
            request.name.clone(),
            Some(request.email.clone()),
            ParticipantRole::Attendee,
        )?;

        Ok(WebinarRegistration {
            id,
            webinar_id,
            email: request.email,
            name: request.name,
            custom_fields,
            status: RegistrationStatus::Confirmed,
            join_link,
            registered_at: Utc::now(),
            confirmed_at: Some(Utc::now()),
            cancelled_at: None,
        })
    }

    pub async fn join_webinar(
        &self,
        webinar_id: Uuid,
        participant_id: Uuid,
    ) -> Result<WebinarParticipant, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if webinar.status != WebinarStatus::Live && webinar.status != WebinarStatus::Scheduled {
            return Err(WebinarError::InvalidState("Webinar is not active".to_string()));
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let status = if webinar.settings.waiting_room_enabled {
            "in_waiting_room"
        } else {
            "joined"
        };

        diesel::sql_query(
            "UPDATE webinar_participants SET status = $1, joined_at = NOW() WHERE id = $2"
        )
        .bind::<Text, _>(status)
        .bind::<DieselUuid, _>(participant_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to join webinar: {e}");
            WebinarError::JoinFailed
        })?;

        self.broadcast_event(
            WebinarEventType::ParticipantJoined,
            webinar_id,
            serde_json::json!({"participant_id": participant_id}),
        );

        self.get_participant(participant_id).await
    }

    pub async fn raise_hand(&self, webinar_id: Uuid, participant_id: Uuid) -> Result<(), WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if !webinar.settings.allow_hand_raise {
            return Err(WebinarError::FeatureDisabled("Hand raising is disabled".to_string()));
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinar_participants SET hand_raised = TRUE, hand_raised_at = NOW() WHERE id = $1 AND webinar_id = $2"
        )
        .bind::<DieselUuid, _>(participant_id)
        .bind::<DieselUuid, _>(webinar_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to raise hand: {e}");
            WebinarError::UpdateFailed
        })?;

        self.broadcast_event(
            WebinarEventType::HandRaised,
            webinar_id,
            serde_json::json!({"participant_id": participant_id}),
        );

        Ok(())
    }

    pub async fn lower_hand(&self, webinar_id: Uuid, participant_id: Uuid) -> Result<(), WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinar_participants SET hand_raised = FALSE, hand_raised_at = NULL WHERE id = $1 AND webinar_id = $2"
        )
        .bind::<DieselUuid, _>(participant_id)
        .bind::<DieselUuid, _>(webinar_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to lower hand: {e}");
            WebinarError::UpdateFailed
        })?;

        self.broadcast_event(
            WebinarEventType::HandLowered,
            webinar_id,
            serde_json::json!({"participant_id": participant_id}),
        );

        Ok(())
    }

    pub async fn get_raised_hands(&self, webinar_id: Uuid) -> Result<Vec<WebinarParticipant>, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, webinar_id, user_id, name, email, role, status,
                   hand_raised, hand_raised_at, is_speaking, video_enabled,
                   audio_enabled, screen_sharing, joined_at, left_at, registration_data
            FROM webinar_participants
            WHERE webinar_id = $1 AND hand_raised = TRUE
            ORDER BY hand_raised_at ASC
            LIMIT $2
        "#;

        let rows: Vec<ParticipantRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(webinar_id)
            .bind::<Integer, _>(MAX_RAISED_HANDS_VISIBLE as i32)
            .load(&mut conn)
            .unwrap_or_default();

        Ok(rows.into_iter().map(|r| self.row_to_participant(r)).collect())
    }

    pub async fn submit_question(
        &self,
        webinar_id: Uuid,
        asker_id: Option<Uuid>,
        asker_name: String,
        request: super::types::SubmitQuestionRequest,
    ) -> Result<QAQuestion, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if !webinar.settings.allow_qa {
            return Err(WebinarError::FeatureDisabled("Q&A is disabled".to_string()));
        }

        if request.question.len() > QA_QUESTION_MAX_LENGTH {
            return Err(WebinarError::InvalidInput("Question too long".to_string()));
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let id = Uuid::new_v4();
        let is_anonymous = request.is_anonymous.unwrap_or(false) && webinar.settings.anonymous_qa;
        let status = if webinar.settings.moderated_qa { "pending" } else { "approved" };
        let display_name = if is_anonymous { "Anonymous".to_string() } else { asker_name };

        let sql = r#"
            INSERT INTO webinar_questions (
                id, webinar_id, asker_id, asker_name, is_anonymous, question,
                status, upvotes, is_pinned, is_highlighted, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, 0, FALSE, FALSE, NOW())
        "#;

        diesel::sql_query(sql)
            .bind::<DieselUuid, _>(id)
            .bind::<DieselUuid, _>(webinar_id)
            .bind::<Nullable<DieselUuid>, _>(asker_id)
            .bind::<Text, _>(&display_name)
            .bind::<Bool, _>(is_anonymous)
            .bind::<Text, _>(&request.question)
            .bind::<Text, _>(status)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to submit question: {e}");
                WebinarError::CreateFailed
            })?;

        self.broadcast_event(
            WebinarEventType::QuestionSubmitted,
            webinar_id,
            serde_json::json!({"question_id": id}),
        );

        Ok(QAQuestion {
            id,
            webinar_id,
            asker_id,
            asker_name: display_name,
            is_anonymous,
            question: request.question,
            status: if webinar.settings.moderated_qa { QuestionStatus::Pending } else { QuestionStatus::Approved },
            upvotes: 0,
            upvoted_by: vec![],
            answer: None,
            answered_by: None,
            answered_at: None,
            is_pinned: false,
            is_highlighted: false,
            created_at: Utc::now(),
        })
    }

    pub async fn answer_question(
        &self,
        question_id: Uuid,
        answerer_id: Uuid,
        request: super::types::AnswerQuestionRequest,
    ) -> Result<QAQuestion, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let status = if request.mark_as_live.unwrap_or(false) { "answered_live" } else { "answered" };

        diesel::sql_query(
            "UPDATE webinar_questions SET answer = $1, answered_by = $2, answered_at = NOW(), status = $3 WHERE id = $4"
        )
        .bind::<Text, _>(&request.answer)
        .bind::<DieselUuid, _>(answerer_id)
        .bind::<Text, _>(status)
        .bind::<DieselUuid, _>(question_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to answer question: {e}");
            WebinarError::UpdateFailed
        })?;

        self.get_question(question_id).await
    }

    pub async fn upvote_question(&self, question_id: Uuid, voter_id: Uuid) -> Result<QAQuestion, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinar_questions SET upvotes = upvotes + 1, upvoted_by = COALESCE(upvoted_by, '[]')::jsonb || $1::jsonb WHERE id = $2"
        )
        .bind::<Text, _>(serde_json::json!([voter_id]).to_string())
        .bind::<DieselUuid, _>(question_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to upvote question: {e}");
            WebinarError::UpdateFailed
        })?;

        self.get_question(question_id).await
    }

    pub async fn get_questions(&self, webinar_id: Uuid, include_pending: bool) -> Result<Vec<QAQuestion>, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let status_filter = if include_pending { "" } else { "AND status != 'pending'" };

        let sql = format!(r#"
            SELECT id, webinar_id, asker_id, asker_name, is_anonymous, question,
                   status, upvotes, upvoted_by, answer, answered_by, answered_at,
                   is_pinned, is_highlighted, created_at
            FROM webinar_questions
            WHERE webinar_id = $1 {status_filter}
            ORDER BY is_pinned DESC, upvotes DESC, created_at ASC
        "#);

        let rows: Vec<QuestionRow> = diesel::sql_query(&sql)
            .bind::<DieselUuid, _>(webinar_id)
            .load(&mut conn)
            .unwrap_or_default();

        Ok(rows.into_iter().map(|r| self.row_to_question(r)).collect())
    }

    async fn get_question(&self, question_id: Uuid) -> Result<QAQuestion, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, webinar_id, asker_id, asker_name, is_anonymous, question,
                   status, upvotes, upvoted_by, answer, answered_by, answered_at,
                   is_pinned, is_highlighted, created_at
            FROM webinar_questions WHERE id = $1
        "#;

        let rows: Vec<QuestionRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(question_id)
            .load(&mut conn)
            .map_err(|_| WebinarError::DatabaseConnection)?;

        let row = rows.into_iter().next().ok_or(WebinarError::NotFound)?;
        Ok(self.row_to_question(row))
    }

    async fn get_participant(&self, participant_id: Uuid) -> Result<WebinarParticipant, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, webinar_id, user_id, name, email, role, status,
                   hand_raised, hand_raised_at, is_speaking, video_enabled,
                   audio_enabled, screen_sharing, joined_at, left_at, registration_data
            FROM webinar_participants WHERE id = $1
        "#;

        let rows: Vec<ParticipantRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(participant_id)
            .load(&mut conn)
            .map_err(|_| WebinarError::DatabaseConnection)?;

        let row = rows.into_iter().next().ok_or(WebinarError::NotFound)?;
        Ok(self.row_to_participant(row))
    }

    fn add_participant_internal(
        &self,
        conn: &mut diesel::PgConnection,
        webinar_id: Uuid,
        user_id: Option<Uuid>,
        name: String,
        email: Option<String>,
        role: ParticipantRole,
    ) -> Result<Uuid, WebinarError> {
        let id = Uuid::new_v4();

        diesel::sql_query(r#"
            INSERT INTO webinar_participants (
                id, webinar_id, user_id, name, email, role, status,
                hand_raised, is_speaking, video_enabled, audio_enabled, screen_sharing
            ) VALUES ($1, $2, $3, $4, $5, $6, 'registered', FALSE, FALSE, FALSE, FALSE, FALSE)
        "#)
        .bind::<DieselUuid, _>(id)
        .bind::<DieselUuid, _>(webinar_id)
        .bind::<Nullable<DieselUuid>, _>(user_id)
        .bind::<Text, _>(&name)
        .bind::<Nullable<Text>, _>(email.as_deref())
        .bind::<Text, _>(role.to_string())
        .execute(conn)
        .map_err(|e| {
            error!("Failed to add participant: {e}");
            WebinarError::CreateFailed
        })?;

        Ok(id)
    }

    fn broadcast_event(&self, event_type: WebinarEventType, webinar_id: Uuid, data: serde_json::Value) {
        let event = super::types::WebinarEvent {
            event_type,
            webinar_id,
            data,
            timestamp: Utc::now(),
        };
        let _ = self.event_sender.send(event);
    }

    fn row_to_webinar(&self, row: WebinarRow) -> Webinar {
        let settings: WebinarSettings = serde_json::from_str(&row.settings_json).unwrap_or_default();
        let status = match row.status.as_str() {
            "draft" => WebinarStatus::Draft,
            "scheduled" => WebinarStatus::Scheduled,
            "live" => WebinarStatus::Live,
            "paused" => WebinarStatus::Paused,
            "ended" => WebinarStatus::Ended,
            "cancelled" => WebinarStatus::Cancelled,
            _ => WebinarStatus::Draft,
        };

        Webinar {
            id: row.id,
            organization_id: row.organization_id,
            meeting_id: row.meeting_id,
            title: row.title,
            description: row.description,
            scheduled_start: row.scheduled_start,
            scheduled_end: row.scheduled_end,
            actual_start: row.actual_start,
            actual_end: row.actual_end,
            status,
            settings,
            registration_required: row.registration_required,
            registration_url: row.registration_url,
            host_id: row.host_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    fn row_to_participant(&self, row: ParticipantRow) -> WebinarParticipant {
        let role = match row.role.as_str() {
            "host" => ParticipantRole::Host,
            "co_host" => ParticipantRole::CoHost,
            "presenter" => ParticipantRole::Presenter,
            "panelist" => ParticipantRole::Panelist,
            _ => ParticipantRole::Attendee,
        };
        let status = match row.status.as_str() {
            "registered" => ParticipantStatus::Registered,
            "in_waiting_room" => ParticipantStatus::InWaitingRoom,
            "joined" => ParticipantStatus::Joined,
            "left" => ParticipantStatus::Left,
            "removed" => ParticipantStatus::Removed,
            _ => ParticipantStatus::Registered,
        };
        let registration_data: Option<HashMap<String, String>> = row
            .registration_data
            .and_then(|d| serde_json::from_str(&d).ok());

        WebinarParticipant {
            id: row.id,
            webinar_id: row.webinar_id,
            user_id: row.user_id,
            name: row.name,
            email: row.email,
            role,
            status,
            hand_raised: row.hand_raised,
            hand_raised_at: row.hand_raised_at,
            is_speaking: row.is_speaking,
            video_enabled: row.video_enabled,
            audio_enabled: row.audio_enabled,
            screen_sharing: row.screen_sharing,
            joined_at: row.joined_at,
            left_at: row.left_at,
            registration_data,
        }
    }

    fn row_to_question(&self, row: QuestionRow) -> QAQuestion {
        let status = match row.status.as_str() {
            "pending" => QuestionStatus::Pending,
            "approved" => QuestionStatus::Approved,
            "answered" => QuestionStatus::Answered,
            "dismissed" => QuestionStatus::Dismissed,
            "answered_live" => QuestionStatus::AnsweredLive,
            _ => QuestionStatus::Pending,
        };
        let upvoted_by: Vec<Uuid> = row
            .upvoted_by
            .and_then(|u| serde_json::from_str(&u).ok())
            .unwrap_or_default();

        QAQuestion {
            id: row.id,
            webinar_id: row.webinar_id,
            asker_id: row.asker_id,
            asker_name: row.asker_name,
            is_anonymous: row.is_anonymous,
            question: row.question,
            status,
            upvotes: row.upvotes,
            upvoted_by,
            answer: row.answer,
            answered_by: row.answered_by,
            answered_at: row.answered_at,
            is_pinned: row.is_pinned,
            is_highlighted: row.is_highlighted,
            created_at: row.created_at,
        }
    }
}
