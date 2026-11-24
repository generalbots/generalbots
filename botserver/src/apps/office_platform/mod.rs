//! Office × platform deep-link contract (#2475).
//!
//! A small, pure layer that lets bots and handlers name an Office surface and
//! hand over a fully validated `app://` link. It owns no routes and no state:
//! the window manager and the command catalog keep their existing roles, this
//! module only makes the *contract* typed, deterministic and hostile-input
//! safe.
//!
//! ```text
//! use botserver::apps::office_platform::{DeepLink, OfficeSurface};
//!
//! // Builders validate each value as it is added:
//! let built = DeepLink::new(OfficeSurface::Docs).file("reports/Q3.docx");
//! // Parsers validate the whole string, after percent-decoding:
//! let parsed = DeepLink::parse("app://docs?file=reports%2FQ3.docx");
//! // `render()` produces the canonical, deterministic form of either link.
//! ```

mod link;
mod surface;

#[cfg(test)]
mod tests;

pub use link::{DeepLink, DeepLinkError};
pub use surface::{OfficeParam, OfficeSurface, DOCS_PARAMS, SHEET_PARAMS, SLIDES_PARAMS};

/// Declaration of one surface's accepted deep-link parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceCapability {
    pub surface: OfficeSurface,
    pub app_id: &'static str,
    pub accepted_params: &'static [OfficeParam],
    pub summary: &'static str,
}

/// The complete capability table: which surface accepts which parameters.
///
/// This is the machine-readable counterpart of the `capabilities` section in
/// issue #2475 and is meant to be injected into prompts/catalogs instead of
/// hand-written prose drifting out of sync with the parser.
pub fn surface_capabilities() -> [SurfaceCapability; 3] {
    [
        SurfaceCapability {
            surface: OfficeSurface::Docs,
            app_id: OfficeSurface::Docs.app_id(),
            accepted_params: DOCS_PARAMS,
            summary: "Word-like editor; opens a Drive file, a VM conversion, a VM source path or a blank document.",
        },
        SurfaceCapability {
            surface: OfficeSurface::Sheet,
            app_id: OfficeSurface::Sheet.app_id(),
            accepted_params: SHEET_PARAMS,
            summary: "Spreadsheet editor; opens a Drive file, a VM conversion, a VM source path or a blank workbook.",
        },
        SurfaceCapability {
            surface: OfficeSurface::Slides,
            app_id: OfficeSurface::Slides.app_id(),
            accepted_params: SLIDES_PARAMS,
            summary: "Presentation editor; opens a Drive file, a VM conversation output or a blank deck (no VM source path).",
        },
    ]
}

/// Render the capability table as a compact, deterministic text description
/// suitable for an LLM prompt or an API catalog entry.
pub fn describe_capabilities() -> String {
    let mut lines = Vec::with_capacity(3);
    for capability in surface_capabilities() {
        let keys: Vec<&str> = capability
            .accepted_params
            .iter()
            .map(|param| param.key())
            .collect();
        lines.push(format!(
            "- {} (app://{}) — {}; params: {}",
            capability.surface,
            capability.app_id,
            capability.summary,
            keys.join(", ")
        ));
    }
    lines.join("\n")
}
