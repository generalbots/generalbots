/**
 * [STATIC] criteria, documented gaps and report-only bugs for #2389.
 *
 * A [STATIC] assertion reads shipped source only: it is a source fact, never a
 * runtime claim. [STATIC][GAP] registrations pass by proving the gap is still
 * registered; they do NOT claim the criterion is met. [KNOWN BUG] entries carry
 * file:line evidence and are reported, never patched by this harness.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var repoFile = h.repoFile;
var suiteFile = h.suiteFile;
var repoExists = h.repoExists;
var gap = h.gap;
var bug = h.bug;

var DIGEST = repoFile("botserver/crates/botsettings/src/settings_digest.rs");
var TZ = repoExists("botserver/crates/botsettings/src/timezone.rs") ?
  repoFile("botserver/crates/botsettings/src/timezone.rs") : "";
var SETTINGS_JS = suiteFile("settings/settings-actions.js");
var APPEARANCE_JS = suiteFile("settings/main/07_appearance.js");
var AUTO_JS = suiteFile("automations/automations-state.js");
var CLOCK_JS = suiteFile("clock/clock.js");

/* --------------------------------------------- shipped [STATIC] criteria */

test("[STATIC] digest resolves IANA zones and DST-aware local dates in source", function () {
  assert(DIGEST.indexOf("pub use crate::timezone::{") >= 0, "digest does not re-export timezone helpers");
  assert(DIGEST.indexOf("match resolve_zone(timezone)") >= 0, "no resolve_zone dispatch in digest");
  assert(DIGEST.indexOf("Zone::Iana(tz)") >= 0, "no IANA branch in next_run_at");
  assert(DIGEST.indexOf("from_local_datetime(&next_local)") >= 0, "no DST-aware local resolution");
  assert(DIGEST.indexOf("day = local_date_in(timezone, now)") >= 0, "week label still UTC-derived");
});

test("[STATIC] timezone.rs resolves real IANA zones before applying a UTC date", function () {
  assert(TZ.indexOf("value.parse::<Tz>()") >= 0, "no chrono-tz IANA parse");
  assert(TZ.indexOf("instant.with_timezone(&tz).date_naive()") >= 0, "local_date not zone-shifted");
  assert(TZ.indexOf("offset().fix().local_minus_utc()") >= 0, "offset not DST-correct for instant");
  assert(DIGEST.indexOf("LocalResult::Ambiguous") >= 0, "ambiguous DST local time not handled in digest");
});

test("[STATIC] unresolvable zone falls back to UTC with a warning, not silently", function () {
  assert(TZ.indexOf("warn_unresolvable(value);") >= 0, "unresolvable value not warned");
  assert(TZ.indexOf("Zone::Iana(Tz::UTC)") >= 0, "no explicit UTC fallback");
  assert(TZ.indexOf("falling back to UTC") >= 0, "no warning text");
  assert(TZ.indexOf("MAX_WARNED_ZONES") >= 0, "warning set is unbounded");
});

test("[STATIC] numeric UTC+/-hh:mm remains a documented fixed-offset fallback", function () {
  assert(TZ.indexOf("fn parse_numeric_offset") >= 0, "numeric fallback missing");
  assert(TZ.indexOf("UTC+05:30") >= 0, "numeric fallback undocumented");
  assert(TZ.indexOf("if hours > 14 || minutes > 59") >= 0, "out-of-range offset not rejected");
});

test("[STATIC] shipped tests cover both DST transitions, month boundary and warning", function () {
  ["2026-03-08", "2026-11-01", "dst_spring_forward_in_new_york_is_dst_correct",
    "dst_fall_back_in_new_york_is_dst_correct", "sao_paulo_month_boundary_is_local",
    "unknown_zone_falls_back_to_utc_and_warns"].forEach(function (needle) {
    assert(TZ.indexOf(needle) >= 0 || DIGEST.indexOf(needle) >= 0,
      "shipped test missing: " + needle);
  });
});

test("[STATIC] settings and clock pickers read the runtime IANA zone list", function () {
  assert(SETTINGS_JS.indexOf("Intl.supportedValuesOf('timeZone')") >= 0 ||
    SETTINGS_JS.indexOf("Intl.supportedValuesOf(\"timeZone\")") >= 0,
  "settings picker not IANA-backed");
  assert(CLOCK_JS.indexOf("Intl.supportedValuesOf(\"timeZone\")") >= 0,
    "clock picker not IANA-backed");
});

test("[STATIC] date_naive classification is documented as a deliberate backlog", function () {
  assert(TZ.indexOf("#2389 `date_naive()` classification") >= 0,
    "date_naive classification note missing");
  assert(TZ.indexOf("intentionally left untouched here") >= 0,
    "backlog no longer described as untouched");
});

/* ------------------------------------------------ unmet criteria (gaps) */

gap("TZ-ORG-ZONE-DB",
  "Per-org/branch IANA zone persisted in the database with a server-locale default is NOT shipped.",
  "Only per-user localStorage is used: settings/settings-actions.js:288 and " +
  "settings/main/07_appearance.js:169 write gb-timezone; no org/branch zone column " +
  "is consumed by these sources. The issue expected a stored tenant zone.");

gap("TZ-DATE-NAIVE-AUDIT",
  "The remaining date_naive() call sites are NOT classified or migrated to tenant-local dates.",
  "107 sites remain under botserver/crates (108 botserver, 109 repo-wide). " +
  "botserver/crates/botsettings/src/timezone.rs:127-133 documents that they are " +
  "intentionally left for per-caller triage, so billing/schedule labels can still be UTC-derived.");

gap("TZ-BROWSER-CI-EVIDENCE",
  "No browser UAT or CI cargo test evidence is produced by this offline harness.",
  "This harness runs Node only. DST/month-boundary Rust unit tests and the live " +
  "suite timezone pickers still need CI cargo test -p botsettings and browser " +
  "verification; those results are reported as outstanding, never fabricated.");

/* -------------------------------------- real defects (reported only) */

bug("TZ-AUTO-HARDCODED-UTC",
  "Automation schedules use a 15-zone hardcoded picker and silently default an " +
  "unlisted local zone to UTC.",
  "botui/ui/suite/automations/automations-state.js:43-48 and :186 " +
  "(AUTO_TIMEZONES.includes(localTz) ? localTz : \"UTC\"); " +
  "unlike settings/clock it never consults Intl.supportedValuesOf.");

bug("TZ-SETTINGS-INVALID-SAVED-ZONE",
  "A saved zone that the IANA list rejects (for example a numeric UTC+05:30 " +
  "fallback accepted by the Rust helper) is appended as a selectable option, " +
  "not warned or normalized.",
  "botui/ui/suite/settings/settings-actions.js:205-211 " +
  "(if (zones.indexOf(saved) === -1) zones = zones.concat([saved])); " +
  "clock.js:146-149 then formats with that value via Intl and can fail.");
