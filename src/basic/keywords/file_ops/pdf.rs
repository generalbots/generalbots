/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the AGPL-3.0.                                                |
|                                                                             |
| According to our dual licensing model, this program can be used either      |
| under the terms of the GNU Affero General Public License, version 3,        |
| or under a proprietary license.                                             |
|                                                                             |
| The texts of the GNU Affero General Public License with an additional       |
| permission and of our proprietary license can be found at and               |
| in the LICENSE file you have received along with this program.              |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the                |
| GNU Affero General Public License for more details.                         |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the AGPLv3 does not imply a              |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

use crate::core::shared::models::schema::bots::dsl::*;
use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::trace;
use rhai::{Dynamic, Engine, Map};
use serde_json::Value;
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::sync::Arc;

use super::basic_io::{execute_read, execute_write};
use super::utils::dynamic_to_json;

pub struct PdfResult {
    pub url: String,
    pub local_name: String,
}

pub fn register_generate_pdf_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["GENERATE", "PDF", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let template = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;
                let output = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!("GENERATE PDF template: {template}, output: {output}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let data_json = dynamic_to_json(&data);

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_generate_pdf(
                                &state_for_task,
                                &user_for_task,
                                &template,
                                data_json,
                                &output,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        log::error!("Failed to send GENERATE PDF result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(120)) {
                    Ok(Ok(result)) => {
                        let mut map: Map = Map::new();
                        map.insert("url".into(), Dynamic::from(result.url));
                        map.insert("localName".into(), Dynamic::from(result.local_name));
                        Ok(Dynamic::from(map))
                    }
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("GENERATE PDF failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "GENERATE PDF timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("GENERATE PDF thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

pub fn register_merge_pdf_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["MERGE", "PDF", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let files = context.eval_expression_tree(&inputs[0])?;
                let output = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("MERGE PDF to: {output}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let file_list: Vec<String> = if files.is_array() {
                    files
                        .into_array()
                        .unwrap_or_default()
                        .iter()
                        .map(|f| f.to_string())
                        .collect()
                } else {
                    vec![files.to_string()]
                };

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_merge_pdf(&state_for_task, &user_for_task, &file_list, &output)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        log::error!("Failed to send MERGE PDF result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(120)) {
                    Ok(Ok(result)) => {
                        let mut map: Map = Map::new();
                        map.insert("url".into(), Dynamic::from(result.url));
                        map.insert("localName".into(), Dynamic::from(result.local_name));
                        Ok(Dynamic::from(map))
                    }
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("MERGE PDF failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "MERGE PDF timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("MERGE PDF thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

pub async fn execute_generate_pdf(
    state: &AppState,
    user: &UserSession,
    template: &str,
    data: Value,
    output: &str,
) -> Result<PdfResult, Box<dyn Error + Send + Sync>> {
    let template_content = execute_read(state, user, template).await?;

    let mut html_content = template_content;
    if let Value::Object(obj) = &data {
        for (key, value) in obj {
            let placeholder = format!("{{{{{key}}}}}");
            let value_str = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            html_content = html_content.replace(&placeholder, &value_str);
        }
    }

    let mut pdf_content = String::from("<!-- PDF Content Generated from Template: ");
    let _ = writeln!(pdf_content, "{template} -->\n{html_content}");

    execute_write(state, user, output, &pdf_content).await?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)?
    };

    let url = format!("s3://{bot_name}.gbai/{bot_name}.gbdrive/{output}");

    trace!("GENERATE_PDF successful: {output}");
    Ok(PdfResult {
        url,
        local_name: output.to_string(),
    })
}

pub async fn execute_merge_pdf(
    state: &AppState,
    user: &UserSession,
    files: &[String],
    output: &str,
) -> Result<PdfResult, Box<dyn Error + Send + Sync>> {
    let mut merged_content = String::from("<!-- Merged PDF -->\n");

    for file in files {
        let content = execute_read(state, user, file).await?;
        let _ = writeln!(merged_content, "\n<!-- From: {file} -->\n{content}");
    }

    execute_write(state, user, output, &merged_content).await?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)?
    };

    let url = format!("s3://{bot_name}.gbai/{bot_name}.gbdrive/{output}");

    trace!(
        "MERGE_PDF successful: {} files merged to {output}",
        files.len()
    );
    Ok(PdfResult {
        url,
        local_name: output.to_string(),
    })
}
