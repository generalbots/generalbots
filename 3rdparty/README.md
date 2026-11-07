# Vendored third-party Rust crates

These crates are **not** part of the workspace. They are path dependencies
declared in the root `Cargo.toml` and excluded via `[workspace] exclude`.

| Directory | Upstream | Version | Local change |
|-----------|----------|---------|--------------|
| `docx-rs/` | [bokuweb/docx-rs](https://github.com/bokuweb/docx-rs) | 0.4.22 (MIT) | `zip` requirement raised from `^8.6.0` to `^9.0.4`; `src/` is byte-for-byte upstream. |
| `umya-spreadsheet/` | [MathNya/umya-spreadsheet](https://github.com/MathNya/umya-spreadsheet) | 3.1.1 (MIT) | `zip` requirement raised from `^8.6.0` to `^9.0.4`, plus one `ZipFile::name()` call adapted to zip 9's `Result<Cow<str>, ZipError>` return in `src/reader/driver.rs`. |

## Why

The published releases of both crates pin `zip ^8.6.0`. That pulls a pre-9 zip
with the extraction advisories RUSTSEC-2026-0367/0368/0369/0429 and their GHSA
aliases. The first stable upstream release of `umya-spreadsheet` that accepts
zip 9 is `4.0.0-beta.1`, and `docx-rs` has no release after 0.4.22 yet. Vendoring
lets the workspace run a single patched zip without shipping beta or unreleased
code.

## Removal condition

Delete each directory and restore the registry requirement in
`[workspace.dependencies]` as soon as the upstream crate ships a **stable**
release that depends on `zip >= 9`. The exact condition is repeated in each
vendored `Cargo.toml`.

## Updating

1. Download the new crate tarball from crates.io (`static.crates.io`).
2. Copy only `Cargo.toml`, `src/`, `README.md` and `LICENSE` (drop examples,
   tests, benches and dev-dependencies).
3. Re-apply the `zip` requirement bump (and any compile fix zip 9 requires).
4. `cargo metadata` to refresh `Cargo.lock`, then
   `cargo tree -i zip --workspace --all-features` to prove only one zip version
   remains.
5. Update this table, `SECURITY.md`, `deny.toml` and `scripts/osv_allowlist.txt`.
