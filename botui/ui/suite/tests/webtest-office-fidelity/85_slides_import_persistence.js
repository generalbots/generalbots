/**
 * #2927 — the Slides import must persist the sanitized deck and return the
 * shared security notice, on every success path.
 *
 * Offline static proof only: pins the real import handler, the shared notice
 * builder and the deterministic persistence tests. No server, browser, cargo
 * or locale edit. The driver owns live UAT and issue closure.
 *
 * GB_SLIDES_IMPORT_PROBE_ROOT points at a /tmp tree holding mutated copies
 * named import.rs / import_notice.rs / import_persistence_tests.rs, so a
 * mutation probe can prove the guard fails when the storage call, the notice
 * or the deterministic test is removed.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var IMPORT = "botserver/crates/botslides/src/handlers/import.rs";
var NOTICE = "botserver/crates/botslides/src/import_notice.rs";
var TESTS = "botserver/crates/botslides/src/handlers/import_persistence_tests.rs";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var KEY = "file-macros-removed";
var PROBE_ROOT = process.env.GB_SLIDES_IMPORT_PROBE_ROOT || "";

function readSrc(rel) {
  var candidate = PROBE_ROOT ? path.join(PROBE_ROOT, path.basename(rel)) : "";
  if (candidate && fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  return h.readRepo(rel);
}
function compact(src) {
  return src.replace(/\s+/g, "");
}
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function countIn(src, needle) {
  return src.split(needle).length - 1;
}
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1 && end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

h.staticTest("#2927 the import handler funnels every success path through one storage call", function () {
  var src = readSrc(IMPORT);
  h.assert(src.length > 9000, "non-vacuity: import.rs must be the real handler, got " + src.length);
  h.assert(src.split("\n").length < 450, "import.rs must stay under 450 lines");
  var flat = compact(src);

  // The persistence seam is the ONLY place that writes a deck during import.
  var persist = span(src, "async fn persist_import", "pub async fn handle_import_presentation");
  h.assert(persist.length > 600, "persist_import must be a substantial function, got " + persist.length);
  contains(persist, "save_presentation_to_drive(drive, user_id, presentation)",
    "the presentation metadata row must be written on every import");
  contains(persist, "save_presentation_pptx(drive, user_id, &presentation.id, bytes)",
    "the sanitized package must be written next to the row");
  contains(persist, "with_notices(presentation, notices)",
    "the notice must be attached to the stored presentation response");
  contains(persist, "map_err(|error|", "storage errors must be propagated, not swallowed");
  contains(flat, "log_and_sanitize_json(&error", "storage failures must use the shared sanitizer");
  absent(flat, "let_=", "no discarded result on the persistence path");
  absent(flat, "is_ok()", "no swallowed storage error on the persistence path");

  var handler = span(src, "pub async fn handle_import_presentation", "#[cfg(test)]\n#[path = \"import_tests.rs\"]");
  h.assert(handler.length > 8000, "handler body must be substantial, got " + handler.length);
  h.assertEqual(countIn(handler, "persist_import("), 2,
    "both new-deck and re-open paths must return through persist_import");
  contains(handler, "strip_executable_parts(&bytes)", "macros are stripped before the gate");
  contains(handler, "validate_upload(", "the shared gate still runs before parsing");
  // #2927 — the redundant second inspection also performed a second malware
  // scan; validate_upload already applies the same gate exactly once.
  absent(handler, "audit_hostile_upload(", "the payload must not be scanned twice");
  contains(handler, "notices.push(crate::import_notice::macro_notice(",
    "a stripped package must build a machine-readable notice");
});

h.staticTest("#2927 the notice reuses the shipped catalogue key in every locale", function () {
  var notice = readSrc(NOTICE);
  h.assert(notice.length > 2000, "non-vacuity: import_notice.rs is the shared builder");
  contains(notice, 'pub(crate) fn macro_notice(', "the shared notice builder must exist");
  contains(notice, 'code: "file-macros-removed"', "stable machine code");
  contains(notice, 'message_key: "file-macros-removed"', "catalogue key, never a user-facing literal");
  contains(notice, 'map.insert("security_notices".to_string(), encoded);',
    "the notice is emitted under the shared field name");
  LOCALES.forEach(function (locale) {
    var dir = path.join(h.REPO_DIR, "botlib", "locales", locale);
    var all = h.fs.readdirSync(dir).filter(function (name) { return /\.ftl$/.test(name); })
      .map(function (name) { return h.fs.readFileSync(path.join(dir, name), "utf8"); }).join("\n");
    h.assert(new RegExp("^" + KEY + "\\s*=", "m").test(all), locale + " is missing key " + KEY);
  });
});

h.staticTest("#2927 deterministic tests pin storage, benign imports and typed failures", function () {
  var tests = readSrc(TESTS);
  h.assert(tests.length > 3000, "non-vacuity: persistence tests must be substantial");
  h.assert(tests.split("\n").length < 450, "test file must stay under 450 lines");
  [
    "a_sanitized_vba_import_persists_the_deck_and_returns_the_notice",
    '"file-macros-removed"',
    "list_presentations_from_drive",
    "vbaproject.bin",
    "a_macro_free_import_is_unchanged_and_has_no_security_notice",
    "security_notices",
    "a_storage_failure_is_a_typed_sanitized_error",
    "INTERNAL_SERVER_ERROR"
  ].forEach(function (needle) {
    contains(tests, needle, "persistence test must pin <" + needle + ">");
  });

  var runner = h.readRepo(RUNNER);
  contains(runner, 'require("./webtest-office-fidelity/85_slides_import_persistence")',
    "this guard must be required by the fidelity runner");
  [IMPORT, NOTICE, TESTS].forEach(function (rel) {
    var src = readSrc(rel);
    h.assert(!/\bhttps?:\/\//i.test(src) && !/\b(?:cdn|unpkg|jsdelivr|cdnjs)\b/i.test(src),
      rel + " must not reference an external CDN");
  });
});

h.note("#slides-import-persistence: offline static proof only — no browser/server/cargo, no new i18n key. Driver owns live Chrome UAT and issue closure.");
