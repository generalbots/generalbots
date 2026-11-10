# Unit test coverage by crate

Deterministic inventory of the offline unit tests in the Rust workspace: one row per crate, the number of `#[test]` / `#[tokio::test]` functions it owns, and the modules those tests live in. It is generated from the tree — never edited by hand:

```bash
python3 scripts/unit_coverage_report.py
```

The count is a *function* count, not a line-coverage percentage. A crate with many tests can still miss a branch; the per-module line and branch numbers come from the `cargo-llvm-cov` job (#2074). What this page guarantees is that no crate is silently absent from the unit gate.

## Summary

- Crates in the workspace: **122**
- `.rs` files scanned: **3953**
- Test functions: **6436**
- Crates with no tests: **12**
- Crates with 1–4 tests: **6**

Run one crate at a time (`cargo test -p <crate> --lib`); the workspace graph does not fit the 4.5 GiB dev container in one go (AGENTS.md).

## Coverage by crate

| Crate | Tests | What the tests cover |
|-------|------:|----------------------|
| `botsecurity-core` | 440 | `adversarial`, `cors`, `cors_policy`, `jwt_secret`, +6 more |
| `bottest` | 399 | `accessibility (integration)`, `anonymous_allowlist (integration)`, `api (integration)`, `auth_flow (integration)`, +47 more |
| `botvibe` | 327 | `domains_caa (integration)`, `git_engine_flow (integration)`, `harness_cmd`, `knowledge_graph (integration)`, +5 more |
| `botbasic_core` | 243 | `resolve_from_scope_row` |
| `botsecurity-auth` | 207 | `hardening`, `secret`, `totp` |
| `botbasic_compiler` | 183 | `blocks`, `convert_if_then`, `convert_multiword`, `convert_while_wend_select`, +6 more |
| `botlib` | 172 | `api`, `attachment`, `bundle_plural`, `command_guard`, +4 more |
| `botllm` | 164 | `extract_base_url`, `from_config`, `hash_embedding`, `model_file_name`, +4 more |
| `botcore` | 151 | `jwt_verifier` |
| `botcompliance` | 146 | `access_review`, `categorize`, `checker`, `export`, +3 more |
| `botdrive` | 139 | `classify`, `gdrive`, `keys`, `storage_backends`, +3 more |
| `botsecurity-crypto` | 131 | `ca`, `encryption`, `mutual_tls`, `pinning`, +2 more |
| `botsheet-core` | 128 | `arrays`, `formats` |
| `botintegrations` | 122 | `auth`, `payload`, `vault_path (integration)` |
| `botsheet` | 114 | `csv`, `find_replace`, `html`, `isolation`, +9 more |
| `botdocs` | 111 | `auth`, `extract_user_from_token`, `legacy`, `review`, +5 more |
| `botbasic_data` | 102 | `csv`, `filter`, `table_definition`, `verified_caller`, +1 more |
| `botbilling` | 99 | `format`, `lib`, `metric`, `plans`, +1 more |
| `botapp` | 97 | inline `#[cfg(test)]` modules |
| `botdomains` | 95 | `cloudflare`, `desec`, `dynadot`, `namecheap`, +1 more |
| `botevents` | 91 | `agui`, `envelope`, `events`, `pipeline`, +5 more |
| `botautotask` | 88 | `compile_prompt`, `extract_json_object`, `log_level`, `manifest`, +4 more |
| `botcoreoauth` | 82 | `cookies`, `oauth_identity`, `pkce`, `providers`, +2 more |
| `botsettings` | 80 | `catalog`, `html_escape`, `jobs`, `permission_inheritance`, +2 more |
| `botwhatsapp` | 78 | `utils`, `webhooks` |
| `botbasic_system` | 77 | `coerce_value`, `cosine_similarity`, `create_task`, `error_type`, +3 more |
| `botslides` | 77 | `create`, `export`, `owner_isolation`, `pptx_preservation (integration)`, +2 more |
| `botcloud` | 76 | `api`, `compute_provisioning`, `domain_provisioning`, `helpers`, +3 more |
| `botanalytics` | 75 | `analytics_types`, `goals_types`, `handlers_charts`, `handlers_ops`, +7 more |
| `botbasic_comms` | 75 | `delegate_to_bot`, `options`, `recurrence`, `sms_priority`, +4 more |
| `botbanking` | 73 | `cashflow_rules`, `dashboard`, `handlers`, `intent`, +2 more |
| `botchannels` | 72 | `channel_type`, `handoff`, `result`, `send_payload`, +2 more |
| `botmemory` | 70 | `api`, `extract`, `import`, `models`, +4 more |
| `botsecurity-protection` | 67 | inline `#[cfg(test)]` modules |
| `botmonitoring` | 62 | `collector`, `policy`, `serializers` |
| `botproviders` | 62 | `digitalocean`, `hetzner`, `oracle`, `ovh`, +1 more |
| `botemail` | 59 | `email_triage`, `models`, `oauth`, `tracking` |
| `bottelegram` | 58 | `payload`, `types`, `webhook`, `webhook_gate` |
| `botcontacts` | 57 | `contacts_csv`, `csv_parse`, `forecast`, `lists`, +1 more |
| `botbrowserpolicy` | 56 | `api`, `budget_adversarial`, `jwt`, `policy_adversarial` |
| `botworkspaces` | 56 | `block_ops`, `domain`, `page_ops`, `pages`, +1 more |
| `botvideo` | 51 | `analytics`, `chat_ai`, `effects`, `media`, +2 more |
| `botpeople` | 47 | `scope`, `time_clock_schedule` |
| `botretail` | 46 | `catalog`, `pos`, `pricing`, `promotion`, +1 more |
| `boteval` | 45 | inline `#[cfg(test)]` modules |
| `botmarketing` | 45 | `analytics`, `click_sign`, `email_tracking_link`, `ip_router`, +1 more |
| `botagent` | 43 | inline `#[cfg(test)]` modules |
| `botconnectors` | 41 | `knowledge` |
| `botinstagram` | 41 | `adapter`, `auth`, `campaign`, `types` |
| `botmsteams` | 41 | `adapter`, `fixtures`, `jwt`, `types` |
| `botbasic_ai` | 39 | `from_config`, `multimodal`, `workflow_dag` |
| `botplan` | 39 | `auth`, `collaboration`, `default_status`, `handler`, +1 more |
| `botbrazil` | 36 | inline `#[cfg(test)]` modules |
| `botsources` | 33 | `bundled_skills (integration)` |
| `botdesktop` | 30 | inline `#[cfg(test)]` modules |
| `botfraud` | 30 | `actions`, `rules_condition`, `scoring_aggregate`, `types` |
| `botdashboards` | 29 | `dashboard`, `data_sources`, `storage` |
| `botweba` | 29 | `base_url (integration)`, `bot_management_page (integration)`, `bot_name (integration)`, `click (integration)`, +8 more |
| `botautomation` | 28 | inline `#[cfg(test)]` modules |
| `botschema` | 27 | `schema_invariants`, `struct_contract` |
| `botattendant` | 26 | `attachments`, `lifecycle` |
| `botpaper` | 26 | `owner_isolation`, `utils` |
| `botproject` | 26 | `export`, `import_parsers` |
| `bottickets` | 25 | `get_bot_context`, `handle_ai_suggest`, `html_escape`, `scope`, +1 more |
| `botconsent` | 24 | inline `#[cfg(test)]` modules |
| `botproducts` | 24 | `pricing`, `write_scope` |
| `botcalendar` | 23 | `convert` |
| `botlearn` | 23 | `certification`, `gamification`, `xp` |
| `botsocial` | 23 | `comment`, `handle_add_comment`, `handle_trending_topics` |
| `botcanvas` | 22 | `convert`, `export`, `models` |
| `botlegal` | 22 | `cookie_info`, `create_deletion_request` |
| `botmarketplace` | 22 | inline `#[cfg(test)]` modules |
| `bottimeseries` | 22 | `config`, `line_protocol`, `message` |
| `botbasic_types` | 19 | `runtime`, `types_contract` |
| `botcorepkg` | 18 | inline `#[cfg(test)]` modules |
| `botcorebot` | 17 | `error`, `trigger_kind`, `utils` |
| `botkyc` | 17 | `intent`, `lifecycle` |
| `botapi` | 16 | `system` |
| `botui` | 15 | `security_headers` |
| `botplayer` | 14 | `media`, `validation` |
| `botuifragments` | 14 | `shells` |
| `botcoredirectory` | 13 | inline `#[cfg(test)]` modules |
| `botminutes` | 13 | `forms`, `parse_dt` |
| `botsampledata` | 13 | `drive`, `sample` |
| `botinventory` | 11 | `intent` |
| `botpos` | 11 | `handlers`, `parse_decimal` |
| `botdeployment` | 10 | inline `#[cfg(test)]` modules |
| `botresearch` | 10 | inline `#[cfg(test)]` modules |
| `bottimeclock` | 10 | inline `#[cfg(test)]` modules |
| `botattendance` | 9 | inline `#[cfg(test)]` modules |
| `botbiometry` | 9 | inline `#[cfg(test)]` modules |
| `botclock` | 9 | inline `#[cfg(test)]` modules |
| `botmaintenance` | 9 | inline `#[cfg(test)]` modules |
| `botmodelsbridge` | 9 | inline `#[cfg(test)]` modules |
| `botm365` | 8 | inline `#[cfg(test)]` modules |
| `botmeet` | 8 | inline `#[cfg(test)]` modules |
| `botdesigner` | 7 | inline `#[cfg(test)]` modules |
| `botgit` | 7 | `resolve_repo` |
| `botlog` | 6 | inline `#[cfg(test)]` modules |
| `botsecurity` | 6 | inline `#[cfg(test)]` modules |
| `bottax` | 6 | inline `#[cfg(test)]` modules |
| `botcoresecrets` | 5 | inline `#[cfg(test)]` modules |
| `botgl` | 5 | `reports` |
| `botsearch` | 5 | inline `#[cfg(test)]` modules |
| `botdatabase` | 4 | inline `#[cfg(test)]` modules |
| `botqdrant` | 4 | inline `#[cfg(test)]` modules |
| `boterp` | 3 | inline `#[cfg(test)]` modules |
| `bottemplates` | 3 | inline `#[cfg(test)]` modules |
| `botvision` | 3 | inline `#[cfg(test)]` modules |
| `botbrowser` | 1 | inline `#[cfg(test)]` modules |
| `botchannelbindings` | 0 | — |
| `botchannels-core` | 0 | — |
| `botcoresession` | 0 | — |
| `boteditor` | 0 | — |
| `bothandoff` | 0 | — |
| `bothr` | 0 | — |
| `botitsm` | 0 | — |
| `botmultimodal` | 0 | — |
| `botnvidia` | 0 | — |
| `botpublic` | 0 | — |
| `botsales` | 0 | — |
| `bottasks` | 0 | — |

## Crates without unit tests

Every crate below has zero test functions. The reason states whether it is thin glue (Tier C: covered by the integration suite and listed in `coverage-exclusions.toml`) or a **gap** — a crate that does hold pure logic and still needs a test.

| Crate | Reason |
|-------|--------|
| `botchannelbindings` | Diesel models plus three routes; the only pure function is the `ChannelBinding::empty()` constructor. |
| `botchannels-core` | Provider metadata and token-expiry helpers that read the system clock; no injected clock, so the expiry branches need a small refactor first. |
| `botcoresession` | Session store over Diesel and Valkey; the one pure helper (`is_placeholder_title`) is a four-line predicate. |
| `boteditor` | File CRUD plus a local `html_escape` that duplicates `botuifragments::htmx_escape` (covered there). |
| `bothandoff` | Thin CRUD: handlers/db/routes/storage over the `handoff_*` tables. |
| `bothr` | Thin CRUD: handlers/db/routes/storage over the `hr_*` tables. |
| `botitsm` | Thin CRUD: handlers/db/routes/storage over the `itsm_*` tables. |
| `botmultimodal` | Configuration readers over the `ConfigProvider` trait; every path is I/O. |
| `botnvidia` | Shells out to `nvidia-smi`; no pure logic. |
| `botpublic` | Twelve lines of route registration. |
| `botsales` | Thin CRUD: leads and quotes over the `sales_*` tables. |
| `bottasks` | **Gap:** the scheduler queries `system_automations`; `should_run_now` reads the clock directly, so it needs an injected `now` before it can be unit-tested. |

## Crates with fewer than five tests

| Crate | Tests | What the tests cover |
|-------|------:|----------------------|
| `botdatabase` | 4 | inline `#[cfg(test)]` modules |
| `botqdrant` | 4 | inline `#[cfg(test)]` modules |
| `boterp` | 3 | inline `#[cfg(test)]` modules |
| `bottemplates` | 3 | inline `#[cfg(test)]` modules |
| `botvision` | 3 | inline `#[cfg(test)]` modules |
| `botbrowser` | 1 | inline `#[cfg(test)]` modules |
