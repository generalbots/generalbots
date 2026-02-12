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

use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;

use super::archive::*;
use super::basic_io::*;
use super::copy_move::*;
use super::pdf::*;
use super::transfer::*;
use super::utils::dynamic_to_file_data;

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
                    serde_json::to_string(&super::utils::dynamic_to_json(&data)).unwrap_or_default()
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
                    let array: rhai::Array = files.iter().map(|f| Dynamic::from(f.clone())).collect();
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
                        let array: rhai::Array = files.iter().map(|f| Dynamic::from(f.clone())).collect();
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
