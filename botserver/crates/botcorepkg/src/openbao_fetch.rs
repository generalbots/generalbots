//! Puts the OpenBao program into the `bin/vault` folder of a stack that still
//! has only the former HashiCorp Vault binary.
//!
//! Only the program is added: nothing under `conf/vault` or `data/vault` is
//! touched and no install step runs, so an upgraded stack keeps its storage
//! until [`crate::openbao_migration`] moves it. The archive comes from the
//! download cache (`BOTSERVER_INSTALLERS_PATH` first) or is downloaded, and
//! its pinned SHA-256 is checked in both cases.

use crate::cache::{CacheResult, DownloadCache};
use crate::secrets_engine::{Engine, StackFacts, BIN_DIR};
use anyhow::{anyhow, Context, Result};
use log::info;
use std::path::Path;

/// Component id whose `3rdparty.toml` entry pins the OpenBao archive.
const COMPONENT: &str = "vault";
/// Scratch folder, inside `bin/vault`, the archive is unpacked into.
const UNPACK_DIR: &str = ".openbao-unpack";

/// The stack runs the former binary and has no OpenBao program yet.
#[must_use]
pub fn needs_fetch(facts: &StackFacts) -> bool {
    facts.legacy_binary && !facts.openbao_binary
}

/// Adds `bin/vault/bao[.exe]` when [`needs_fetch`] holds. Returns whether the
/// program was placed.
pub async fn ensure_openbao_binary(stack: &Path) -> Result<bool> {
    if !needs_fetch(&StackFacts::read(stack)) {
        return Ok(false);
    }
    let url = crate::installer::get_component_url(COMPONENT)
        .ok_or_else(|| anyhow!("no OpenBao download is pinned for this platform"))?;
    let expected = crate::installer::checksum_for_url(&url)
        .filter(|digest| !digest.is_empty())
        .ok_or_else(|| anyhow!("the OpenBao download has no pinned SHA-256"))?;
    let cache = DownloadCache::new(stack.parent().unwrap_or(stack))
        .or_else(|_| DownloadCache::new(stack))
        .context("download cache")?;
    let archive = match cache.resolve_component_url(COMPONENT, &url) {
        CacheResult::Cached(path) => path,
        CacheResult::Download { url, cache_path } => {
            info!("Downloading OpenBao for the secrets server");
            crate::facade_download::download_with_reqwest(&url, &cache_path, COMPONENT).await?;
            cache_path
        }
    };
    crate::checksum::verify_with_policy(&archive, &expected, COMPONENT, true)?;
    place_program(&archive, stack)?;
    info!("OpenBao placed in {BIN_DIR}");
    Ok(true)
}

/// Unpacks `archive` into a scratch folder and moves only the program next
/// to the former binary.
fn place_program(archive: &Path, stack: &Path) -> Result<()> {
    let bin = stack.join(BIN_DIR);
    let unpack = bin.join(UNPACK_DIR);
    if unpack.exists() {
        std::fs::remove_dir_all(&unpack)?;
    }
    std::fs::create_dir_all(&unpack)?;
    let placed = crate::facade_download::handle_downloaded_file(
        archive,
        &unpack,
        Some(Engine::OpenBao.binary_name()),
        false,
    )
    .and_then(|()| {
        let name = Engine::OpenBao.file_name();
        let found = [unpack.join(&name), unpack.join("bin").join(&name)]
            .into_iter()
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| anyhow!("the OpenBao archive has no {name}"))?;
        std::fs::rename(found, Engine::OpenBao.path_in(stack))?;
        Ok(())
    });
    let cleaned = std::fs::remove_dir_all(&unpack);
    placed?;
    cleaned?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_stack_with_the_former_binary_alone_fetches() {
        let legacy_only = StackFacts { legacy_binary: true, ..StackFacts::default() };
        assert!(needs_fetch(&legacy_only));
        let both = StackFacts { openbao_binary: true, ..legacy_only };
        assert!(!needs_fetch(&both));
        assert!(!needs_fetch(&StackFacts::default()));
    }

    #[test]
    fn the_pinned_archive_has_a_digest() {
        let url = crate::installer::get_component_url(COMPONENT).unwrap_or_default();
        assert!(url.contains("openbao"));
        assert_eq!(crate::installer::checksum_for_url(&url).map(|d| d.len()), Some(64));
    }
}
