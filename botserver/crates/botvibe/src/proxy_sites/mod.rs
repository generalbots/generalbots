//! Split from `proxy_sites.rs` per #1443 (AGENTS.md 450-line rule).
//! #1288 — published vibe sites served from the proxy container, not from a
//! dedicated prod VM. Enterprise-grade operation:
//!
//! - **Concurrency**: a process-wide publish mutex serializes every site
//!   mutation (payload swap + Caddy config rewrite) so two simultaneous
//!   publishes can never interleave config pulls/pushes and corrupt the
//!   shared Caddyfile section. Blocking incus work runs via
//!   `spawn_blocking` — never on the async executor.
//! - **Input hardening**: slugs are validated against a strict pattern and a
//!   reserved-name list (no `proxy`, `caddy`, `grafana`, …, no config
//!   collision); payload size and file counts are capped before any bytes
//!   reach the proxy; python payloads must contain `app.py`, static ones an
//!   `index.html` (no publishing a site that can only 404).
//! - **Zero-downtime config**: the new Caddyfile is validated as a
//!   *candidate file* BEFORE the swap; the current config is backed up
//!   (rotated, 10 kept); a failed reload auto-restores the backup.
//! - **Verification**: after publish, the route is probed through Caddy
//!   itself (Host header + `--resolve`-style dial inside the proxy), not
//!   just the backend — a route that never serves is a failed publish.
//! - **Releases & rollback**: each payload swap keeps the previous release
//!   in `<site>.prev-N` (10 retained); `rollback_site` re-activates one and
//!   `unpublish_site` removes route + service + payload.
//! - **Two websites per project**: the working copy lives at
//!   `{slug}-test.{domain}` (`websites/{slug}-test`) and the public page at
//!   `{slug}.{domain}` (`websites/{slug}`). Every site publish defaults to the
//!   test twin; only the deploy pipeline promotes a release to the public
//!   slug, so a change under test can never blank the live site.
//!
//! Transport: the bot container drives the proxy via nested
//! `incus exec proxy -- …` / `incus file push|pull` (verified on prod).
//! Every invocation goes through the harness command guard (`incus` is on
//! the allowlist; `sh` is deliberately not — inner commands are passed as
//! direct argv, never through a shell).

mod caddy;
mod caddy_2;
mod caddy_3;
mod caddy_4;
mod release;
mod sites;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;
use crate::site_env::{SiteEnv, SiteTarget};

pub use caddy::{python_port, validate_slug};
pub(crate) use caddy::{PROXY_CADDY_CONFIG, check_python_runtime, check_serveability, dir_is_vibe_owned, must_run, proxy_exec, python_port_for, rotate_release, stage_payload, tls_directive, tls_internal_from_env};
#[cfg(test)]
pub(crate) use caddy::{PROXY_SITES_ROOT, site_block_with_mode};
pub(crate) use caddy_2::{remove_site_config, site_block_for_target, upsert_site_config};
pub use caddy_3::{deploy_site_to_proxy_env};
pub(crate) use caddy_3::{ensure_python_service_for, probe_python_service, verify_route_serving};
pub use caddy_4::{looks_like_python, promote_site_test_to_prod, rollback_site, rollback_site_test, unpublish_site, unpublish_site_test};
pub(crate) use release::{MARKER_FILE, RELEASE_RETENTION, lock_publish};
pub use sites::{site_slug};
pub(crate) use sites::{MAX_FILES, MAX_SINGLE_FILE_BYTES, SECTION_BEGIN, SECTION_END, extract_section, payload_bytes, site_unit_name};
