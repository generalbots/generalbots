/**
 * [STATIC] characterizations for #2544, plus honest bug reporting.
 *
 * GAP assertions pass by confirming a gap still exists; they are observations,
 * not claims that a criterion is met. Real bugs are recorded with file:line
 * evidence and are NOT patched here. Criteria that need cargo, a live server or
 * CI are registered with defer() and are explicitly NOT counted as PASS.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var defer = h.defer;
var bug = h.bug;
var assert = h.assert;
var equal = h.equal;
var all = h.all;
var read = h.read;
var findLine = h.findLine;

var CRM = "bottest/tests/integration/crm_flow.rs";
var CAMPAIGN = "bottest/tests/integration/campaign_flow.rs";

function has(text, needle, label) {
  assert(text.indexOf(needle) >= 0, label + ": missing marker " + needle);
}

// ── Shipped criteria, verified statically ────────────────────────────────

test("[STATIC] CI invokes the binary-target test command and AGENTS.md documents it", function () {
  var ci = read(".github/workflows/ci.yml");
  var line = findLine(".github/workflows/ci.yml", "cargo test -p botserver --bin botserver");
  assert(line > 0, "ci.yml has no --bin botserver command");
  has(ci, "cargo test -p botserver --bin botserver", "ci.yml");
  has(read("AGENTS.md"), "cargo test -p botserver --bin botserver", "AGENTS.md");
});

test("[STATIC] CI ships per-crate coverage jobs and an informational baseline", function () {
  var ci = read(".github/workflows/ci.yml");
  has(ci, "unit-coverage", "ci.yml unit-coverage job");
  has(ci, "coverage-plan", "ci.yml coverage-plan job");
  has(ci, "scripts/check_coverage.py", "ci.yml coverage script");
  assert(all().coverageBaseline, "issues/coverage/baseline.json missing");
});

test("[STATIC] CRM flow test encodes contact -> lead -> convert -> close-won steps", function () {
  var src = read(CRM);
  has(src, "test_crm_flow_contact_lead_convert_close_won", "CRM test name");
  has(src, "diesel::sql_query", "CRM DB read-back");
  has(src, 'contact.status, "active"', "contact state assertion");
  has(src, 'lead.stage, "new"', "lead state assertion");
  has(src, 'opportunity.stage, "qualification"', "opportunity state assertion");
  has(src, 'converted_lead.stage, "converted"', "convert state assertion");
  has(src, "won_flag", "close-won flag assertion");
});

test("[STATIC] Campaign flow test encodes list -> preview -> publish -> suppression -> unsubscribe", function () {
  var src = read(CAMPAIGN);
  has(src, "test_campaign_flow_list_preview_publish_suppression_unsubscribe", "Campaign test name");
  has(src, "preview_counts", "segment preview counts");
  has(src, 'draft.status, "draft"', "new campaign starts as draft");
  has(src, "recipient_count(&mut conn, campaign_id)", "recipient row read-back");
  has(src, '"suppressed"', "suppression state");
  has(src, '"unsubscribed"', "unsubscribe state");
  has(src, "suppressed_all, baseline_suppressed + 2", "final suppression assertion");
});

// ── Real bugs / gaps, asserted as still-present and reported with evidence ──

test("[STATIC][GAP] botserver has no lib target, so `cargo test -p botserver` runs no tests", function () {
  var m = all();
  var pkgLine = findLine("botserver/Cargo.toml", 'name = "botserver"');
  equal(m.libSection, false, "botserver [lib] section");
  equal(m.serverLibFile, false, "botserver src/lib.rs");
  bug("B1", "botserver is binary-only: AGENTS.md and ci.yml document that the bare command " +
      "fails with 'no library targets found'; the suite only builds via " +
      "`cargo test -p botserver --bin botserver` (ci.yml:1139). Not re-measured offline.", [
    "botserver/Cargo.toml:" + pkgLine + " [package] with no [lib] section",
    "botserver/src/main.rs present; botserver/src/lib.rs absent",
    "AGENTS.md:" + findLine("AGENTS.md", "no library target") + " documents the requirement",
  ]);
});

test("[STATIC][GAP] 8 zero-test crates have no test-exemption entry", function () {
  var m = all();
  equal(m.unexempted.length, 8, "unexempted zero-test crate count");
  var ev = m.unexempted.map(function (name) {
    return "botserver/crates/" + name + "/Cargo.toml:1 zero #[test]/#[tokio::test], no exemption";
  });
  bug("B2", "8 crates still ship zero unit tests and are not covered by " +
      "botserver/test-exemptions.toml (only botnvidia and botpublic are exempt).", ev);
});

test("[STATIC][GAP] CRM and Campaign flow tests can silently skip their steps", function () {
  function skips(rel) {
    var lines = read(rel).split(/\r?\n/);
    var out = [];
    lines.forEach(function (line, i) {
      if (line.indexOf("Skipping ") >= 0) out.push(rel + ":" + (i + 1) + " " + line.trim());
    });
    return out;
  }
  var crmSkips = skips(CRM);
  var campSkips = skips(CAMPAIGN);
  assert(crmSkips.length > 0 && campSkips.length > 0, "expected soft-skip paths to exist");
  bug("B3", "Both integration flow tests return early on unavailable endpoints/DB, so an " +
      "environment gap can report PASS without exercising the CRM/Campaign steps " +
      "(crm=" + crmSkips.length + " skip sites, campaign=" + campSkips.length + " skip sites).",
    crmSkips.slice(0, 3).concat(campSkips.slice(0, 3)));
});

test("[STATIC][GAP] test-floor acceptance criteria overstate the shipped floor", function () {
  var m = all();
  assert(m.zero.length > 0, "issue-era '38 zero-test crates' claim no longer reflects the tree");
  assert(m.integration.crmNamed.length > 0 && m.integration.campaignNamed.length > 0,
    "CRM/Campaign integration files unexpectedly absent");
  bug("B4", "Issue #2544's 2025 snapshot is stale: the current tree has 119 crates, " +
      "10 zero-test (down from 38), CRM/Campaign integration files, --bin CI and coverage jobs. " +
      "Only the residual gaps above remain.", [
    "10 zero-test crates observed; 8 unexempted",
    "bottest/tests/integration/crm_flow.rs and campaign_flow.rs present",
  ]);
});

// ── Criteria that CANNOT be measured offline: deferred, never a fake PASS ──

defer("cargo test -p botserver --bin botserver actual result",
  "needs a cargo toolchain and compiled binary; CI job at .github/workflows/ci.yml:1139");
defer("cargo test -p bottest --features integration tenant_isolation actual result",
  "needs botserver + migrated DB; documented in AGENTS.md:936");
defer("per-crate coverage report and floor gate result",
  "needs the CI unit-coverage matrix; baseline at issues/coverage/baseline.json");
defer("CRM flow DB state verified end to end at runtime",
  "needs live botserver + migrated PostgreSQL; static test markers only");
defer("Campaign flow DB state verified end to end at runtime",
  "needs live botserver + migrated PostgreSQL; static test markers only");
defer("browser verification of any changed web-facing behaviour",
  "no web-facing file changed by this static audit; requires Chrome CDP when UI changes");
