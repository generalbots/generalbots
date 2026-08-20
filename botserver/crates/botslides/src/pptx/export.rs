use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::types::{ElementStyle, Presentation, Slide, SlideElement};

const SLIDE_W_EMU: i64 = 12_192_000; // 13.333in
const SLIDE_H_EMU: i64 = 6_858_000; // 7.5in
const PX_TO_EMU: f64 = 12_700.0; // 914400 / 72

fn emu(px: f64) -> i64 {
    (px * PX_TO_EMU).round() as i64
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn hex_color(c: Option<&String>, default: &str) -> String {
    c.map(|v| v.trim_start_matches('#').to_string())
        .filter(|v| v.len() == 6 && v.chars().all(|ch| ch.is_ascii_hexdigit()))
        .unwrap_or_else(|| default.to_string())
}

struct ImagePart {
    ext: String,
    bytes: Vec<u8>,
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

fn fill_xml(style: &ElementStyle, default: &str) -> String {
    let color = hex_color(style.fill.as_ref(), default);
    format!(
        "<a:solidFill><a:srgbClr val=\"{}\"/></a:solidFill>",
        color
    )
}

fn xfrm_xml(x: f64, y: f64, w: f64, h: f64) -> String {
    format!(
        "<a:xfrm><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>",
        emu(x),
        emu(y),
        emu(w),
        emu(h)
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
    let font_family = element.style.font_family.as_deref().unwrap_or("Arial");
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

fn slide_xml(slide: &Slide, images: &mut Vec<ImagePart>) -> String {
    let bg = hex_color(slide.background.color.as_ref(), "FFFFFF");
    let mut shapes = String::new();
    for (i, element) in slide.elements.iter().enumerate() {
        shapes.push_str(&slide_shape_xml(element, i + 1, images));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <p:sld xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\">\
         <p:cSld><p:bg><p:bgPr><a:solidFill><a:srgbClr val=\"{bg}\"/></a:solidFill><a:effectLst/></p:bgPr></p:bg>\
         <p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>\
         <p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>\
         {shapes}</p:spTree></p:cSld><p:clrMapOvr><a:overrideClrMapping/></p:clrMapOvr></p:sld>"
    )
}

fn notes_slide_xml(notes: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <p:notes xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\">\
         <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>\
         <p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>\
         <p:sp><p:nvSpPr><p:cNvPr id=\"2\" name=\"Notes\"/><p:cNvSpPr txBox=\"1\"/><p:nvPr/></p:nvSpPr>\
         <p:spPr><a:xfrm><a:off x=\"685800\" y=\"685800\"/><a:ext cx=\"10820400\" cy=\"5486400\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></p:spPr>\
         <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang=\"en-US\" sz=\"1200\"/><a:t>{}</a:t></a:r></a:p></p:txBody></p:sp>\
         </p:spTree></p:cSld></p:notes>",
        xml_escape(notes)
    )
}

fn content_types_xml(slide_count: usize, notes: &[bool]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
         <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
         <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
         <Default Extension=\"png\" ContentType=\"image/png\"/>\
         <Default Extension=\"jpg\" ContentType=\"image/jpeg\"/>\
         <Override PartName=\"/ppt/presentation.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml\"/>\
         <Override PartName=\"/ppt/slideMasters/slideMaster1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml\"/>\
         <Override PartName=\"/ppt/slideLayouts/slideLayout1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml\"/>\
         <Override PartName=\"/ppt/theme/theme1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.theme+xml\"/>",
    );
    for i in 1..=slide_count {
        out.push_str(&format!(
            "<Override PartName=\"/ppt/slides/slide{i}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slide+xml\"/>"
        ));
        if notes.get(i - 1).copied().unwrap_or(false) {
            out.push_str(&format!(
                "<Override PartName=\"/ppt/notesSlides/notesSlide{i}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml\"/>"
            ));
        }
    }
    out.push_str("</Types>");
    out
}

fn root_rels_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
     <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
     <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"ppt/presentation.xml\"/>\
     </Relationships>"
        .to_string()
}

fn presentation_xml(slide_count: usize) -> String {
    let mut sld_ids = String::new();
    for i in 1..=slide_count {
        sld_ids.push_str(&format!(
            "<p:sldId id=\"{i}\" r:id=\"rId{i}\"/>"
        ));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <p:presentation xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\">\
         <p:sldMasterIdLst><p:sldMasterId id=\"2147483648\" r:id=\"rId1\"/></p:sldMasterIdLst>\
         <p:sldIdLst>{sld_ids}</p:sldIdLst>\
         <p:sldSz cx=\"{SLIDE_W_EMU}\" cy=\"{SLIDE_H_EMU}\"/>\
         <p:notesSz cx=\"{SLIDE_H_EMU}\" cy=\"{SLIDE_W_EMU}\"/>\
         </p:presentation>"
    )
}

fn presentation_rels_xml(slide_count: usize, notes: &[bool]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
         <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster\" Target=\"slideMasters/slideMaster1.xml\"/>\
         <Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" Target=\"theme/theme1.xml\"/>",
    );
    for i in 1..=slide_count {
        out.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide\" Target=\"slides/slide{i}.xml\"/>",
            i + 2
        ));
    }
    for (i, has_notes) in notes.iter().enumerate() {
        if *has_notes {
            out.push_str(&format!(
                "<Relationship Id=\"rIdN{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide\" Target=\"notesSlides/notesSlide{}.xml\"/>",
                i + 1,
                i + 1
            ));
        }
    }
    out.push_str("</Relationships>");
    out
}

fn slide_rels_xml(images: &[ImagePart]) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
         <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout\" Target=\"../slideLayouts/slideLayout1.xml\"/>",
    );
    for (i, img) in images.iter().enumerate() {
        out.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"../media/image{}.{}\"/>",
            i + 2,
            i + 1,
            img.ext
        ));
    }
    out.push_str("</Relationships>");
    out
}

fn theme_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
     <a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" name=\"Default\">\
     <a:themeElements><a:clrScheme name=\"Default\"><a:dk1><a:sysClr val=\"windowText\" lastClr=\"000000\"/></a:dk1>\
     <a:lt1><a:sysClr val=\"window\" lastClr=\"FFFFFF\"/></a:lt1><a:dk2><a:srgbClr val=\"1F497D\"/></a:dk2>\
     <a:lt2><a:srgbClr val=\"EEECE1\"/></a:lt2><a:accent1><a:srgbClr val=\"4F81BD\"/></a:accent1>\
     <a:accent2><a:srgbClr val=\"C0504D\"/></a:accent2><a:accent3><a:srgbClr val=\"9BBB59\"/></a:accent3>\
     <a:accent4><a:srgbClr val=\"8064A2\"/></a:accent4><a:accent5><a:srgbClr val=\"4BACC6\"/></a:accent5>\
     <a:accent6><a:srgbClr val=\"F79646\"/></a:accent6><a:hlink><a:srgbClr val=\"0000FF\"/></a:hlink>\
     <a:folHlink><a:srgbClr val=\"800080\"/></a:folHlink></a:clrScheme>\
     <a:fontScheme name=\"Default\"><a:majorFont><a:latin typeface=\"Calibri\"/></a:majorFont>\
     <a:minorFont><a:latin typeface=\"Calibri\"/></a:minorFont></a:fontScheme>\
     <a:fmtScheme name=\"Default\"><a:fillStyleLst/><a:lnStyleLst/><a:effectStyleLst/><a:bgFillStyleLst/></a:fmtScheme>\
     </a:themeElements></a:theme>"
        .to_string()
}

fn slide_master_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
     <p:sldMaster xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\">\
     <p:cSld><p:bg><p:bgPr><a:solidFill><a:srgbClr val=\"FFFFFF\"/></a:solidFill><a:effectLst/></p:bgPr></p:bg>\
     <p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>\
     <p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>\
     </p:spTree></p:cSld><p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/>\
     <p:sldLayoutIdLst><p:sldLayoutId id=\"2147483649\" r:id=\"rId1\"/></p:sldLayoutIdLst></p:sldMaster>"
        .to_string()
}

fn slide_master_rels_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
     <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
     <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout\" Target=\"../slideLayouts/slideLayout1.xml\"/>\
     <Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" Target=\"../theme/theme1.xml\"/>\
     </Relationships>"
        .to_string()
}

fn slide_layout_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
     <p:sldLayout xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" type=\"blank\" preserve=\"1\">\
     <p:cSld name=\"Blank\"><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>\
     <p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>\
     </p:spTree></p:cSld><p:clrMapOvr><a:overrideClrMapping/></p:clrMapOvr></p:sldLayout>"
        .to_string()
}

fn slide_layout_rels_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
     <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
     <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster\" Target=\"../slideMasters/slideMaster1.xml\"/>\
     </Relationships>"
        .to_string()
}

/// Serialize a presentation to a valid `.pptx` byte stream.
pub fn export_to_pptx(presentation: &Presentation) -> Result<Vec<u8>, String> {
    let slide_count = presentation.slides.len();
    let notes: Vec<bool> = presentation
        .slides
        .iter()
        .map(|s| s.notes.as_deref().map(|n| !n.trim().is_empty()).unwrap_or(false))
        .collect();

    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buf);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let write_part = |zip: &mut ZipWriter<&mut Cursor<Vec<u8>>>, name: &str, data: &str| -> Result<(), String> {
            zip.start_file(name, opts).map_err(|e| format!("zip start {name}: {e}"))?;
            zip.write_all(data.as_bytes()).map_err(|e| format!("zip write {name}: {e}"))?;
            Ok(())
        };

        write_part(&mut zip, "[Content_Types].xml", &content_types_xml(slide_count, &notes))?;
        write_part(&mut zip, "_rels/.rels", &root_rels_xml())?;
        write_part(&mut zip, "ppt/presentation.xml", &presentation_xml(slide_count))?;
        write_part(&mut zip, "ppt/_rels/presentation.xml.rels", &presentation_rels_xml(slide_count, &notes))?;
        write_part(&mut zip, "ppt/theme/theme1.xml", &theme_xml())?;
        write_part(&mut zip, "ppt/slideMasters/slideMaster1.xml", &slide_master_xml())?;
        write_part(&mut zip, "ppt/slideMasters/_rels/slideMaster1.xml.rels", &slide_master_rels_xml())?;
        write_part(&mut zip, "ppt/slideLayouts/slideLayout1.xml", &slide_layout_xml())?;
        write_part(&mut zip, "ppt/slideLayouts/_rels/slideLayout1.xml.rels", &slide_layout_rels_xml())?;

        for (i, slide) in presentation.slides.iter().enumerate() {
            let mut images: Vec<ImagePart> = Vec::new();
            let slide_xml = slide_xml(slide, &mut images);
            let n = i + 1;
            write_part(&mut zip, &format!("ppt/slides/slide{n}.xml"), &slide_xml)?;
            write_part(&mut zip, &format!("ppt/slides/_rels/slide{n}.xml.rels"), &slide_rels_xml(&images))?;

            for (j, img) in images.iter().enumerate() {
                let name = format!("ppt/media/image{}.{}", j + 1, img.ext);
                zip.start_file(&name, opts).map_err(|e| format!("zip start {name}: {e}"))?;
                zip.write_all(&img.bytes).map_err(|e| format!("zip write {name}: {e}"))?;
            }

            if notes[i] {
                if let Some(ref note_text) = slide.notes {
                    write_part(
                        &mut zip,
                        &format!("ppt/notesSlides/notesSlide{n}.xml"),
                        &notes_slide_xml(note_text),
                    )?;
                }
            }
        }

        zip.finish().map_err(|e| format!("zip finish: {e}"))?;
    }

    Ok(buf.into_inner())
}
