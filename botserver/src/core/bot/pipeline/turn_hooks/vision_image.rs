//! Shrinks an attached picture the way the providers' own chat apps do before
//! it goes to a vision model: the long edge is cut to [`LONG_EDGE`] pixels and
//! the result is re-encoded, so a 25 MB poster photo arrives as a few hundred
//! kilobytes instead of being refused and read only as OCR text.

use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;

/// Longest side sent to the model. Claude's guidance: larger images are
/// downscaled by the provider anyway and only cost time and tokens.
pub(super) const LONG_EDGE: u32 = 1568;
/// A picture already this small and this narrow is sent untouched.
const PASS_THROUGH_BYTES: usize = 3 * 1024 * 1024;

/// The bytes and media type to send for an image, or `None` when the image
/// cannot be decoded (the turn then falls back to the recognised text).
pub(crate) fn prepare(data: &[u8], media_type: &str) -> Option<(Vec<u8>, &'static str)> {
    let format = match media_type {
        "image/png" => ImageFormat::Png,
        "image/jpeg" => ImageFormat::Jpeg,
        "image/gif" => ImageFormat::Gif,
        // WebP is not decoded here; small files go as they are.
        "image/webp" => return (data.len() <= PASS_THROUGH_BYTES).then(|| (data.to_vec(), "image/webp")),
        _ => return None,
    };
    let picture = image::load_from_memory_with_format(data, format).ok()?;
    let (w, h) = picture.dimensions();
    if data.len() <= PASS_THROUGH_BYTES && w.max(h) <= LONG_EDGE {
        return Some((data.to_vec(), static_type(format)));
    }
    let scaled = if w.max(h) > LONG_EDGE {
        picture.resize(LONG_EDGE, LONG_EDGE, FilterType::Triangle)
    } else {
        picture
    };
    encode(scaled)
}

fn static_type(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "image/png",
        ImageFormat::Gif => "image/gif",
        _ => "image/jpeg",
    }
}

/// PNG keeps transparency (logos, diagrams); everything else becomes JPEG.
fn encode(picture: DynamicImage) -> Option<(Vec<u8>, &'static str)> {
    let mut out = Cursor::new(Vec::new());
    if picture.color().has_alpha() {
        picture.write_to(&mut out, ImageFormat::Png).ok()?;
        return Some((out.into_inner(), "image/png"));
    }
    let rgb = DynamicImage::ImageRgb8(picture.to_rgb8());
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 85);
    encoder.encode_image(&rgb).ok()?;
    Some((out.into_inner(), "image/jpeg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    fn jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = RgbImage::from_fn(w, h, |x, y| Rgb([(x % 256) as u8, (y % 256) as u8, 128]));
        let mut out = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img).write_to(&mut out, ImageFormat::Jpeg).unwrap_or_default();
        out.into_inner()
    }

    #[test]
    fn a_large_picture_is_cut_to_the_long_edge() {
        let (bytes, mt) = prepare(&jpeg(4000, 2000), "image/jpeg").unwrap_or_default();
        assert_eq!(mt, "image/jpeg");
        let out = image::load_from_memory(&bytes).map(|i| i.dimensions()).unwrap_or((0, 0));
        assert_eq!(out, (LONG_EDGE, LONG_EDGE / 2));
    }

    #[test]
    fn a_small_picture_goes_untouched() {
        let original = jpeg(800, 600);
        let (bytes, _) = prepare(&original, "image/jpeg").unwrap_or_default();
        assert_eq!(bytes, original);
    }

    #[test]
    fn garbage_is_not_sent() {
        assert!(prepare(b"not an image", "image/png").is_none());
    }
}
