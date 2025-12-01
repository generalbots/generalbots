//! CARD keyword - Creates beautiful Instagram-style posts from prompts
//!
//! Syntax:
//!   CARD image_prompt, text_prompt TO variable
//!   CARD image_prompt, text_prompt, style TO variable
//!   CARD image_prompt, text_prompt, style, count TO variable
//!
//! Examples:
//!   CARD "sunset over mountains", "inspirational quote about nature" TO post
//!   CARD "modern office", "productivity tips", "minimal" TO cards
//!   CARD "healthy food", "nutrition facts", "vibrant", 5 TO carousel

use crate::basic::runtime::{BasicRuntime, BasicValue};
use crate::llm::LLMProvider;
use anyhow::{anyhow, Result};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Card style presets for Instagram posts
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CardStyle {
    #[default]
    Modern,
    Minimal,
    Vibrant,
    Dark,
    Light,
    Gradient,
    Polaroid,
    Magazine,
    Story,
    Carousel,
}

impl From<&str> for CardStyle {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "minimal" => CardStyle::Minimal,
            "vibrant" => CardStyle::Vibrant,
            "dark" => CardStyle::Dark,
            "light" => CardStyle::Light,
            "gradient" => CardStyle::Gradient,
            "polaroid" => CardStyle::Polaroid,
            "magazine" => CardStyle::Magazine,
            "story" => CardStyle::Story,
            "carousel" => CardStyle::Carousel,
            _ => CardStyle::Modern,
        }
    }
}

/// Card dimensions for different formats
#[derive(Debug, Clone, Copy)]
pub struct CardDimensions {
    pub width: u32,
    pub height: u32,
}

impl CardDimensions {
    pub const INSTAGRAM_SQUARE: Self = Self {
        width: 1080,
        height: 1080,
    };
    pub const INSTAGRAM_PORTRAIT: Self = Self {
        width: 1080,
        height: 1350,
    };
    pub const INSTAGRAM_STORY: Self = Self {
        width: 1080,
        height: 1920,
    };
    pub const INSTAGRAM_LANDSCAPE: Self = Self {
        width: 1080,
        height: 566,
    };

    pub fn for_style(style: &CardStyle) -> Self {
        match style {
            CardStyle::Story => Self::INSTAGRAM_STORY,
            CardStyle::Carousel => Self::INSTAGRAM_SQUARE,
            _ => Self::INSTAGRAM_SQUARE,
        }
    }
}

/// Text overlay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOverlay {
    pub text: String,
    pub font_size: f32,
    pub color: [u8; 4],
    pub position: TextPosition,
    pub max_width_ratio: f32,
    pub shadow: bool,
    pub background: Option<[u8; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TextPosition {
    Top,
    #[default]
    Center,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Generated card result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardResult {
    pub image_path: String,
    pub image_url: Option<String>,
    pub text_content: String,
    pub hashtags: Vec<String>,
    pub caption: String,
    pub style: String,
    pub dimensions: (u32, u32),
}

/// Card generation configuration
#[derive(Debug, Clone)]
pub struct CardConfig {
    pub style: CardStyle,
    pub dimensions: CardDimensions,
    pub text_position: TextPosition,
    pub include_hashtags: bool,
    pub include_caption: bool,
    pub brand_watermark: Option<String>,
}

impl Default for CardConfig {
    fn default() -> Self {
        Self {
            style: CardStyle::Modern,
            dimensions: CardDimensions::INSTAGRAM_SQUARE,
            text_position: TextPosition::Center,
            include_hashtags: true,
            include_caption: true,
            brand_watermark: None,
        }
    }
}

/// CARD keyword implementation
pub struct CardKeyword {
    llm_provider: Arc<dyn LLMProvider>,
    output_dir: String,
}

impl CardKeyword {
    pub fn new(llm_provider: Arc<dyn LLMProvider>, output_dir: String) -> Self {
        Self {
            llm_provider,
            output_dir,
        }
    }

    /// Execute CARD keyword
    pub async fn execute(
        &self,
        image_prompt: &str,
        text_prompt: &str,
        style: Option<&str>,
        count: Option<usize>,
    ) -> Result<Vec<CardResult>> {
        let card_style = style.map(CardStyle::from).unwrap_or_default();
        let card_count = count.unwrap_or(1).min(10); // Max 10 cards

        let config = CardConfig {
            style: card_style.clone(),
            dimensions: CardDimensions::for_style(&card_style),
            ..Default::default()
        };

        let mut results = Vec::with_capacity(card_count);

        for i in 0..card_count {
            let result = self
                .generate_single_card(image_prompt, text_prompt, &config, i)
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Generate a single card
    async fn generate_single_card(
        &self,
        image_prompt: &str,
        text_prompt: &str,
        config: &CardConfig,
        index: usize,
    ) -> Result<CardResult> {
        // Step 1: Generate optimized text content using LLM
        let text_content = self.generate_text_content(text_prompt, config).await?;

        // Step 2: Generate image using image generation API
        let base_image = self.generate_image(image_prompt, config).await?;

        // Step 3: Apply style and text overlay
        let styled_image = self.apply_style_and_text(&base_image, &text_content, config)?;

        // Step 4: Generate hashtags and caption
        let (hashtags, caption) = self.generate_social_content(&text_content, config).await?;

        // Step 5: Save the final image
        let filename = format!(
            "card_{}_{}.png",
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            index
        );
        let image_path = format!("{}/{}", self.output_dir, filename);

        styled_image.save(&image_path)?;

        Ok(CardResult {
            image_path: image_path.clone(),
            image_url: None, // Will be set after upload to storage
            text_content,
            hashtags,
            caption,
            style: format!("{:?}", config.style),
            dimensions: (config.dimensions.width, config.dimensions.height),
        })
    }

    /// Generate optimized text content for the card
    async fn generate_text_content(
        &self,
        text_prompt: &str,
        config: &CardConfig,
    ) -> Result<String> {
        let style_instruction = match config.style {
            CardStyle::Minimal => "Keep it very short, 1-2 impactful words or a brief phrase.",
            CardStyle::Vibrant => "Make it energetic and exciting with action words.",
            CardStyle::Dark => "Create a mysterious, sophisticated tone.",
            CardStyle::Light => "Keep it uplifting and positive.",
            CardStyle::Magazine => "Write like a magazine headline, catchy and professional.",
            CardStyle::Story => "Create engaging story-style text that draws people in.",
            _ => "Create compelling, shareable text perfect for social media.",
        };

        let prompt = format!(
            r#"Create text for an Instagram post image overlay.

Topic/Theme: {}

Style Guidelines:
- {}
- Maximum 50 characters for main text
- Should be visually impactful when overlaid on an image
- Use proper capitalization for visual appeal
- No hashtags in the main text (those come separately)

Respond with ONLY the text content, nothing else."#,
            text_prompt, style_instruction
        );

        let response = self.llm_provider.complete(&prompt, None).await?;

        // Clean up the response
        let text = response.trim().to_string();

        // Ensure it's not too long
        if text.len() > 100 {
            Ok(text.chars().take(100).collect::<String>() + "...")
        } else {
            Ok(text)
        }
    }

    /// Generate the base image
    async fn generate_image(
        &self,
        image_prompt: &str,
        config: &CardConfig,
    ) -> Result<DynamicImage> {
        let enhanced_prompt = self.enhance_image_prompt(image_prompt, config);

        // Call image generation service
        let image_bytes = self
            .llm_provider
            .generate_image(
                &enhanced_prompt,
                config.dimensions.width,
                config.dimensions.height,
            )
            .await?;

        let image = image::load_from_memory(&image_bytes)?;
        Ok(image)
    }

    /// Enhance the image prompt based on style
    fn enhance_image_prompt(&self, base_prompt: &str, config: &CardConfig) -> String {
        let style_modifiers = match config.style {
            CardStyle::Minimal => {
                "minimalist, clean, simple composition, lots of negative space, muted colors"
            }
            CardStyle::Vibrant => "vibrant colors, high saturation, dynamic, energetic, bold",
            CardStyle::Dark => "dark moody atmosphere, dramatic lighting, deep shadows, cinematic",
            CardStyle::Light => "bright, airy, soft lighting, pastel colors, ethereal",
            CardStyle::Gradient => "smooth color gradients, abstract, flowing colors",
            CardStyle::Polaroid => "vintage polaroid style, slightly faded, warm tones, nostalgic",
            CardStyle::Magazine => "high fashion, editorial style, professional photography, sharp",
            CardStyle::Story => "vertical composition, immersive, storytelling, atmospheric",
            CardStyle::Carousel => "consistent style, series-ready, cohesive aesthetic",
            CardStyle::Modern => "modern, trendy, instagram aesthetic, high quality",
        };

        format!(
            "{}, {}, perfect for Instagram, professional quality, 4K, highly detailed",
            base_prompt, style_modifiers
        )
    }

    /// Apply style effects and text overlay to the image
    fn apply_style_and_text(
        &self,
        image: &DynamicImage,
        text: &str,
        config: &CardConfig,
    ) -> Result<DynamicImage> {
        let mut rgba_image = image.to_rgba8();

        // Apply style-specific filters
        self.apply_style_filter(&mut rgba_image, &config.style);

        // Add text overlay
        self.add_text_overlay(&mut rgba_image, text, config)?;

        // Add watermark if configured
        if let Some(ref watermark) = config.brand_watermark {
            self.add_watermark(&mut rgba_image, watermark)?;
        }

        Ok(DynamicImage::ImageRgba8(rgba_image))
    }

    /// Apply style-specific image filters
    fn apply_style_filter(&self, image: &mut RgbaImage, style: &CardStyle) {
        match style {
            CardStyle::Dark => {
                // Darken and increase contrast
                for pixel in image.pixels_mut() {
                    pixel[0] = (pixel[0] as f32 * 0.7) as u8;
                    pixel[1] = (pixel[1] as f32 * 0.7) as u8;
                    pixel[2] = (pixel[2] as f32 * 0.7) as u8;
                }
            }
            CardStyle::Light => {
                // Brighten slightly
                for pixel in image.pixels_mut() {
                    pixel[0] = ((pixel[0] as f32 * 1.1).min(255.0)) as u8;
                    pixel[1] = ((pixel[1] as f32 * 1.1).min(255.0)) as u8;
                    pixel[2] = ((pixel[2] as f32 * 1.1).min(255.0)) as u8;
                }
            }
            CardStyle::Polaroid => {
                // Add warm vintage tint
                for pixel in image.pixels_mut() {
                    pixel[0] = ((pixel[0] as f32 * 1.05).min(255.0)) as u8;
                    pixel[1] = ((pixel[1] as f32 * 0.95).min(255.0)) as u8;
                    pixel[2] = ((pixel[2] as f32 * 0.85).min(255.0)) as u8;
                }
            }
            CardStyle::Vibrant => {
                // Increase saturation
                for pixel in image.pixels_mut() {
                    let r = pixel[0] as f32;
                    let g = pixel[1] as f32;
                    let b = pixel[2] as f32;
                    let avg = (r + g + b) / 3.0;
                    let factor = 1.3;
                    pixel[0] = ((r - avg) * factor + avg).clamp(0.0, 255.0) as u8;
                    pixel[1] = ((g - avg) * factor + avg).clamp(0.0, 255.0) as u8;
                    pixel[2] = ((b - avg) * factor + avg).clamp(0.0, 255.0) as u8;
                }
            }
            _ => {}
        }
    }

    /// Add text overlay to the image
    fn add_text_overlay(
        &self,
        image: &mut RgbaImage,
        text: &str,
        config: &CardConfig,
    ) -> Result<()> {
        let (width, height) = (image.width(), image.height());

        // Load font (embedded or from file)
        let font_data = include_bytes!("../../../assets/fonts/Inter-Bold.ttf");
        let font = Font::try_from_bytes(font_data as &[u8])
            .ok_or_else(|| anyhow!("Failed to load font"))?;

        // Calculate font size based on image dimensions and text length
        let base_size = (width as f32 * 0.08).min(height as f32 * 0.1);
        let scale = Scale::uniform(base_size);

        // Calculate text position
        let (text_width, text_height) = text_size(scale, &font, text);
        let (x, y) = self.calculate_text_position(
            width,
            height,
            text_width as u32,
            text_height as u32,
            &config.text_position,
        );

        // Draw text shadow for better readability
        let shadow_color = Rgba([0u8, 0u8, 0u8, 180u8]);
        draw_text_mut(image, shadow_color, x + 3, y + 3, scale, &font, text);

        // Draw main text
        let text_color = match config.style {
            CardStyle::Dark => Rgba([255u8, 255u8, 255u8, 255u8]),
            CardStyle::Light => Rgba([30u8, 30u8, 30u8, 255u8]),
            _ => Rgba([255u8, 255u8, 255u8, 255u8]),
        };
        draw_text_mut(image, text_color, x, y, scale, &font, text);

        Ok(())
    }

    /// Calculate text position based on configuration
    fn calculate_text_position(
        &self,
        img_width: u32,
        img_height: u32,
        text_width: u32,
        text_height: u32,
        position: &TextPosition,
    ) -> (i32, i32) {
        let padding = (img_width as f32 * 0.05) as i32;

        match position {
            TextPosition::Top => (
                ((img_width - text_width) / 2) as i32,
                padding + text_height as i32,
            ),
            TextPosition::Center => (
                ((img_width - text_width) / 2) as i32,
                ((img_height - text_height) / 2) as i32,
            ),
            TextPosition::Bottom => (
                ((img_width - text_width) / 2) as i32,
                (img_height - text_height) as i32 - padding,
            ),
            TextPosition::TopLeft => (padding, padding + text_height as i32),
            TextPosition::TopRight => (
                (img_width - text_width) as i32 - padding,
                padding + text_height as i32,
            ),
            TextPosition::BottomLeft => (padding, (img_height - text_height) as i32 - padding),
            TextPosition::BottomRight => (
                (img_width - text_width) as i32 - padding,
                (img_height - text_height) as i32 - padding,
            ),
        }
    }

    /// Add brand watermark
    fn add_watermark(&self, image: &mut RgbaImage, watermark: &str) -> Result<()> {
        let font_data = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
        let font = Font::try_from_bytes(font_data as &[u8])
            .ok_or_else(|| anyhow!("Failed to load font"))?;

        let scale = Scale::uniform(image.width() as f32 * 0.025);
        let color = Rgba([255u8, 255u8, 255u8, 128u8]);

        let padding = 20i32;
        let x = padding;
        let y = (image.height() - 30) as i32;

        draw_text_mut(image, color, x, y, scale, &font, watermark);

        Ok(())
    }

    /// Generate hashtags and caption for the post
    async fn generate_social_content(
        &self,
        text_content: &str,
        config: &CardConfig,
    ) -> Result<(Vec<String>, String)> {
        if !config.include_hashtags && !config.include_caption {
            return Ok((vec![], String::new()));
        }

        let prompt = format!(
            r#"Based on this Instagram post text: "{}"

Generate:
1. A short, engaging caption (1-2 sentences max)
2. 5-10 relevant hashtags (without the # symbol)

Format your response exactly like this:
CAPTION: [your caption here]
HASHTAGS: tag1, tag2, tag3, tag4, tag5"#,
            text_content
        );

        let response = self.llm_provider.complete(&prompt, None).await?;

        // Parse the response
        let mut caption = String::new();
        let mut hashtags = Vec::new();

        for line in response.lines() {
            if line.starts_with("CAPTION:") {
                caption = line.trim_start_matches("CAPTION:").trim().to_string();
            } else if line.starts_with("HASHTAGS:") {
                let tags = line.trim_start_matches("HASHTAGS:").trim();
                hashtags = tags.split(',').map(|t| format!("#{}", t.trim())).collect();
            }
        }

        Ok((hashtags, caption))
    }
}

/// Register CARD keyword with the BASIC runtime
pub fn register_card_keyword(runtime: &mut BasicRuntime, llm_provider: Arc<dyn LLMProvider>) {
    let output_dir = runtime
        .get_config("output_dir")
        .unwrap_or_else(|| "/tmp/gb_cards".to_string());

    let keyword = Arc::new(Mutex::new(CardKeyword::new(llm_provider, output_dir)));

    runtime.register_keyword("CARD", move |args, _ctx| {
        let keyword = keyword.clone();
        Box::pin(async move {
            if args.len() < 2 {
                return Err(anyhow!(
                    "CARD requires at least 2 arguments: image_prompt, text_prompt"
                ));
            }

            let image_prompt = args[0].as_string()?;
            let text_prompt = args[1].as_string()?;
            let style = args.get(2).map(|v| v.as_string()).transpose()?;
            let count = args
                .get(3)
                .map(|v| v.as_number().map(|n| n as usize))
                .transpose()?;

            let kw = keyword.lock().await;
            let results = kw
                .execute(&image_prompt, &text_prompt, style.as_deref(), count)
                .await?;

            // Convert results to BasicValue
            let value = if results.len() == 1 {
                BasicValue::Object(serde_json::to_value(&results[0])?)
            } else {
                BasicValue::Array(
                    results
                        .into_iter()
                        .map(|r| BasicValue::Object(serde_json::to_value(&r).unwrap()))
                        .collect(),
                )
            };

            Ok(value)
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_style_from_string() {
        assert!(matches!(CardStyle::from("minimal"), CardStyle::Minimal));
        assert!(matches!(CardStyle::from("VIBRANT"), CardStyle::Vibrant));
        assert!(matches!(CardStyle::from("unknown"), CardStyle::Modern));
    }

    #[test]
    fn test_card_dimensions() {
        assert_eq!(CardDimensions::INSTAGRAM_SQUARE.width, 1080);
        assert_eq!(CardDimensions::INSTAGRAM_SQUARE.height, 1080);
        assert_eq!(CardDimensions::INSTAGRAM_STORY.height, 1920);
    }

    #[test]
    fn test_text_position_calculation() {
        // Create a mock keyword for testing
        // In real tests, we'd use a mock LLM provider
    }
}
