//! #2044 — office fidelity contract for DOCX, PPTX and XLSX.
//!
//! Every office format the suite edits follows the same four rules:
//!
//! 1. **Originals are durable.** An import persists the source package beside
//!    the model, so a restart cannot degrade fidelity.
//! 2. **Save patches.** Edits are written into the original package; only the
//!    affected OOXML parts change and every untouched part stays byte-identical.
//! 3. **Export reuses the save path**, so a download cannot lose what a save
//!    preserves.
//! 4. **No silent fallback.** A missing or unpatchable package produces the
//!    explicit `regenerated` signal (or an error that leaves the original
//!    untouched) — never a quiet rebuild.
//!
//! The two attached fixtures (`Rice_and_Beans_Nutrition.pptx`,
//! `Alex_Rivera_Resume_Futuristic.docx`) are the round-trip material. The XLSX
//! case uses a synthetic package carrying parts the model cannot re-emit
//! (chart, drawing, media, comments, theme, styles) so the per-part assertion
//! is deterministic.
//!
//! This target runs with no database, no Drive, no MinIO and no LLM. It drives
//! the public APIs the per-format fixes added: `botslides::pptx::preserve_*`,
//! `botdocs::storage::{persist_original_docx, docx_bytes_with_fidelity}` and
//! `botsheet::storage::{plan_for, produce_xlsx}`.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[path = "office_fidelity/docx.rs"]
mod docx;
#[path = "office_fidelity/memory_drive.rs"]
mod memory_drive;
#[path = "office_fidelity/pptx.rs"]
mod pptx;
#[path = "office_fidelity/xlsx.rs"]
mod xlsx;

pub const PPTX_FIXTURE: &str = "Rice_and_Beans_Nutrition.pptx";
pub const DOCX_FIXTURE: &str = "Alex_Rivera_Resume_Futuristic.docx";

/// The attached regression fixture, when the checkout still carries it.
pub fn fixture(name: &str) -> Option<Vec<u8>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("office-fidelity")
        .join(name);
    std::fs::read(path).ok()
}

/// Zip entries as (part name, uncompressed content), sorted by name so the
/// comparison ignores entry order (a rewrite may reorder the central directory).
pub fn parts(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).expect("zip package");
    let mut out = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("zip entry");
        let name = entry
            .name()
            .map(|name| name.to_string())
            .unwrap_or_default();
        let mut content = Vec::new();
        entry.read_to_end(&mut content).expect("read entry");
        out.push((name, content));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

pub fn part(bytes: &[u8], name: &str) -> String {
    parts(bytes)
        .into_iter()
        .find(|(part_name, _)| part_name == name)
        .map(|(_, content)| String::from_utf8_lossy(&content).to_string())
        .unwrap_or_else(|| panic!("missing part {name}"))
}

/// FNV-1a over the part content — the per-part hash the contract compares.
fn part_hash(content: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in content {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

pub fn part_hashes(bytes: &[u8]) -> BTreeMap<String, String> {
    parts(bytes)
        .into_iter()
        .map(|(name, content)| (name, part_hash(&content)))
        .collect()
}

/// Part names whose content differs between two packages, sorted.
pub fn changed_parts(before: &[u8], after: &[u8]) -> Vec<String> {
    let before = part_hashes(before);
    let after = part_hashes(after);
    let mut changed: Vec<String> = before
        .iter()
        .filter(|(name, hash)| after.get(*name) != Some(*hash))
        .map(|(name, _)| name.clone())
        .collect();
    changed.extend(
        after
            .keys()
            .filter(|name| !before.contains_key(*name))
            .cloned(),
    );
    changed.sort();
    changed
}
