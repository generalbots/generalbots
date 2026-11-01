# Security Policy

<p align="center"><img src="logo.svg" alt="General Bots" width="200"></p>

---

## Reporting a Vulnerability

Please, open an issue or contact us at security@pragmatismo.com.br.

## Dependency Advisories

### `zip` (GHSA-m665-v7pm-7grj, GHSA-q7v9-x93c-xqv3, GHSA-8hxf-qr8p-62x4)

First-party archive handling uses the workspace `zip` dependency, pinned to
**9.2.0** (patched; all three advisories are fixed in 9.0.5). That covers the
DOCX/XLSX text extraction, the DOCX/PPTX builders, `botsheet`, `botsources`,
`botcorepkg`, `botdrive` and the `archive` BASIC keyword.

Transitive copies of `zip` remain in the dependency graph because the crates
that embed them have not adopted `zip >= 9.0.5` yet:

| Crate | `zip` requirement | Latest upstream |
|-------|-------------------|-----------------|
| `docx-rs` 0.4.x | `^0.6.3` (0.4.22 moves to `^8.6`) | still below 9.0.5 |
| `umya-spreadsheet` 2.3.3 | `^2.2.1` (3.1.1 uses `^8.6`) | 4.0.0-beta.1 only |
| `ooxmlsdk` 0.3.0 | `^4.2.0` (0.14.0 uses `^9.0.5`) | upgrade tracked |
| `calamine` 0.26.1 | `^2.1` (0.37.0 uses `^9.0`) | upgrade tracked |
| `rust_xlsxwriter` 0.77.0 | `^2.1.3` (0.102 uses `^9.0`) | upgrade tracked |

Residual exposure of those copies is limited and accepted:

- The advisories require `ZipArchive::extract()` (writes attacker-chosen entry
  names to disk), symlink entries, or an AES-encrypted archive. First-party
  code never calls `extract`; it opens named entries in memory with
  `by_name`/`by_index` and reads them, never passes a password, and the only
  extraction loop that writes to disk (`botcorepkg` facade download) resolves
  every entry with `enclosed_name()` and rejects the rest.
- `docx-rs` and `umya-spreadsheet` are used to build the DOCX/XLSX documents
  the assistant generates (write path) and to parse files the user uploads
  (read path, in memory, size-capped).
- Dependabot alerts for these transitive copies are dismissed as
  **tolerable risk** with a link to this section; they are re-checked on every
  dependency review and will be closed by the feature-crate upgrades above.

### `accelerate` 1.1.1 (GHSA-4j2p-28q2-5m79)

`botmodels/requirements.txt` pins `accelerate==1.1.1`, which has no patched
release for the sharded-checkpoint `weight_map` path traversal / denial of
service advisory. Exposure is limited to operator-run local model tooling:
the service is not exposed to end users and only loads checkpoints the
operator places on disk. The pin is retained and re-checked on every
dependency review; replace it as soon as an upstream fix is published.

