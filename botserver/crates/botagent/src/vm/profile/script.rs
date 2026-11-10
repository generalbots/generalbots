//! `profile::script` — shell generation for one image profile.
//!
//! Split from `profile.rs` per the AGENTS.md file-size rule. The generated
//! block is appended to the bootstrap and workspace-preparation scripts; a
//! failing artifact aborts it (exit 78) after recording the reason in
//! `/opt/vibe/.profile-error`, so a half-provisioned VM is never mistaken for
//! a ready workspace.

use super::{
    Artifact, ArtifactChannel, VmProfile, ARTIFACT_CACHE_ENV, DEFAULT_ARTIFACT_CACHE,
    OFFICE_MAX_DELTA_BYTES, PROFILE_ERROR_FILE,
};

/// Shell that installs one profile, generated from [`Artifact`] data.
///
/// A failure is fatal: a half-installed profile answers document requests with
/// a missing engine, so the reason lands in [`PROFILE_ERROR_FILE`] and the
/// script exits 78 instead of pretending the workspace is ready.
pub(crate) fn install_block(profile: VmProfile) -> String {
    if profile == VmProfile::Base {
        return String::new();
    }
    let mut script = String::new();
    script.push_str(&format!(
        "\n# ── profile '{profile}' (#2062) ────────────────────────────────────────\n\
         # Baked into the prefabricated image by scripts/build_vm_image.sh\n\
         # --profile {profile}; a failure below is fatal, never a silent downgrade.\n\
         # Budget: <= {} MB over today's base image.\n\
         gb_profile_fail() {{\n  \
           printf 'vm-profile: artifact %s failed on profile {profile} (log: /tmp/agent-profile.log)\\n' \"$1\" >&2\n  \
           printf 'vm-profile: artifact %s failed on profile {profile}\\n' \"$1\" > {PROFILE_ERROR_FILE}\n  \
           exit 78\n}}\n",
        OFFICE_MAX_DELTA_BYTES / 1_000_000
    ));
    for artifact in profile.artifacts() {
        script.push_str(&artifact_step(artifact));
    }
    script.push_str(&html_to_pdf_wrapper());
    script
}

fn artifact_step(artifact: &Artifact) -> String {
    if artifact.channel == ArtifactChannel::Static {
        return static_step(artifact);
    }
    let install = match artifact.channel {
        ArtifactChannel::Apt => format!(
            "apt-get -o DPkg::Lock::Timeout=300 install -y -q --no-install-recommends {}",
            artifact.packages.join(" ")
        ),
        ArtifactChannel::Pip => format!(
            "python3 -m pip install --break-system-packages --quiet --disable-pip-version-check {}",
            artifact.packages.join(" ")
        ),
        ArtifactChannel::Static => String::new(),
    };
    format!(
        "\n# {id} ({channel:?}, {mb} MB)\n\
         if ! {probe} >/dev/null 2>&1; then\n  \
           {install} >>/tmp/agent-profile.log 2>&1 || gb_profile_fail {id}\n\
         fi\n",
        id = artifact.id,
        channel = artifact.channel,
        mb = artifact.installed_bytes / 1_000_000,
        probe = artifact.probe,
        install = install,
    )
}

/// Static builds come from the artifact cache (the prefabricated image ships
/// them); the distro package is the online fallback only.
fn static_step(artifact: &Artifact) -> String {
    let cache_file = artifact.cache_file.unwrap_or(artifact.id);
    let fallback = artifact.packages.join(" ");
    format!(
        "\n# {id} ({channel:?}, {mb} MB): static build from the cache, distro package only as\n\
         # the online fallback (it drags ~230 MB of dependencies).\n\
         if ! {probe} >/dev/null 2>&1; then\n  \
           if [ -f \"${{{ARTIFACT_CACHE_ENV}:-{DEFAULT_ARTIFACT_CACHE}}}/{cache_file}\" ]; then\n    \
             mkdir -p /opt/chromium && tar -xf \
             \"${{{ARTIFACT_CACHE_ENV}:-{DEFAULT_ARTIFACT_CACHE}}}/{cache_file}\" -C /opt/chromium \
             >>/tmp/agent-profile.log 2>&1 || gb_profile_fail {id}\n  \
           else\n    \
             apt-get -o DPkg::Lock::Timeout=300 install -y -q --no-install-recommends {fallback} \
             >>/tmp/agent-profile.log 2>&1 || gb_profile_fail {id}\n  \
           fi\n\
         fi\n",
        id = artifact.id,
        channel = artifact.channel,
        mb = artifact.installed_bytes / 1_000_000,
        probe = artifact.probe,
    )
}

/// `html2pdf <input.html> <output.pdf>` — the WeasyPrint replacement: one
/// Chromium invocation, no python HTML engine, no cold start.
fn html_to_pdf_wrapper() -> String {
    "\n# html2pdf — HTML->PDF through the embedded headless Chromium.\n\
     cat > /usr/local/bin/html2pdf <<'EOS'\n\
     #!/bin/sh\n\
     # usage: html2pdf input.html output.pdf\n\
     set -eu\n\
     chrome=${GB_CHROME_BIN:-}\n\
     if [ -z \"$chrome\" ]; then\n  \
       if [ -x /opt/chromium/chrome ]; then chrome=/opt/chromium/chrome; else chrome=$(command -v chromium) || chrome=; fi\n\
     fi\n\
     [ -n \"$chrome\" ] || { echo 'html2pdf: no chromium in this VM' >&2; exit 78; }\n\
     exec \"$chrome\" --headless=new --no-sandbox --disable-gpu --print-to-pdf=\"$2\" \"file://$(realpath \"$1\")\"\n\
     EOS\n\
     chmod +x /usr/local/bin/html2pdf\n"
        .to_string()
}

