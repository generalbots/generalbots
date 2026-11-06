//! ODP export (#1715): ODF packages, not bare content.xml.

use super::export::xml_escape;
use crate::types::Presentation;

pub fn export_to_odp_content(presentation: &Presentation) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
xmlns:presentation="urn:oasis:names:tc:opendocument:xmlns:presentation:1.0"
xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
office:version="1.2">
<office:body>
<office:presentation>
"#);

    for (idx, slide) in presentation.slides.iter().enumerate() {
        xml.push_str(&format!(
            "<draw:page draw:name=\"Slide{}\" draw:style-name=\"dp1\">\n",
            idx + 1
        ));

        for element in &slide.elements {
            match element.element_type.as_str() {
                "text" => {
                    let text = element.content.text.as_deref().unwrap_or("");
                    xml.push_str(&format!(
                        r#" <draw:frame svg:x="{}pt" svg:y="{}pt" svg:width="{}pt" svg:height="{}pt">
<draw:text-box>
<text:p>{}</text:p>
</draw:text-box>
</draw:frame>
"#,
                        element.x, element.y, element.width, element.height, xml_escape(text)
                    ));
                }
                "shape" => {
                    let shape_type = element.content.shape_type.as_deref().unwrap_or("rectangle");
                    let fill = element.style.fill.as_deref().unwrap_or("#cccccc");

                    match shape_type {
                        "rectangle" | "rect" => {
                            xml.push_str(&format!(
                                r#" <draw:rect svg:x="{}pt" svg:y="{}pt" svg:width="{}pt" svg:height="{}pt" draw:fill-color="{}"/>
"#,
                                element.x, element.y, element.width, element.height, fill
                            ));
                        }
                        "circle" | "ellipse" => {
                            xml.push_str(&format!(
                                r#" <draw:ellipse svg:x="{}pt" svg:y="{}pt" svg:width="{}pt" svg:height="{}pt" draw:fill-color="{}"/>
"#,
                                element.x, element.y, element.width, element.height, fill
                            ));
                        }
                        _ => {}
                    }
                }
                "image" => {
                    if let Some(ref src) = element.content.src {
                        xml.push_str(&format!(
                            r#" <draw:frame svg:x="{}pt" svg:y="{}pt" svg:width="{}pt" svg:height="{}pt">
<draw:image xlink:href="{}"/>
</draw:frame>
"#,
                            element.x, element.y, element.width, element.height, src
                        ));
                    }
                }
                _ => {}
            }
        }

        xml.push_str("</draw:page>\n");
    }

    xml.push_str("</office:presentation>\n</office:body>\n</office:document-content>");
    xml
}


pub fn export_to_odp(presentation: &Presentation) -> Result<Vec<u8>, String> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    let content = export_to_odp_content(presentation);
    let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
<manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.presentation"/>
<manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
</manifest:manifest>
"#;

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut cursor);
        let stored = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        let deflated = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("mimetype", stored).map_err(|e| e.to_string())?;
        zip.write_all(b"application/vnd.oasis.opendocument.presentation")
            .map_err(|e| e.to_string())?;
        zip.start_file("content.xml", deflated)
            .map_err(|e| e.to_string())?;
        zip.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
        zip.start_file("META-INF/manifest.xml", deflated)
            .map_err(|e| e.to_string())?;
        zip.write_all(manifest.as_bytes()).map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
    }
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod odp_tests {
    use super::{export_to_odp, export_to_odp_content};
    use crate::types::Presentation;

    fn deck() -> Presentation {
        crate::storage::create_new_presentation()
    }

    #[test]
    fn odp_export_is_a_package_with_the_stored_mimetype() {
        let presentation = deck();
        assert!(export_to_odp_content(&presentation).contains("<office:presentation>"));
        let bytes = export_to_odp(&presentation).expect("odp");
        assert_eq!(&bytes[..2], b"PK");
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
        {
            let mut mime = zip.by_name("mimetype").expect("mimetype entry");
            let mut text = String::new();
            std::io::Read::read_to_string(&mut mime, &mut text).expect("read");
            assert_eq!(text, "application/vnd.oasis.opendocument.presentation");
        }
        assert!(zip.by_name("content.xml").is_ok());
        assert!(zip.by_name("META-INF/manifest.xml").is_ok());
    }
}
