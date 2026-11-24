//! Typed identities and parameter keys for the Office surfaces (#2475).
//!
//! This is the vocabulary of the Office × platform deep-link contract. It is
//! deliberately dependency-free and feature-flag-free: the same types are
//! available to handlers, the LLM command catalog and tests regardless of
//! which Office apps a given edition ships.
//!
//! App ids match the suite registry (`docs`, `sheet`, `slides`) so a link built
//! here can be handed to the existing window manager unchanged.

/// The three Office applications that are first-class GB surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OfficeSurface {
    Docs,
    Sheet,
    Slides,
}

impl OfficeSurface {
    /// Canonical, lowercase app id used in `app://<id>?...` links and in the
    /// suite app registry.
    pub const fn app_id(self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Sheet => "sheet",
            Self::Slides => "slides",
        }
    }

    /// Human-readable product name.
    pub const fn title(self) -> &'static str {
        match self {
            Self::Docs => "Docs",
            Self::Sheet => "Sheets",
            Self::Slides => "Slides",
        }
    }

    /// Resolve an app id. Unknown ids are refused — there is no fallback to a
    /// "default" Office app, because a wrong fallback silently opens the wrong
    /// editor.
    pub fn from_app_id(app_id: &str) -> Option<Self> {
        match app_id {
            "docs" => Some(Self::Docs),
            "sheet" | "sheets" => Some(Self::Sheet),
            "slides" => Some(Self::Slides),
            _ => None,
        }
    }

    /// All surfaces, in catalog order.
    pub const fn all() -> &'static [Self] {
        &[Self::Docs, Self::Sheet, Self::Slides]
    }

    /// Parameter keys this surface understands (see `super::capabilities`).
    pub const fn accepted_params(self) -> &'static [OfficeParam] {
        match self {
            Self::Docs => DOCS_PARAMS,
            Self::Sheet => SHEET_PARAMS,
            Self::Slides => SLIDES_PARAMS,
        }
    }

    /// Whether `param` is part of this surface's declared capability.
    pub fn accepts(self, param: OfficeParam) -> bool {
        self.accepted_params().contains(&param)
    }
}

impl std::fmt::Display for OfficeSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.title())
    }
}

/// Query parameters understood by the Office deep-link contract.
///
/// The set is intentionally small: it is the platform-independent vocabulary
/// a bot can emit without knowing which Office editor is installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OfficeParam {
    /// A Drive object reference (file key or path) to open.
    File,
    /// Id of a Vibe-VM conversation whose output becomes the document.
    VmConvId,
    /// Source path inside the Vibe VM to import/convert.
    VmPath,
    /// Boolean flag (`true`/`false`): open a new, empty document.
    Blank,
}

impl OfficeParam {
    /// Exact query-string key.
    pub const fn key(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::VmConvId => "vmConvId",
            Self::VmPath => "vmPath",
            Self::Blank => "blank",
        }
    }

    /// Resolve a query-string key. Unknown keys are refused.
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "file" => Some(Self::File),
            "vmConvId" => Some(Self::VmConvId),
            "vmPath" => Some(Self::VmPath),
            "blank" => Some(Self::Blank),
            _ => None,
        }
    }

    /// All keys, in the canonical order used when rendering a link.
    pub const fn all() -> &'static [Self] {
        &[
            Self::File,
            Self::VmConvId,
            Self::VmPath,
            Self::Blank,
        ]
    }

    /// True for parameters that carry a filesystem-like value, where `..`
    /// segments must be rejected.
    pub const fn is_path_like(self) -> bool {
        matches!(self, Self::File | Self::VmPath)
    }
}

impl std::fmt::Display for OfficeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

/// Capability matrix.
///
/// `vmPath` is offered by Docs and Sheets (both import a source file from a
/// Vibe VM). Slides decks are produced from a Drive file or a conversation,
/// so a bare VM source path is not a valid way to open a deck — the parser
/// refuses it for `slides`.
pub const DOCS_PARAMS: &[OfficeParam] = &[
    OfficeParam::File,
    OfficeParam::VmConvId,
    OfficeParam::VmPath,
    OfficeParam::Blank,
];
pub const SHEET_PARAMS: &[OfficeParam] = &[
    OfficeParam::File,
    OfficeParam::VmConvId,
    OfficeParam::VmPath,
    OfficeParam::Blank,
];
pub const SLIDES_PARAMS: &[OfficeParam] = &[
    OfficeParam::File,
    OfficeParam::VmConvId,
    OfficeParam::Blank,
];
