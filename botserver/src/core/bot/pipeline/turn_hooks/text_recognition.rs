//! Text recognition for attachments that carry no text layer: images and
//! scanned PDFs.
//!
//! On Windows it uses the engine that ships with the system
//! (`Windows.Media.Ocr`; scanned PDF pages are rendered with
//! `Windows.Data.Pdf` first): nothing is downloaded and nothing leaves the
//! machine. Other systems have no recognizer here, so the attachment keeps
//! its honest "cannot read" note.

/// Most pages read from a scanned PDF.
const MAX_PDF_PAGES: u32 = 20;
/// Characters kept, the same bound as the other attachment extractors.
const MAX_CHARS: usize = 12_000;

/// The text found in an image or a scanned PDF, or `None` when there is no
/// recognizer, it failed, or what it found is not text.
pub(crate) async fn recognize(name: &str, data: Vec<u8>) -> Option<String> {
    let pdf = name.to_lowercase().ends_with(".pdf");
    let started = std::time::Instant::now();
    let outcome = tokio::task::spawn_blocking(move || platform::recognize(&data, pdf)).await;
    let text = match outcome {
        Ok(Ok(text)) => text,
        Ok(Err(error)) => {
            log::warn!("text recognition failed for {name}: {error}");
            return None;
        }
        Err(error) => {
            log::warn!("text recognition task failed for {name}: {error}");
            return None;
        }
    };
    let text = bounded(&text);
    log::info!(
        "text recognition: {name}: {} characters in {} ms",
        text.chars().count(),
        started.elapsed().as_millis()
    );
    has_words(&text).then_some(text)
}

fn bounded(text: &str) -> String {
    let trimmed = text.trim();
    match trimmed.char_indices().nth(MAX_CHARS) {
        Some((end, _)) => trimmed[..end].to_string(),
        None => trimmed.to_string(),
    }
}

/// Whether the recognised text is worth handing to the model: a photo often
/// yields a few stray characters, which are not text, while a poster or a
/// sign may carry a single word.
fn has_words(text: &str) -> bool {
    let sizes: Vec<usize> = text
        .split_whitespace()
        .map(|word| word.chars().filter(|c| c.is_alphanumeric()).count())
        .collect();
    sizes.iter().any(|&size| size >= 4) || sizes.iter().filter(|&&size| size >= 2).count() >= 3
}

#[cfg(windows)]
mod platform {
    use windows::core::{Result, HSTRING};
    use windows::Data::Pdf::{PdfDocument, PdfPageRenderOptions};
    use windows::Globalization::Language;
    use windows::Graphics::Imaging::{
        BitmapAlphaMode, BitmapDecoder, BitmapInterpolationMode, BitmapPixelFormat,
        BitmapTransform, ColorManagementMode, ExifOrientationMode,
    };
    use windows::Media::Ocr::OcrEngine;
    use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};

    pub(super) fn recognize(data: &[u8], pdf: bool) -> Result<String> {
        let engine = engine()?;
        let source = stream_from(data)?;
        if !pdf {
            return read_picture(&engine, &source);
        }
        let document = PdfDocument::LoadFromStreamAsync(&source)?.get()?;
        let pages = document.PageCount()?.min(super::MAX_PDF_PAGES);
        let largest = OcrEngine::MaxImageDimension()?;
        let mut text = String::new();
        for index in 0..pages {
            let page = document.GetPage(index)?;
            let rendered = InMemoryRandomAccessStream::new()?;
            let options = PdfPageRenderOptions::new()?;
            // Twice the page's own size: small print is lost at screen size.
            let width = (page.Size()?.Width * 2.0).max(1.0) as u32;
            options.SetDestinationWidth(width.min(largest))?;
            page.RenderWithOptionsToStreamAsync(&rendered, &options)?
                .get()?;
            let page_text = read_image(&engine, &rendered, 1.0)?;
            if !page_text.trim().is_empty() {
                if !text.is_empty() {
                    text.push_str("\n\n");
                }
                text.push_str(&page_text);
            }
            if text.chars().count() >= super::MAX_CHARS {
                break;
            }
        }
        Ok(text)
    }

    /// Portuguese first (the edition's language); the languages of the
    /// Windows profile when that pack is not installed.
    fn engine() -> Result<OcrEngine> {
        let portuguese = Language::CreateLanguage(&HSTRING::from("pt-BR"))?;
        match OcrEngine::TryCreateFromLanguage(&portuguese) {
            Ok(engine) => Ok(engine),
            Err(error) => {
                log::info!(
                    "text recognition: no pt-BR engine ({error}), using the profile languages"
                );
                OcrEngine::TryCreateFromUserProfileLanguages()
            }
        }
    }

    fn stream_from(data: &[u8]) -> Result<InMemoryRandomAccessStream> {
        let stream = InMemoryRandomAccessStream::new()?;
        let writer = DataWriter::CreateDataWriter(&stream)?;
        writer.WriteBytes(data)?;
        writer.StoreAsync()?.get()?;
        writer.FlushAsync()?.get()?;
        writer.DetachStream()?;
        stream.Seek(0)?;
        Ok(stream)
    }

    /// The recognizer sometimes finds no line in a picture at its own size
    /// (a single word in large print beside a shape) and finds it at half or
    /// twice that size, so a picture with no text is read again at those.
    fn read_picture(engine: &OcrEngine, stream: &InMemoryRandomAccessStream) -> Result<String> {
        for scale in [1.0, 0.5, 2.0] {
            let text = read_image(engine, stream, scale)?;
            if !text.trim().is_empty() {
                return Ok(text);
            }
        }
        Ok(String::new())
    }

    fn read_image(
        engine: &OcrEngine,
        stream: &InMemoryRandomAccessStream,
        scale: f64,
    ) -> Result<String> {
        stream.Seek(0)?;
        let decoder = BitmapDecoder::CreateAsync(stream)?.get()?;
        let largest = f64::from(OcrEngine::MaxImageDimension()?);
        let (width, height) = (f64::from(decoder.PixelWidth()?), f64::from(decoder.PixelHeight()?));
        // The engine refuses an image larger than its maximum on either side.
        let scale = scale.min(largest / width.max(height).max(1.0));
        let transform = BitmapTransform::new()?;
        if (scale - 1.0).abs() > f64::EPSILON {
            transform.SetScaledWidth(((width * scale) as u32).max(1))?;
            transform.SetScaledHeight(((height * scale) as u32).max(1))?;
            transform.SetInterpolationMode(BitmapInterpolationMode::Fant)?;
        }
        let bitmap = decoder
            .GetSoftwareBitmapTransformedAsync(
                BitmapPixelFormat::Bgra8,
                BitmapAlphaMode::Premultiplied,
                &transform,
                ExifOrientationMode::RespectExifOrientation,
                ColorManagementMode::DoNotColorManage,
            )?
            .get()?;
        let result = engine.RecognizeAsync(&bitmap)?.get()?;
        let lines = result.Lines()?;
        let mut out = Vec::new();
        for index in 0..lines.Size()? {
            out.push(lines.GetAt(index)?.Text()?.to_string());
        }
        Ok(out.join("\n"))
    }
}

#[cfg(not(windows))]
mod platform {
    pub(super) fn recognize(data: &[u8], pdf: bool) -> Result<String, String> {
        std::hint::black_box((data, pdf));
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::{bounded, has_words, MAX_CHARS};

    #[test]
    fn stray_characters_are_not_text() {
        assert!(!has_words(""));
        assert!(!has_words("a | ~ 1"));
        assert!(!has_words("IV ."));
        assert!(has_words("Ofício nº 1.234/2026 Coordenação de Tecnologia"));
        assert!(has_words("AZUL"));
    }

    #[test]
    fn the_text_is_bounded_on_a_character() {
        let long = "ç".repeat(MAX_CHARS + 50);
        assert_eq!(bounded(&long).chars().count(), MAX_CHARS);
        assert_eq!(bounded("  texto  "), "texto");
    }
}
