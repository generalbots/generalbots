use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ExportBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub enum ExportError {
    InvalidFormat(String),
    RenderError(String),
    IoError(String),
    EmptyCanvas,
    InvalidDimensions,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(s) => write!(f, "Invalid format: {s}"),
            Self::RenderError(s) => write!(f, "Render error: {s}"),
            Self::IoError(s) => write!(f, "IO error: {s}"),
            Self::EmptyCanvas => write!(f, "Empty canvas"),
            Self::InvalidDimensions => write!(f, "Invalid dimensions"),
        }
    }
}

impl std::error::Error for ExportError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Png,
    Pdf,
    Svg,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub width: u32,
    pub height: u32,
    pub background_color: Option<String>,
    pub include_grid: bool,
    pub scale: f32,
    pub padding: u32,
    pub quality: u8,
    pub include_metadata: bool,
    pub selected_shapes_only: bool,
    pub selected_shape_ids: Vec<Uuid>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            width: 1920,
            height: 1080,
            background_color: Some("#ffffff".to_string()),
            include_grid: false,
            scale: 1.0,
            padding: 20,
            quality: 90,
            include_metadata: false,
            selected_shapes_only: false,
            selected_shape_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub id: Uuid,
    pub whiteboard_id: Uuid,
    pub format: ExportFormat,
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub whiteboard_name: String,
    pub shape_count: u32,
    pub export_dimensions: (u32, u32),
    pub original_dimensions: (u32, u32),
    pub exported_by: String,
    pub export_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardShape {
    pub id: Uuid,
    pub shape_type: ShapeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub fill_color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: f32,
    pub opacity: f32,
    pub points: Vec<Point>,
    pub text: Option<String>,
    pub font_size: Option<f32>,
    pub font_family: Option<String>,
    pub z_index: i32,
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Freehand,
    Text,
    Image,
    Sticky,
    Connector,
    Triangle,
    Diamond,
    Star,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardData {
    pub id: Uuid,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub background_color: String,
    pub shapes: Vec<WhiteboardShape>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct WhiteboardExportService {
    export_history: Arc<RwLock<HashMap<Uuid, Vec<ExportResult>>>>,
}

impl WhiteboardExportService {
    pub fn new() -> Self {
        Self {
            export_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn export(
        &self,
        whiteboard: &WhiteboardData,
        options: ExportOptions,
        user_id: Uuid,
        user_name: &str,
    ) -> Result<ExportResult, ExportError> {
        let shapes = if options.selected_shapes_only {
            whiteboard
                .shapes
                .iter()
                .filter(|s| options.selected_shape_ids.contains(&s.id))
                .cloned()
                .collect()
        } else {
            whiteboard.shapes.clone()
        };

        let bounds = self.calculate_bounds(&shapes, &options);

        let (data, content_type, extension) = match options.format {
            ExportFormat::Png => {
                let png_data = self.render_to_png(&shapes, &bounds, &options)?;
                (png_data, "image/png".to_string(), "png")
            }
            ExportFormat::Pdf => {
                let pdf_data = self.render_to_pdf(&shapes, &bounds, &options, whiteboard)?;
                (pdf_data, "application/pdf".to_string(), "pdf")
            }
            ExportFormat::Svg => {
                let svg_data = self.render_to_svg(&shapes, &bounds, &options)?;
                (svg_data.into_bytes(), "image/svg+xml".to_string(), "svg")
            }
            ExportFormat::Json => {
                let json_data = self.export_to_json(whiteboard, &shapes)?;
                (json_data.into_bytes(), "application/json".to_string(), "json")
            }
        };

        let file_name = format!(
            "{}_{}.{}",
            sanitize_filename(&whiteboard.name),
            Utc::now().format("%Y%m%d_%H%M%S"),
            extension
        );

        let result = ExportResult {
            id: Uuid::new_v4(),
            whiteboard_id: whiteboard.id,
            format: options.format.clone(),
            file_name,
            file_size: data.len() as u64,
            content_type,
            data,
            created_at: Utc::now(),
            created_by: user_id,
            metadata: ExportMetadata {
                whiteboard_name: whiteboard.name.clone(),
                shape_count: shapes.len() as u32,
                export_dimensions: (bounds.width as u32, bounds.height as u32),
                original_dimensions: (whiteboard.width, whiteboard.height),
                exported_by: user_name.to_string(),
                export_time: Utc::now(),
            },
        };

        let mut history = self.export_history.write().await;
        history
            .entry(whiteboard.id)
            .or_default()
            .push(result.clone());

        Ok(result)
    }

    fn calculate_bounds(&self, shapes: &[WhiteboardShape], options: &ExportOptions) -> ExportBounds {
        if shapes.is_empty() {
            return ExportBounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: options.width as f64,
                max_y: options.height as f64,
                width: options.width as f64,
                height: options.height as f64,
            };
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for shape in shapes {
            let shape_bounds = self.get_shape_bounds(shape);
            min_x = min_x.min(shape_bounds.0);
            min_y = min_y.min(shape_bounds.1);
            max_x = max_x.max(shape_bounds.2);
            max_y = max_y.max(shape_bounds.3);
        }

        let padding = options.padding as f64;
        min_x -= padding;
        min_y -= padding;
        max_x += padding;
        max_y += padding;

        let width = (max_x - min_x) * options.scale as f64;
        let height = (max_y - min_y) * options.scale as f64;

        ExportBounds {
            min_x,
            min_y,
            max_x,
            max_y,
            width,
            height,
        }
    }

    fn get_shape_bounds(&self, shape: &WhiteboardShape) -> (f64, f64, f64, f64) {
        match shape.shape_type {
            ShapeType::Freehand | ShapeType::Line | ShapeType::Arrow => {
                if shape.points.is_empty() {
                    return (shape.x, shape.y, shape.x + shape.width, shape.y + shape.height);
                }
                let min_x = shape.points.iter().map(|p| p.x).fold(f64::MAX, f64::min);
                let min_y = shape.points.iter().map(|p| p.y).fold(f64::MAX, f64::min);
                let max_x = shape.points.iter().map(|p| p.x).fold(f64::MIN, f64::max);
                let max_y = shape.points.iter().map(|p| p.y).fold(f64::MIN, f64::max);
                (min_x, min_y, max_x, max_y)
            }
            _ => (shape.x, shape.y, shape.x + shape.width, shape.y + shape.height),
        }
    }

    fn render_to_png(
        &self,
        shapes: &[WhiteboardShape],
        bounds: &ExportBounds,
        options: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        let width = bounds.width.max(1.0) as u32;
        let height = bounds.height.max(1.0) as u32;

        let mut pixels = vec![255u8; (width * height * 4) as usize];

        if let Some(bg_color) = &options.background_color {
            let (r, g, b) = parse_hex_color(bg_color).unwrap_or((255, 255, 255));
            for chunk in pixels.chunks_mut(4) {
                chunk[0] = r;
                chunk[1] = g;
                chunk[2] = b;
                chunk[3] = 255;
            }
        }

        let mut sorted_shapes = shapes.to_vec();
        sorted_shapes.sort_by_key(|s| s.z_index);

        for shape in &sorted_shapes {
            self.render_shape_to_pixels(shape, &mut pixels, width, height, bounds, options);
        }

        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);

            if let Ok(mut writer) = encoder.write_header() {
                let _ = writer.write_image_data(&pixels);
            }
        }

        if png_data.is_empty() {
            png_data = self.create_placeholder_png(width, height, options)?;
        }

        Ok(png_data)
    }

    fn render_shape_to_pixels(
        &self,
        shape: &WhiteboardShape,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        bounds: &ExportBounds,
        options: &ExportOptions,
    ) {
        let scale = options.scale as f64;
        let offset_x = bounds.min_x;
        let offset_y = bounds.min_y;

        let x = ((shape.x - offset_x) * scale) as i32;
        let y = ((shape.y - offset_y) * scale) as i32;
        let w = (shape.width * scale) as i32;
        let h = (shape.height * scale) as i32;

        let fill = shape
            .fill_color
            .as_ref()
            .and_then(|c| parse_hex_color(c));

        let stroke = shape
            .stroke_color
            .as_ref()
            .and_then(|c| parse_hex_color(c))
            .unwrap_or((0, 0, 0));

        let alpha = (shape.opacity * 255.0) as u8;

        match shape.shape_type {
            ShapeType::Rectangle | ShapeType::Sticky => {
                if let Some((r, g, b)) = fill {
                    self.fill_rect(pixels, width, height, x, y, w, h, r, g, b, alpha);
                }
                self.draw_rect_outline(pixels, width, height, x, y, w, h, stroke.0, stroke.1, stroke.2, alpha);
            }
            ShapeType::Ellipse => {
                if let Some((r, g, b)) = fill {
                    self.fill_ellipse(pixels, width, height, x, y, w, h, r, g, b, alpha);
                }
            }
            ShapeType::Line | ShapeType::Arrow | ShapeType::Freehand => {
                for i in 0..shape.points.len().saturating_sub(1) {
                    let p1 = &shape.points[i];
                    let p2 = &shape.points[i + 1];
                    let x1 = ((p1.x - offset_x) * scale) as i32;
                    let y1 = ((p1.y - offset_y) * scale) as i32;
                    let x2 = ((p2.x - offset_x) * scale) as i32;
                    let y2 = ((p2.y - offset_y) * scale) as i32;
                    self.draw_line(pixels, width, height, x1, y1, x2, y2, stroke.0, stroke.1, stroke.2, alpha);
                }
            }
            _ => {
                if let Some((r, g, b)) = fill {
                    self.fill_rect(pixels, width, height, x, y, w, h, r, g, b, alpha);
                }
            }
        }
    }

    fn fill_rect(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        let x_start = x.max(0) as u32;
        let y_start = y.max(0) as u32;
        let x_end = ((x + w) as u32).min(width);
        let y_end = ((y + h) as u32).min(height);

        for py in y_start..y_end {
            for px in x_start..x_end {
                let idx = ((py * width + px) * 4) as usize;
                if idx + 3 < pixels.len() {
                    self.blend_pixel(pixels, idx, r, g, b, a);
                }
            }
        }
    }

    fn draw_rect_outline(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        self.draw_line(pixels, width, height, x, y, x + w, y, r, g, b, a);
        self.draw_line(pixels, width, height, x + w, y, x + w, y + h, r, g, b, a);
        self.draw_line(pixels, width, height, x + w, y + h, x, y + h, r, g, b, a);
        self.draw_line(pixels, width, height, x, y + h, x, y, r, g, b, a);
    }

    fn fill_ellipse(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        let cx = x + w / 2;
        let cy = y + h / 2;
        let rx = w / 2;
        let ry = h / 2;

        if rx <= 0 || ry <= 0 {
            return;
        }

        let x_start = (x.max(0)) as u32;
        let y_start = (y.max(0)) as u32;
        let x_end = ((x + w) as u32).min(width);
        let y_end = ((y + h) as u32).min(height);

        for py in y_start..y_end {
            for px in x_start..x_end {
                let dx = (px as i32 - cx) as f64 / rx as f64;
                let dy = (py as i32 - cy) as f64 / ry as f64;
                if dx * dx + dy * dy <= 1.0 {
                    let idx = ((py * width + px) * 4) as usize;
                    if idx + 3 < pixels.len() {
                        self.blend_pixel(pixels, idx, r, g, b, a);
                    }
                }
            }
        }
    }

    fn draw_line(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && y >= 0 && (x as u32) < width && (y as u32) < height {
                let idx = ((y as u32 * width + x as u32) * 4) as usize;
                if idx + 3 < pixels.len() {
                    self.blend_pixel(pixels, idx, r, g, b, a);
                }
            }

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn blend_pixel(&self, pixels: &mut [u8], idx: usize, r: u8, g: u8, b: u8, a: u8) {
        let alpha = a as f32 / 255.0;
        let inv_alpha = 1.0 - alpha;

        pixels[idx] = (r as f32 * alpha + pixels[idx] as f32 * inv_alpha) as u8;
        pixels[idx + 1] = (g as f32 * alpha + pixels[idx + 1] as f32 * inv_alpha) as u8;
        pixels[idx + 2] = (b as f32 * alpha + pixels[idx + 2] as f32 * inv_alpha) as u8;
        pixels[idx + 3] = 255;
    }

    fn create_placeholder_png(
        &self,
        width: u32,
        height: u32,
        options: &ExportOptions,
    ) -> Result<Vec<u8>, ExportError> {
        let mut pixels = vec![255u8; (width * height * 4) as usize];

        if let Some(bg) = &options.background_color {
            if let Some((r, g, b)) = parse_hex_color(bg) {
                for chunk in pixels.chunks_mut(4) {
                    chunk[0] = r;
                    chunk[1] = g;
                    chunk[2] = b;
                    chunk[3] = 255;
                }
            }
        }

        let mut png_data = Vec::new();
        let mut encoder = png::Encoder::new(&mut png_data, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder
            .write_header()
            .map_err(|e| ExportError::RenderError(e.to_string()))?;

        writer
            .write_image_data(&pixels)
            .map_err(|e| ExportError::RenderError(e.to_string()))?;

        Ok(png_data)
    }

    fn render_to_pdf(
        &self,
        shapes: &[WhiteboardShape],
        bounds: &ExportBounds,
        options: &ExportOptions,
        whiteboard: &WhiteboardData,
    ) -> Result<Vec<u8>, ExportError> {
        let mut pdf = PdfDocument::new(&whiteboard.name);

        let page_width = bounds.width.max(595.0);
        let page_height = bounds.height.max(842.0);

        pdf.add_page(page_width, page_height);

        if let Some(bg_color) = &options.background_color {
            pdf.set_fill_color(bg_color);
            pdf.draw_rect(0.0, 0.0, page_width, page_height, true, false);
        }

        let mut sorted_shapes = shapes.to_vec();
        sorted_shapes.sort_by_key(|s| s.z_index);

        for shape in &sorted_shapes {
            self.render_shape_to_pdf(&mut pdf, shape, bounds, options);
        }

        if options.include_metadata {
            pdf.add_metadata(&whiteboard.name, &Utc::now().to_rfc3339());
        }

        Ok(pdf.to_bytes())
    }

    fn render_shape_to_pdf(
        &self,
        pdf: &mut PdfDocument,
        shape: &WhiteboardShape,
        bounds: &ExportBounds,
        options: &ExportOptions,
    ) {
        let scale = options.scale as f64;
        let x = (shape.x - bounds.min_x) * scale;
        let y = (shape.y - bounds.min_y) * scale;
        let w = shape.width * scale;
        let h = shape.height * scale;

        if let Some(fill) = &shape.fill_color {
            pdf.set_fill_color(fill);
        }
        if let Some(stroke) = &shape.stroke_color {
            pdf.set_stroke_color(stroke);
        }
        pdf.set_line_width(shape.stroke_width as f64);

        match shape.shape_type {
            ShapeType::Rectangle | ShapeType::Sticky => {
                pdf.draw_rect(x, y, w, h, shape.fill_color.is_some(), shape.stroke_color.is_some());
            }
            ShapeType::Ellipse => {
                pdf.draw_ellipse(x + w / 2.0, y + h / 2.0, w / 2.0, h / 2.0, shape.fill_color.is_some(), shape.stroke_color.is_some());
            }
            ShapeType::Line | ShapeType::Arrow | ShapeType::Freehand => {
                if !shape.points.is_empty() {
                    let points: Vec<(f64, f64)> = shape
                        .points
                        .iter()
                        .map(|p| {
                            ((p.x - bounds.min_x) * scale, (p.y - bounds.min_y) * scale)
                        })
                        .collect();
                    pdf.draw_path(&points);
                }
            }
            ShapeType::Text => {
                if let Some(text) = &shape.text {
                    let font_size = shape.font_size.unwrap_or(12.0) * options.scale;
                    pdf.draw_text(text, x, y, font_size as f64);
                }
            }
            ShapeType::Triangle => {
                let points = vec![
                    (x + w / 2.0, y),
                    (x + w, y + h),
                    (x, y + h),
                    (x + w / 2.0, y),
                ];
                pdf.draw_path(&points);
            }
            ShapeType::Diamond => {
                let points = vec![
                    (x + w / 2.0, y),
                    (x + w, y + h / 2.0),
                    (x + w / 2.0, y + h),
                    (x, y + h / 2.0),
                    (x + w / 2.0, y),
                ];
                pdf.draw_path(&points);
            }
            _ => {
                pdf.draw_rect(x, y, w, h, shape.fill_color.is_some(), shape.stroke_color.is_some());
            }
        }
    }

    fn render_to_svg(
        &self,
        shapes: &[WhiteboardShape],
        bounds: &ExportBounds,
        options: &ExportOptions,
    ) -> Result<String, ExportError> {
        let width = bounds.width.max(1.0);
        let height = bounds.height.max(1.0);

        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            width, height, width, height
        ));

        svg.push_str(r#"
<defs>
    <style>
        .shape { transition: all 0.2s ease; }
        .shape:hover { filter: brightness(1.1); }
    </style>
</defs>"#);

        if let Some(bg_color) = &options.background_color {
            svg.push_str(&format!(
                r#"<rect width="100%" height="100%" fill="{}"/>"#,
                bg_color
            ));
        }

        if options.include_grid {
            svg.push_str(&self.generate_svg_grid(width, height));
        }

        let mut sorted_shapes = shapes.to_vec();
        sorted_shapes.sort_by_key(|s| s.z_index);

        for shape in &sorted_shapes {
            svg.push_str(&self.shape_to_svg(shape, bounds, options));
        }

        svg.push_str("</svg>");

        Ok(svg)
    }

    fn generate_svg_grid(&self, width: f64, height: f64) -> String {
        let grid_size = 20.0;
        let mut grid = String::new();

        grid.push_str(r##"<g class="grid" stroke="#e0e0e0" stroke-width="0.5">"##);

        let mut x = 0.0;
        while x <= width {
            grid.push_str(&format!(
                r#"<line x1="{}" y1="0" x2="{}" y2="{}"/>"#,
                x, x, height
            ));
            x += grid_size;
        }

        let mut y = 0.0;
        while y <= height {
            grid.push_str(&format!(
                r#"<line x1="0" y1="{}" x2="{}" y2="{}"/>"#,
                y, width, y
            ));
            y += grid_size;
        }

        grid.push_str("</g>");
        grid
    }

    fn shape_to_svg(
        &self,
        shape: &WhiteboardShape,
        bounds: &ExportBounds,
        options: &ExportOptions,
    ) -> String {
        let scale = options.scale as f64;
        let x = (shape.x - bounds.min_x) * scale;
        let y = (shape.y - bounds.min_y) * scale;
        let w = shape.width * scale;
        let h = shape.height * scale;

        let fill = shape
            .fill_color
            .as_ref()
            .map(|c| c.as_str())
            .unwrap_or("none");

        let stroke = shape
            .stroke_color
            .as_ref()
            .map(|c| c.as_str())
            .unwrap_or("none");

        let stroke_width = shape.stroke_width * options.scale;
        let opacity = shape.opacity;

        let transform = if shape.rotation != 0.0 {
            format!(
                r#" transform="rotate({} {} {})""#,
                shape.rotation,
                x + w / 2.0,
                y + h / 2.0
            )
        } else {
            String::new()
        };

        match shape.shape_type {
            ShapeType::Rectangle | ShapeType::Sticky => {
                format!(
                    r#"<rect class="shape" x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"{}/>"#,
                    x, y, w, h, fill, stroke, stroke_width, opacity, transform
                )
            }
            ShapeType::Ellipse => {
                format!(
                    r#"<ellipse class="shape" cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"{}/>"#,
                    x + w / 2.0,
                    y + h / 2.0,
                    w / 2.0,
                    h / 2.0,
                    fill,
                    stroke,
                    stroke_width,
                    opacity,
                    transform
                )
            }
            ShapeType::Line | ShapeType::Arrow | ShapeType::Freehand => {
                if shape.points.is_empty() {
                    return String::new();
                }
                let points: Vec<String> = shape
                    .points
                    .iter()
                    .map(|p| {
                        format!(
                            "{},{}",
                            (p.x - bounds.min_x) * scale,
                            (p.y - bounds.min_y) * scale
                        )
                    })
                    .collect();

                if shape.shape_type == ShapeType::Freehand {
                    let mut path = format!("M {}", points[0]);
                    for point in points.iter().skip(1) {
                        path.push_str(&format!(" L {}", point));
                    }
                    format!(
                        r#"<path class="shape" d="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"{}/>"#,
                        path, stroke, stroke_width, opacity, transform
                    )
                } else {
                    let line_points = points.join(" ");
                    let marker = if shape.shape_type == ShapeType::Arrow {
                        r#" marker-end="url(#arrowhead)""#
                    } else {
                        ""
                    };
                    format!(
                        r#"<polyline class="shape" points="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}"{}{}/>"#,
                        line_points, stroke, stroke_width, opacity, marker, transform
                    )
                }
            }
            ShapeType::Text => {
                let font_size = shape.font_size.unwrap_or(16.0) * scale;
                let text_content = shape.text.as_deref().unwrap_or("");
                format!(
                    r#"<text class="shape" x="{}" y="{}" font-size="{}" fill="{}" opacity="{}"{}>{}</text>"#,
                    x, y + font_size, font_size, fill, opacity, transform, text_content
                )
            }
            ShapeType::Image => {
                if let Some(src) = &shape.image_url {
                    format!(
                        r#"<image class="shape" x="{}" y="{}" width="{}" height="{}" href="{}" opacity="{}"{}/>"#,
                        x, y, w, h, src, opacity, transform
                    )
                } else {
                    String::new()
                }
            }
        }
    }
}
