use std::io::{Cursor, Read};
use std::collections::HashMap;
use base64::Engine;
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
        let mut content = String::new();
        {
            let mut file = archive.by_name(&name).map_err(|e| format!("open {name}: {e}"))?;
            file.read_to_string(&mut content).map_err(|e| format!("read {name}: {e}"))?;
        }
        let rels = slide_rels(&mut archive, &name);
        slides.push(parse_slide_xml(&content, &mut archive, &rels));
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

fn parse_slide_xml(xml: &str, archive: &mut ZipArchive<Cursor<&[u8]>>, rels: &HashMap<String, String>) -> Slide {
    // One fragment per shape, from the same scan the preserving save uses, so
    // element order and paragraph order agree with the package (#2041).
    let mut elements: Vec<SlideElement> = shape_blocks(xml)
        .into_iter()
        .filter_map(|(start, end)| parse_shape_element(&xml[start..end]))
        .collect();
    // Pictures are a separate element type in OOXML (`p:pic`) and used to be
    // dropped entirely, so every imported photo became a grey rectangle
    // (#2151). Resolve each one against the slide rels and embed the bytes as
    // a data URL so the Slides app can render it without a Drive round-trip.
    let mut cursor = 0;
    while let Some(start) = crate::ooxml::scan::find_open(xml, cursor, "<p:pic") {
        let Some(close) = xml[start..].find("</p:pic>") else {
            break;
        };
        let end = start + close + "</p:pic>".len();
        if let Some(element) = parse_pic_element(&xml[start..end], rels, archive) {
            elements.push(element);
        }
        cursor = end;
    }
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

/// Map `rId` -> `ppt/media/...` for one slide.
fn slide_rels(archive: &mut ZipArchive<Cursor<&[u8]>>, slide_name: &str) -> HashMap<String, String> {
    let base = slide_name.rsplit('/').next().unwrap_or(slide_name);
    let rels_name = format!("ppt/slides/_rels/{base}.rels");
    let mut map = HashMap::new();
    let Ok(mut file) = archive.by_name(&rels_name) else {
        return map;
    };
    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        return map;
    }
    let mut cursor = 0;
    while let Some(pos) = content[cursor..].find("<Relationship ") {
        let start = cursor + pos;
        let Some(tag_end) = content[start..].find('>') else {
            break;
        };
        let tag = &content[start..start + tag_end + 1];
        let id = attr_value(tag, "Id");
        let target = attr_value(tag, "Target");
        if let (Some(id), Some(target)) = (id, target) {
            let normalised = if target.starts_with("../") {
                format!("ppt/{}", target.trim_start_matches("../"))
            } else if target.starts_with('/') {
                target.trim_start_matches('/').to_string()
            } else {
                format!("ppt/slides/{target}")
            };
            map.insert(id, normalised);
        }
        cursor = start + tag_end + 1;
    }
    map
}

fn parse_pic_element(
    pic_xml: &str,
    rels: &HashMap<String, String>,
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Option<SlideElement> {
    let (x, y, w, h) = parse_xfrm(pic_xml);
    let id = uuid::Uuid::new_v4().to_string();

    // r:embed="rId1" → the relationship id carrying the media part.
    let r_id = attr_value(pic_xml, "r:embed").or_else(|| attr_value(pic_xml, "embed"))?;
    let media_path = rels.get(&r_id)?;
    let mut file = archive.by_name(media_path).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;

    let mime = match media_path.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "image/png",
    };
    let src = format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    );

    Some(SlideElement {
        id,
        element_type: "image".to_string(),
        x,
        y,
        width: w.max(20.0),
        height: h.max(20.0),
        rotation: 0.0,
        content: ElementContent {
            src: Some(src),
            ..Default::default()
        },
        style: ElementStyle::default(),
        animations: Vec::new(),
        z_index: 0,
        locked: false,
    })
}
