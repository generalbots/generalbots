use super::*;

fn pdf_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\n' | '\r' | '\t' => out.push(' '),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

fn slide_text_lines(slide: &Slide, idx: usize) -> Vec<(String, f32)> {
    let mut lines: Vec<(String, f32, f32, f32)> = Vec::new();
    for element in &slide.elements {
        if element.element_type == "text" {
            if let Some(ref text) = element.content.text {
                let t = text.trim();
                if t.is_empty() {
                    continue;
                }
                let font_size = element.style.font_size.unwrap_or(18.0) as f32;
                lines.push((t.to_string(), font_size, element.y as f32, element.x as f32));
            }
        } else if element.element_type == "table" {
            if let Some(ref table) = element.content.table_data {
                for row in &table.cells {
                    let row_text = row
                        .iter()
                        .map(|c| c.content.trim())
                        .filter(|c| !c.is_empty())
                        .collect::<Vec<_>>()
                        .join(" | ");
                    if !row_text.is_empty() {
                        lines.push((row_text, 12.0_f32, element.y as f32, element.x as f32));
                    }
                }
            }
        }
    }
    if lines.is_empty() {
        lines.push((format!("Slide {}", idx + 1), 24.0, 72.0, 72.0));
    }
    lines.sort_by(|a, b| {
        a.2.partial_cmp(&b.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
    });
    lines.into_iter().map(|(t, s, _, _)| (t, s)).collect()
}

pub fn export_to_pdf(presentation: &Presentation) -> Result<Vec<u8>, String> {
    let page_w = 960.0_f32;
    let page_h = 540.0_f32;
    let mut objects: Vec<Vec<u8>> = Vec::new();
    // 1: catalog, 2: pages, 3: font
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objects.push(Vec::new()); // pages placeholder
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());

    let mut page_ids: Vec<usize> = Vec::new();
    let slides: Vec<&Slide> = if presentation.slides.is_empty() {
        Vec::new()
    } else {
        presentation.slides.iter().collect()
    };
    let slide_count = if slides.is_empty() { 1 } else { slides.len() };

    for i in 0..slide_count {
        let lines = if slides.is_empty() {
            vec![(presentation.name.clone(), 28.0_f32)]
        } else {
            slide_text_lines(slides[i], i)
        };

        let mut content2 = String::new();
        content2.push_str("BT\n");
        let mut y2 = page_h - 72.0;
        for (text, font_size) in &lines {
            let size = (*font_size).clamp(10.0, 48.0);
            let max_chars = ((page_w - 144.0) / (size * 0.52)).max(10.0) as usize;
            let mut remaining = text.as_str();
            while !remaining.is_empty() {
                if y2 < 48.0 {
                    break;
                }
                let take = remaining
                    .char_indices()
                    .nth(max_chars)
                    .map(|(idx, _)| idx)
                    .unwrap_or(remaining.len());
                let (part, rest) = remaining.split_at(take);
                content2.push_str(&format!(
                    "/F1 {:.1} Tf 1 0 0 1 72 {:.1} Tm ({}) Tj\n",
                    size,
                    y2,
                    pdf_escape(part)
                ));
                y2 -= size * 1.6;
                remaining = rest.trim_start();
            }
        }
        content2.push_str("ET\n");

        let content_bytes = content2.into_bytes();
        let content_id = objects.len() + 1;
        objects.push(
            format!(
                "<< /Length {} >>\nstream\n{}\nendstream",
                content_bytes.len(),
                String::from_utf8_lossy(&content_bytes)
            )
            .into_bytes(),
        );

        let page_id = objects.len() + 1;
        let page_obj = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.0} {:.0}] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            page_w, page_h, content_id
        );
        objects.push(page_obj.into_bytes());
        page_ids.push(page_id);
    }

    let kids = page_ids
        .iter()
        .map(|id| format!("{id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    objects[1] = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids,
        page_ids.len()
    )
    .into_bytes();

    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
    let mut offsets: Vec<usize> = Vec::with_capacity(objects.len() + 1);
    offsets.push(0);
    for (idx, obj) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", idx + 1).as_bytes());
        out.extend_from_slice(obj);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref_start = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in offsets.iter().skip(1) {
        out.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_start
        )
        .as_bytes(),
    );
    Ok(out)
}
