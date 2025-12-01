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
use image::Luma;
use log::{error, trace};
use qrcode::QrCode;
use rhai::{Dynamic, Engine};
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

    // Render to image
    let image = code.render::<Luma<u8>>().min_dimensions(size, size).build();

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

    // Save image
    image.save(&final_path)?;

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
    use image::{Rgb, RgbImage};

    let code = QrCode::new(data.as_bytes())?;
    let qr_image = code.render::<Luma<u8>>().min_dimensions(size, size).build();

    // Convert to RGB with custom colors
    let mut rgb_image = RgbImage::new(qr_image.width(), qr_image.height());

    for (x, y, pixel) in qr_image.enumerate_pixels() {
        let color = if pixel[0] == 0 {
            Rgb(foreground)
        } else {
            Rgb(background)
        };
        rgb_image.put_pixel(x, y, color);
    }

    rgb_image.save(output_path)?;
    Ok(output_path.to_string())
}

/// Generate QR code with logo overlay
pub fn generate_qr_code_with_logo(
    data: &str,
    size: u32,
    logo_path: &str,
    output_path: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use image::{imageops, DynamicImage, GenericImageView, Rgba, RgbaImage};

    // Generate QR code with higher error correction for logo overlay
    let code = QrCode::with_error_correction_level(data.as_bytes(), qrcode::EcLevel::H)?;
    let qr_image = code.render::<Luma<u8>>().min_dimensions(size, size).build();

    // Convert to RGBA
    let mut rgba_image = RgbaImage::new(qr_image.width(), qr_image.height());
    for (x, y, pixel) in qr_image.enumerate_pixels() {
        let color = if pixel[0] == 0 {
            Rgba([0, 0, 0, 255])
        } else {
            Rgba([255, 255, 255, 255])
        };
        rgba_image.put_pixel(x, y, color);
    }

    // Load and resize logo
    let logo = image::open(logo_path)?;
    let logo_size = size / 5; // Logo should be about 20% of QR code size
    let resized_logo = logo.resize(logo_size, logo_size, imageops::FilterType::Lanczos3);

    // Calculate center position
    let center_x = (rgba_image.width() - resized_logo.width()) / 2;
    let center_y = (rgba_image.height() - resized_logo.height()) / 2;

    // Overlay logo
    let mut final_image = DynamicImage::ImageRgba8(rgba_image);
    imageops::overlay(
        &mut final_image,
        &resized_logo,
        center_x.into(),
        center_y.into(),
    );

    final_image.save(output_path)?;
    Ok(output_path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_code_generation() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_qr.png");

        // Create a mock state and user for testing
        // In real tests, you'd set up proper test fixtures
        let result = QrCode::new(b"https://example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_qr_code_with_unicode() {
        let result = QrCode::new("Hello 世界 🌍".as_bytes());
        assert!(result.is_ok());
    }

    #[test]
    fn test_qr_code_long_data() {
        let long_data = "A".repeat(1000);
        let result = QrCode::new(long_data.as_bytes());
        assert!(result.is_ok());
    }

    #[test]
    fn test_qr_code_url() {
        let url = "https://example.com/path?param=value&other=123";
        let result = QrCode::new(url.as_bytes());
        assert!(result.is_ok());
    }

    #[test]
    fn test_qr_code_json() {
        let json = r#"{"id": 123, "name": "Test", "active": true}"#;
        let result = QrCode::new(json.as_bytes());
        assert!(result.is_ok());
    }
}
