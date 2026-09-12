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

// Re-export all functionality from the file_ops module
// This maintains backward compatibility with existing imports
pub use crate::keywords::file_ops::*;

use std::sync::Arc;
use botbasic_types::{BasicRuntime, UserSession};
use rhai::Engine;

pub fn register_file_operations_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    crate::keywords::file_ops::register_file_ops_keywords(state, user, engine);
}
