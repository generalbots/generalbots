use super::*;
use crate::types::{ElementContent, ElementStyle, Slide, SlideElement};
use uuid::Uuid;

pub fn slides_from_markdown(md: &str) -> Vec<Slide> {
    let theme = create_default_theme();
    let mut slides = Vec::new();
    let sections: Vec<&str> = md.split("\n---\n").collect();

    for section in sections {
        let lines: Vec<&str> = section.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.is_empty() {
            continue;
        }

        let mut slide = create_blank_slide(&theme);
        let mut y_offset = 50.0;

        for line in lines {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("# ") {
                slide.elements.push(create_text_element(
                    stripped,
                    &TextElementParams {
                        x: 50.0,
                        y: y_offset,
                        width: 860.0,
                        height: 60.0,
                        font_size: 44.0,
                        bold: true,
                    },
                    &theme,
                ));
                y_offset += 80.0;
            } else if let Some(stripped) = trimmed.strip_prefix("## ") {
                slide.elements.push(create_text_element(
                    stripped,
                    &TextElementParams {
                        x: 50.0,
                        y: y_offset,
                        width: 860.0,
                        height: 50.0,
                        font_size: 32.0,
                        bold: true,
                    },
                    &theme,
                ));
                y_offset += 60.0;
            } else if let Some(stripped) = trimmed.strip_prefix("### ") {
                slide.elements.push(create_text_element(
                    stripped,
                    &TextElementParams {
                        x: 50.0,
                        y: y_offset,
                        width: 860.0,
                        height: 40.0,
                        font_size: 24.0,
                        bold: true,
                    },
                    &theme,
                ));
                y_offset += 50.0;
            } else if trimmed.starts_with("![") {
                if let Some(start) = trimmed.find('(') {
                    if let Some(end) = trimmed.find(')') {
                        let src = &trimmed[start + 1..end];
                        slide.elements.push(SlideElement {
                            id: Uuid::new_v4().to_string(),
                            element_type: "image".to_string(),
                            x: 50.0,
                            y: y_offset,
                            width: 400.0,
                            height: 300.0,
                            rotation: 0.0,
                            content: ElementContent {
                                text: None,
                                html: None,
                                src: Some(src.to_string()),
                                shape_type: None,
                                chart_data: None,
                                table_data: None,
                            },
                            style: ElementStyle::default(),
                            animations: vec![],
                            z_index: slide.elements.len() as i32,
                            locked: false,
                        });
                        y_offset += 320.0;
                    }
                }
            } else if !trimmed.is_empty() {
                slide.elements.push(create_text_element(
                    trimmed,
                    &TextElementParams {
                        x: 50.0,
                        y: y_offset,
                        width: 860.0,
                        height: 30.0,
                        font_size: 18.0,
                        bold: false,
                    },
                    &theme,
                ));
                y_offset += 40.0;
            }
        }

        slides.push(slide);
    }

    if slides.is_empty() {
        slides.push(create_title_slide(&theme));
    }

    slides
}
