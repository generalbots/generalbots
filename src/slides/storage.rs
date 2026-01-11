use crate::shared::state::AppState;
use crate::slides::ooxml::{load_pptx_preserving, update_pptx_text};
use crate::slides::types::{
    ElementContent, ElementStyle, Presentation, PresentationMetadata, Slide,
    SlideBackground, SlideElement,
};
use crate::slides::utils::{create_content_slide, create_default_theme, create_title_slide};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

static PRESENTATION_CACHE: once_cell::sync::Lazy<RwLock<HashMap<String, (Vec<u8>, DateTime<Utc>)>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

const CACHE_TTL_SECS: i64 = 3600;

pub fn get_user_presentations_path(user_id: &str) -> String {
    format!("users/{user_id}/presentations")
}

pub fn get_current_user_id() -> String {
    "default-user".to_string()
}

pub fn generate_presentation_id() -> String {
    Uuid::new_v4().to_string()
}

pub async fn cache_presentation_bytes(pres_id: &str, bytes: Vec<u8>) {
    let mut cache = PRESENTATION_CACHE.write().await;
    cache.insert(pres_id.to_string(), (bytes, Utc::now()));

    let now = Utc::now();
    cache.retain(|_, (_, modified)| (now - *modified).num_seconds() < CACHE_TTL_SECS);
}

pub async fn get_cached_presentation_bytes(pres_id: &str) -> Option<Vec<u8>> {
    let cache = PRESENTATION_CACHE.read().await;
    cache.get(pres_id).map(|(bytes, _)| bytes.clone())
}

pub async fn remove_from_cache(pres_id: &str) {
    let mut cache = PRESENTATION_CACHE.write().await;
    cache.remove(pres_id);
}

fn extract_id_from_path(path: &str) -> String {
    path.split('/')
        .last()
        .unwrap_or_default()
        .trim_end_matches(".json")
        .trim_end_matches(".pptx")
        .to_string()
}

pub async fn save_presentation_to_drive(
    state: &Arc<AppState>,
    user_id: &str,
    presentation: &Presentation,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation.id
    );
    let content = serde_json::to_string_pretty(presentation)
        .map_err(|e| format!("Serialization error: {e}"))?;

    drive
        .put_object()
        .bucket("gbo")
        .key(&path)
        .body(content.into_bytes().into())
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to save presentation: {e}"))?;

    Ok(())
}

pub async fn save_presentation_as_pptx(
    state: &Arc<AppState>,
    user_id: &str,
    presentation: &Presentation,
) -> Result<Vec<u8>, String> {
    let pptx_bytes = if let Some(original_bytes) = get_cached_presentation_bytes(&presentation.id).await {
        let slide_texts: Vec<Vec<String>> = presentation.slides.iter().map(|slide| {
            slide.elements.iter().filter_map(|el| {
                if let ElementContent::Text { text, .. } = &el.content {
                    Some(text.clone())
                } else {
                    None
                }
            }).collect()
        }).collect();
        update_pptx_text(&original_bytes, &slide_texts).unwrap_or_else(|_| {
            convert_to_pptx(presentation).unwrap_or_default()
        })
    } else {
        convert_to_pptx(presentation)?
    };

    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!(
        "{}/{}.pptx",
        get_user_presentations_path(user_id),
        presentation.id
    );

    drive
        .put_object()
        .bucket("gbo")
        .key(&path)
        .body(pptx_bytes.clone().into())
        .content_type("application/vnd.openxmlformats-officedocument.presentationml.presentation")
        .send()
        .await
        .map_err(|e| format!("Failed to save PPTX: {e}"))?;

    Ok(pptx_bytes)
}

pub fn convert_to_pptx(presentation: &Presentation) -> Result<Vec<u8>, String> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buf);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", options)
            .map_err(|e| format!("Failed to create content types: {e}"))?;
        zip.write_all(generate_content_types_xml(presentation.slides.len()).as_bytes())
            .map_err(|e| format!("Failed to write content types: {e}"))?;

        zip.start_file("_rels/.rels", options)
            .map_err(|e| format!("Failed to create rels: {e}"))?;
        zip.write_all(generate_rels_xml().as_bytes())
            .map_err(|e| format!("Failed to write rels: {e}"))?;

        zip.start_file("ppt/presentation.xml", options)
            .map_err(|e| format!("Failed to create presentation.xml: {e}"))?;
        zip.write_all(generate_presentation_xml(presentation).as_bytes())
            .map_err(|e| format!("Failed to write presentation.xml: {e}"))?;

        zip.start_file("ppt/_rels/presentation.xml.rels", options)
            .map_err(|e| format!("Failed to create presentation rels: {e}"))?;
        zip.write_all(generate_presentation_rels_xml(presentation.slides.len()).as_bytes())
            .map_err(|e| format!("Failed to write presentation rels: {e}"))?;

        for (idx, slide) in presentation.slides.iter().enumerate() {
            let slide_num = idx + 1;

            zip.start_file(format!("ppt/slides/slide{slide_num}.xml"), options)
                .map_err(|e| format!("Failed to create slide{slide_num}.xml: {e}"))?;
            zip.write_all(generate_slide_xml(slide, slide_num).as_bytes())
                .map_err(|e| format!("Failed to write slide{slide_num}.xml: {e}"))?;

            zip.start_file(format!("ppt/slides/_rels/slide{slide_num}.xml.rels"), options)
                .map_err(|e| format!("Failed to create slide{slide_num} rels: {e}"))?;
            zip.write_all(generate_slide_rels_xml().as_bytes())
                .map_err(|e| format!("Failed to write slide{slide_num} rels: {e}"))?;
        }

        zip.start_file("ppt/slideLayouts/slideLayout1.xml", options)
            .map_err(|e| format!("Failed to create slideLayout1.xml: {e}"))?;
        zip.write_all(generate_slide_layout_xml().as_bytes())
            .map_err(|e| format!("Failed to write slideLayout1.xml: {e}"))?;

        zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", options)
            .map_err(|e| format!("Failed to create slideLayout1 rels: {e}"))?;
        zip.write_all(generate_slide_layout_rels_xml().as_bytes())
            .map_err(|e| format!("Failed to write slideLayout1 rels: {e}"))?;

        zip.start_file("ppt/slideMasters/slideMaster1.xml", options)
            .map_err(|e| format!("Failed to create slideMaster1.xml: {e}"))?;
        zip.write_all(generate_slide_master_xml().as_bytes())
            .map_err(|e| format!("Failed to write slideMaster1.xml: {e}"))?;

        zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", options)
            .map_err(|e| format!("Failed to create slideMaster1 rels: {e}"))?;
        zip.write_all(generate_slide_master_rels_xml().as_bytes())
            .map_err(|e| format!("Failed to write slideMaster1 rels: {e}"))?;

        zip.start_file("ppt/theme/theme1.xml", options)
            .map_err(|e| format!("Failed to create theme1.xml: {e}"))?;
        zip.write_all(generate_theme_xml(presentation).as_bytes())
            .map_err(|e| format!("Failed to write theme1.xml: {e}"))?;

        zip.start_file("docProps/app.xml", options)
            .map_err(|e| format!("Failed to create app.xml: {e}"))?;
        zip.write_all(generate_app_xml(presentation).as_bytes())
            .map_err(|e| format!("Failed to write app.xml: {e}"))?;

        zip.start_file("docProps/core.xml", options)
            .map_err(|e| format!("Failed to create core.xml: {e}"))?;
        zip.write_all(generate_core_xml(presentation).as_bytes())
            .map_err(|e| format!("Failed to write core.xml: {e}"))?;

        zip.finish().map_err(|e| format!("Failed to finish ZIP: {e}"))?;
    }

    Ok(buf.into_inner())
}

fn generate_content_types_xml(slide_count: usize) -> String {
    let mut slides_types = String::new();
    for i in 1..=slide_count {
        slides_types.push_str(&format!(
            r#"<Override PartName="/ppt/slides/slide{i}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
<Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
<Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
<Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
{slides_types}
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>"#
    )
}

fn generate_rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#.to_string()
}

fn generate_presentation_xml(presentation: &Presentation) -> String {
    let mut slide_ids = String::new();
    for (idx, _) in presentation.slides.iter().enumerate() {
        let id = 256 + idx as u32;
        let rid = format!("rId{}", idx + 2);
        slide_ids.push_str(&format!(r#"<p:sldId id="{id}" r:id="{rid}"/>"#));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" saveSubsetFonts="1">
<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>
<p:sldIdLst>{slide_ids}</p:sldIdLst>
<p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>
<p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#
    )
}

fn generate_presentation_rels_xml(slide_count: usize) -> String {
    let mut rels = String::new();
    rels.push_str(r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>"#);

    for i in 1..=slide_count {
        let rid = format!("rId{}", i + 1);
        rels.push_str(&format!(
            r#"<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{i}.xml"/>"#
        ));
    }

    let theme_rid = format!("rId{}", slide_count + 2);
    rels.push_str(&format!(
        r#"<Relationship Id="{theme_rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>"#
    ));

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
{rels}
</Relationships>"#
    )
}

fn generate_slide_xml(slide: &Slide, _slide_num: usize) -> String {
    let mut shapes = String::new();
    let mut shape_id = 2u32;

    for element in &slide.elements {
        let x = (element.x * 9144.0) as i64;
        let y = (element.y * 9144.0) as i64;
        let cx = (element.width * 9144.0) as i64;
        let cy = (element.height * 9144.0) as i64;

        if let Some(ref text) = element.content.text {
            let font_size = element.style.font_size.unwrap_or(18.0);
            let font_size_emu = (font_size * 100.0) as i32;
            let escaped_text = escape_xml(text);

            let bold_attr = if element.style.font_weight.as_deref() == Some("bold") {
                r#" b="1""#
            } else {
                ""
            };

            let italic_attr = if element.style.font_style.as_deref() == Some("italic") {
                r#" i="1""#
            } else {
                ""
            };

            shapes.push_str(&format!(
                r#"<p:sp>
<p:nvSpPr><p:cNvPr id="{shape_id}" name="TextBox {shape_id}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
<p:txBody><a:bodyPr wrap="square" rtlCol="0"/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="{font_size_emu}"{bold_attr}{italic_attr}/><a:t>{escaped_text}</a:t></a:r></a:p></p:txBody>
</p:sp>"#
            ));
            shape_id += 1;
        } else if let Some(ref shape_type) = element.content.shape_type {
            let preset = match shape_type.as_str() {
                "rectangle" => "rect",
                "ellipse" | "circle" => "ellipse",
                "triangle" => "triangle",
                "diamond" => "diamond",
                "star" => "star5",
                "arrow" => "rightArrow",
                _ => "rect",
            };

            let fill_color = element
                .style
                .fill
                .as_ref()
                .map(|c| c.trim_start_matches('#').to_uppercase())
                .unwrap_or_else(|| "4472C4".to_string());

            shapes.push_str(&format!(
                r#"<p:sp>
<p:nvSpPr><p:cNvPr id="{shape_id}" name="Shape {shape_id}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="{preset}"><a:avLst/></a:prstGeom><a:solidFill><a:srgbClr val="{fill_color}"/></a:solidFill></p:spPr>
<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang="en-US"/></a:p></p:txBody>
</p:sp>"#
            ));
            shape_id += 1;
        } else if let Some(ref src) = element.content.src {
            shapes.push_str(&format!(
                r#"<p:sp>
<p:nvSpPr><p:cNvPr id="{shape_id}" name="Image {shape_id}" descr="{}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang="en-US"/></a:p></p:txBody>
</p:sp>"#,
                escape_xml(src)
            ));
            shape_id += 1;
        }
    }

    let bg_fill = if slide.background.bg_type == "solid" {
        let color_hex = slide.background.color.as_ref()
            .map(|c| c.trim_start_matches('#').to_uppercase())
            .unwrap_or_else(|| "FFFFFF".to_string());
        format!(r#"<p:bg><p:bgPr><a:solidFill><a:srgbClr val="{color_hex}"/></a:solidFill></p:bgPr></p:bg>"#)
    } else {
        String::new()
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld>{bg_fill}<p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>{shapes}</p:spTree></p:cSld>
<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sld>"#
    )
}

fn generate_slide_rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#.to_string()
}

fn generate_slide_layout_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="blank">
<p:cSld name="Blank"><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr></p:spTree></p:cSld>
<p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sldLayout>"#.to_string()
}

fn generate_slide_layout_rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#.to_string()
}

fn generate_slide_master_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr></p:spTree></p:cSld>
<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
<p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst>
</p:sldMaster>"#.to_string()
}

fn generate_slide_master_rels_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#.to_string()
}

fn generate_theme_xml(presentation: &Presentation) -> String {
    let accent1 = presentation
        .theme
        .colors
        .accent
        .trim_start_matches('#')
        .to_uppercase();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
<a:themeElements>
<a:clrScheme name="Office">
<a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>
<a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1>
<a:dk2><a:srgbClr val="44546A"/></a:dk2>
<a:lt2><a:srgbClr val="E7E6E6"/></a:lt2>
<a:accent1><a:srgbClr val="{accent1}"/></a:accent1>
<a:accent2><a:srgbClr val="ED7D31"/></a:accent2>
<a:accent3><a:srgbClr val="A5A5A5"/></a:accent3>
<a:accent4><a:srgbClr val="FFC000"/></a:accent4>
<a:accent5><a:srgbClr val="5B9BD5"/></a:accent5>
<a:accent6><a:srgbClr val="70AD47"/></a:accent6>
<a:hlink><a:srgbClr val="0563C1"/></a:hlink>
<a:folHlink><a:srgbClr val="954F72"/></a:folHlink>
</a:clrScheme>
<a:fontScheme name="Office">
<a:majorFont><a:latin typeface="Calibri Light"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>
<a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont>
</a:fontScheme>
<a:fmtScheme name="Office">
<a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst>
<a:lnStyleLst><a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst>
<a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst>
<a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst>
</a:fmtScheme>
</a:themeElements>
</a:theme>"#
    )
}

fn generate_app_xml(presentation: &Presentation) -> String {
    let slide_count = presentation.slides.len();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
<Application>General Bots Suite</Application>
<Slides>{slide_count}</Slides>
<Company>General Bots</Company>
</Properties>"#
    )
}

fn generate_core_xml(presentation: &Presentation) -> String {
    let title = escape_xml(&presentation.name);
    let created = presentation.created_at.to_rfc3339();
    let modified = presentation.updated_at.to_rfc3339();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<dc:title>{title}</dc:title>
<dc:creator>{}</dc:creator>
<dcterms:created xsi:type="dcterms:W3CDTF">{created}</dcterms:created>
<dcterms:modified xsi:type="dcterms:W3CDTF">{modified}</dcterms:modified>
</cp:coreProperties>"#,
        escape_xml(&presentation.owner_id)
    )
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub async fn load_pptx_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    file_path: &str,
) -> Result<Presentation, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let result = drive
        .get_object()
        .bucket("gbo")
        .key(file_path)
        .send()
        .await
        .map_err(|e| format!("Failed to load PPTX: {e}"))?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read PPTX: {e}"))?
        .into_bytes()
        .to_vec();

    load_pptx_from_bytes(&bytes, user_id, file_path).await
}

pub async fn load_pptx_from_bytes(
    bytes: &[u8],
    user_id: &str,
    file_path: &str,
) -> Result<Presentation, String> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open PPTX archive: {e}"))?;

    let file_name = file_path
        .split('/')
        .last()
        .unwrap_or("Untitled")
        .trim_end_matches(".pptx")
        .trim_end_matches(".ppt");

    let pres_id = generate_presentation_id();

    cache_presentation_bytes(&pres_id, bytes.to_vec()).await;

    let mut slides = Vec::new();
    let mut slide_num = 1;

    loop {
        let slide_path = format!("ppt/slides/slide{slide_num}.xml");
        match archive.by_name(&slide_path) {
            Ok(mut file) => {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    let slide = parse_slide_xml(&content, slide_num);
                    slides.push(slide);
                }
                slide_num += 1;
            }
            Err(_) => break,
        }
    }

    if slides.is_empty() {
        slides.push(create_title_slide(&create_default_theme()));
    }

    Ok(Presentation {
        id: pres_id,
        name: file_name.to_string(),
        owner_id: user_id.to_string(),
        slides,
        theme: create_default_theme(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    })
}

fn parse_slide_xml(xml_content: &str, slide_num: usize) -> Slide {
    let mut elements = Vec::new();
    let mut element_id = 1;

    let mut in_sp = false;
    let mut current_text = String::new();
    let mut x: f64 = 100.0;
    let mut y: f64 = 100.0;
    let mut cx: f64 = 200.0;
    let mut cy: f64 = 50.0;

    for line in xml_content.lines() {
        if line.contains("<p:sp>") || line.contains("<p:sp ") {
            in_sp = true;
            current_text.clear();
        }

        if in_sp {
            if let Some(start) = line.find("x=\"") {
                if let Some(end) = line[start + 3..].find('"') {
                    if let Ok(val) = line[start + 3..start + 3 + end].parse::<f64>() {
                        x = val / 9144.0;
                    }
                }
            }
            if let Some(start) = line.find("y=\"") {
                if let Some(end) = line[start + 3..].find('"') {
                    if let Ok(val) = line[start + 3..start + 3 + end].parse::<f64>() {
                        y = val / 9144.0;
                    }
                }
            }
            if let Some(start) = line.find("cx=\"") {
                if let Some(end) = line[start + 4..].find('"') {
                    if let Ok(val) = line[start + 4..start + 4 + end].parse::<f64>() {
                        cx = val / 9144.0;
                    }
                }
            }
            if let Some(start) = line.find("cy=\"") {
                if let Some(end) = line[start + 4..].find('"') {
                    if let Ok(val) = line[start + 4..start + 4 + end].parse::<f64>() {
                        cy = val / 9144.0;
                    }
                }
            }

            if let Some(start) = line.find("<a:t>") {
                if let Some(end) = line.find("</a:t>") {
                    let text = &line[start + 5..end];
                    current_text.push_str(text);
                }
            }
        }

        if line.contains("</p:sp>") && in_sp {
            in_sp = false;
            if !current_text.is_empty() {
                elements.push(SlideElement {
                    id: format!("elem_{slide_num}_{element_id}"),
                    element_type: "text".to_string(),
                    x,
                    y,
                    width: cx.max(100.0),
                    height: cy.max(30.0),
                    rotation: 0.0,
                    z_index: element_id as i32,
                    locked: false,
                    content: ElementContent {
                        text: Some(current_text.clone()),
                        html: None,
                        src: None,
                        shape_type: None,
                        chart_data: None,
                        table_data: None,
                    },
                    style: ElementStyle {
                        font_family: Some("Calibri".to_string()),
                        font_size: Some(18.0),
                        font_weight: None,
                        font_style: None,
                        color: Some("#000000".to_string()),
                        fill: None,
                        stroke: None,
                        stroke_width: None,
                        opacity: Some(1.0),
                        shadow: None,
                        border_radius: None,
                        text_align: None,
                        vertical_align: None,
                        line_height: None,
                    },
                    animations: Vec::new(),
                });
                element_id += 1;
            }
            current_text.clear();
        }
    }

    Slide {
        id: format!("slide_{slide_num}"),
        layout: "blank".to_string(),
        elements,
        background: SlideBackground {
            bg_type: "solid".to_string(),
            color: Some("#FFFFFF".to_string()),
            gradient: None,
            image_url: None,
            image_fit: None,
        },
        notes: None,
        transition: None,
    }
}

pub async fn load_presentation_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    presentation_id: &Option<String>,
) -> Result<Presentation, String> {
    let presentation_id = presentation_id
        .as_ref()
        .ok_or_else(|| "Presentation ID is required".to_string())?;

    load_presentation_by_id(state, user_id, presentation_id).await
}

pub async fn load_presentation_by_id(
    state: &Arc<AppState>,
    user_id: &str,
    presentation_id: &str,
) -> Result<Presentation, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation_id
    );

    let result = drive
        .get_object()
        .bucket("gbo")
        .key(&path)
        .send()
        .await
        .map_err(|e| format!("Failed to load presentation: {e}"))?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read presentation: {e}"))?
        .into_bytes();

    let presentation: Presentation =
        serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse presentation: {e}"))?;

    Ok(presentation)
}

pub async fn list_presentations_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
) -> Result<Vec<PresentationMetadata>, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let prefix = format!("{}/", get_user_presentations_path(user_id));

    let result = drive
        .list_objects_v2()
        .bucket("gbo")
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| format!("Failed to list presentations: {e}"))?;

    let mut presentations = Vec::new();

    if let Some(contents) = result.contents {
        for obj in contents {
            if let Some(key) = obj.key {
                if key.ends_with(".json") {
                    let id = extract_id_from_path(&key);
                    if let Ok(presentation) = load_presentation_by_id(state, user_id, &id).await {
                        presentations.push(PresentationMetadata {
                            id: presentation.id,
                            name: presentation.name,
                            owner_id: presentation.owner_id,
                            slide_count: presentation.slides.len(),
                            created_at: presentation.created_at,
                            updated_at: presentation.updated_at,
                        });
                    }
                }
            }
        }
    }

    presentations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(presentations)
}

pub async fn delete_presentation_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    presentation_id: &Option<String>,
) -> Result<(), String> {
    let presentation_id = presentation_id
        .as_ref()
        .ok_or_else(|| "Presentation ID is required".to_string())?;

    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let json_path = format!(
        "{}/{}.json",
        get_user_presentations_path(user_id),
        presentation_id
    );
    let pptx_path = format!(
        "{}/{}.pptx",
        get_user_presentations_path(user_id),
        presentation_id
    );

    let _ = drive
        .delete_object()
        .bucket("gbo")
        .key(&json_path)
        .send()
        .await;

    let _ = drive
        .delete_object()
        .bucket("gbo")
        .key(&pptx_path)
        .send()
        .await;

    Ok(())
}

pub fn create_new_presentation() -> Presentation {
    let theme = create_default_theme();
    let id = generate_presentation_id();

    Presentation {
        id,
        name: "Untitled Presentation".to_string(),
        owner_id: get_current_user_id(),
        slides: vec![create_title_slide(&theme)],
        theme,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn create_slide_with_layout(layout: &str, theme: &crate::slides::types::PresentationTheme) -> Slide {
    match layout {
        "title" => create_title_slide(theme),
        "content" => create_content_slide(theme),
        "blank" => crate::slides::utils::create_blank_slide(theme),
        _ => create_content_slide(theme),
    }
}
