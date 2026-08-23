use std::fmt::Debug;

use crate::traits::{
    BoxFutureString, BoxFutureUnit, BoxFutureVecString,
};

pub trait ScriptRunner: Send + Sync + Debug {
    fn run_script(
        &self,
        script: &str,
        session_id: uuid::Uuid,
        bot_id: &str,
    ) -> BoxFutureString;

    fn get_suggestions(
        &self,
        session_id: &uuid::Uuid,
        bot_id: &str,
    ) -> Result<Vec<crate::models::Suggestion>, String>;
}

pub trait TaskOrchestrator: Send + Sync + Debug {
    fn manifest(&self) -> String;
}

pub trait SessionStore: Send + Sync + Debug {
    fn get_session(
        &self,
        session_id: &uuid::Uuid,
    ) -> Result<Option<crate::models::Session>, String>;

    fn create_session(
        &self,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
    ) -> Result<crate::models::Session, String>;
}

pub trait KnowledgeBase: Send + Sync + Debug {
    fn query(&self, query: &str, limit: usize) -> BoxFutureVecString;

    fn index_document(
        &self,
        doc_id: &str,
        content: &str,
    ) -> BoxFutureUnit;
}

pub trait SessionManagerService: Send + Debug {
    fn get_session_by_id(&mut self, session_id: uuid::Uuid) -> Result<Option<crate::models::UserSession>, String>;

    fn get_or_create_user_session(
        &mut self,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
        session_title: &str,
    ) -> Result<Option<crate::models::UserSession>, String>;

    fn get_or_create_anonymous_user(
        &mut self,
        user_id: Option<uuid::Uuid>,
    ) -> Result<uuid::Uuid, String>;

    fn create_session(
        &mut self,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
        session_title: &str,
    ) -> Result<crate::models::UserSession, String>;

    fn get_or_create_session_by_id(
        &mut self,
        session_id: uuid::Uuid,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
        session_title: &str,
    ) -> Result<crate::models::UserSession, String>;

    fn save_message(
        &mut self,
        session_id: uuid::Uuid,
        user_id: uuid::Uuid,
        role: i32,
        content: &str,
        message_type: i32,
    ) -> Result<(), String>;

    fn get_conversation_history(
        &mut self,
        session_id: uuid::Uuid,
        user_id: uuid::Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<(String, String)>, String>;

    /// Returns the first user message of a conversation (decrypted) — used as
    /// the human-readable title in chat history listings. `None` when the
    /// session has no user messages yet.
    fn get_first_user_message(&mut self, session_id: uuid::Uuid) -> Result<Option<String>, String>;

    fn get_session_context_data(
        &self,
        session_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
    ) -> Result<String, String>;

    fn update_session_context(
        &mut self,
        session_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        context_data: String,
    ) -> Result<(), String>;

    fn get_user_sessions(
        &mut self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<crate::models::UserSession>, String>;

    fn update_user_id(
        &mut self,
        session_id: uuid::Uuid,
        new_user_id: uuid::Uuid,
    ) -> Result<(), String>;

    fn mark_waiting(&mut self, session_id: uuid::Uuid);

    fn active_count(&self) -> usize;
}
