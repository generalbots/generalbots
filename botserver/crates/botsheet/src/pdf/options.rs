//! Print layout selected in the sheet print dialog.

/// Print layout selected in the sheet print dialog.
#[derive(Debug, Clone, Copy)]
pub struct PdfOptions {
    pub landscape: bool,
    /// 0.5–2.0; scales the grid metrics, never the page.
    pub scale: f32,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self { landscape: false, scale: 1.0 }
    }
}
