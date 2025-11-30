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

use crate::shared::models::schema::bots::dsl::*;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use flate2::read::GzDecoder;
use log::{error, trace};
use rhai::{Array, Dynamic, Engine, Map};
use serde_json::Value;
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use tar::Archive;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

/// Register all file operation keywords
pub fn register_file_operations(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_read_keyword(state.clone(), user.clone(), engine);
    register_write_keyword(state.clone(), user.clone(), engine);
    register_delete_file_keyword(state.clone(), user.clone(), engine);
    register_copy_keyword(state.clone(), user.clone(), engine);
    register_move_keyword(state.clone(), user.clone(), engine);
    register_list_keyword(state.clone(), user.clone(), engine);
    register_compress_keyword(state.clone(), user.clone(), engine);
    register_extract_keyword(state.clone(), user.clone(), engine);
    register_upload_keyword(state.clone(), user.clone(), engine);
    register_download_keyword(state.clone(), user.clone(), engine);
    register_generate_pdf_keyword(state.clone(), user.clone(), engine);
    register_merge_pdf_keyword(state.clone(), user.clone(), engine);
}

/// READ "path"
/// Reads content from a file in .gbdrive
pub fn register_read_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(&["READ", "$expr$"], false, move |context, inputs| {
            let path = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("READ file: {}", path);

            let state_for_task = Arc::clone(&state_clone);
            let user_for_task = user_clone.clone();
            let path_clone = path.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_read(&state_for_task, &user_for_task, &path_clone).await
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
                    format!("READ failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "READ timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("READ thread failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .unwrap();
}

/// WRITE "path", data
/// Writes content to a file in .gbdrive
pub fn register_write_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["WRITE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let path = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;

                trace!("WRITE to file: {}", path);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let path_clone = path.clone();
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
                            execute_write(&state_for_task, &user_for_task, &path_clone, &data_str)
                                .await
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
                        format!("WRITE failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "WRITE timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("WRITE thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// DELETE FILE "path" / DELETE_FILE "path"
/// Deletes a file from .gbdrive
pub fn register_delete_file_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    // DELETE FILE (space-separated - preferred)
    engine
        .register_custom_syntax(
            &["DELETE", "FILE", "$expr$"],
            false,
            move |context, inputs| {
                let path = context.eval_expression_tree(&inputs[0])?.to_string();

                trace!("DELETE FILE: {}", path);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let path_clone = path.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_delete_file(&state_for_task, &user_for_task, &path_clone).await
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
                        format!("DELETE FILE failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "DELETE FILE timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DELETE FILE thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();

    // DELETE_FILE (underscore - backwards compatibility)
    engine
        .register_custom_syntax(&["DELETE_FILE", "$expr$"], false, move |context, inputs| {
            let path = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("DELETE_FILE: {}", path);

            let state_for_task = Arc::clone(&state_clone2);
            let user_for_task = user_clone2.clone();
            let path_clone = path.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_delete_file(&state_for_task, &user_for_task, &path_clone).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send DELETE_FILE result from thread");
                }
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(_)) => Ok(Dynamic::UNIT),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("DELETE_FILE failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "DELETE_FILE timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("DELETE_FILE thread failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .unwrap();
}

/// COPY "source", "destination"
/// Copies a file within .gbdrive
pub fn register_copy_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["COPY", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let source = context.eval_expression_tree(&inputs[0])?.to_string();
                let destination = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("COPY from {} to {}", source, destination);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let source_clone = source.clone();
                let dest_clone = destination.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_copy(
                                &state_for_task,
                                &user_for_task,
                                &source_clone,
                                &dest_clone,
                            )
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
                        format!("COPY failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "COPY timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("COPY thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// MOVE "source", "destination"
/// Moves/renames a file within .gbdrive
pub fn register_move_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["MOVE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let source = context.eval_expression_tree(&inputs[0])?.to_string();
                let destination = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("MOVE from {} to {}", source, destination);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let source_clone = source.clone();
                let dest_clone = destination.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_move(
                                &state_for_task,
                                &user_for_task,
                                &source_clone,
                                &dest_clone,
                            )
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
                        format!("MOVE failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "MOVE timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("MOVE thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// LIST "path"
/// Lists contents of a directory in .gbdrive
pub fn register_list_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(&["LIST", "$expr$"], false, move |context, inputs| {
            let path = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("LIST directory: {}", path);

            let state_for_task = Arc::clone(&state_clone);
            let user_for_task = user_clone.clone();
            let path_clone = path.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        execute_list(&state_for_task, &user_for_task, &path_clone).await
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
                    format!("LIST failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "LIST timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("LIST thread failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .unwrap();
}

/// COMPRESS files, "archive.zip"
/// Creates a ZIP archive from files
pub fn register_compress_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["COMPRESS", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let files = context.eval_expression_tree(&inputs[0])?;
                let archive_name = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("COMPRESS to: {}", archive_name);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let archive_clone = archive_name.clone();

                // Convert files to Vec<String>
                let file_list: Vec<String> = if files.is_array() {
                    files
                        .clone()
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
                                &archive_clone,
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
                        format!("COMPRESS failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "COMPRESS timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("COMPRESS thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// EXTRACT "archive.zip", "destination/"
/// Extracts an archive to a destination folder
pub fn register_extract_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["EXTRACT", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let archive = context.eval_expression_tree(&inputs[0])?.to_string();
                let destination = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("EXTRACT {} to {}", archive, destination);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let archive_clone = archive.clone();
                let dest_clone = destination.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_extract(
                                &state_for_task,
                                &user_for_task,
                                &archive_clone,
                                &dest_clone,
                            )
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
                        format!("EXTRACT failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "EXTRACT timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("EXTRACT thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// UPLOAD file, "destination_path"
/// Uploads a file to .gbdrive storage
pub fn register_upload_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["UPLOAD", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let file = context.eval_expression_tree(&inputs[0])?;
                let destination = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("UPLOAD to: {}", destination);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let dest_clone = destination.clone();
                let file_data = dynamic_to_file_data(&file);

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_upload(&state_for_task, &user_for_task, file_data, &dest_clone)
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
                        format!("UPLOAD failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "UPLOAD timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("UPLOAD thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// DOWNLOAD "url", "local_path"
/// Downloads a file from URL to local path
pub fn register_download_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["DOWNLOAD", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let url = context.eval_expression_tree(&inputs[0])?.to_string();
                let local_path = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("DOWNLOAD {} to {}", url, local_path);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let url_clone = url.clone();
                let path_clone = local_path.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            execute_download(
                                &state_for_task,
                                &user_for_task,
                                &url_clone,
                                &path_clone,
                            )
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
                        format!("DOWNLOAD failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "DOWNLOAD timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("DOWNLOAD thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// GENERATE_PDF template, data, "output.pdf"
/// Generates a PDF from a template with data
pub fn register_generate_pdf_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["GENERATE_PDF", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let template = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;
                let output = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!("GENERATE_PDF template: {}, output: {}", template, output);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let template_clone = template.clone();
                let output_clone = output.clone();
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
                                &template_clone,
                                data_json,
                                &output_clone,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send GENERATE_PDF result from thread");
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
                        format!("GENERATE_PDF failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "GENERATE_PDF timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("GENERATE_PDF thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// MERGE_PDF files, "merged.pdf"
/// Merges multiple PDF files into one
pub fn register_merge_pdf_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["MERGE_PDF", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let files = context.eval_expression_tree(&inputs[0])?;
                let output = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("MERGE_PDF to: {}", output);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();
                let output_clone = output.clone();

                // Convert files to Vec<String>
                let file_list: Vec<String> = if files.is_array() {
                    files
                        .clone()
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
                            execute_merge_pdf(
                                &state_for_task,
                                &user_for_task,
                                &file_list,
                                &output_clone,
                            )
                            .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send MERGE_PDF result from thread");
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
                        format!("MERGE_PDF failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "MERGE_PDF timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("MERGE_PDF thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

// ============================================================================
// Implementation Functions
// ============================================================================

/// Read file content from .gbdrive
async fn execute_read(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {}", e);
                e
            })?
    };

    let bucket_name = format!("{}.gbai", bot_name);
    let key = format!("{}.gbdrive/{}", bot_name, path);

    let response = client
        .get_object()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await
        .map_err(|e| format!("S3 get failed: {}", e))?;

    let data = response.body.collect().await?.into_bytes();
    let content =
        String::from_utf8(data.to_vec()).map_err(|_| "File content is not valid UTF-8")?;

    trace!("READ successful: {} bytes", content.len());
    Ok(content)
}

/// Write content to file in .gbdrive
async fn execute_write(
    state: &AppState,
    user: &UserSession,
    path: &str,
    content: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {}", e);
                e
            })?
    };

    let bucket_name = format!("{}.gbai", bot_name);
    let key = format!("{}.gbdrive/{}", bot_name, path);

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(content.as_bytes().to_vec().into())
        .send()
        .await
        .map_err(|e| format!("S3 put failed: {}", e))?;

    trace!("WRITE successful: {} bytes to {}", content.len(), path);
    Ok(())
}

/// Delete file from .gbdrive
async fn execute_delete_file(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {}", e);
                e
            })?
    };

    let bucket_name = format!("{}.gbai", bot_name);
    let key = format!("{}.gbdrive/{}", bot_name, path);

    client
        .delete_object()
        .bucket(&bucket_name)
        .key(&key)
        .send()
        .await
        .map_err(|e| format!("S3 delete failed: {}", e))?;

    trace!("DELETE_FILE successful: {}", path);
    Ok(())
}

/// Copy file within .gbdrive
async fn execute_copy(
    state: &AppState,
    user: &UserSession,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {}", e);
                e
            })?
    };

    let bucket_name = format!("{}.gbai", bot_name);
    let source_key = format!("{}.gbdrive/{}", bot_name, source);
    let dest_key = format!("{}.gbdrive/{}", bot_name, destination);

    let copy_source = format!("{}/{}", bucket_name, source_key);

    client
        .copy_object()
        .bucket(&bucket_name)
        .key(&dest_key)
        .copy_source(&copy_source)
        .send()
        .await
        .map_err(|e| format!("S3 copy failed: {}", e))?;

    trace!("COPY successful: {} -> {}", source, destination);
    Ok(())
}

/// Move/rename file within .gbdrive
async fn execute_move(
    state: &AppState,
    user: &UserSession,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Copy first
    execute_copy(state, user, source, destination).await?;

    // Then delete source
    execute_delete_file(state, user, source).await?;

    trace!("MOVE successful: {} -> {}", source, destination);
    Ok(())
}

/// List directory contents in .gbdrive
async fn execute_list(
    state: &AppState,
    user: &UserSession,
    path: &str,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {}", e);
                e
            })?
    };

    let bucket_name = format!("{}.gbai", bot_name);
    let prefix = format!("{}.gbdrive/{}", bot_name, path);

    let response = client
        .list_objects_v2()
        .bucket(&bucket_name)
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| format!("S3 list failed: {}", e))?;

    let files: Vec<String> = response
        .contents()
        .iter()
        .filter_map(|obj| {
            obj.key().map(|k| {
                k.strip_prefix(&format!("{}.gbdrive/", bot_name))
                    .unwrap_or(k)
                    .to_string()
            })
        })
        .collect();

    trace!("LIST successful: {} files", files.len());
    Ok(files)
}

/// Create ZIP archive from files
async fn execute_compress(
    state: &AppState,
    user: &UserSession,
    files: &[String],
    archive_name: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {}", e);
                e
            })?
    };

    // Create temporary file for the archive
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

        zip.start_file(file_name, options.clone())?;
        zip.write_all(content.as_bytes())?;
    }

    zip.finish()?;

    // Upload the archive to .gbdrive
    let archive_content = fs::read(&archive_path)?;
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;
    let bucket_name = format!("{}.gbai", bot_name);
    let key = format!("{}.gbdrive/{}", bot_name, archive_name);

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(archive_content.into())
        .send()
        .await
        .map_err(|e| format!("S3 put failed: {}", e))?;

    // Clean up temp file
    fs::remove_file(&archive_path).ok();

    trace!("COMPRESS successful: {}", archive_name);
    Ok(archive_name.to_string())
}

/// Extract archive to destination
async fn execute_extract(
    state: &AppState,
    user: &UserSession,
    archive: &str,
    destination: &str,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {}", e);
                e
            })?
    };

    let bucket_name = format!("{}.gbai", bot_name);
    let archive_key = format!("{}.gbdrive/{}", bot_name, archive);

    // Download the archive
    let response = client
        .get_object()
        .bucket(&bucket_name)
        .key(&archive_key)
        .send()
        .await
        .map_err(|e| format!("S3 get failed: {}", e))?;

    let data = response.body.collect().await?.into_bytes();

    // Create temp file for extraction
    let temp_dir = std::env::temp_dir();
    let archive_path = temp_dir.join(archive);
    fs::write(&archive_path, &data)?;

    let mut extracted_files = Vec::new();

    // Extract based on file type
    if archive.ends_with(".zip") {
        let file = File::open(&archive_path)?;
        let mut zip = ZipArchive::new(file)?;

        for i in 0..zip.len() {
            let mut zip_file = zip.by_index(i)?;
            let file_name = zip_file.name().to_string();

            let mut content = Vec::new();
            zip_file.read_to_end(&mut content)?;

            let dest_path = format!("{}/{}", destination.trim_end_matches('/'), file_name);

            // Upload extracted file
            let dest_key = format!("{}.gbdrive/{}", bot_name, dest_path);
            client
                .put_object()
                .bucket(&bucket_name)
                .key(&dest_key)
                .body(content.into())
                .send()
                .await
                .map_err(|e| format!("S3 put failed: {}", e))?;

            extracted_files.push(dest_path);
        }
    } else if archive.ends_with(".tar.gz") || archive.ends_with(".tgz") {
        let file = File::open(&archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut tar = Archive::new(decoder);

        for entry in tar.entries()? {
            let mut entry = entry?;
            let file_name = entry.path()?.to_string_lossy().to_string();

            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;

            let dest_path = format!("{}/{}", destination.trim_end_matches('/'), file_name);

            // Upload extracted file
            let dest_key = format!("{}.gbdrive/{}", bot_name, dest_path);
            client
                .put_object()
                .bucket(&bucket_name)
                .key(&dest_key)
                .body(content.into())
                .send()
                .await
                .map_err(|e| format!("S3 put failed: {}", e))?;

            extracted_files.push(dest_path);
        }
    }

    // Clean up temp file
    fs::remove_file(&archive_path).ok();

    trace!("EXTRACT successful: {} files", extracted_files.len());
    Ok(extracted_files)
}

/// File data structure for uploads
struct FileData {
    content: Vec<u8>,
    filename: String,
}

/// Upload file to .gbdrive
async fn execute_upload(
    state: &AppState,
    user: &UserSession,
    file_data: FileData,
    destination: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query bot name: {}", e);
                e
            })?
    };

    let bucket_name = format!("{}.gbai", bot_name);
    let key = format!("{}.gbdrive/{}", bot_name, destination);

    client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(file_data.content.into())
        .send()
        .await
        .map_err(|e| format!("S3 put failed: {}", e))?;

    let url = format!("s3://{}/{}", bucket_name, key);
    trace!("UPLOAD successful: {}", url);
    Ok(url)
}

/// Download file from URL
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
        .map_err(|e| format!("Download failed: {}", e))?;

    let content = response.bytes().await?;

    // Save to .gbdrive
    execute_write(state, user, local_path, &String::from_utf8_lossy(&content)).await?;

    trace!("DOWNLOAD successful: {} -> {}", url, local_path);
    Ok(local_path.to_string())
}

/// PDF generation result
struct PdfResult {
    url: String,
    local_name: String,
}

/// Generate PDF from template
async fn execute_generate_pdf(
    state: &AppState,
    user: &UserSession,
    template: &str,
    data: Value,
    output: &str,
) -> Result<PdfResult, Box<dyn Error + Send + Sync>> {
    // Read template
    let template_content = execute_read(state, user, template).await?;

    // Simple template replacement
    let mut html_content = template_content;
    if let Value::Object(obj) = &data {
        for (key, value) in obj {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            html_content = html_content.replace(&placeholder, &value_str);
        }
    }

    // For now, we save as HTML with instructions
    // In production, use a proper PDF generation library like wkhtmltopdf or headless Chrome
    let pdf_content = format!(
        "<!-- PDF Content Generated from Template: {} -->\n{}",
        template, html_content
    );

    // Save the output
    execute_write(state, user, output, &pdf_content).await?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)?
    };

    let url = format!("s3://{}.gbai/{}.gbdrive/{}", bot_name, bot_name, output);

    trace!("GENERATE_PDF successful: {}", output);
    Ok(PdfResult {
        url,
        local_name: output.to_string(),
    })
}

/// Merge multiple PDFs
async fn execute_merge_pdf(
    state: &AppState,
    user: &UserSession,
    files: &[String],
    output: &str,
) -> Result<PdfResult, Box<dyn Error + Send + Sync>> {
    let mut merged_content = String::from("<!-- Merged PDF -->\n");

    for file in files {
        let content = execute_read(state, user, file).await?;
        merged_content.push_str(&format!("\n<!-- From: {} -->\n{}\n", file, content));
    }

    // Save merged content
    execute_write(state, user, output, &merged_content).await?;

    let bot_name: String = {
        let mut db_conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;
        bots.filter(id.eq(&user.bot_id))
            .select(name)
            .first(&mut *db_conn)?
    };

    let url = format!("s3://{}.gbai/{}.gbdrive/{}", bot_name, bot_name, output);

    trace!(
        "MERGE_PDF successful: {} files merged to {}",
        files.len(),
        output
    );
    Ok(PdfResult {
        url,
        local_name: output.to_string(),
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert Dynamic to JSON Value
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

/// Convert Dynamic to FileData
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
