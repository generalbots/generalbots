//! Pure builders for the Bot API request bodies the adapter POSTs.
//!
//! Every outbound call used to assemble its own `serde_json` shape inline in an
//! `async` method, which made the wire contract (field names, `parse_mode`,
//! `reply_markup` presence) reachable only through a live `reqwest` call. The
//! builders keep the contract in one place and let the tests pin it offline.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct TelegramSendMessage {
    pub(crate) chat_id: String,
    pub(crate) text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parse_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reply_markup: Option<TelegramReplyMarkup>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TelegramReplyMarkup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) inline_keyboard: Option<Vec<Vec<TelegramInlineButton>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) keyboard: Option<Vec<Vec<TelegramKeyboardButton>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) one_time_keyboard: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resize_keyboard: Option<bool>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TelegramInlineButton {
    pub(crate) text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) callback_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) url: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TelegramKeyboardButton {
    pub(crate) text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) request_contact: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) request_location: Option<bool>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TelegramSendPhoto {
    pub(crate) chat_id: String,
    pub(crate) photo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) caption: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TelegramSendDocument {
    pub(crate) chat_id: String,
    pub(crate) document: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) caption: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TelegramSendLocation {
    pub(crate) chat_id: String,
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
}

/// Bot API `sendMessage` body for a plain text message.
pub(crate) fn text_payload(
    chat_id: &str,
    text: &str,
    parse_mode: Option<&str>,
) -> TelegramSendMessage {
    TelegramSendMessage {
        chat_id: chat_id.to_string(),
        text: text.to_string(),
        parse_mode: parse_mode.map(String::from),
        reply_markup: None,
    }
}

/// Bot API `sendMessage` body carrying one inline button per row. An empty
/// `buttons` list omits `reply_markup` entirely: Telegram rejects an empty
/// `inline_keyboard` array, so "no buttons" must be "no keyboard".
pub(crate) fn buttons_payload(
    chat_id: &str,
    text: &str,
    buttons: Vec<(String, String)>,
) -> TelegramSendMessage {
    let inline_buttons: Vec<Vec<TelegramInlineButton>> = buttons
        .into_iter()
        .map(|(label, callback)| {
            vec![TelegramInlineButton {
                text: label,
                callback_data: Some(callback),
                url: None,
            }]
        })
        .collect();

    let reply_markup = if inline_buttons.is_empty() {
        None
    } else {
        Some(TelegramReplyMarkup {
            inline_keyboard: Some(inline_buttons),
            keyboard: None,
            one_time_keyboard: None,
            resize_keyboard: None,
        })
    };

    TelegramSendMessage {
        chat_id: chat_id.to_string(),
        text: text.to_string(),
        parse_mode: Some("HTML".to_string()),
        reply_markup,
    }
}

pub(crate) fn photo_payload(
    chat_id: &str,
    photo_url: &str,
    caption: Option<&str>,
) -> TelegramSendPhoto {
    TelegramSendPhoto {
        chat_id: chat_id.to_string(),
        photo: photo_url.to_string(),
        caption: caption.map(String::from),
    }
}

pub(crate) fn document_payload(
    chat_id: &str,
    document_url: &str,
    caption: Option<&str>,
) -> TelegramSendDocument {
    TelegramSendDocument {
        chat_id: chat_id.to_string(),
        document: document_url.to_string(),
        caption: caption.map(String::from),
    }
}

pub(crate) fn location_payload(chat_id: &str, latitude: f64, longitude: f64) -> TelegramSendLocation {
    TelegramSendLocation {
        chat_id: chat_id.to_string(),
        latitude,
        longitude,
    }
}

#[cfg(test)]
#[path = "payload_tests.rs"]
mod payload_tests;
