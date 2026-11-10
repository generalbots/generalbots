use std::io::{Cursor, Read};
use zip::ZipArchive;

use crate::ooxml::{paragraph_texts, shape_blocks};
use crate::types::{ElementContent, ElementStyle, Presentation, Slide, SlideBackground, SlideElement};

const PX_TO_EMU: f64 = 12_700.0; // 914400 / 72

/// Parse a `.pptx` into the internal model (text boxes, shapes and images).
///
/// The model has to line up with the package: slides come in presentation
/// order and every text element carries the same paragraphs the preserving
/// scan sees, because a save writes the model back into those very parts
/// (#2041). Any divergence here turns an untouched save into a rebuild.
pub fn load_pptx(bytes: &[u8]) -> Result<Presentation, String> {
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader).map_err(|e| format!("open pptx: {e}"))?;

    let slide_names = super::order::slide_part_names(bytes)?;

    let theme = crate::utils::create_default_theme();
    let mut slides = Vec::new();
    for name in slide_names {
        let mut file = archive.by_name(&name).map_err(|e| format!("open {name}: {e}"))?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| format!("read {name}: {e}"))?;
        slides.push(parse_slide_xml(&content));
    }

    Ok(Presentation {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Imported Presentation".to_string(),
        owner_id: String::new(),
        slides,
        theme,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        source_bucket: None,
        source_path: None,
    })
}

fn parse_slide_xml(xml: &str) -> Slide {
    // One fragment per shape, from the same scan the preserving save uses, so
    // element order and paragraph order agree with the package (#2041).
    let elements = shape_blocks(xml)
        .into_iter()
        .filter_map(|(start, end)| parse_shape_element(&xml[start..end]))
        .collect();
    Slide {
        id: uuid::Uuid::new_v4().to_string(),
        layout: "blank".to_string(),
        elements,
        background: SlideBackground::default(),
        notes: None,
        transition: None,
        transition_config: None,
        media: None,
    }
}

fn parse_shape_element(sp_xml: &str) -> Option<SlideElement> {
    let (x, y, w, h) = parse_xfrm(sp_xml);
    let text = extract_drawing_text(sp_xml);
    let is_tx_box = sp_xml.contains("txBox=\"1\"");
    let id = uuid::Uuid::new_v4().to_string();

    if !text.trim().is_empty() || is_tx_box {
        let style = ElementStyle {
            font_size: Some(24.0),
            ..Default::default()
        };
        return Some(SlideElement {
            id,
            element_type: "text".to_string(),
            x,
            y,
            width: w.max(100.0),
            height: h.max(40.0),
            rotation: 0.0,
            content: ElementContent { text: Some(text), ..Default::default() },
            style,
            animations: Vec::new(),
            z_index: 0,
            locked: false,
        });
    }

    // Fall back to a shape so non-text content is still represented.
    let style = ElementStyle {
        fill: Some("#cccccc".to_string()),
        ..ElementStyle::default()
    };
    Some(SlideElement {
        id,
        element_type: "shape".to_string(),
        x,
        y,
        width: w.max(20.0),
        height: h.max(20.0),
        rotation: 0.0,
        content: ElementContent { shape_type: Some("rectangle".to_string()), ..Default::default() },
        style,
        animations: Vec::new(),
        z_index: 0,
        locked: false,
    })
}

fn parse_xfrm(sp_xml: &str) -> (f64, f64, f64, f64) {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut w = 400.0;
    let mut h = 100.0;
    if let Some(off_start) = sp_xml.find("<a:off ") {
        if let Some(x_attr) = attr_value(&sp_xml[off_start..], "x") {
            x = x_attr.parse::<f64>().unwrap_or(0.0) / PX_TO_EMU;
        }
        if let Some(y_attr) = attr_value(&sp_xml[off_start..], "y") {
            y = y_attr.parse::<f64>().unwrap_or(0.0) / PX_TO_EMU;
        }
    }
    if let Some(ext_start) = sp_xml.find("<a:ext ") {
        if let Some(cx) = attr_value(&sp_xml[ext_start..], "cx") {
            w = cx.parse::<f64>().unwrap_or(0.0) / PX_TO_EMU;
        }
        if let Some(cy) = attr_value(&sp_xml[ext_start..], "cy") {
            h = cy.parse::<f64>().unwrap_or(0.0) / PX_TO_EMU;
        }
    }
    (x, y, w, h)
}

fn attr_value(tag: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

fn extract_drawing_text(xml: &str) -> String {
    // One line per visible paragraph — the same rule the preserving scan uses,
    // so `slide_paragraphs` reads back exactly what the package holds.
    paragraph_texts(xml).join("\n")
}
