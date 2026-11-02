/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the MIT License.                                             |
|                                                                             |
| This program is free software: you can redistribute it and/or modify        |
| it under the terms of the MIT License.                                      |
|                                                                             |
| The text of the MIT License can be found in the LICENSE file you have       |
| received along with this program.                                           |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.                        |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the MIT License does not imply a         |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

pub(crate) use botcore::config::ConfigManager;
pub(crate) use botbasic_types::UserSession;
pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use log::trace;
pub(crate) use rhai::{Dynamic, Engine};
pub(crate) use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmsProvider {
    Twilio,
    AwsSns,
    Vonage,
    MessageBird,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum SmsPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsSendResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub provider: String,
    pub to: String,
    pub priority: String,
    pub error: Option<String>,
}

use std::sync::Arc;
use uuid::Uuid;

mod sms_priority;
mod register_sms_keywords;
mod register_send_sms_full_keyword;
mod send_via_twilio;

pub use register_sms_keywords::*;
pub use register_send_sms_full_keyword::*;
use send_via_twilio::*;
