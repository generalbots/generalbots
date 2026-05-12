pub mod add_bot;
pub mod add_member;
pub mod add_suggestion;
pub mod book;
pub mod create_draft;
pub mod messaging;
pub mod on_email;
pub mod play;
pub mod send_mail;
pub mod send_template;
pub mod sms;
pub mod social;
pub mod social_media;
pub mod switcher;
pub mod transfer_to_human;
pub mod universal_messaging;
pub mod weather;
pub mod webhook;

use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;
use std::sync::Arc;

pub fn register_comms_keywords(
    state: &Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    let s = state.clone();
    add_bot::register_bot_keywords(s.clone(), &user, engine);
    add_member::add_member_keyword(s.clone(), user.clone(), engine);
    add_suggestion::add_suggestion_keyword(s.clone(), user.clone(), engine);
    book::book_keyword(s.clone(), user.clone(), engine);
    create_draft::create_draft_keyword(&s.clone(), user.clone(), engine);
    messaging::register_messaging_keywords(s.clone(), user.clone(), engine);
    on_email::on_email_keyword(&s.clone(), user.clone(), engine);
    play::play_keyword(s.clone(), user.clone(), engine);
    send_mail::send_mail_keyword(s.clone(), user.clone(), engine);
    send_template::register_send_template_keywords(s.clone(), user.clone(), engine);
    sms::register_sms_keywords(s.clone(), user.clone(), engine);
    social::register_social_media_keywords(s.clone(), user.clone(), engine);
    social_media::register_social_media_keywords(s.clone(), user.clone(), engine);
    switcher::add_switcher_keyword(s.clone(), user.clone(), engine);
    transfer_to_human::register_transfer_to_human_keyword(s.clone(), user.clone(), engine);
    universal_messaging::register_universal_messaging(s.clone(), user.clone(), engine);
    weather::weather_keyword(s.clone(), user.clone(), engine);
    webhook::webhook_keyword(&s.clone(), user.clone(), engine);
}
