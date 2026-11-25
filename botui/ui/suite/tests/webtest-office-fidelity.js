/**
 * Offline verification for #2457 Office-fidelity harness and QA lab slice.
 * Run with: node botui/ui/suite/tests/webtest-office-fidelity.js
 *
 * Ordered modules under ./webtest-office-fidelity/:
 *   00_harness.js      counters, assertions, shared-vm loader, coverage print
 *   01_host.js         deterministic Node host; real export IIFEs loaded once
 *   02_zip.js          independent store-only ZIP reader + OOXML primitives
 *   03_features.js     fixed feature schema + deterministic scoring
 *   04_corpus.js       generated + hand-built OOXML corpus fixtures
 *   10_roundtrip.js    structural feature checks + known degradations
 *   11_helpers.js      real render/export/preview helper runtime checks
 *   12_fuzz.js         malformed ZIP and resource-cap checks
 *   20_static.js       [STATIC] production-source assertions
 *   25_recent_fixes.js [STATIC] guards for the #2457 fixes already landed
 *   28_agent_undo.js   [KNOWN BUG] #2448 criterion 4: no batch-identity undo verb
 *   30_coverage.js     matrix derived from recorded results
 *
 * No server, browser, cargo, CDN or i18n change is involved. Nothing is
 * committed by this suite; the driver owns issue closure and deployment.
 */
"use strict";

var harness = require("./webtest-office-fidelity/00_harness");

require("./webtest-office-fidelity/10_roundtrip");
require("./webtest-office-fidelity/11_helpers");
require("./webtest-office-fidelity/12_fuzz");
require("./webtest-office-fidelity/20_static");
require("./webtest-office-fidelity/25_recent_fixes");
require("./webtest-office-fidelity/26_kb_citation");
require("./webtest-office-fidelity/27_kb_backfill");
require("./webtest-office-fidelity/28_agent_undo");
require("./webtest-office-fidelity/29_sheet_scale_guard");
require("./webtest-office-fidelity/30_kb_edit_eviction");
require("./webtest-office-fidelity/31_chat_citation_inject");
require("./webtest-office-fidelity/32_kb_clause_fallback");
require("./webtest-office-fidelity/33_access_audit");
require("./webtest-office-fidelity/34_perf_marks");
require("./webtest-office-fidelity/35_sharing_gaps");
require("./webtest-office-fidelity/36_link_expiry");
require("./webtest-office-fidelity/37_sheet_toolbar_theme");
require("./webtest-office-fidelity/38_chunker_prose");
require("./webtest-office-fidelity/39_admin_verb_rbac");
require("./webtest-office-fidelity/40_agent_parity_gaps");
require("./webtest-office-fidelity/41_consent_preview_ui");
require("./webtest-office-fidelity/42_fidelity_coverage_matrix");
require("./webtest-office-fidelity/43_citation_directive");
require("./webtest-office-fidelity/44_sharing_acl_gaps");
require("./webtest-office-fidelity/45_sheet_scale_fixes");
require("./webtest-office-fidelity/46_export_acl_gate");
require("./webtest-office-fidelity/47_activity_ip_wiring");
require("./webtest-office-fidelity/48_sheet_a11y_live");
require("./webtest-office-fidelity/49_track_changes_admin_gate");
require("./webtest-office-fidelity/50_docs_collab_docid");
require("./webtest-office-fidelity/51_docs_collab_lifecycle");
require("./webtest-office-fidelity/52_docs_blank_collab");
require("./webtest-office-fidelity/53_interop_gaps");
require("./webtest-office-fidelity/54_view_ip_hook");
require("./webtest-office-fidelity/55_link_download_policy");
require("./webtest-office-fidelity/56_clippy_baseline_gate");
require("./webtest-office-fidelity/57_link_inventory");
require("./webtest-office-fidelity/58_agent_undo_gap");
require("./webtest-office-fidelity/59_confirm_cancel_gate");
require("./webtest-office-fidelity/30_coverage");

harness.finish();
