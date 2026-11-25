/**
 * [KNOWN BUG] #2463 — source-backed guards for the coexistence / interop gaps
 * confirmed by browser UAT. The book's "Office interoperability and coexistence
 * (status)" table is the disclosure of record; the code is the proof:
 *
 *   (a) no WOPI / Office-for-the-web route — only the generic `/api/docs/*`
 *       router ships (import, export, collab, ...);
 *   (b) no Google Workspace import connector — only a generic `/api/docs/import`
 *       and an Excel/Google-Sheets clipboard HTML-table paste;
 *   (c) no font substitution / metric-compatible fallback / font embedding —
 *       the PDF writer emits built-in Helvetica only;
 *   (d) no embedded-object/OLE round-trip and no PDF form fill — the import
 *       security pass explicitly never reads non-XML parts;
 *   (e) BLOCKED: no ODF 1.3 conformance claim and no LibreOffice round-trip
 *       validator — ODT is hand-built as an ODF 1.2 package.
 *
 * The only shipped PASSes in this slice are ODF import-route existence (used as
 * a non-vacuity anchor below) and the malware-scan fail-closed gate (checked as
 * one positive [STATIC] case at the end). Everything else is DEGRADED here.
 *
 * NON-VACUITY: every case first proves the shipped artifacts exist (the book
 * "Not implemented" row, the docs router, the PDF writer, the import security
 * pass, the ODF 1.2 markers), then runs a directory scan and asserts that it
 * actually covered many source files/bytes before asserting that the fix tokens
 * are absent. A missing, empty, truncated or garbled file therefore cannot
 * satisfy a case. Because the assertions are the ABSENCE of a fix, each case
 * flips to FAIL the moment a real implementation lands a matching route,
 * connector, substitution module, object/form handler or validator — at which
 * point this knownBug must be replaced by a runtime test of the real feature.
 *
 * No user-facing string, no server, no browser, no cargo, no network.
 */
"use strict";

var h = require("./00_harness");
var fs = h.fs;
var path = h.path;

var BOOK = "botbook/src/08-rest-api-tools/document-processing.md";
var DOCS_LIB = "botserver/crates/botdocs/src/lib.rs";
var DOCS_IMPORT = "botserver/crates/botdocs/src/handlers_api/import.rs";
var ODT_BUILDER = "botserver/crates/botdocs/src/utils_ooxml.rs";
var PDF_WRITER = "botserver/crates/botdocs/src/pdf/writer.rs";
var IMPORT_SECURITY = "botserver/crates/botdocs/src/import_security.rs";
var PASTE_HTML = "botserver/crates/botsheet/src/handlers/paste_html.rs";
var UPLOAD_HOSTILE = "botserver/crates/botsecurity-core/src/upload_hostile.rs";

var OFFICE_CRATES = [
  "botserver/crates/botdocs/src",
  "botserver/crates/botsheet/src",
  "botserver/crates/botslides/src"
];
var WOPI_DIRS = [
  "botserver/src",
  "botserver/crates",
  "botlib/src",
  "botui/ui/suite"
];
var VALIDATOR_DIRS = OFFICE_CRATES.concat([".github/workflows"]);

var SKIP_DIRS = ["vendor", "node_modules", "target", "tests", "webtest-office-fidelity"];

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function between(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  var span = src.slice(start, end);
  h.assert(span.length > 0, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return span;
}

/**
 * Case-insensitive literal scan for fix tokens over the given repo-relative
 * directories. The harness/test dirs are skipped so this module's own literals
 * cannot make a scan non-empty. Returns coverage stats plus every hit.
 */
function scanSources(relDirs, extensions, tokens) {
  var lowerTokens = tokens.map(function (token) { return token.toLowerCase(); });
  var stats = { files: 0, bytes: 0 };
  var hits = [];
  relDirs.forEach(function (relDir) {
    var root = path.join(h.REPO_DIR, relDir);
    h.assert(fs.existsSync(root), "scan root must ship: " + relDir);
    (function walk(dir) {
      fs.readdirSync(dir, { withFileTypes: true })
        .sort(function (a, b) { return a.name < b.name ? -1 : a.name > b.name ? 1 : 0; })
        .forEach(function (entry) {
          var full = path.join(dir, entry.name);
          if (entry.isDirectory()) {
            if (SKIP_DIRS.indexOf(entry.name) === -1) walk(full);
            return;
          }
          if (extensions.indexOf(path.extname(entry.name)) === -1) return;
          var src = fs.readFileSync(full, "utf8").toLowerCase();
          stats.files++;
          stats.bytes += src.length;
          lowerTokens.forEach(function (token) {
            if (src.indexOf(token) !== -1) {
              hits.push(path.relative(h.REPO_DIR, full).split(path.sep).join("/") + " :: " + token);
            }
          });
        });
    })(root);
  });
  return { stats: stats, hits: hits };
}

/** Non-vacuity first, then the documented absence of a fix. */
function assertAbsent(scan, what) {
  h.assert(scan.stats.files > 20,
    "non-vacuity: " + what + " scan covered only " + scan.stats.files + " files");
  h.assert(scan.stats.bytes > 20000,
    "non-vacuity: " + what + " scan read only " + scan.stats.bytes + " bytes");
  h.assertEqual(scan.hits.length, 0,
    what + " must still be absent; found: " + scan.hits.slice(0, 6).join(" | "));
}

// (a) The docs router is large and generic, but has no WOPI/Collabora route.
h.knownBug("#2463 no WOPI / Office-for-the-web route: only the generic /api/docs/* router ships", function () {
  contains(BOOK, "| WOPI / Office for the web / Office Online | Not implemented |",
    "book discloses the WOPI gap");
  contains(BOOK, "No `wopi` reference exists in `botserver/src`",
    "book pins the scanned source trees");
  contains(DOCS_LIB, "pub fn configure_docs_routes()", "docs router ships");
  contains(DOCS_LIB, '"/api/docs/import"', "generic import route ships");
  contains(DOCS_LIB, '"/api/docs/export/docx"', "generic export route ships");
  h.assert(h.readRepo(DOCS_LIB).split('.route(').length > 30,
    "non-vacuity: docs router must register many routes");

  var scan = scanSources(WOPI_DIRS, [".rs", ".js", ".html"],
    ["wopi", "checkfileinfo", "wopisrc", "x-wopi", "coolwsd"]);
  assertAbsent(scan, "a WOPI / Office-for-the-web route");
});

// (b) Generic import + clipboard paste are shipped; no Google API connector.
h.knownBug("#2463 no Google Workspace import connector or conversion report", function () {
  contains(BOOK,
    "| Google Workspace import/export connector with a conversion report | Not implemented |",
    "book discloses the Google connector gap");
  contains(DOCS_LIB, '"/api/docs/import"', "generic import route ships");
  contains(DOCS_IMPORT, "pub async fn handle_import_document(", "generic import handler ships");
  contains(DOCS_IMPORT, '"odt" => odt_zip_to_html(effective)', "ODF import dispatch ships");
  contains(PASTE_HTML,
    "Excel and Google Sheets put an HTML table flavour on the clipboard.",
    "clipboard-paste fallback is the only Google-adjacent path");

  var scan = scanSources(OFFICE_CRATES, [".rs"],
    ["googleapis", "google_drive", "google_docs", "google_sheets_import",
      "drive.files", "conversion_report", "conversionreport", "google_oauth"]);
  assertAbsent(scan, "a Google Workspace import connector");
});

// (c) The PDF writer uses only the built-in Helvetica family (no embed), and
//     no crate implements substitution or metric-compatible fallback.
h.knownBug("#2463 no font substitution, metric-compatible fallback or embedded fonts on export", function () {
  contains(BOOK,
    "| Font substitution, metric-compatible fallback, font embedding on export | Not implemented |",
    "book discloses the font gap");
  contains(PDF_WRITER,
    "Only the built-in Helvetica family is used, so no font embedding is",
    "built-in Helvetica policy is shipped");
  contains(PDF_WRITER, "fn font_object(", "font-object writer is shipped");
  contains(PDF_WRITER, 'content: font_object("Helvetica"),', "Helvetica object is emitted");
  contains(DOCS_LIB, '"/api/docs/export/pdf"', "PDF export route ships");

  var scan = scanSources(OFFICE_CRATES, [".rs"],
    ["font_substitut", "substitute_font", "font_fallback", "fallback_font",
      "metric_compatible", "font_embed", "embed_font", "font_embedding"]);
  assertAbsent(scan, "font substitution / embedding");
});

// (d) Non-XML parts are deliberately never read; no object/OLE or form-fill
//     handling exists anywhere in the office crates.
h.knownBug("#2463 no embedded-object/OLE round-trip and no PDF form fill", function () {
  contains(BOOK, "| Embedded objects / OLE round-trip | Not implemented |",
    "book discloses the embedded-object gap");
  contains(BOOK, "| PDF form fill and save | Not implemented |",
    "book discloses the PDF form-fill gap");
  contains(IMPORT_SECURITY, "Non-XML parts (images, embedded fonts,",
    "import pass explicitly skips non-XML parts");
  contains("botserver/crates/botsheet/src/storage/import/odf.rs",
    "fn is_embedded_object_path(", "the only embedded-object code detects parts to refuse");
  contains("botserver/crates/botsheet/src/storage/import/odf/tests.rs",
    "refuses_an_embedded_object_part", "embedded-object refusal is what is tested");
  contains(PDF_WRITER, "fn font_object(", "PDF writer ships without a form path");
  contains(DOCS_LIB, '"/api/docs/export/pdf"', "PDF export route ships");

  // `is_embedded_object_path` only DETECTS objects so they can be refused —
  // that is not a round-trip handler. These tokens would only appear with a
  // real OLE-preservation / form-fill implementation.
  var scan = scanSources(OFFICE_CRATES, [".rs"],
    ["ole_object", "object_roundtrip", "roundtrip_embedded", "preserve_embedded",
      "acroform", "form_fill", "fill_form", "formfield", "widget_annotation"]);
  assertAbsent(scan, "an embedded-object round-trip or PDF form-fill handler");
});

// (e) BLOCKED: ODT is hand-built as ODF 1.2; there is no conformance claim and
//     no LibreOffice round-trip validator in code or CI.
h.knownBug("#2463 BLOCKED: no ODF 1.3 conformance claim and no LibreOffice round-trip validator", function () {
  contains(BOOK,
    "There is no ODF 1.3 conformance claim and no LibreOffice round-trip validator in the repository.",
    "book discloses the validator gap");
  contains(ODT_BUILDER, 'office:version="1.2">', "ODT builder still declares ODF 1.2");
  contains(ODT_BUILDER, 'manifest:version="1.2">', "ODT manifest still declares 1.2");
  contains(DOCS_LIB, '"/api/docs/export/odt"', "ODF export route ships");

  var scan = scanSources(VALIDATOR_DIRS, [".rs", ".yml"],
    ["soffice", "libreoffice --headless", "libreoffice-headless",
      "libreoffice_validator", "odf_validator", "odfvalidator",
      "round-trip validator", "roundtrip validator", "validate_odf"]);
  assertAbsent(scan, "a LibreOffice round-trip validator");
});

// The malware gate is a shipped PASS: a missing scanner is a typed rejection,
// never a fabricated Clean. This is a positive [STATIC] case, not a knownBug.
h.staticTest("#2463 malware scan fails closed: NotConfigured is a typed rejection, never Clean", function () {
  contains(UPLOAD_HOSTILE, "pub fn inspect_hostile_upload_with_scanner(",
    "hostile-file gate ships");
  contains(UPLOAD_HOSTILE, "pub fn malware_decision(", "malware decision helper ships");
  contains(UPLOAD_HOSTILE, "require_malware_scanner: true,",
    "default policy requires a scanner");
  contains(UPLOAD_HOSTILE, "None => MalwareScanDecision::NotConfigured,",
    "a missing hook reports NotConfigured, not Clean");
  contains(UPLOAD_HOSTILE, "Infected { scanner: String, signature: String }",
    "infected signature is a typed decision");

  var gate = between(h.readRepo(UPLOAD_HOSTILE),
    "pub fn malware_decision(", "/// Inspects a zip container with the default policy.");
  h.assert(gate.indexOf("MalwareScanDecision::NotConfigured if require_scanner =>") !== -1,
    "NotConfigured must be gated on require_scanner");
  h.assert(gate.indexOf("Err(UploadError::MalwareScannerMissing)") !== -1,
    "missing scanner must produce a typed rejection");
  h.assert(gate.indexOf("Err(UploadError::MalwareDetected") !== -1,
    "infected payload must be refused");
  h.assert(gate.indexOf("Err(UploadError::MalwareScanFailed)") !== -1,
    "a failed scan must be refused");
});

h.note("GAP #2463: WOPI, Google Workspace connector, font substitution, embedded objects, PDF form fill and an ODF 1.3/LibreOffice validator are all documented 'Not implemented'; only ODF import routing and malware-scan fail-closed ship, and each guard here flips to FAIL when a real fix lands.");
