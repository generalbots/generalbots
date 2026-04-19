//! WeChat Official Account and Mini Program API Integration
//!
//! Provides messaging, media upload, and content publishing capabilities.
//! Supports both Official Account and Mini Program APIs.

mod client;
mod content;
mod menu;
mod messages;
mod provider;
mod qrcode;
mod types;
mod user;

// Re-export the main provider
pub use client::WeChatProvider;

// Re-export all types for public API
pub use types::{
    ArticleDetail, ArticleItem, ActionInfo, CustomerMessage, DraftResponse,
    FollowerData, FollowerList, FollowerListResponse, IncomingMessage, MediaContent,
    MediaUploadResponse, MediaUploadResult, Menu, MenuButton, MiniProgram, MusicContent,
    NewsContent, NewsItem, PermanentMediaResponse, PermanentMediaResult, PublishResponse,
    PublishResult, PublishStatus, PublishStatusResponse, QRCodeRequest, QRCodeResponse,
    QRCodeResult, ReplyArticle, ReplyContent, ReplyMessage, Scene, ShortUrlResponse,
    TemplateDataItem, TemplateMessage, TemplateMessageResult, TextContent, VideoContent,
    VideoDescription, WeChatApiResponse, WeChatErrorCodes, WeChatUser, WeChatUserResponse,
    AccessTokenResponse, MediaType,
};
