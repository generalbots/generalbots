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

//! QR Code generation keyword
//!
//! Provides BASIC keywords:
//! - QR CODE data -> generates QR code image, returns file path
//! - QR CODE data, size -> generates QR code with specified size
//! - QR CODE data, size, output_path -> generates QR code to specific path

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use png::{BitDepth, ColorType, Encoder};
use qrcode::QrCode;
use rhai::{Dynamic, Engine};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// Register QR code keywords
pub fn register_qrcode_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_qr_code_keyword(state.clone(), user.clone(), engine);
    register_qr_code_with_size_keyword(state.clone(), user.clone(), engine);
    register_qr_code_full_keyword(state, user, engine);
}

/// QR CODE data
/// Generates a QR code image with default size (256x256)
pub fn register_qr_code_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(&["QR", "CODE", "$expr$"], false, move |context, inputs| {
            let data = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("QR CODE: Generating QR code for data: {}", data);

            let state_for_task = Arc::clone(&state_clone);
            let user_for_task = user_clone.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let result =
                    execute_qr_code_generation(&state_for_task, &user_for_task, &data, 256, None);
                if tx.send(result).is_err() {
                    error!("Failed to send QR CODE result from thread");
                }
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(result)) => Ok(Dynamic::from(result)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("QR CODE failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "QR CODE generation timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("QR CODE thread failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .unwrap();
}

/// QR CODE data, size
/// Generates a QR code image with specified size
pub fn register_qr_code_with_size_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["QR", "CODE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let data = context.eval_expression_tree(&inputs[0])?.to_string();
                let size = context
                    .eval_expression_tree(&inputs[1])?
                    .as_int()
                    .unwrap_or(256) as u32;

                trace!(
                    "QR_CODE: Generating QR code with size {} for: {}",
                    size,
                    data
                );

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let result = execute_qr_code_generation(
                        &state_for_task,
                        &user_for_task,
                        &data,
                        size,
                        None,
                    );
                    if tx.send(result).is_err() {
                        error!("Failed to send QR_CODE result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("QR_CODE failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "QR_CODE generation timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("QR_CODE thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

/// QR_CODE data, size, output_path
/// Generates a QR code image with specified size and output path
pub fn register_qr_code_full_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["QR_CODE", "$expr$", ",", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let data = context.eval_expression_tree(&inputs[0])?.to_string();
                let size = context
                    .eval_expression_tree(&inputs[1])?
                    .as_int()
                    .unwrap_or(256) as u32;
                let output_path = context.eval_expression_tree(&inputs[2])?.to_string();

                trace!(
                    "QR_CODE: Generating QR code with size {} to {} for: {}",
                    size,
                    output_path,
                    data
                );

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user_clone.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let result = execute_qr_code_generation(
                        &state_for_task,
                        &user_for_task,
                        &data,
                        size,
                        Some(&output_path),
                    );
                    if tx.send(result).is_err() {
                        error!("Failed to send QR_CODE result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("QR_CODE failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "QR_CODE generation timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("QR_CODE thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

fn execute_qr_code_generation(
    state: &AppState,
    user: &UserSession,
    data: &str,
    size: u32,
    output_path: Option<&str>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Generate QR code
    let code = QrCode::new(data.as_bytes())?;

    // Get the QR code as a matrix of bools
    let matrix = code.to_colors();
    let qr_width = code.width();

    // Calculate scale factor to reach target size
    let scale = (size as usize) / qr_width;
    let actual_size = qr_width * scale;

    // Create grayscale pixel buffer
    let mut pixels: Vec<u8> = Vec::with_capacity(actual_size * actual_size);

    for y in 0..actual_size {
        for x in 0..actual_size {
            let qr_x = x / scale;
            let qr_y = y / scale;
            let idx = qr_y * qr_width + qr_x;
            let is_dark = matrix
                .get(idx)
                .map(|c| *c == qrcode::Color::Dark)
                .unwrap_or(false);
            pixels.push(if is_dark { 0 } else { 255 });
        }
    }

    // Determine output path
    let data_dir = state
        .config
        .as_ref()
        .map(|c| c.data_dir.as_str())
        .unwrap_or("./botserver-stack/data");

    let final_path = match output_path {
        Some(path) => {
            if Path::new(path).is_absolute() {
                path.to_string()
            } else {
                format!("{}/bots/{}/gbdrive/{}", data_dir, user.bot_id, path)
            }
        }
        None => {
            let filename = format!("qrcode_{}.png", Uuid::new_v4());
            let base_path = format!("{}/bots/{}/gbdrive", data_dir, user.bot_id);

            // Ensure directory exists
            std::fs::create_dir_all(&base_path)?;

            format!("{}/{}", base_path, filename)
        }
    };

    // Ensure parent directory exists
    if let Some(parent) = Path::new(&final_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Save as PNG using png crate directly
    let file = File::create(&final_path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(w, actual_size as u32, actual_size as u32);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;

    trace!("QR code generated: {}", final_path);
    Ok(final_path)
}

/// Generate QR code with custom colors
pub fn generate_qr_code_colored(
    data: &str,
    size: u32,
    foreground: [u8; 3],
    background: [u8; 3],
    output_path: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let code = QrCode::new(data.as_bytes())?;

    // Get the QR code as a matrix of bools
    let matrix = code.to_colors();
    let qr_width = code.width();

    // Calculate scale factor to reach target size
    let scale = (size as usize) / qr_width;
    let actual_size = qr_width * scale;

    // Create RGB pixel buffer (3 bytes per pixel)
    let mut pixels: Vec<u8> = Vec::with_capacity(actual_size * actual_size * 3);

    for y in 0..actual_size {
        for x in 0..actual_size {
            let qr_x = x / scale;
            let qr_y = y / scale;
            let idx = qr_y * qr_width + qr_x;
            let is_dark = matrix
                .get(idx)
                .map(|c| *c == qrcode::Color::Dark)
                .unwrap_or(false);
            let color = if is_dark { foreground } else { background };
            pixels.extend_from_slice(&color);
        }
    }

    // Save as PNG
    let file = File::create(output_path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(w, actual_size as u32, actual_size as u32);
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;

    Ok(output_path.to_string())
}

/// Generate QR code with logo overlay
/// Note: Logo overlay requires the image crate. This simplified version
/// generates a QR code with a white center area where a logo can be placed manually.
pub fn generate_qr_code_with_logo(
    data: &str,
    size: u32,
    _logo_path: &str,
    output_path: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Generate QR code with higher error correction for logo overlay
    let code = QrCode::with_error_correction_level(data.as_bytes(), qrcode::EcLevel::H)?;

    // Get the QR code as a matrix
    let matrix = code.to_colors();
    let qr_width = code.width();

    // Calculate scale factor
    let scale = (size as usize) / qr_width;
    let actual_size = qr_width * scale;

    // Calculate logo area (center 20% of the QR code)
    let logo_size = actual_size / 5;
    let logo_start = (actual_size - logo_size) / 2;
    let logo_end = logo_start + logo_size;

    // Create RGBA pixel buffer (4 bytes per pixel)
    let mut pixels: Vec<u8> = Vec::with_capacity(actual_size * actual_size * 4);

    for y in 0..actual_size {
        for x in 0..actual_size {
            // Check if we're in the logo area
            if x >= logo_start && x < logo_end && y >= logo_start && y < logo_end {
                // White background for logo area
                pixels.extend_from_slice(&[255, 255, 255, 255]);
            } else {
                let qr_x = x / scale;
                let qr_y = y / scale;
                let idx = qr_y * qr_width + qr_x;
                let is_dark = matrix
                    .get(idx)
                    .map(|c| *c == qrcode::Color::Dark)
                    .unwrap_or(false);
                if is_dark {
                    pixels.extend_from_slice(&[0, 0, 0, 255]);
                } else {
                    pixels.extend_from_slice(&[255, 255, 255, 255]);
                }
            }
        }
    }

    // Save as PNG
    let file = File::create(output_path)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(w, actual_size as u32, actual_size as u32);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;

    // Note: Logo overlay not supported without image crate
    // The QR code has a white center area where a logo can be placed manually
    trace!("QR code with logo placeholder generated: {}", output_path);

    Ok(output_path.to_string())
}
