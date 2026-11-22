//! Versioned event names for the Office document platform (#2503).
//!
//! A name is a stable wire identifier: consumers filter on it and the schema
//! registry publishes an example for every member of `ALL`. Adding a name is a
//! contract change; renaming one is a breaking change.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Version of the `DomainEvent` payload contract. Bump when a field is added
/// or its meaning changes; consumers refuse versions they do not understand.
pub const PAYLOAD_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OfficeEventType {
    #[serde(rename = "doc.created")]
    DocCreated,
    #[serde(rename = "doc.updated")]
    DocUpdated,
    #[serde(rename = "doc.commented")]
    DocCommented,
    #[serde(rename = "doc.review_requested")]
    DocReviewRequested,
    #[serde(rename = "doc.approved")]
    DocApproved,
    #[serde(rename = "doc.published")]
    DocPublished,
    #[serde(rename = "doc.exported")]
    DocExported,
    #[serde(rename = "doc.deleted")]
    DocDeleted,
    #[serde(rename = "doc.comment.added")]
    DocCommentAdded,
    #[serde(rename = "doc.comment.updated")]
    DocCommentUpdated,
    #[serde(rename = "doc.comment.deleted")]
    DocCommentDeleted,
    #[serde(rename = "doc.comment.mention.added")]
    DocCommentMentionAdded,
    #[serde(rename = "slides.created")]
    SlidesCreated,
    #[serde(rename = "slides.updated")]
    SlidesUpdated,
    #[serde(rename = "slides.presented")]
    SlidesPresented,
    #[serde(rename = "slides.exported")]
    SlidesExported,
    #[serde(rename = "slides.deleted")]
    SlidesDeleted,
    #[serde(rename = "sheets.created")]
    SheetsCreated,
    #[serde(rename = "sheets.updated")]
    SheetsUpdated,
    #[serde(rename = "sheets.refreshed")]
    SheetsRefreshed,
    #[serde(rename = "sheets.exported")]
    SheetsExported,
    #[serde(rename = "sheets.deleted")]
    SheetsDeleted,
}

impl OfficeEventType {
    /// Every published event name. The schema registry and the tests iterate
    /// this list so a new variant cannot be shipped without a documented
    /// example.
    pub const ALL: [OfficeEventType; 22] = [
        Self::DocCreated,
        Self::DocUpdated,
        Self::DocCommented,
        Self::DocReviewRequested,
        Self::DocApproved,
        Self::DocPublished,
        Self::DocExported,
        Self::DocDeleted,
        Self::DocCommentAdded,
        Self::DocCommentUpdated,
        Self::DocCommentDeleted,
        Self::DocCommentMentionAdded,
        Self::SlidesCreated,
        Self::SlidesUpdated,
        Self::SlidesPresented,
        Self::SlidesExported,
        Self::SlidesDeleted,
        Self::SheetsCreated,
        Self::SheetsUpdated,
        Self::SheetsRefreshed,
        Self::SheetsExported,
        Self::SheetsDeleted,
    ];

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DocCreated => "doc.created",
            Self::DocUpdated => "doc.updated",
            Self::DocCommented => "doc.commented",
            Self::DocReviewRequested => "doc.review_requested",
            Self::DocApproved => "doc.approved",
            Self::DocPublished => "doc.published",
            Self::DocExported => "doc.exported",
            Self::DocDeleted => "doc.deleted",
            Self::DocCommentAdded => "doc.comment.added",
            Self::DocCommentUpdated => "doc.comment.updated",
            Self::DocCommentDeleted => "doc.comment.deleted",
            Self::DocCommentMentionAdded => "doc.comment.mention.added",
            Self::SlidesCreated => "slides.created",
            Self::SlidesUpdated => "slides.updated",
            Self::SlidesPresented => "slides.presented",
            Self::SlidesExported => "slides.exported",
            Self::SlidesDeleted => "slides.deleted",
            Self::SheetsCreated => "sheets.created",
            Self::SheetsUpdated => "sheets.updated",
            Self::SheetsRefreshed => "sheets.refreshed",
            Self::SheetsExported => "sheets.exported",
            Self::SheetsDeleted => "sheets.deleted",
        }
    }

    /// The suite app that emits this event, used by subscription filters.
    #[must_use]
    pub fn app(self) -> &'static str {
        match self {
            Self::SlidesCreated
            | Self::SlidesUpdated
            | Self::SlidesPresented
            | Self::SlidesExported
            | Self::SlidesDeleted => "slides",
            Self::SheetsCreated
            | Self::SheetsUpdated
            | Self::SheetsRefreshed
            | Self::SheetsExported
            | Self::SheetsDeleted => "sheets",
            Self::DocCreated
            | Self::DocUpdated
            | Self::DocCommented
            | Self::DocReviewRequested
            | Self::DocApproved
            | Self::DocPublished
            | Self::DocExported
            | Self::DocDeleted
            | Self::DocCommentAdded
            | Self::DocCommentUpdated
            | Self::DocCommentDeleted
            | Self::DocCommentMentionAdded => "documents",
        }
    }

    /// Short, human-readable description used by the registry.
    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::DocCreated => "A document was created",
            Self::DocUpdated => "A document's metadata was updated",
            Self::DocCommented => "A comment thread changed on a document",
            Self::DocReviewRequested => "A review was requested on a document",
            Self::DocApproved => "A document was approved",
            Self::DocPublished => "A document was published",
            Self::DocExported => "A document was exported",
            Self::DocDeleted => "A document was deleted",
            Self::DocCommentAdded => "A comment was added to a document",
            Self::DocCommentUpdated => "A comment was edited",
            Self::DocCommentDeleted => "A comment was removed",
            Self::DocCommentMentionAdded => "A user was mentioned in a comment",
            Self::SlidesCreated => "A slide deck was created",
            Self::SlidesUpdated => "A slide deck was updated",
            Self::SlidesPresented => "A slide deck was presented",
            Self::SlidesExported => "A slide deck was exported",
            Self::SlidesDeleted => "A slide deck was deleted",
            Self::SheetsCreated => "A spreadsheet was created",
            Self::SheetsUpdated => "A spreadsheet was updated",
            Self::SheetsRefreshed => "A spreadsheet's data was refreshed",
            Self::SheetsExported => "A spreadsheet was exported",
            Self::SheetsDeleted => "A spreadsheet was deleted",
        }
    }

    /// Looks up a name emitted by `as_str`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|event_type| event_type.as_str() == name)
    }
}

impl FromStr for OfficeEventType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_name(value).ok_or_else(|| format!("unknown Office event '{value}'"))
    }
}

impl fmt::Display for OfficeEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::{OfficeEventType, PAYLOAD_SCHEMA_VERSION};

    #[test]
    fn every_name_is_unique() {
        let mut names: Vec<&str> = OfficeEventType::ALL
            .iter()
            .map(|event_type| event_type.as_str())
            .collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "duplicate event name in taxonomy");
    }

    #[test]
    fn names_round_trip_through_strings() {
        for event_type in OfficeEventType::ALL {
            let name = event_type.as_str();
            assert_eq!(OfficeEventType::from_name(name), Some(event_type));
            assert_eq!(name.parse::<OfficeEventType>().ok(), Some(event_type));
            let json = serde_json::to_string(&event_type).unwrap_or_default();
            assert_eq!(json, format!("\"{name}\""));
        }
    }

    #[test]
    fn apps_match_the_emitting_suite_app() {
        assert_eq!(OfficeEventType::DocCommentAdded.app(), "documents");
        assert_eq!(OfficeEventType::SlidesPresented.app(), "slides");
        assert_eq!(OfficeEventType::SheetsExported.app(), "sheets");
        assert_eq!(OfficeEventType::DocCommentMentionAdded.app(), "documents");
    }

    #[test]
    fn payload_contract_is_version_one() {
        assert_eq!(PAYLOAD_SCHEMA_VERSION, 1);
    }
}
