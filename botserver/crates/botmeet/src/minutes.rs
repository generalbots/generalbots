use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::transcription::TranscriptionSegment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub minutes_id: String,
    pub description: String,
    pub assignee: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: ActionItemStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionItemStatus {
    Open,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingMinutes {
    pub id: String,
    pub meeting_id: String,
    pub title: String,
    pub date: DateTime<Utc>,
    pub attendees: Vec<String>,
    pub agenda: Vec<String>,
    pub discussions: Vec<DiscussionPoint>,
    pub decisions: Vec<String>,
    pub action_items: Vec<ActionItem>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionPoint {
    pub topic: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub speaker: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinutesTemplate {
    pub title: String,
    pub date: String,
    pub attendees: String,
    pub agenda: String,
    pub discussions: Vec<DiscussionPoint>,
    pub decisions: Vec<String>,
    pub action_items: Vec<ActionItem>,
}

pub struct MinutesService;

impl MinutesService {
    pub fn generate_from_transcription(
        segments: &[TranscriptionSegment],
        meeting_title: &str,
        attendees: Vec<String>,
    ) -> MeetingMinutes {
        info!("Generating minutes from {} transcription segments", segments.len());

        let meeting_id = segments
            .first()
            .map(|s| s.meeting_id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let now = Utc::now();

        let discussions = Self::extract_discussion_topics(segments);
        let action_items = Self::extract_action_items(segments);
        let decisions = Self::extract_decisions(segments);
        let agenda = Self::extract_agenda(segments);

        MeetingMinutes {
            id: Uuid::new_v4().to_string(),
            meeting_id,
            title: meeting_title.to_string(),
            date: now,
            attendees,
            agenda,
            discussions,
            decisions,
            action_items,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn extract_discussion_topics(segments: &[TranscriptionSegment]) -> Vec<DiscussionPoint> {
        let mut discussions: Vec<DiscussionPoint> = Vec::new();
        let mut current_topic = String::new();
        let mut current_points: Vec<String> = Vec::new();
        let mut current_speaker: Option<String> = None;

        for seg in segments {
            let text_lower = seg.text.to_lowercase();

            if text_lower.starts_with("next") || text_lower.starts_with("moving on") || text_lower.starts_with("now let") {
                if !current_topic.is_empty() {
                    discussions.push(DiscussionPoint {
                        topic: current_topic.clone(),
                        summary: current_points.join(". "),
                        key_points: current_points.clone(),
                        speaker: current_speaker.clone(),
                    });
                }
                current_topic = seg.text.clone();
                current_points.clear();
            }

            current_points.push(seg.text.clone());
            current_speaker = Some(seg.speaker.clone());

            if !current_topic.is_empty() && text_lower.contains("question") || text_lower.contains("thoughts") || text_lower.contains("anyone") {
                current_points.push(format!("Discussion point: {}", seg.text));
            }
        }

        if !current_topic.is_empty() {
            discussions.push(DiscussionPoint {
                topic: current_topic,
                summary: current_points.join(". "),
                key_points: current_points,
                speaker: current_speaker,
            });
        }

        if discussions.is_empty() && !segments.is_empty() {
            discussions.push(DiscussionPoint {
                topic: "General Discussion".to_string(),
                summary: segments.iter().map(|s| s.text.clone()).collect::<Vec<_>>().join(" "),
                key_points: segments.iter().map(|s| s.text.clone()).collect(),
                speaker: None,
            });
        }

        discussions
    }

    pub fn extract_action_items(segments: &[TranscriptionSegment]) -> Vec<ActionItem> {
        let mut items: Vec<ActionItem> = Vec::new();
        let minutes_id = Uuid::new_v4().to_string();

        let action_phrases = [
            "i will", "i'll", "action item", "to-do", "todo", "need to",
            "must", "should", "follow up", "follow-up", "assign",
        ];

        for seg in segments {
            let text_lower = seg.text.to_lowercase();
            let has_action = action_phrases.iter().any(|p| text_lower.contains(p));
            if !has_action {
                continue;
            }

            let assignee = Self::extract_assignee(&seg.text);
            items.push(ActionItem {
                id: Uuid::new_v4().to_string(),
                minutes_id: minutes_id.clone(),
                description: seg.text.clone(),
                assignee,
                due_date: None,
                status: ActionItemStatus::Open,
                created_at: Utc::now(),
            });
        }

        items
    }

    fn extract_assignee(text: &str) -> Option<String> {
        let patterns = [
            "assign to ",
            "assigned to ",
            "@",
            "for ",
        ];

        for pattern in &patterns {
            if let Some(pos) = text.to_lowercase().find(pattern) {
                let start = pos + pattern.len();
                let remaining = &text[start..];
                let end = remaining.find(|c: char| c == '.' || c == ',' || c == ' ').unwrap_or(remaining.len());
                let name = remaining[..end].trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }

        None
    }

    pub fn extract_decisions(segments: &[TranscriptionSegment]) -> Vec<String> {
        let mut decisions: Vec<String> = Vec::new();

        let decision_phrases = [
            "decided", "agreed", "consensus", "we will", "let's go with",
            "approved", "signed off", "confirmed", "finalized",
        ];

        for seg in segments {
            let text_lower = seg.text.to_lowercase();
            if decision_phrases.iter().any(|p| text_lower.contains(p)) {
                decisions.push(seg.text.clone());
            }
        }

        decisions
    }

    fn extract_agenda(segments: &[TranscriptionSegment]) -> Vec<String> {
        let mut agenda: Vec<String> = Vec::new();

        for seg in segments {
            let text_lower = seg.text.to_lowercase();
            if text_lower.starts_with("agenda") || text_lower.contains("first item") || text_lower.contains("topic #") {
                agenda.push(seg.text.clone());
            }
        }

        if agenda.is_empty() {
            agenda.push("General Meeting".to_string());
        }

        agenda
    }

    pub fn format_minutes_template(minutes: &MeetingMinutes) -> String {
        let mut html = String::new();

        html.push_str(&format!(
            r#"<div class="meeting-minutes">
                <h1>{title}</h1>
                <div class="meta">Date: {date} | Meeting ID: {meeting_id}</div>
                <hr>
                <h2>Attendees</h2>
                <ul>{attendees}</ul>
                <h2>Agenda</h2>
                <ol>{agenda}</ol>"#,
            title = minutes.title,
            date = minutes.date.format("%B %d, %Y at %H:%M"),
            meeting_id = minutes.meeting_id,
            attendees = minutes.attendees.iter().map(|a| format!("<li>{}</li>", a)).collect::<Vec<_>>().join(""),
            agenda = minutes.agenda.iter().map(|a| format!("<li>{}</li>", a)).collect::<Vec<_>>().join(""),
        ));

        if !minutes.discussions.is_empty() {
            html.push_str("<h2>Discussion Points</h2>");
            for d in &minutes.discussions {
                html.push_str(&format!(
                    "<div class='discussion'><h3>{}</h3><p>{}</p><ul>{}</ul></div>",
                    d.topic,
                    d.summary,
                    d.key_points.iter().map(|kp| format!("<li>{}</li>", kp)).collect::<Vec<_>>().join(""),
                ));
            }
        }

        if !minutes.decisions.is_empty() {
            html.push_str("<h2>Decisions</h2><ul>");
            for d in &minutes.decisions {
                html.push_str(&format!("<li>{}</li>", d));
            }
            html.push_str("</ul>");
        }

        if !minutes.action_items.is_empty() {
            html.push_str("<h2>Action Items</h2><table><tr><th>Description</th><th>Assignee</th><th>Status</th></tr>");
            for ai in &minutes.action_items {
                let status_emoji = match ai.status {
                    ActionItemStatus::Open => "🔴",
                    ActionItemStatus::InProgress => "🟡",
                    ActionItemStatus::Completed => "🟢",
                    ActionItemStatus::Cancelled => "⚪",
                };
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{} {:?}</td></tr>",
                    ai.description,
                    ai.assignee.as_deref().unwrap_or("Unassigned"),
                    status_emoji,
                    ai.status,
                ));
            }
            html.push_str("</table>");
        }

        html.push_str("</div>");
        html
    }

    pub fn format_minutes_markdown(minutes: &MeetingMinutes) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {}\n\n", minutes.title));
        md.push_str(&format!("**Date:** {}\n\n", minutes.date.format("%B %d, %Y at %H:%M")));
        md.push_str(&format!("**Meeting ID:** `{}`\n\n", minutes.meeting_id));

        md.push_str("## Attendees\n");
        for a in &minutes.attendees {
            md.push_str(&format!("- {}\n", a));
        }
        md.push('\n');

        md.push_str("## Agenda\n");
        for (i, a) in minutes.agenda.iter().enumerate() {
            md.push_str(&format!("{}. {}\n", i + 1, a));
        }
        md.push('\n');

        if !minutes.discussions.is_empty() {
            md.push_str("## Discussion Points\n");
            for d in &minutes.discussions {
                md.push_str(&format!("### {}\n{}\n\n", d.topic, d.summary));
            }
        }

        if !minutes.decisions.is_empty() {
            md.push_str("## Decisions\n");
            for d in &minutes.decisions {
                md.push_str(&format!("- {}\n", d));
            }
            md.push('\n');
        }

        if !minutes.action_items.is_empty() {
            md.push_str("## Action Items\n");
            md.push_str("| Description | Assignee | Status |\n");
            md.push_str("|---|---|---|\n");
            for ai in &minutes.action_items {
                md.push_str(&format!(
                    "| {} | {} | {:?} |\n",
                    ai.description,
                    ai.assignee.as_deref().unwrap_or("-"),
                    ai.status,
                ));
            }
            md.push('\n');
        }

        md
    }

    pub fn update_minutes(minutes: &mut MeetingMinutes, notes: String) {
        minutes.notes = Some(notes);
        minutes.updated_at = Utc::now();
    }

    pub fn resolve_action_item(item: &mut ActionItem) {
        item.status = ActionItemStatus::Completed;
    }

    pub fn reopen_action_item(item: &mut ActionItem) {
        item.status = ActionItemStatus::Open;
    }
}
