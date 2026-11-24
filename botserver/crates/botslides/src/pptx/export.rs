use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use super::chart::{self, ChartPart};
use super::static_parts::{self, ImagePart};
use super::table;
use crate::types::{ElementStyle, Presentation, Slide, SlideElement};

const PX_TO_EMU: f64 = 12_700.0; // 914400 / 72

pub(super) fn emu(px: f64) -> i64 {
    (px * PX_TO_EMU).round() as i64
}

pub(super) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(super) fn hex_color(c: Option<&String>, default: &str) -> String {
    c.map(|v| v.trim_start_matches('#').to_string())
        .filter(|v| v.len() == 6 && v.chars().all(|ch| ch.is_ascii_hexdigit()))
        .unwrap_or_else(|| default.to_string())
}

fn decode_data_uri(src: &str) -> Option<(String, Vec<u8>)> {
    let rest = src.strip_prefix("data:")?;
    let (mime, b64) = rest.split_once(',')?;
    let b64 = b64.split(';').next()?;
    let ext = match mime {
        _ if mime.contains("png") => "png",
        _ if mime.contains("jpeg") || mime.contains("jpg") => "jpg",
        _ if mime.contains("gif") => "gif",
        _ => return None,
    };
    use base64::{engine::general_purpose, Engine as _};
    let bytes = general_purpose::STANDARD.decode(b64).ok()?;
    Some((ext.to_string(), bytes))
}

pub(super) fn fill_xml(style: &ElementStyle, default: &str) -> String {
    let color = hex_color(style.fill.as_ref(), default);
    format!(
        "<a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill>",
        color
    )
}

pub(super) fn xfrm_emu_xml(x: i64, y: i64, cx: i64, cy: i64) -> String {
    format!(
        "<a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>",
        x, y, cx, cy
    )
}

pub(super) fn xfrm_xml(x: f64, y: f64, w: f64, h: f64) -> String {
    // Untrusted JSON can carry NaN/∞ or enormous coordinates; clamp before
    // converting so the generated XML never contains invalid EMU values.
    let clamped = |value: f64, floor: f64| {
        if value.is_finite() {
            value.clamp(floor, 20_000.0)
        } else {
            floor
        }
    };
    xfrm_emu_xml(
        emu(clamped(x, -20_000.0)),
        emu(clamped(y, -20_000.0)),
        emu(clamped(w, 0.0)),
        emu(clamped(h, 0.0)),
    )
}

fn prst_geom(shape_type: &str) -> &'static str {
    match shape_type {
        "circle" | "ellipse" => "ellipse",
        "triangle" => "triangle",
        "line" => "line",
        "diamond" => "diamond",
        _ => "rect",
    }
}

fn slide_shape_xml(
    element: &SlideElement,
    id: usize,
    images: &mut Vec<ImagePart>,
) -> String {
    let name = format!("Shape {id}");
    let xfrm = xfrm_xml(element.x, element.y, element.width, element.height);

    if element.element_type == "image" {
        let src = element.content.src.as_deref().unwrap_or("");
        if let Some((ext, bytes)) = decode_data_uri(src) {
            let rel_id = images.len() + 2; // rId1 = slide layout
            images.push(ImagePart { ext, bytes });
            return format!(
                "<p:pic><p:nvPicPr><p:cNvPr id=\"{id}\" name=\"{name}\"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr>\
                 <p:blipFill><a:blip r:embed=\"rId{rel_id}\"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>\
                 <p:spPr>{xfrm}<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></p:spPr></p:pic>"
            );
        }
        // Non-data-URI image sources are not embeddable offline; emit nothing.
        return String::new();
    }

    if element.element_type == "shape" {
        let shape_type = element.content.shape_type.as_deref().unwrap_or("rectangle");
        let geom = prst_geom(shape_type);
        let fill = fill_xml(&element.style, "cccccc");
        return format!(
            "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{name}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>\
             <p:spPr>{xfrm}<a:prstGeom prst=\"{geom}\"><a:avLst/></a:prstGeom>{fill}</p:spPr>\
             <p:txBody><a:bodyPr/><a:lstStyle/><a:p/></p:txBody></p:sp>"
        );
    }

    // Text box (default for "text"/"title"/anything else with text content).
    let text = element
        .content
        .text
        .as_deref()
        .or(element.content.html.as_deref())
        .unwrap_or("");
    let font_size = element.style.font_size.unwrap_or(18.0);
    let sz = (font_size * 75.0).round() as i32; // px -> hundredths of a point
    let color = hex_color(element.style.color.as_ref(), "000000");
    let font_family = xml_escape(element.style.font_family.as_deref().unwrap_or("Arial"));
    let bold = if element.style.font_weight.as_deref() == Some("bold") { " b=\"1\"" } else { "" };

    let mut paragraphs = String::new();
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.is_empty() {
        paragraphs.push_str("<a:p><a:endParaRPr lang=\"en-US\"/></a:p>");
    }
    for line in lines {
        paragraphs.push_str(&format!(
            "<a:p><a:r><a:rPr lang=\"en-US\" sz=\"{sz}\"{bold}><a:solidFill><a:srgbClr val=\"{color}\"/></a:solidFill><a:latin typeface=\"{font_family}\"/></a:rPr><a:t>{}</a:t></a:r><a:endParaRPr lang=\"en-US\"/></a:p>",
            xml_escape(line)
        ));
    }

    format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{name}\"/><p:cNvSpPr txBox=\"1\"/><p:nvPr/></p:nvSpPr>\
         <p:spPr>{xfrm}<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:noFill/></p:spPr>\
         <p:txBody><a:bodyPr wrap=\"square\"><a:spAutoFit/></a:bodyPr><a:lstStyle/>{paragraphs}</p:txBody></p:sp>"
    )
}

fn slide_xml(slide: &Slide, images: &mut Vec<ImagePart>, charts: &mut Vec<ChartPart>) -> String {
    let bg = hex_color(slide.background.color.as_ref(), "FFFFFF");
    let mut shapes = String::new();
    for (i, element) in slide.elements.iter().enumerate() {
        let id = i + 1;
        // #2457 — the old "Tables/charts have no native shape writer here"
        // degradation is gone: tables become native a:tbl graphicFrames and
        // charts become real chart parts with cached series, never text boxes.
        if table::is_table_element(element) {
            shapes.push_str(&table::table_shape_xml(element, id));
        } else if chart::is_chart_element(element) {
            shapes.push_str(&chart::chart_frame_xml(element, id, charts));
        } else {
            if matches!(element.element_type.as_str(), "table" | "chart") {
                // No structured data to write: keep any title text visible and
                // say so in the log instead of inventing a flattened data dump.
                log::warn!(
                    "pptx export: {} element '{}' has no structured data",
                    element.element_type,
                    element.id
                );
            }
            shapes.push_str(&slide_shape_xml(element, id, images));
        }
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <p:sld xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\">\
         <p:cSld><p:bg><p:bgPr><a:solidFill><a:srgbClr val=\"{bg}\"/></a:solidFill><a:effectLst/></p:bgPr></p:bg>\
         <p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>\
         <p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>\
         {shapes}</p:spTree></p:cSld><p:clrMapOvr><a:overrideClrMapping/></p:clrMapOvr></p:sld>"
    )
}

/// Serialize a presentation to a valid `.pptx` byte stream.
pub fn export_to_pptx(presentation: &Presentation) -> Result<Vec<u8>, String> {
    let slide_count = presentation.slides.len();
    let notes: Vec<bool> = presentation
        .slides
        .iter()
        .map(|s| s.notes.as_deref().map(|n| !n.trim().is_empty()).unwrap_or(false))
        .collect();
    let chart_count = presentation
        .slides
        .iter()
        .flat_map(|slide| slide.elements.iter())
        .filter(|element| chart::is_chart_element(*element))
        .count();

    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let write_part = |zip: &mut ZipWriter<&mut Cursor<Vec<u8>>>, name: &str, data: &str| -> Result<(), String> {
            zip.start_file(name, opts).map_err(|e| format!("zip start {name}: {e}"))?;
            zip.write_all(data.as_bytes()).map_err(|e| format!("zip write {name}: {e}"))?;
            Ok(())
        };

        write_part(&mut zip, "[Content_Types].xml", &static_parts::content_types_xml(slide_count, &notes, chart_count))?;
        write_part(&mut zip, "_rels/.rels", &static_parts::root_rels_xml())?;
        write_part(&mut zip, "ppt/presentation.xml", &static_parts::presentation_xml(slide_count))?;
        write_part(&mut zip, "ppt/_rels/presentation.xml.rels", &static_parts::presentation_rels_xml(slide_count, &notes))?;
        write_part(&mut zip, "ppt/theme/theme1.xml", &static_parts::theme_xml())?;
        write_part(&mut zip, "ppt/slideMasters/slideMaster1.xml", &static_parts::slide_master_xml())?;
        write_part(&mut zip, "ppt/slideMasters/_rels/slideMaster1.xml.rels", &static_parts::slide_master_rels_xml())?;
        write_part(&mut zip, "ppt/slideLayouts/slideLayout1.xml", &static_parts::slide_layout_xml())?;
        write_part(&mut zip, "ppt/slideLayouts/_rels/slideLayout1.xml.rels", &static_parts::slide_layout_rels_xml())?;

        let mut chart_sequence = 0usize;
        for (i, slide) in presentation.slides.iter().enumerate() {
            let mut images: Vec<ImagePart> = Vec::new();
            let mut charts: Vec<ChartPart> = Vec::new();
            let slide_xml = slide_xml(slide, &mut images, &mut charts);
            for (offset, chart) in charts.iter_mut().enumerate() {
                chart.file_name = format!("chart{}.xml", chart_sequence + offset + 1);
            }
            chart_sequence += charts.len();
            let n = i + 1;
            write_part(&mut zip, &format!("ppt/slides/slide{n}.xml"), &slide_xml)?;
            write_part(&mut zip, &format!("ppt/slides/_rels/slide{n}.xml.rels"), &static_parts::slide_rels_xml(&images, &charts))?;

            for (j, img) in images.iter().enumerate() {
                let name = format!("ppt/media/image{}.{}", j + 1, img.ext);
                zip.start_file(&name, opts).map_err(|e| format!("zip start {name}: {e}"))?;
                zip.write_all(&img.bytes).map_err(|e| format!("zip write {name}: {e}"))?;
            }

            for chart in &charts {
                let name = format!("ppt/charts/{}", chart.file_name);
                write_part(&mut zip, &name, &chart.xml)?;
            }

            if notes[i] {
                if let Some(ref note_text) = slide.notes {
                    write_part(
                        &mut zip,
                        &format!("ppt/notesSlides/notesSlide{n}.xml"),
                        &static_parts::notes_slide_xml(note_text),
                    )?;
                }
            }
        }

        zip.finish().map_err(|e| format!("zip finish: {e}"))?;
    }

    Ok(buf.into_inner())
}
