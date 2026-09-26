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

//! CREATE FILE keyword (#749 — doc/code mismatch: the keyword was documented
//! in botbook/AGENTS.md but never registered in the engine).

use botbasic_types::{BasicRuntime, UserSession};
use log::trace;
use rhai::{Dynamic, Engine, EvalAltResult};
use std::sync::Arc;

use crate::keywords::file_ops::basic_io::execute_create_file;

pub fn register_create_file_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    register_create_file_fn(state.clone(), user.clone(), engine);
    register_create_file_syntax(state, user, engine);
}

/// Function form `create_file(path, data)`, emitted by
/// `convert_multiword_keywords` for `CREATE FILE … WITH …`. The custom-syntax
/// form cannot be relied on at runtime because CREATE SITE (registered later)
/// shares the same Rhai first-token key and overrides it.
fn register_create_file_fn(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    engine.register_fn("create_file", move |path: &str, data: &str| -> Result<rhai::Dynamic, Box<EvalAltResult>> {
        let state_for_task = Arc::clone(&state);
        let user_for_task = user.clone();
        let path = path.to_string();
        let data_str = data.to_string();

        let (tx, rx) = std::sync::mpsc::channel();
        let spawn_result = std::thread::Builder::new()
            .name("create-file".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_create_file(&state_for_task, &user_for_task, &path, &data_str).await
                    });
                    tx.send(result.map(|_| rhai::Dynamic::UNIT)).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".into())).err()
                };
                if send_err.is_some() {
                    log::error!("Failed to send CREATE FILE result from thread");
                }
            });

        if spawn_result.is_err() {
            return Err(Box::new(EvalAltResult::ErrorRuntime(
                "CREATE FILE thread failed".into(),
                rhai::Position::NONE,
            )));
        }

        match rx.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok(Ok(_)) => Ok(rhai::Dynamic::UNIT),
            Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                format!("CREATE FILE failed: {e}").into(),
                rhai::Position::NONE,
            ))),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(Box::new(
                EvalAltResult::ErrorRuntime("CREATE FILE timed out".into(), rhai::Position::NONE),
            )),
            Err(e) => Err(Box::new(EvalAltResult::ErrorRuntime(
                format!("CREATE FILE thread failed: {e}").into(),
                rhai::Position::NONE,
            ))),
        }
    });
}

fn register_create_file_syntax(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine
        .register_custom_syntax(
            ["CREATE", "FILE", "$expr$", "WITH", "$expr$"],
            false,
            move |context, inputs| {
                let path = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;

                trace!("CREATE FILE: {path}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let data_str = if data.is_string() {
                    data.to_string()
                } else {
                    serde_json::to_string(&crate::keywords::file_ops::utils::dynamic_to_json(&data))
                        .unwrap_or_default()
                };

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_create_file(&state_for_task, &user_for_task, &path, &data_str).await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        log::error!("Failed to send CREATE FILE result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(_)) => Ok(Dynamic::UNIT),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("CREATE FILE failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "CREATE FILE timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("CREATE FILE thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
    {
        log::error!("Failed to register the custom syntax: {e}");
    }
}