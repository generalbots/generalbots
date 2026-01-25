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

use crate::basic::keywords::use_account::{
    get_account_credentials, is_account_path, parse_account_path,
};
use crate::shared::models::schema::bots::dsl::*;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use flate2::read::GzDecoder;
use log::{error, trace};
use rhai::{Array, Dynamic, Engine, Map};
use serde_json::Value;
use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use tar::Archive;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

pub fn register_file_operations(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_read_keyword(Arc::clone(&state), user.clone(), engine);
    register_write_keyword(Arc::clone(&state), user.clone(), engine);
    register_delete_file_keyword(Arc::clone(&state), user.clone(), engine);
    register_copy_keyword(Arc::clone(&state), user.clone(), engine);
    register_move_keyword(Arc::clone(&state), user.clone(), engine);
    register_list_keyword(Arc::clone(&state), user.clone(), engine);
    register_compress_keyword(Arc::clone(&state), user.clone(), engine);
    register_extract_keyword(Arc::clone(&state), user.clone(), engine);
    register_upload_keyword(Arc::clone(&state), user.clone(), engine);
    register_download_keyword(Arc::clone(&state), user.clone(), engine);
    register_generate_pdf_keyword(Arc::clone(&state), user.clone(), engine);
    register_merge_pdf_keyword(state, user, engine);
}

pub fn register_read_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["READ", "$expr$"], false, move |context, inputs| {
            let path = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("READ file: {path}");

            let state_for_task = Arc::clone(&state);
            let user_for_task = user.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_read(&state_for_task, &user_for_task, &path).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send READ result from thread");
                }
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(content)) => Ok(Dynamic::from(content)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("READ failed: {e}").into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "READ timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("READ thread failed: {e}").into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");
}

pub fn register_write_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["WRITE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let path = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;

                trace!("WRITE to file: {path}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let data_str = if data.is_string() {
                    data.to_string()
                } else {
                    serde_json::to_string(&dynamic_to_json(&data)).unwrap_or_default()
                };

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_write(&state_for_task, &user_for_task, &path, &data_str).await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send WRITE result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(_)) => Ok(Dynamic::UNIT),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("WRITE failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "WRITE timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("WRITE thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

pub fn register_delete_file_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user;

    engine
        .register_custom_syntax(
            ["DELETE", "FILE", "$expr$"],
            false,
            move |context, inputs| {
                let path = context.eval_expression_tree(&inputs[0])?.to_string();

                trace!("DELETE FILE: {path}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_delete_file(&state_for_task, &user_for_task, &path).await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send DELETE FILE result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(_)) => Ok(Dynamic::UNIT),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE FILE failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "DELETE FILE timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE FILE thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");

    engine
        .register_custom_syntax(
            ["DELETE", "FILE", "$expr$"],
            false,
            move |context, inputs| {
                let path = context.eval_expression_tree(&inputs[0])?.to_string();

                trace!("DELETE FILE: {path}");

                let state_for_task = Arc::clone(&state_clone2);
                let user_for_task = user_clone2.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_delete_file(&state_for_task, &user_for_task, &path).await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send DELETE FILE result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(_)) => Ok(Dynamic::UNIT),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE FILE failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "DELETE FILE timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE FILE thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

pub fn register_copy_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["COPY", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let source = context.eval_expression_tree(&inputs[0])?.to_string();
                let destination = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("COPY from {source} to {destination}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_copy(&state_for_task, &user_for_task, &source, &destination)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send COPY result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(_)) => Ok(Dynamic::UNIT),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("COPY failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "COPY timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("COPY thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

pub fn register_move_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["MOVE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let source = context.eval_expression_tree(&inputs[0])?.to_string();
                let destination = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("MOVE from {source} to {destination}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_move(&state_for_task, &user_for_task, &source, &destination)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send MOVE result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(_)) => Ok(Dynamic::UNIT),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("MOVE failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "MOVE timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("MOVE thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

pub fn register_list_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(["LIST", "$expr$"], false, move |context, inputs| {
            let path = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("LIST directory: {path}");

            let state_for_task = Arc::clone(&state_clone);
            let user_for_task = user_clone.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_list(&state_for_task, &user_for_task, &path).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send LIST result from thread");
                }
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(files)) => {
                    let array: Array = files.iter().map(|f| Dynamic::from(f.clone())).collect();
                    Ok(Dynamic::from(array))
                }
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("LIST failed: {e}").into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "LIST timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("LIST thread failed: {e}").into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");
}

pub fn register_compress_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["COMPRESS", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let files = context.eval_expression_tree(&inputs[0])?;
                let archive_name = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("COMPRESS to: {archive_name}");

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
                            execute_compress(
                                &state_for_task,
                                &user_for_task,
                                &file_list,
                                &archive_name,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send COMPRESS result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(120)) {
                    Ok(Ok(path)) => Ok(Dynamic::from(path)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("COMPRESS failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "COMPRESS timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("COMPRESS thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

pub fn register_extract_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["EXTRACT", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let archive = context.eval_expression_tree(&inputs[0])?.to_string();
                let destination = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("EXTRACT {archive} to {destination}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_extract(&state_for_task, &user_for_task, &archive, &destination)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send EXTRACT result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(120)) {
                    Ok(Ok(files)) => {
                        let array: Array = files.iter().map(|f| Dynamic::from(f.clone())).collect();
                        Ok(Dynamic::from(array))
                    }
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("EXTRACT failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "EXTRACT timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("EXTRACT thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

pub fn register_upload_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["UPLOAD", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let file = context.eval_expression_tree(&inputs[0])?;
                let destination = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("UPLOAD to: {destination}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let file_data = dynamic_to_file_data(&file);

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_upload(&state_for_task, &user_for_task, file_data, &destination)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send UPLOAD result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(300)) {
                    Ok(Ok(url)) => Ok(Dynamic::from(url)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("UPLOAD failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "UPLOAD timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("UPLOAD thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

pub fn register_download_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["DOWNLOAD", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                let local_path = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("DOWNLOAD {url} to {local_path}");

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_download(&state_for_task, &user_for_task, &url, &local_path)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send DOWNLOAD result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(300)) {
                    Ok(Ok(path)) => Ok(Dynamic::from(path)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DOWNLOAD failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "DOWNLOAD timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DOWNLOAD thread failed: {e}").into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
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
                        error!("Failed to send GENERATE PDF result from thread");
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
                        error!("Failed to send MERGE PDF result from thread");
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

async fn execute_read(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {e}");
                e
            })?
    };

    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{path}");

    let response = client
        .get_object()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await
        .map_err(|e| format!("S3 get failed: {e}"))?;

    let data = response.body.collect().await?.into_bytes();
    let content =
        String::from_utf8(data.to_vec()).map_err(|_| "File content is not valid UTF-8")?;

    trace!("READ successful: {} bytes", content.len());
    Ok(content)
}

async fn execute_write(
    state: &AppState,
    user: &UserSession,
    path: &str,
    content: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {e}");
                e
            })?
    };

    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{path}");

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(content.as_bytes().to_vec().into())
        .send()
        .await
        .map_err(|e| format!("S3 put failed: {e}"))?;

    trace!("WRITE successful: {} bytes to {path}", content.len());
    Ok(())
}

async fn execute_delete_file(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {e}");
                e
            })?
    };

    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{path}");

    client
        .delete_object()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await
        .map_err(|e| format!("S3 delete failed: {e}"))?;

    trace!("DELETE_FILE successful: {path}");
    Ok(())
}

async fn execute_copy(
    state: &AppState,
    user: &UserSession,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let source_is_account = is_account_path(source);
    let dest_is_account = is_account_path(destination);

    if source_is_account || dest_is_account {
        return execute_copy_with_account(state, user, source, destination).await;
    }

    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {e}");
                e
            })?
    };

    let bucket_name = format!("{bot_name}.gbai");
    let source_key = format!("{bot_name}.gbdrive/{source}");
    let dest_key = format!("{bot_name}.gbdrive/{destination}");

    let copy_source = format!("{bucket_name}/{source_key}");

    client
        .copy_object()
        .bucket(&bucket_name)
        .key(&dest_key)
        .copy_source(&copy_source)
        .send()
        .await
        .map_err(|e| format!("S3 copy failed: {e}"))?;

    trace!("COPY successful: {source} -> {destination}");
    Ok(())
}

async fn execute_copy_with_account(
    state: &AppState,
    user: &UserSession,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let source_is_account = is_account_path(source);
    let dest_is_account = is_account_path(destination);

    let content = if source_is_account {
        let (email, path) = parse_account_path(source).ok_or("Invalid account:// path format")?;
        let creds = get_account_credentials(&state.conn, &email, user.bot_id)
            .await
            .map_err(|e| format!("Failed to get credentials: {e}"))?;
        download_from_account(&creds, &path).await?
    } else {
        read_from_local(state, user, source).await?
    };

    if dest_is_account {
        let (email, path) =
            parse_account_path(destination).ok_or("Invalid account:// path format")?;
        let creds = get_account_credentials(&state.conn, &email, user.bot_id)
            .await
            .map_err(|e| format!("Failed to get credentials: {e}"))?;
        upload_to_account(&creds, &path, &content).await?;
    } else {
        write_to_local(state, user, destination, &content).await?;
    }

    trace!("COPY with account successful: {source} -> {destination}");
    Ok(())
}

async fn download_from_account(
    creds: &crate::basic::keywords::use_account::AccountCredentials,
    path: &str,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    match creds.provider.as_str() {
        "gmail" | "google" => {
            let url = format!(
                "https://www.googleapis.com/drive/v3/files/{}?alt=media",
                urlencoding::encode(path)
            );
            let resp = client
                .get(&url)
                .bearer_auth(&creds.access_token)
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(format!("Google Drive download failed: {}", resp.status()).into());
            }
            Ok(resp.bytes().await?.to_vec())
        }
        "outlook" | "microsoft" => {
            let url = format!(
                "https://graph.microsoft.com/v1.0/me/drive/root:/{}:/content",
                urlencoding::encode(path)
            );
            let resp = client
                .get(&url)
                .bearer_auth(&creds.access_token)
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(format!("OneDrive download failed: {}", resp.status()).into());
            }
            Ok(resp.bytes().await?.to_vec())
        }
        _ => Err(format!("Unsupported provider: {}", creds.provider).into()),
    }
}

async fn upload_to_account(
    creds: &crate::basic::keywords::use_account::AccountCredentials,
    path: &str,
    content: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();

    match creds.provider.as_str() {
        "gmail" | "google" => {
            let url = format!(
                "https://www.googleapis.com/upload/drive/v3/files?uploadType=media&name={}",
                urlencoding::encode(path)
            );
            let resp = client
                .post(&url)
                .bearer_auth(&creds.access_token)
                .body(content.to_vec())
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(format!("Google Drive upload failed: {}", resp.status()).into());
            }
        }
        "outlook" | "microsoft" => {
            let url = format!(
                "https://graph.microsoft.com/v1.0/me/drive/root:/{}:/content",
                urlencoding::encode(path)
            );
            let resp = client
                .put(&url)
                .bearer_auth(&creds.access_token)
                .body(content.to_vec())
                .send()
                .await?;
            if !resp.status().is_success() {
                return Err(format!("OneDrive upload failed: {}", resp.status()).into());
            }
        }
        _ => return Err(format!("Unsupported provider: {}", creds.provider).into()),
    }
    Ok(())
}

async fn read_from_local(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;
    let bot_name: String = {
        let mut db_conn = state.conn.get()?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)?
    };
    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{path}");

    let result = client
        .get_object()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await?;
    let bytes = result.body.collect().await?.into_bytes();
    Ok(bytes.to_vec())
}

async fn write_to_local(
    state: &AppState,
    user: &UserSession,
    path: &str,
    content: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;
    let bot_name: String = {
        let mut db_conn = state.conn.get()?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)?
    };
    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{path}");

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(content.to_vec().into())
        .send()
        .await?;
    Ok(())
}

async fn execute_move(
    state: &AppState,
    user: &UserSession,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    execute_copy(state, user, source, destination).await?;

    execute_delete_file(state, user, source).await?;

    trace!("MOVE successful: {source} -> {destination}");
    Ok(())
}

async fn execute_list(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {e}");
                e
            })?
    };

    let bucket_name = format!("{bot_name}.gbai");
    let prefix = format!("{bot_name}.gbdrive/{path}");

    let response = client
        .list_objects_v2()
        .bucket(&bucket_name)
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| format!("S3 list failed: {e}"))?;

    let files: Vec<String> = response
        .contents()
        .iter()
        .filter_map(|obj| {
            obj.key().map(|k| {
                k.strip_prefix(&format!("{bot_name}.gbdrive/"))
                    .unwrap_or(k)
                    .to_string()
            })
        })
        .collect();

    trace!("LIST successful: {} files", files.len());
    Ok(files)
}

async fn execute_compress(
    state: &AppState,
    user: &UserSession,
    files: &[String],
    archive_name: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {e}");
                e
            })?
    };

    let temp_dir = std::env::temp_dir();
    let archive_path = temp_dir.join(archive_name);
    let file = File::create(&archive_path)?;
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    for file_path in files {
        let content = execute_read(state, user, file_path).await?;
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path);

        zip.start_file(file_name, options)?;
        zip.write_all(content.as_bytes())?;
    }

    zip.finish()?;

    let archive_content = fs::read(&archive_path)?;
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;
    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{archive_name}");

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(archive_content.into())
        .send()
        .await
        .map_err(|e| format!("S3 put failed: {e}"))?;

    fs::remove_file(&archive_path).ok();

    trace!("COMPRESS successful: {archive_name}");
    Ok(archive_name.to_string())
}

fn has_zip_extension(archive: &str) -> bool {
    Path::new(archive)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}

fn has_tar_gz_extension(archive: &str) -> bool {
    let path = Path::new(archive);
    if let Some(ext) = path.extension() {
        if ext.eq_ignore_ascii_case("tgz") {
            return true;
        }
        if ext.eq_ignore_ascii_case("gz") {
            if let Some(stem) = path.file_stem() {
                return Path::new(stem)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("tar"));
            }
        }
    }
    false
}

async fn execute_extract(
    state: &AppState,
    user: &UserSession,
    archive: &str,
    destination: &str,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {e}");
                e
            })?
    };

    let bucket_name = format!("{bot_name}.gbai");
    let archive_key = format!("{bot_name}.gbdrive/{archive}");

    let response = client
        .get_object()
        .bucket(&bucket_name)
        .key(&archive_key)
        .send()
        .await
        .map_err(|e| format!("S3 get failed: {e}"))?;

    let data = response.body.collect().await?.into_bytes();

    let temp_dir = std::env::temp_dir();
    let archive_path = temp_dir.join(archive);
    fs::write(&archive_path, &data)?;

    let mut extracted_files = Vec::new();

    if has_zip_extension(archive) {
        let file = File::open(&archive_path)?;
        let mut zip = ZipArchive::new(file)?;

        for i in 0..zip.len() {
            let mut zip_file = zip.by_index(i)?;
            let file_name = zip_file.name().to_string();

            let mut content = Vec::new();
            zip_file.read_to_end(&mut content)?;

            let dest_path = format!("{}/{file_name}", destination.trim_end_matches('/'));

            let dest_key = format!("{bot_name}.gbdrive/{dest_path}");
            client
                .put_object()
                .bucket(&bucket_name)
                .key(&dest_key)
                .body(content.into())
                .send()
                .await
                .map_err(|e| format!("S3 put failed: {e}"))?;

            extracted_files.push(dest_path);
        }
    } else if has_tar_gz_extension(archive) {
        let file = File::open(&archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut tar = Archive::new(decoder);

        for entry in tar.entries()? {
            let mut entry = entry?;
            let file_name = entry.path()?.to_string_lossy().to_string();

            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;

            let dest_path = format!("{}/{file_name}", destination.trim_end_matches('/'));

            let dest_key = format!("{bot_name}.gbdrive/{dest_path}");
            client
                .put_object()
                .bucket(&bucket_name)
                .key(&dest_key)
                .body(content.into())
                .send()
                .await
                .map_err(|e| format!("S3 put failed: {e}"))?;

            extracted_files.push(dest_path);
        }
    }

    fs::remove_file(&archive_path).ok();

    trace!("EXTRACT successful: {} files", extracted_files.len());
    Ok(extracted_files)
}

struct FileData {
    content: Vec<u8>,
    filename: String,
}

async fn execute_upload(
    state: &AppState,
    user: &UserSession,
    file_data: FileData,
    destination: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {e}");
                e
            })?
    };

    let bucket_name = format!("{bot_name}.gbai");
    let key = format!("{bot_name}.gbdrive/{destination}");

    let content_disposition = format!("attachment; filename=\"{}\"", file_data.filename);

    trace!(
        "Uploading file '{}' to {bucket_name}/{key} ({} bytes)",
        file_data.filename,
        file_data.content.len()
    );

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .content_disposition(&content_disposition)
        .body(file_data.content.into())
        .send()
        .await
        .map_err(|e| format!("S3 put failed: {e}"))?;

    let url = format!("s3://{bucket_name}/{key}");
    trace!(
        "UPLOAD successful: {url} (original filename: {})",
        file_data.filename
    );
    Ok(url)
}

async fn execute_download(
    state: &AppState,
    user: &UserSession,
    url: &str,
    local_path: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    let content = response.bytes().await?;

    execute_write(state, user, local_path, &String::from_utf8_lossy(&content)).await?;

    trace!("DOWNLOAD successful: {url} -> {local_path}");
    Ok(local_path.to_string())
}

struct PdfResult {
    url: String,
    local_name: String,
}

async fn execute_generate_pdf(
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

async fn execute_merge_pdf(
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

fn dynamic_to_json(value: &Dynamic) -> Value {
    if value.is_unit() {
        Value::Null
    } else if value.is_bool() {
        Value::Bool(value.as_bool().unwrap_or(false))
    } else if value.is_int() {
        Value::Number(value.as_int().unwrap_or(0).into())
    } else if value.is_float() {
        if let Ok(f) = value.as_float() {
            serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        }
    } else if value.is_string() {
        Value::String(value.to_string())
    } else if value.is_array() {
        let arr = value.clone().into_array().unwrap_or_default();
        Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if value.is_map() {
        let map = value.clone().try_cast::<Map>().unwrap_or_default();
        let obj: serde_json::Map<String, Value> = map
            .iter()
            .map(|(k, v)| (k.to_string(), dynamic_to_json(v)))
            .collect();
        Value::Object(obj)
    } else {
        Value::String(value.to_string())
    }
}

fn dynamic_to_file_data(value: &Dynamic) -> FileData {
    if value.is_map() {
        let map = value.clone().try_cast::<Map>().unwrap_or_default();
        let content = map
            .get("data")
            .map(|v| v.to_string().into_bytes())
            .unwrap_or_default();
        let filename = map
            .get("filename")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "file".to_string());

        FileData { content, filename }
    } else {
        FileData {
            content: value.to_string().into_bytes(),
            filename: "file".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Dynamic;
    use serde_json::Value;

    #[test]
    fn test_dynamic_to_json() {
        let dynamic = Dynamic::from("hello");
        let json = dynamic_to_json(&dynamic);
        assert_eq!(json, Value::String("hello".to_string()));
    }

    #[test]
    fn test_dynamic_to_file_data() {
        let dynamic = Dynamic::from("test content");
        let file_data = dynamic_to_file_data(&dynamic);
        assert_eq!(file_data.filename, "file");
        assert!(!file_data.content.is_empty());
    }
}
