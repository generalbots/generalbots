# Input Guardrails: SSRF, DLP, Injection, Files 🟡 BETA

This page separates the guards that are **wired into a request path** from the
guards that exist only as library code in `botsecurity-core`. The distinction
matters: a module with no callers protects nothing.

Authority: `botserver/crates/botsecurity-core/src/` (`validation/`, `dlp/`,
`prompt_security/`, `file_validation.rs`, `antivirus/`),
`botserver/crates/botsecurity-crypto/src/cert_pinning/`, and the call sites in
`botserver/crates/botbasic_data/src/keywords/get/execute_get_from.rs`.

## Wired vs library-only

| Guard | Module | Wired? | Call site |
|---|---|---|---|
| SSRF host filter for outbound HTTP in BASIC | `execute_get_from.rs` (`is_safe_path`) | **Yes** | `GET FROM` / HTTP keyword path |
| SQL identifier and value sanitising | `botsecurity-core/src/sql_guard.rs` (re-exported by `botcore::shared`) | **Yes** | `botbasic_data`, `botdatabase` handlers |
| Command execution allowlist | `botsecurity-core/src/command_guard.rs` (`SafeCommand`) | **Yes** | `manager_ops`, console status panel |
| Path and filename sanitising | `botsecurity-core/src/path_guard/` | **Yes** | whiteboard export, app server |
| Error sanitising | `botsecurity-core/src/error_sanitizer/` | **Yes** | HTTP handlers such as `bottelegram`, `botdrive`, `botintegrations` |
| Security headers and CSRF | `security/headers`, `server/run_axum_server.rs` | **Yes** | global middleware layers |
| URL validation helper | `validation/validate_url_ssrf.rs` | **No** | no caller outside the crate |
| DLP scanning and redaction | `dlp/` | **No** | no caller outside the crate |
| Prompt-injection detection | `prompt_security/` | **No** | no caller outside the crate |
| File magic-byte validation | `file_validation.rs` | **No** | no caller outside the crate |
| Antivirus (ClamAV / Defender) | `antivirus/` | **No** | not referenced from `botserver/src` |
| Certificate pinning | `botsecurity-crypto/src/cert_pinning/` | **No** | `create_pinned_client` has no callers |

## SSRF

Two implementations exist with different reach.

### The wired guard

`is_safe_path` in `botbasic_data/src/keywords/get/execute_get_from.rs` runs
before an outbound `GET`:

```rust
if host_lower == "localhost"
    || host_lower.contains("169.254")
    || host_lower.starts_with("127.")
    || host_lower.starts_with("10.")
    || host_lower.starts_with("192.168.")
    || host_lower.starts_with("172.")
    || host_lower == "::1"
    || host_lower.contains("0x7f")
    || host_lower.contains("metadata.google.internal")
{
    return false; // Prevent obvious SSRF
}
```

What it catches: loopback IPv4, link-local (including the cloud metadata
address `169.254.169.254`), RFC 1918 IPv4, IPv6 loopback, hex-encoded loopback,
and the GCP metadata hostname.

What it misses:

- **IPv4-mapped IPv6.** `http://[::ffff:10.0.0.1]/` parses to host
  `::ffff:10.0.0.1`, which matches none of the prefixes, so it passes and
  `reqwest` connects to the private address.
- **Internal DNS names.** `http://internal.corp/` is not resolved before the
  check, so any name that resolves to a private address passes.
- **Other IPv6 forms.** ULA addresses (`fc00::/7`) other than the literal `::1`
  comparison are not inspected.
- **Over-blocking.** The `172.` prefix rejects the whole `172.0.0.0/8` block,
  not just `172.16.0.0/12`; `contains("169.254")` also matches a public hostname
  containing that digit string.

### The library validator

`validate_url_ssrf` first runs `validate_url`, whose regex requires a dotted
hostname with no port and no brackets. Consequences, verified against the
regex:

| Input | Result |
|---|---|
| `https://example.com/a` | accepted |
| `http://127.0.0.1/` | rejected by the host blocklist |
| `http://[::ffff:10.0.0.1]/` | rejected earlier — brackets fail the URL regex |
| `https://example.com:8443/` | rejected earlier — ports fail the URL regex |
| `http://internal.corp/` | accepted (no DNS resolution) |

So the IPv4-mapped gap is **not** reachable through this validator (it rejects
all IPv6 literals), but the validator also rejects every URL with an explicit
port, which makes it unsuitable as a drop-in outbound check. It is currently
unused.

## DLP

`dlp/` defines `DlpConfig`, `DlpPolicy`, `SensitiveDataType` (email, phone,
credit card, SSN, IP, API key, password, AWS access/secret key, private key, JWT,
bank account, passport, driver's licence, health id, custom), four severity
levels and five actions (`Allow`, `Warn`, `Redact`, `Block`, `Quarantine`).

Defaults in `dlp/dlp_config.rs`:

| Field | Default |
|---|---|
| `enabled` | `true` |
| `scan_inbound` / `scan_outbound` | `true` / `true` |
| `block_on_violation` | **`false`** |
| `log_violations` | `true` |
| `redact_sensitive_data` | `true` |
| `max_content_length` | 10 MiB |

These defaults describe a scanner that reports and redacts but does not block.
Because nothing calls the scanner, the effective behaviour on a live request is
"no DLP". Any page or policy that claims prompt traffic is scanned for card
numbers or keys is describing the library, not the running system.

## Prompt injection

`prompt_security/` builds 25 regex patterns grouped by `InjectionType`:
instruction override ("ignore all previous instructions"), role manipulation
("you are now a…"), system-prompt leak, delimiter injection (`<|im_start|>`,
fenced `system` blocks), encoding bypass (`\xNN`, `\uNNNN`, HTML entities,
`base64:`), jailbreak (`DAN mode`, `developer mode`), data exfiltration and
recursive injection (`{{…}}`, `${…}`).

`PromptSecurityConfig` adds length ceilings (100,000 characters in, 500,000
out), `block_on_injection`, `sanitize_inputs`, `validate_outputs`,
`jailbreak_detection` and `content_filtering`, plus an output validator that
looks for leaked system prompts and sensitive data.

Like DLP, `PromptSecurityManager` has no caller in `botserver/src`. User text
reaches the model unfiltered. Regex detection of this kind is also weak by
construction: paraphrase, translation and base64 with a different prefix all
evade it, so even when wired it should be treated as a signal, not a boundary.

## File uploads

`file_validation.rs` holds a magic-byte table for images (JPEG, PNG, GIF, BMP,
TIFF), documents (PDF, ZIP, RAR, gzip, bzip2, xz, 7z), video (MP4, WebM, ASF),
audio (MP3, Ogg, FLAC, M4A, WAV, AIFF) and a config with:

| Setting | Default |
|---|---|
| `max_size` | 100 MiB |
| `allowed_types` | jpeg, png, gif, pdf, text/plain, zip |
| `block_executables` | `true` |
| `check_magic_bytes` | `true` |
| `defang_pdf` | `true` |

The upload handlers in `botserver/crates/botdrive/src/` do not call this module,
so a file's declared content type is not cross-checked against its bytes on the
way into Drive. Treat `allowed_types` above as an intended policy, not an
enforced one.

## Certificate pinning

`cert_pinning` supports leaf, intermediate and root pins with a per-host map, a
validation cache (`cache_ttl_secs`, default 3600), backup pins, a
`require_pins` switch and a `report_only` mode that logs instead of failing.
`create_pinned_client(hostname)` builds a `reqwest::Client` that validates
against the pin set.

The configuration defaults to `enabled: true` with an **empty pin map** and
`require_pins: false`, which is equivalent to "no pins configured". No caller
outside the crate builds a pinned client, so outbound TLS uses the system trust
store.

## Practical reading

| Question | Answer today |
|---|---|
| Is outbound HTTP from BASIC filtered against private networks? | Yes, by a prefix blocklist with the gaps listed above |
| Is inbound prompt text scanned for injection? | No |
| Is outbound model output scanned for secrets? | No |
| Are uploaded bytes checked against their extension? | No |
| Is TLS pinned for outbound calls? | No |
| Are SQL identifiers and shell commands constrained? | Yes, via `sql_guard` and `SafeCommand` |

## Related pages

- [Browser Policy and Budgets](./browser-policy.md) — the browsing control plane.
- [Security Features](./security-features.md) — the enforced surface.
- [System Limits & Rate Limiting](./system-limits.md) — request limits.
- [Security Protection Tools](./protection-tools.md) — the ClamAV/NocoDB
  protection endpoints, which are separate from the unused `antivirus` module.
