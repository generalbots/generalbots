//! `app://` deep-link builder, parser and validator for Office surfaces.
//!
//! The grammar is intentionally a strict subset of the existing suite deep
//! links so a link produced here is accepted by the window manager as-is:
//!
//! ```text
//! app://<app_id>?key=value&key=value
//! ```
//!
//! - `<app_id>` must be a known [`OfficeSurface`] id; unknown apps are refused.
//! - `key` must be a known [`OfficeParam`] supported by that surface.
//! - `value` is percent-encoded (unreserved bytes only, uppercase hex), so
//!   rendering is deterministic and parse → render → parse is a fixed point.
//! - Values are validated *after* decoding: script schemes, control
//!   characters and `..` path segments are rejected.

use super::surface::{OfficeParam, OfficeSurface};
use std::collections::BTreeMap;
use std::fmt;

/// Why a deep link could not be built or parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeepLinkError {
    /// The `app://` scheme or its `?` separator is missing/malformed.
    Malformed(String),
    /// The host is not a registered Office app id.
    UnknownApp(String),
    /// A query key is not part of the Office vocabulary.
    UnknownParam(String),
    /// A query key appeared more than once.
    DuplicateParam(OfficeParam),
    /// The surface does not declare this parameter as a capability.
    UnsupportedParam {
        surface: OfficeSurface,
        param: OfficeParam,
    },
    /// A `%` escape is truncated, non-hex or not valid UTF-8.
    InvalidEncoding(String),
    /// A decoded value failed validation.
    InvalidValue {
        param: OfficeParam,
        value: String,
        reason: &'static str,
    },
}

impl fmt::Display for DeepLinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(input) => {
                write!(f, "malformed Office deep link {input:?}")
            }
            Self::UnknownApp(app) => {
                write!(f, "unknown Office app {app:?}")
            }
            Self::UnknownParam(key) => {
                write!(f, "unknown Office deep-link parameter {key:?}")
            }
            Self::DuplicateParam(param) => {
                write!(f, "duplicate Office deep-link parameter {param}")
            }
            Self::UnsupportedParam { surface, param } => {
                write!(f, "{surface} does not accept the {param} parameter")
            }
            Self::InvalidEncoding(value) => {
                write!(f, "invalid percent-encoding in Office deep link: {value:?}")
            }
            Self::InvalidValue {
                param,
                value,
                reason,
            } => {
                write!(
                    f,
                    "unsafe value for Office parameter {param}: {reason} ({value:?})"
                )
            }
        }
    }
}

impl std::error::Error for DeepLinkError {}

/// A validated Office deep link.
///
/// Parameters are stored in a `BTreeMap` keyed by [`OfficeParam`], whose
/// derived ordering defines the canonical rendering order
/// (`file`, `vmConvId`, `vmPath`, `blank`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeepLink {
    surface: OfficeSurface,
    params: BTreeMap<OfficeParam, String>,
}

impl DeepLink {
    /// Start a link for `surface` with no parameters.
    pub fn new(surface: OfficeSurface) -> Self {
        Self {
            surface,
            params: BTreeMap::new(),
        }
    }

    /// Target surface of this link.
    pub const fn surface(&self) -> OfficeSurface {
        self.surface
    }

    /// Decoded value of a parameter, if present.
    pub fn get(&self, param: OfficeParam) -> Option<&str> {
        self.params.get(&param).map(String::as_str)
    }

    /// Number of parameters carried by this link.
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// True when the link carries no parameters (`app://docs`).
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// All parameters in canonical order.
    pub fn params(&self) -> impl Iterator<Item = (OfficeParam, &str)> + '_ {
        self.params.iter().map(|(key, value)| (*key, value.as_str()))
    }

    /// Add one validated parameter. An existing value for the same key is
    /// replaced, which makes repeated builder calls deterministic.
    pub fn with_param(
        mut self,
        param: OfficeParam,
        value: impl Into<String>,
    ) -> Result<Self, DeepLinkError> {
        let value = value.into();
        self.check_supported(param)?;
        validate_value(param, &value)?;
        self.params.insert(param, value);
        Ok(self)
    }

    /// Open a Drive object (`file`).
    pub fn file(self, value: impl Into<String>) -> Result<Self, DeepLinkError> {
        self.with_param(OfficeParam::File, value)
    }

    /// Open the output of a Vibe-VM conversation (`vmConvId`).
    pub fn vm_conv_id(self, value: impl Into<String>) -> Result<Self, DeepLinkError> {
        self.with_param(OfficeParam::VmConvId, value)
    }

    /// Import a source file from a Vibe VM path (`vmPath`).
    pub fn vm_path(self, value: impl Into<String>) -> Result<Self, DeepLinkError> {
        self.with_param(OfficeParam::VmPath, value)
    }

    /// Open a new, empty document (`blank=true` / `blank=false`).
    pub fn blank(self, enabled: bool) -> Result<Self, DeepLinkError> {
        let value = if enabled { "true" } else { "false" };
        self.with_param(OfficeParam::Blank, value)
    }

    fn check_supported(&self, param: OfficeParam) -> Result<(), DeepLinkError> {
        if self.surface.accepts(param) {
            Ok(())
        } else {
            Err(DeepLinkError::UnsupportedParam {
                surface: self.surface,
                param,
            })
        }
    }

    /// Render the canonical `app://...?...` string.
    pub fn render(&self) -> String {
        let mut out = format!("app://{}", self.surface.app_id());
        if !self.params.is_empty() {
            out.push('?');
            let mut first = true;
            for (key, value) in &self.params {
                if !first {
                    out.push('&');
                }
                first = false;
                out.push_str(key.key());
                out.push('=');
                out.push_str(&percent_encode(value));
            }
        }
        out
    }

    /// Parse and validate a deep link. The whole string must be in canonical
    /// form: a trailing `?`, an empty pair, a repeated key or a non-canonical
    /// escape sequence is refused rather than silently normalised.
    pub fn parse(input: &str) -> Result<Self, DeepLinkError> {
        const PREFIX: &str = "app://";
        let Some(prefix) = input.get(..PREFIX.len()) else {
            return Err(DeepLinkError::Malformed(input.to_string()));
        };
        if !prefix.eq_ignore_ascii_case(PREFIX) {
            return Err(DeepLinkError::Malformed(input.to_string()));
        }
        // The prefix is ASCII, so byte 6 is a valid UTF-8 boundary here.
        let rest = &input[PREFIX.len()..];
        let (app_id, query) = match rest.split_once('?') {
            Some((app, query)) => (app, Some(query)),
            None => (rest, None),
        };
        let surface = OfficeSurface::from_app_id(app_id)
            .ok_or_else(|| DeepLinkError::UnknownApp(app_id.to_string()))?;
        let mut link = Self::new(surface);
        let Some(query) = query else { return Ok(link) };
        if query.is_empty() {
            return Err(DeepLinkError::Malformed(input.to_string()));
        }
        for pair in query.split('&') {
            if pair.is_empty() {
                return Err(DeepLinkError::Malformed(input.to_string()));
            }
            let (raw_key, raw_value) = pair
                .split_once('=')
                .ok_or_else(|| DeepLinkError::Malformed(input.to_string()))?;
            let param = OfficeParam::from_key(raw_key)
                .ok_or_else(|| DeepLinkError::UnknownParam(raw_key.to_string()))?;
            if link.params.contains_key(&param) {
                return Err(DeepLinkError::DuplicateParam(param));
            }
            link.check_supported(param)?;
            let value = percent_decode(raw_value)?;
            validate_value(param, &value)?;
            link.params.insert(param, value);
        }
        Ok(link)
    }
}

impl fmt::Display for DeepLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

/// Reject unsafe values before they can reach an app shell or a Drive path.
fn validate_value(
    param: OfficeParam,
    value: &str,
) -> Result<(), DeepLinkError> {
    let reject = |reason: &'static str| DeepLinkError::InvalidValue {
        param,
        value: value.to_string(),
        reason,
    };

    if value.is_empty() {
        return Err(reject("empty values are not allowed"));
    }
    if value.chars().any(|c| c.is_control()) {
        return Err(reject("control characters are not allowed"));
    }
    // Collapse whitespace so `java script:` and ` JavaScript:` cannot slip past
    // the scheme check (control characters were already rejected above).
    let compact: String = value
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();
    let lowered = compact.to_ascii_lowercase();
    if lowered.starts_with("javascript:")
        || lowered.starts_with("data:")
        || lowered.starts_with("vbscript:")
    {
        return Err(reject("script/data schemes are not allowed"));
    }
    if param == OfficeParam::Blank {
        if lowered != "true" && lowered != "false" {
            return Err(reject("blank must be true or false"));
        }
        return Ok(());
    }
    if param.is_path_like() {
        let normalised = value.replace('\\', "/");
        if normalised
            .split('/')
            .any(|segment| segment.trim() == "..")
        {
            return Err(reject("path traversal is not allowed"));
        }
    }
    Ok(())
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

const fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

/// Percent-encode every byte outside the RFC 3986 unreserved set. Using a
/// single canonical form (uppercase hex, no `+` for space) is what makes
/// rendering deterministic.
fn percent_encode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for &byte in raw.as_bytes() {
        if unreserved(byte) {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0F) as usize] as char);
        }
    }
    out
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Strict percent-decoder. A stray `%`, a bad hex pair or invalid UTF-8 is an
/// error — never a lossy replacement character.
fn percent_decode(encoded: &str) -> Result<String, DeepLinkError> {
    let bytes = encoded.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(DeepLinkError::InvalidEncoding(encoded.to_string()));
            }
            let high = hex_value(bytes[index + 1]);
            let low = hex_value(bytes[index + 2]);
            match (high, low) {
                (Some(high), Some(low)) => out.push((high << 4) | low),
                _ => return Err(DeepLinkError::InvalidEncoding(encoded.to_string())),
            }
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).map_err(|_| DeepLinkError::InvalidEncoding(encoded.to_string()))
}
