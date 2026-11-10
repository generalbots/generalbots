//! #2062 — VM image profiles as data.
//!
//! The workspace VM is described by a profile instead of a pile of ad-hoc
//! `apt-get`/`pip` lines:
//!
//! | Profile | Contents | Delta over today's base |
//! |---------|----------|-------------------------|
//! | `base` | Debian 13 slim + python3 + node + git | — |
//! | `office` (default) | base + embedded headless Chromium + fonts + document/PDF libraries | ≤ +200 MB |
//! | `media` (opt-in) | office + ffmpeg | +250 MB |
//! | `data` (opt-in) | office + numpy/pandas/matplotlib/scipy | +180 MB |
//!
//! `office` fits the issue's budget because documents are generated with
//! libraries (`python-docx`, `openpyxl`, `python-pptx`, `reportlab`) and the
//! only browser is the static headless Chromium build — a single binary with
//! no X server and no LibreOffice cold start. HTML becomes PDF through
//! `chromium --headless=new --print-to-pdf` (the `/usr/local/bin/html2pdf`
//! wrapper), which replaces WeasyPrint and its ~55 MB Python stack.
//!
//! Audio/video/ML work runs in BotModels, so no profile bakes torch,
//! tensorflow, transformers or a CUDA build of ffmpeg. A bot that produces
//! media asks BotModels; the `media` profile exists only for offline hosts.
//!
//! The pre-fabricated image is built with `scripts/build_vm_image.sh
//! --profile office`, which bakes every artifact below into the Incus image so
//! a fresh container needs no package download. On a host that has neither
//! that image nor network access [`ensure_available`] refuses to provision a
//! degraded VM instead of silently falling back to a smaller profile.

use std::fmt;

/// Issue #2062: the office delta over today's base must stay within +200 MB.
pub(super) const OFFICE_MAX_DELTA_BYTES: u64 = 200_000_000;
/// Issue #2062: the opt-in media profile adds ffmpeg (+250 MB).
pub(super) const MEDIA_EXTRA_BYTES: u64 = 250_000_000;
/// Issue #2062: the opt-in data profile adds scientific python (+180 MB).
pub(super) const DATA_EXTRA_BYTES: u64 = 180_000_000;

/// Selects the profile explicitly (`base`, `office`, `media`, `data`).
pub(super) const PROFILE_ENV: &str = "GB_VM_PROFILE";
/// `1`/`true`/`yes` when the host cannot reach a package mirror.
pub(super) const OFFLINE_ENV: &str = "GB_VM_OFFLINE";
/// Directory the prefabricated image carries the static artifacts in.
pub(super) const ARTIFACT_CACHE_ENV: &str = "GB_VM_ARTIFACT_CACHE";
pub(super) const DEFAULT_ARTIFACT_CACHE: &str = "/opt/vibe/artifacts";
/// Why a profile could not be honoured; read by the chat/API surface.
pub(super) const PROFILE_ERROR_FILE: &str = "/opt/vibe/.profile-error";

/// Today's base image: what the host already had before #2062.
pub(super) const BASE_IMAGE: &str = "images:debian/13";
/// Image alias produced by `scripts/build_vm_image.sh`.
pub(super) const DEFAULT_PREFAB_IMAGE: &str = "gb-workspace-office";

/// Workload kind that gets the document image: bots answer "gere um .docx e
/// um PDF" on a cold workspace, so `office` is their default.
pub(super) const KIND_BOT: &str = "bot";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VmProfile {
    Base,
    Office,
    Media,
    Data,
}

impl VmProfile {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Office => "office",
            Self::Media => "media",
            Self::Data => "data",
        }
    }

    /// Case-insensitive parse of a requested profile name.
    pub(super) fn parse(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "base" => Some(Self::Base),
            "office" => Some(Self::Office),
            "media" => Some(Self::Media),
            "data" => Some(Self::Data),
            _ => None,
        }
    }

    /// Every artifact the profile installs, in installation order. `media` and
    /// `data` are supersets of `office`: a bot that transcodes still writes
    /// documents.
    pub(super) fn artifacts(self) -> Vec<&'static Artifact> {
        match self {
            Self::Base => Vec::new(),
            Self::Office => vec![
                &CHROMIUM_HEADLESS,
                &FONTS,
                &DOC_LIBS,
                &PDF_LIBS,
                &HTML_LIBS,
            ],
            Self::Media => {
                let mut all = Self::Office.artifacts();
                all.push(&FFMPEG);
                all
            }
            Self::Data => {
                let mut all = Self::Office.artifacts();
                all.push(&SCIENTIFIC_PYTHON);
                all
            }
        }
    }

    /// Installed bytes the profile adds over today's base image.
    pub(super) fn extra_bytes(self) -> u64 {
        self.artifacts().iter().map(|a| a.installed_bytes).sum()
    }

    /// Marker written to `/opt/vibe/.provisioned`; a profile change re-runs the
    /// idempotent preparation exactly once.
    pub(super) fn prepare_version(self) -> String {
        format!("v7-{}", self.as_str())
    }
}

impl fmt::Display for VmProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How an artifact is installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ArtifactChannel {
    /// `apt-get install --no-install-recommends`.
    Apt,
    /// `python3 -m pip install --break-system-packages`.
    Pip,
    /// Static build unpacked from the artifact cache (no download at runtime).
    Static,
}

/// One installable unit of a profile. `installed_bytes` is the number the
/// issue's budget is accounted against, so a change here must move the
/// `budget` tests too.
#[derive(Debug, Clone, Copy)]
pub(super) struct Artifact {
    pub(super) id: &'static str,
    pub(super) channel: ArtifactChannel,
    pub(super) installed_bytes: u64,
    /// apt package names, pip requirement specs, or the online fallback
    /// package of a static build.
    pub(super) packages: &'static [&'static str],
    /// Shell probe: exit 0 means the artifact is already installed.
    pub(super) probe: &'static str,
    /// File name expected in [`ARTIFACT_CACHE_ENV`] for static builds.
    pub(super) cache_file: Option<&'static str>,
}

/// The static Chromium build: ~120 MB, single binary, no X, no LibreOffice.
/// Debian's `chromium` package is only the online fallback because it drags
/// ~230-260 MB of dependencies with it.
pub(super) const CHROMIUM_HEADLESS: Artifact = Artifact {
    id: "chromium-headless",
    channel: ArtifactChannel::Static,
    installed_bytes: 120_000_000,
    packages: &["chromium"],
    probe: "test -x /opt/chromium/chrome",
    cache_file: Some("chromium-headless.tar.zst"),
};

/// Document fonts: without them every generated PDF is full of tofu boxes.
pub(super) const FONTS: Artifact = Artifact {
    id: "fonts",
    channel: ArtifactChannel::Apt,
    installed_bytes: 25_000_000,
    packages: &["fonts-noto-core", "fonts-dejavu-core"],
    probe: "test -d /usr/share/fonts/truetype/dejavu",
    cache_file: None,
};

/// `.docx`/`.xlsx`/`.pptx` generation, library-first (no LibreOffice).
pub(super) const DOC_LIBS: Artifact = Artifact {
    id: "doc-libs",
    channel: ArtifactChannel::Pip,
    installed_bytes: 12_000_000,
    packages: &["python-docx", "openpyxl", "python-pptx", "lxml"],
    probe: "python3 -c 'import docx, openpyxl, pptx, lxml'",
    cache_file: None,
};

/// PDF writing and post-processing.
pub(super) const PDF_LIBS: Artifact = Artifact {
    id: "pdf-libs",
    channel: ArtifactChannel::Pip,
    installed_bytes: 15_000_000,
    packages: &["reportlab", "pikepdf", "pypdf"],
    probe: "python3 -c 'import reportlab, pikepdf, pypdf'",
    cache_file: None,
};

/// Markdown/Jinja templating feeding the Chromium print-to-PDF step. This is
/// what used to be WeasyPrint (55 MB); the browser already does it better.
pub(super) const HTML_LIBS: Artifact = Artifact {
    id: "html-libs",
    channel: ArtifactChannel::Pip,
    installed_bytes: 3_000_000,
    packages: &["markdown", "jinja2"],
    probe: "python3 -c 'import markdown, jinja2'",
    cache_file: None,
};

/// Opt-in `media`: local transcode only, for offline hosts. Generation, music,
/// video and AI stay in BotModels.
pub(super) const FFMPEG: Artifact = Artifact {
    id: "ffmpeg",
    channel: ArtifactChannel::Apt,
    installed_bytes: MEDIA_EXTRA_BYTES,
    packages: &["ffmpeg"],
    probe: "command -v ffmpeg",
    cache_file: None,
};

/// Opt-in `data`: scientific python for notebooks and joins. Kept out of the
/// default image because it is the largest python addition.
pub(super) const SCIENTIFIC_PYTHON: Artifact = Artifact {
    id: "scientific-python",
    channel: ArtifactChannel::Pip,
    installed_bytes: DATA_EXTRA_BYTES,
    packages: &["numpy", "pandas", "matplotlib", "scipy"],
    probe: "python3 -c 'import numpy, pandas, matplotlib, scipy'",
    cache_file: None,
};

/// Pure selector: an explicit request wins, otherwise the kind decides.
///
/// `office` is the default for bots; app/website VMs serve HTTP and keep
/// today's base image. `media`/`data` are never a default — they are only used
/// when the operator asks for them through `GB_VM_PROFILE`, and an unknown
/// value falls back to the kind default (office for bots) instead of leaving a
/// workspace without document support.
pub(super) fn profile_for_bot(kind: &str, env_override: Option<&str>) -> VmProfile {
    if let Some(requested) = env_override.and_then(VmProfile::parse) {
        return requested;
    }
    match kind.trim().to_ascii_lowercase().as_str() {
        KIND_BOT => VmProfile::Office,
        _ => VmProfile::Base,
    }
}

/// The profile this process provisions with, from `GB_VM_PROFILE`.
pub(super) fn active_profile() -> VmProfile {
    let requested = std::env::var(PROFILE_ENV).ok();
    let profile = profile_for_bot(KIND_BOT, requested.as_deref());
    if let Some(value) = requested.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        if VmProfile::parse(value).is_none() {
            tracing::warn!(
                "unknown {PROFILE_ENV}='{value}'; provisioning profile '{}'",
                profile.as_str()
            );
        }
    }
    profile
}

/// Which image to launch and whether it is the prefabricated one.
///
/// An explicit `AGENT_VM_IMAGE` wins (a host that pins an image keeps it),
/// then the prefabricated workspace image when the host has that alias, then
/// today's base image.
pub(super) fn resolve_image(
    requested: Option<&str>,
    prefab: &str,
    available: &[String],
) -> (String, bool) {
    if let Some(image) = requested.map(str::trim).filter(|v| !v.is_empty()) {
        return (image.to_string(), image == prefab);
    }
    if available.iter().any(|alias| alias.trim() == prefab) {
        return (prefab.to_string(), true);
    }
    (BASE_IMAGE.to_string(), false)
}

/// What the host can offer a profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Availability {
    /// The launch image already carries every artifact of the profile.
    pub(super) prefab_image: bool,
    /// No package mirror reachable: nothing can be installed at first boot.
    pub(super) offline: bool,
}

impl Availability {
    pub(super) fn from_env(prefab_image: bool) -> Self {
        Self {
            prefab_image,
            offline: env_flag(OFFLINE_ENV),
        }
    }
}

fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref().map(str::trim),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

/// A profile the host cannot honour. Carries the missing artifacts so the
/// message tells the operator exactly what to bake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CapabilityError {
    pub(super) profile: VmProfile,
    pub(super) artifacts: Vec<&'static str>,
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "vm-profile: the '{}' profile needs {} baked into the image, but this host is \
             offline ({} is set) and has no '{}' image. Build it with \
             `scripts/build_vm_image.sh --profile {}` or launch the prefabricated image. \
             Refusing to fall back to a smaller profile: document/PDF generation would fail \
             at run time. Media and ML jobs run in BotModels, not in the VM.",
            self.profile,
            self.artifacts.join(", "),
            OFFLINE_ENV,
            DEFAULT_PREFAB_IMAGE,
            self.profile,
        )
    }
}

impl std::error::Error for CapabilityError {}

/// Refuses a profile the host cannot deliver. A prefabricated image has every
/// artifact baked in and an online host can install the apt/pip ones, so only
/// an offline host without that image fails — never silently degrading.
pub(super) fn ensure_available(
    profile: VmProfile,
    availability: &Availability,
) -> Result<(), CapabilityError> {
    if profile == VmProfile::Base || availability.prefab_image || !availability.offline {
        return Ok(());
    }
    Err(CapabilityError {
        profile,
        artifacts: profile.artifacts().iter().map(|a| a.id).collect(),
    })
}

mod script;
#[cfg(test)]
mod tests;

pub(super) use script::install_block;
