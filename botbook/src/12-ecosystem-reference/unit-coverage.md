# Unit test coverage by crate

Deterministic inventory of the offline unit tests in the Rust workspace: one row per crate, the number of `#[test]` / `#[tokio::test]` functions it owns, and the modules those tests live in. It is generated from the tree — never edited by hand:

```bash
python3 scripts/unit_coverage_report.py
```

The count is a *function* count, not a line-coverage percentage. A crate with many tests can still miss a branch; the per-module line and branch numbers come from the `cargo-llvm-cov` job (#2074). What this page guarantees is that no crate is silently absent from the unit gate.

## Summary

- Crates in the workspace: **122**
- `.rs` files scanned: **3784**
- Test functions: **5611**
- Crates with no tests: **13**
- Crates with 1–4 tests: **10**

Run one crate at a time (`cargo test -p <crate> --lib`); the workspace graph does not fit the 4.5 GiB dev container in one go (AGENTS.md).

## Coverage by crate

| Crate | Tests | What the tests cover |
|-------|------:|----------------------|
| `botsecurity-core` | 349 | `adversarial`, `sql_guard`, `ssrf`, `tenant_adversarial` |
| `bottest` | 337 | `accessibility (integration)`, `api (integration)`, `auth_flow (integration)`, `basic_runtime (integration)`, +29 more |
| `botvibe` | 325 | `domains_caa (integration)`, `git_engine_flow (integration)`, `harness_cmd`, `knowledge_graph (integration)`, +5 more |
| `botbasic_core` | 243 | `resolve_from_scope_row` |
| `botbasic_compiler` | 183 | `blocks`, `convert_if_then`, `convert_multiword`, `convert_while_wend_select`, +6 more |
| `botsecurity-auth` | 176 | inline `#[cfg(test)]` modules |
| `botlib` | 168 | `api`, `attachment`, `bundle_plural`, `command_guard`, +4 more |
| `botllm` | 151 | `extract_base_url`, `hash_embedding`, `model_file_name`, `openai_client`, +3 more |
| `botcore` | 143 | inline `#[cfg(test)]` modules |
| `botcompliance` | 131 | `access_review`, `categorize`, `checker`, `export`, +1 more |
| `botsecurity-crypto` | 131 | `ca`, `encryption`, `mutual_tls`, `pinning`, +2 more |
| `botdrive` | 129 | `classify`, `gdrive`, `keys`, `storage_backends`, +2 more |
| `botsheet-core` | 126 | `arrays`, `formats` |
| `botintegrations` | 122 | `auth`, `payload`, `vault_path (integration)` |
| `botapp` | 97 | inline `#[cfg(test)]` modules |
| `botdomains` | 95 | `cloudflare`, `desec`, `dynadot`, `namecheap`, +1 more |
| `botbasic_data` | 93 | `csv`, `filter`, `table_definition`, `with_user_id` |
| `botevents` | 91 | `agui`, `envelope`, `events`, `pipeline`, +5 more |
| `botautotask` | 88 | `compile_prompt`, `extract_json_object`, `log_level`, `manifest`, +4 more |
| `botsheet` | 88 | `csv`, `find_replace`, `html`, `json`, +7 more |
| `botsettings` | 80 | `catalog`, `html_escape`, `jobs`, `permission_inheritance`, +2 more |
| `botdocs` | 79 | `review`, `review_pair`, `storage_core`, `storage_docx_source`, +2 more |
| `botwhatsapp` | 78 | `utils`, `webhooks` |
| `botbasic_system` | 77 | `coerce_value`, `cosine_similarity`, `create_task`, `error_type`, +3 more |
| `botbasic_comms` | 75 | `delegate_to_bot`, `options`, `recurrence`, `sms_priority`, +4 more |
| `botcloud` | 75 | `api`, `compute_provisioning`, `domain_provisioning`, `helpers`, +3 more |
| `botbanking` | 73 | `cashflow_rules`, `dashboard`, `handlers`, `intent`, +2 more |
| `botchannels` | 72 | `channel_type`, `handoff`, `result`, `send_payload`, +2 more |
| `botmemory` | 69 | `api`, `extract`, `import`, `models`, +4 more |
| `botsecurity-protection` | 67 | inline `#[cfg(test)]` modules |
| `botbilling` | 66 | `lib`, `totals` |
| `botmonitoring` | 62 | `collector`, `policy`, `serializers` |
| `botproviders` | 62 | `digitalocean`, `hetzner`, `oracle`, `ovh`, +1 more |
| `bottelegram` | 55 | `payload`, `types`, `webhook` |
| `botemail` | 50 | `email_triage`, `models`, `oauth` |
| `botbrowserpolicy` | 47 | `api`, `budget_adversarial`, `policy_adversarial` |
| `botretail` | 46 | `catalog`, `pos`, `pricing`, `promotion`, +1 more |
| `boteval` | 45 | inline `#[cfg(test)]` modules |
| `botagent` | 43 | inline `#[cfg(test)]` modules |
| `botworkspaces` | 43 | `block_ops`, `domain`, `page_ops`, `transform` |
| `botpeople` | 42 | `time_clock_schedule` |
| `botanalytics` | 41 | `analytics_types`, `goals_types`, `handlers_charts`, `insights_types` |
| `botconnectors` | 41 | `knowledge` |
| `botbasic_ai` | 39 | `from_config`, `multimodal`, `workflow_dag` |
| `botbrazil` | 36 | inline `#[cfg(test)]` modules |
| `botslides` | 35 | `pptx_preservation (integration)` |
| `botcontacts` | 34 | `contacts_csv`, `csv_parse`, `forecast` |
| `botsources` | 33 | `bundled_skills (integration)` |
| `botdesktop` | 30 | inline `#[cfg(test)]` modules |
| `botfraud` | 30 | `actions`, `rules_condition`, `scoring_aggregate`, `types` |
| `botinstagram` | 30 | `adapter`, `campaign`, `types` |
| `botautomation` | 28 | inline `#[cfg(test)]` modules |
| `botschema` | 27 | `schema_invariants`, `struct_contract` |
| `botvideo` | 27 | `chat_ai`, `media`, `safe_command` |
| `botattendant` | 26 | `attachments`, `lifecycle` |
| `botconsent` | 24 | inline `#[cfg(test)]` modules |
| `botdashboards` | 23 | `dashboard`, `data_sources` |
| `botsocial` | 23 | `comment`, `handle_add_comment`, `handle_trending_topics` |
| `botcanvas` | 22 | `convert`, `export`, `models` |
| `botweba` | 20 | `base_url (integration)`, `bot_management_page (integration)`, `bot_name (integration)`, `click (integration)`, +7 more |
| `botbasic_types` | 19 | `runtime`, `types_contract` |
| `botmarketplace` | 19 | inline `#[cfg(test)]` modules |
| `botmsteams` | 19 | `adapter`, `types` |
| `botplan` | 19 | `collaboration`, `default_status` |
| `botcorepkg` | 18 | inline `#[cfg(test)]` modules |
| `botkyc` | 17 | `intent`, `lifecycle` |
| `botpaper` | 17 | `utils` |
| `botproducts` | 17 | `pricing` |
| `botmarketing` | 16 | `analytics` |
| `bottickets` | 15 | `handle_ai_suggest`, `html_escape`, `stats_helpers` |
| `botcalendar` | 14 | inline `#[cfg(test)]` modules |
| `botuifragments` | 14 | `shells` |
| `botcorebot` | 13 | `trigger_kind`, `utils` |
| `botcoredirectory` | 13 | inline `#[cfg(test)]` modules |
| `botcoreoauth` | 13 | `pkce` |
| `botsampledata` | 13 | `drive`, `sample` |
| `botlearn` | 12 | `certification`, `xp` |
| `bottimeseries` | 12 | `line_protocol` |
| `botresearch` | 10 | inline `#[cfg(test)]` modules |
| `bottimeclock` | 10 | inline `#[cfg(test)]` modules |
| `botapi` | 9 | `system` |
| `botattendance` | 9 | inline `#[cfg(test)]` modules |
| `botbiometry` | 9 | inline `#[cfg(test)]` modules |
| `botclock` | 9 | inline `#[cfg(test)]` modules |
| `botmaintenance` | 9 | inline `#[cfg(test)]` modules |
| `botmodelsbridge` | 9 | inline `#[cfg(test)]` modules |
| `botdeployment` | 8 | inline `#[cfg(test)]` modules |
| `botm365` | 8 | inline `#[cfg(test)]` modules |
| `botmeet` | 8 | inline `#[cfg(test)]` modules |
| `botplayer` | 8 | `media` |
| `botgit` | 7 | `resolve_repo` |
| `botlog` | 6 | inline `#[cfg(test)]` modules |
| `botminutes` | 6 | `parse_dt` |
| `botsecurity` | 6 | inline `#[cfg(test)]` modules |
| `bottax` | 6 | inline `#[cfg(test)]` modules |
| `botbrowser` | 5 | inline `#[cfg(test)]` modules |
| `botgl` | 5 | `reports` |
| `botinventory` | 5 | inline `#[cfg(test)]` modules |
| `botsearch` | 5 | inline `#[cfg(test)]` modules |
| `botdatabase` | 4 | inline `#[cfg(test)]` modules |
| `botdesigner` | 4 | inline `#[cfg(test)]` modules |
| `botpos` | 4 | `parse_decimal` |
| `botproject` | 4 | inline `#[cfg(test)]` modules |
| `botqdrant` | 4 | inline `#[cfg(test)]` modules |
| `botcoresecrets` | 3 | inline `#[cfg(test)]` modules |
| `boterp` | 3 | inline `#[cfg(test)]` modules |
| `bottemplates` | 3 | inline `#[cfg(test)]` modules |
| `botvision` | 3 | inline `#[cfg(test)]` modules |
| `botui` | 1 | inline `#[cfg(test)]` modules |
| `botchannelbindings` | 0 | — |
| `botchannels-core` | 0 | — |
| `botcoresession` | 0 | — |
| `boteditor` | 0 | — |
| `bothandoff` | 0 | — |
| `bothr` | 0 | — |
| `botitsm` | 0 | — |
| `botlegal` | 0 | — |
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
| `botlegal` | Contract DTOs and expiry helpers that read the clock; no pure parser. |
| `botmultimodal` | Configuration readers over the `ConfigProvider` trait; every path is I/O. |
| `botnvidia` | Shells out to `nvidia-smi`; no pure logic. |
| `botpublic` | Twelve lines of route registration. |
| `botsales` | Thin CRUD: leads and quotes over the `sales_*` tables. |
| `bottasks` | **Gap:** the scheduler queries `system_automations`; `should_run_now` reads the clock directly, so it needs an injected `now` before it can be unit-tested. |

## Crates with fewer than five tests

| Crate | Tests | What the tests cover |
|-------|------:|----------------------|
| `botdatabase` | 4 | inline `#[cfg(test)]` modules |
| `botdesigner` | 4 | inline `#[cfg(test)]` modules |
| `botpos` | 4 | `parse_decimal` |
| `botproject` | 4 | inline `#[cfg(test)]` modules |
| `botqdrant` | 4 | inline `#[cfg(test)]` modules |
| `botcoresecrets` | 3 | inline `#[cfg(test)]` modules |
| `boterp` | 3 | inline `#[cfg(test)]` modules |
| `bottemplates` | 3 | inline `#[cfg(test)]` modules |
| `botvision` | 3 | inline `#[cfg(test)]` modules |
| `botui` | 1 | inline `#[cfg(test)]` modules |
