// Webinar API module - re-exports for backward compatibility
// This module has been split into the webinar_api subdirectory for better organization

pub mod webinar_api {
    pub use super::webinar_api::*;
}

// Re-export all public items for backward compatibility
pub use webinar_api::{
    // Constants
    MAX_RAISED_HANDS_VISIBLE, MAX_WEBINAR_PARTICIPANTS, QA_QUESTION_MAX_LENGTH,

    // Types
    AnswerQuestionRequest, CreatePollRequest, CreateWebinarRequest, FieldType,
    GetTranscriptionRequest, PanelistInvite, PollOption, PollStatus, PollType, PollVote,
    QAQuestion, QuestionStatus, RecordingQuality, RecordingStatus, RegisterRequest,
    RegistrationField, RegistrationStatus, RetentionPoint, RoleChangeRequest,
    StartRecordingRequest, SubmitQuestionRequest, TranscriptionFormat,
    TranscriptionSegment, TranscriptionStatus, TranscriptionWord, Webinar,
    WebinarAnalytics, WebinarEvent, WebinarEventType, WebinarParticipant,
    WebinarPoll, WebinarRecording, WebinarRegistration, WebinarSettings,
    WebinarStatus, WebinarTranscription, ParticipantRole, ParticipantStatus,

    // Error
    WebinarError,

    // Service
    WebinarService,

    // Routes
    webinar_routes,

    // Migrations
    create_webinar_tables_migration,
};
