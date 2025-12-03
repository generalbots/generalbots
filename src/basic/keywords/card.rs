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

//! CARD keyword - Creates beautiful Instagram-style posts from prompts
//!
//! This module generates social media cards by combining AI-generated images
//! with AI-generated text. The LLM handles both image generation and text
//! composition, eliminating the need for local image processing.
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
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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

/// Text position configuration
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
/// Uses LLM for both image generation and text composition
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

        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir)?;

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

        // Step 2: Create enhanced prompt that includes text overlay instructions
        let enhanced_image_prompt = self.create_card_prompt(image_prompt, &text_content, config);

        // Step 3: Generate image with text baked in via LLM image generation
        let image_bytes = self
            .llm_provider
            .generate_image(
                &enhanced_image_prompt,
                config.dimensions.width,
                config.dimensions.height,
            )
            .await?;

        // Step 4: Save the image
        let filename = format!(
            "card_{}_{}.png",
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            index
        );
        let image_path = format!("{}/{}", self.output_dir, filename);

        // Ensure parent directory exists
        if let Some(parent) = Path::new(&image_path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&image_path, &image_bytes)?;

        // Step 5: Generate hashtags and caption
        let (hashtags, caption) = self.generate_social_content(&text_content, config).await?;

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

    /// Create an enhanced prompt for image generation that includes text overlay
    fn create_card_prompt(
        &self,
        image_prompt: &str,
        text_content: &str,
        config: &CardConfig,
    ) -> String {
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

        let text_position = match config.text_position {
            TextPosition::Top => "at the top",
            TextPosition::Center => "in the center",
            TextPosition::Bottom => "at the bottom",
            TextPosition::TopLeft => "in the top left corner",
            TextPosition::TopRight => "in the top right corner",
            TextPosition::BottomLeft => "in the bottom left corner",
            TextPosition::BottomRight => "in the bottom right corner",
        };

        let text_color = match config.style {
            CardStyle::Dark => "white",
            CardStyle::Light => "dark gray or black",
            _ => "white with a subtle shadow",
        };

        format!(
            r#"Create an Instagram-ready image with the following specifications:

Background/Scene: {}
Style: {}, perfect for Instagram, professional quality, 4K, highly detailed

Text Overlay Requirements:
- Display the text "{}" {} of the image
- Use bold, modern typography
- Text color should be {} for readability
- Add a subtle text shadow or background blur behind text for contrast
- Leave appropriate padding around text

The image should be {} x {} pixels, optimized for social media.
Make the text an integral part of the design, not just overlaid."#,
            image_prompt,
            style_modifiers,
            text_content,
            text_position,
            text_color,
            config.dimensions.width,
            config.dimensions.height
        )
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
            let line_trimmed = line.trim();
            if line_trimmed.starts_with("CAPTION:") {
                caption = line_trimmed
                    .trim_start_matches("CAPTION:")
                    .trim()
                    .to_string();
            } else if line_trimmed.starts_with("HASHTAGS:") {
                let tags = line_trimmed.trim_start_matches("HASHTAGS:").trim();
                hashtags = tags
                    .split(',')
                    .map(|t| {
                        let tag = t.trim().trim_start_matches('#');
                        format!("#{}", tag)
                    })
                    .filter(|t| t.len() > 1)
                    .collect();
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
                .map(|v| v.as_integer().map(|i| i as usize))
                .transpose()?;

            let keyword = keyword.lock().await;
            let results = keyword
                .execute(&image_prompt, &text_prompt, style.as_deref(), count)
                .await?;

            // Convert results to BasicValue
            let result_values: Vec<BasicValue> = results
                .into_iter()
                .map(|r| {
                    BasicValue::Object(serde_json::json!({
                        "image_path": r.image_path,
                        "image_url": r.image_url,
                        "text": r.text_content,
                        "hashtags": r.hashtags,
                        "caption": r.caption,
                        "style": r.style,
                        "width": r.dimensions.0,
                        "height": r.dimensions.1,
                    }))
                })
                .collect();

            if result_values.len() == 1 {
                Ok(result_values.into_iter().next().unwrap())
            } else {
                Ok(BasicValue::Array(result_values))
            }
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
        assert!(matches!(CardStyle::from("dark"), CardStyle::Dark));
        assert!(matches!(CardStyle::from("unknown"), CardStyle::Modern));
    }

    #[test]
    fn test_card_dimensions_for_style() {
        let story_dims = CardDimensions::for_style(&CardStyle::Story);
        assert_eq!(story_dims.width, 1080);
        assert_eq!(story_dims.height, 1920);

        let square_dims = CardDimensions::for_style(&CardStyle::Modern);
        assert_eq!(square_dims.width, 1080);
        assert_eq!(square_dims.height, 1080);
    }

    #[test]
    fn test_card_config_default() {
        let config = CardConfig::default();
        assert!(matches!(config.style, CardStyle::Modern));
        assert!(config.include_hashtags);
        assert!(config.include_caption);
        assert!(config.brand_watermark.is_none());
    }
}
