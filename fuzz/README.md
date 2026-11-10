# Fuzz targets (#2119)

libfuzzer coverage for the parsers that eat untrusted bytes. `fuzz/` is
deliberately **detached from the workspace** (`[workspace]` in
`fuzz/Cargo.toml`): it needs a nightly toolchain and never builds during a
normal `cargo test`.

## Targets

| Target | Surface | Invariant |
|--------|---------|-----------|
| `jwt_claims` | `JwtManager::validate_*`, claims JSON | an unsigned token never validates or decodes; claims JSON never panics |
| `webhook_body` | WhatsApp JSON / Twilio form bodies, `sha256=` header | parse garbage into `Err`/`false`; an empty app secret never validates; verification is deterministic |
| `basic_compile` | `goto_transform`, `syntax_transforms` | pure transforms: garbage in, same text out, twice |
| `office_import` | `import_spreadsheet_bytes` (xlsx/xls/ods/csv) | any bytes import or return `Err(String)` |

## Running

```bash
rustup toolchain install nightly
cargo install cargo-fuzz --locked
cargo fuzz list
cargo fuzz run jwt_claims -- -max_total_time=90 -rss_limit_mb=4096
```

`fuzz-nightly` in `.github/workflows/ci.yml` runs every target for 90 s on the
nightly schedule and uploads `fuzz/artifacts/<target>/` on a finding.

## Regressions

Two places, on purpose:

* `fuzz/corpus/<target>/` — the seed corpus libfuzzer replays first;
* `bottest/tests/fuzz_regressions.rs` — the **stable** fallback that runs on
  every PR with a deterministic mutation corpus (no nightly, no new dependency).
  Add the minimized bytes of a fixed crash to the matching `*_SEEDS` slice so the
  regression keeps running on the stable gate.

A crash that is not reproducible on stable is still a finding: keep the artifact
in the corpus and open the issue before closing the fuzz run.
