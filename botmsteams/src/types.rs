use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TeamsOutboundActivity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<TeamsAttachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_actions: Option<TeamsSuggestedActions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsAttachment {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsSuggestedActions {
    pub actions: Vec<TeamsCardAction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsCardAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsHeroCard {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub text: Option<String>,
    pub images: Vec<TeamsCardImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<TeamsCardAction>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsCardImage {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsMember {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    pub role: Option<String>,
}

pub fn create_adaptive_card(
    title: &str,
    body: Vec<serde_json::Value>,
    actions: Vec<serde_json::Value>,
) -> serde_json::Value {
    let mut all_body_items = vec![serde_json::json!({
        "type": "TextBlock",
        "text": title,
        "weight": "Bolder",
        "size": "Medium"
    })];
    all_body_items.extend(body);

    serde_json::json!({
        "type": "AdaptiveCard",
        "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
        "version": "1.3",
        "body": all_body_items,
        "actions": actions
    })
}

pub fn create_thumbnail_card(
    title: &str,
    subtitle: Option<&str>,
    text: Option<&str>,
    image_url: Option<&str>,
    buttons: Vec<(&str, &str, &str)>,
) -> serde_json::Value {
    let mut card = serde_json::json!({
        "title": title
    });

    if let Some(sub) = subtitle {
        card["subtitle"] = serde_json::Value::String(sub.to_string());
    }
    if let Some(txt) = text {
        card["text"] = serde_json::Value::String(txt.to_string());
    }
    if let Some(img) = image_url {
        card["images"] = serde_json::json!([{"url": img}]);
    }

    let button_list: Vec<serde_json::Value> = buttons
        .into_iter()
        .map(|(action_type, btn_title, value)| {
            serde_json::json!({
                "type": action_type,
                "title": btn_title,
                "value": value
            })
        })
        .collect();

    if !button_list.is_empty() {
        card["buttons"] = serde_json::Value::Array(button_list);
    }

    card
}

pub fn create_message_with_mentions(
    text: &str,
    mentions: Vec<(&str, &str)>,
) -> (String, Vec<serde_json::Value>) {
    let mut message = text.to_string();
    let mention_entities: Vec<serde_json::Value> = mentions
        .into_iter()
        .map(|(user_id, display_name)| {
            let mention_text = format!("<at>{}</at>", display_name);
            message = message.replace(&format!("@{}", display_name), &mention_text);

            serde_json::json!({
                "type": "mention",
                "mentioned": {
                    "id": user_id,
                    "name": display_name
                },
                "text": mention_text
            })
        })
        .collect();

    (message, mention_entities)
}
