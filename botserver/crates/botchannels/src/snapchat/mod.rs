//! Snapchat Marketing API Integration
//!
//! Provides Snap Ads management, audience targeting, and content publishing capabilities.
//! Supports OAuth 2.0 authentication flow for Snapchat Marketing API.

pub(crate) use crate::{ ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType, PostContent, PostResult, };
pub(crate) use serde::{Deserialize, Serialize};

mod snapchat_provider;
mod get_me;
mod create_creative;
mod create_audience;
mod campaign_create_request;
mod ad_create_request;
mod oauth_token_response;

pub use snapchat_provider::*;
pub use campaign_create_request::*;
pub use ad_create_request::*;
pub use oauth_token_response::*;
