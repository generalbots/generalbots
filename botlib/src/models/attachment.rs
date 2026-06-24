use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub attachment_type: AttachmentType,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentType {
    Image,
    Audio,
    Video,
    Document,
    File,
}

impl Attachment {
    #[must_use]
    pub fn new(attachment_type: AttachmentType, url: impl Into<String>) -> Self {
        Self {
            attachment_type,
            url: url.into(),
            mime_type: None,
            filename: None,
            size: None,
            thumbnail_url: None,
        }
    }

    #[must_use]
    pub fn image(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::Image, url)
    }

    #[must_use]
    pub fn audio(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::Audio, url)
    }

    #[must_use]
    pub fn video(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::Video, url)
    }

    #[must_use]
    pub fn document(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::Document, url)
    }

    #[must_use]
    pub fn file(url: impl Into<String>) -> Self {
        Self::new(AttachmentType::File, url)
    }

    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    #[must_use]
    pub const fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    #[must_use]
    pub fn with_thumbnail(mut self, thumbnail_url: impl Into<String>) -> Self {
        self.thumbnail_url = Some(thumbnail_url.into());
        self
    }

    #[must_use]
    pub const fn is_image(&self) -> bool {
        matches!(self.attachment_type, AttachmentType::Image)
    }

    #[must_use]
    pub const fn is_media(&self) -> bool {
        matches!(
            self.attachment_type,
            AttachmentType::Image | AttachmentType::Audio | AttachmentType::Video
        )
    }
}
