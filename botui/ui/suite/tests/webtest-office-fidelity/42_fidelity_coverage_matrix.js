/**
 * #2457 — static coverage for the SHIPPED offline fidelity-lab building blocks:
 * the three-format round-trip corpus, the generated coverage matrix, the
 * structural comparator and the importer fuzz entrypoint.
 *
 * All assertions are literal substrings of repo source or file-size checks.
 * Nothing is executed, no server, no browser, no cargo, no network.
 *
 * NON-VACUITY: every case first proves the artifact ships (format-specific
 * builders/parts, a results-derived status expression, a libfuzzer target,
 * non-empty on-disk fixtures/seeds). Removing any one format builder, the
 * results-derived matrix, the fuzz target or a seed makes a case FAIL.
 * The claims are deliberately narrow: the licensed 100+ file corpus, the
 * visual diff dashboard, the 24h run and the four-browser smoke matrix are
 * NOT shipped. The last criterion is characterised with h.knownBug from the
 * documented "Not implemented" state, never asserted as a PASS.
 */
"use strict";

var h = require("./00_harness");

var FIXTURES = "bottest/tests/fixtures/office-fidelity/";
var SYNTH = "bottest/tools/fidelity/fx_fixtures.py";
var CONTRACT = "bottest/tests/office_fidelity.rs";
var CORPUS = "botui/ui/suite/tests/webtest-office-fidelity/04_corpus.js";
var README = "bottest/tools/fidelity/README.md";
var REPORT = "bottest/tools/fidelity/fx_report.py";
var MATRIX = "botui/ui/suite/tests/webtest-office-fidelity/30_coverage.js";
var HARNESS = "botui/ui/suite/tests/webtest-office-fidelity/00_harness.js";
var FUZZ_TARGET = "fuzz/fuzz_targets/office_import.rs";
var FUZZ_CARGO = "fuzz/Cargo.toml";
var FUZZ_STABLE = "bottest/tests/fuzz_regressions.rs";
var SEED_DIR = "fuzz/corpus/office_import";
var BOOK = "botbook/src/07-user-interface/apps/office-fidelity.md";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  var src = h.readRepo(relPath);
  h.assert(src.indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function span(src, startMarker, endMarker, msg) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, (msg || "missing start marker") + " <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, (msg || "missing end marker") + " <" + endMarker + ">");
  h.assert(end > start, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

/** Size of a shipped binary/text artifact, in bytes. */
function sizeOf(relPath) {
  return h.fs.statSync(h.path.join(h.REPO_DIR, relPath)).size;
}

h.staticTest("#2457 offline round-trip corpus ships DOCX/PPTX/XLSX fixtures and a deterministic generator (not the licensed 100+ corpus)", function () {
  // Non-vacuity: three real, non-empty on-disk fixtures — one per format.
  ["Alex_Rivera_Resume_Futuristic.docx", "Rice_and_Beans_Nutrition.pptx",
    "Painel_Financeiro_Completo.xlsx"].forEach(function (name) {
      h.assert(sizeOf(FIXTURES + name) > 1000, name + " must be a non-empty fixture");
    });

  // The Python comparator's corpus generator has one builder per format.
  contains(SYNTH, 'ORIGINAL_SPECS: dict[str, tuple[str, dict[str, int]]] = {',
    "feature-tagged original specs ship");
  contains(SYNTH, '"doc_a.docx": ("docx", {', "DOCX spec is shipped");
  contains(SYNTH, '"slides_b.pptx": ("pptx", {', "PPTX spec is shipped");
  contains(SYNTH, '"nested/sheet_c.xlsx": ("xlsx", {', "XLSX spec is shipped");
  contains(SYNTH, 'BUILDERS = {"docx": _docx, "pptx": _pptx, "xlsx": _xlsx}',
    "one builder per format is registered");

  var synth = h.readRepo(SYNTH);
  var docxBody = span(synth, "def _docx(", "def _pptx(", "DOCX builder body");
  var pptxBody = span(synth, "def _pptx(", "def _xlsx(", "PPTX builder body");
  var xlsxBody = span(synth, "def _xlsx(", "BUILDERS = ", "XLSX builder body");
  [docxBody, pptxBody, xlsxBody].forEach(function (body) {
    h.assert(body.indexOf("return _zip(") !== -1, "each builder returns a stored ZIP package");
  });
  h.assert(synth.indexOf("zipfile.ZIP_STORED") !== -1, "builders emit deterministic stored ZIPs");
  contains(SYNTH, "def write_corpus(", "corpus writer is shipped");

  // The Rust round-trip contract drives DOCX/PPTX fixtures + an XLSX module.
  contains(CONTRACT, 'pub const PPTX_FIXTURE: &str = "Rice_and_Beans_Nutrition.pptx";',
    "PPTX round-trip fixture is pinned");
  contains(CONTRACT, 'pub const DOCX_FIXTURE: &str = "Alex_Rivera_Resume_Futuristic.docx";',
    "DOCX round-trip fixture is pinned");
  contains(CONTRACT, '#[path = "office_fidelity/xlsx.rs"]', "XLSX round-trip module is pinned");
  contains(CONTRACT, "pub fn part_hashes(", "per-part comparison is shipped");

  // The JS harness carries its own deterministic three-format corpus.
  var corpus = h.readRepo(CORPUS);
  h.assert(corpus.indexOf("function docParts()") !== -1 &&
    corpus.indexOf("function pptParts()") !== -1 &&
    corpus.indexOf("function xlsxParts()") !== -1,
    "all three corpus builders must ship");
  ["word/document.xml", "ppt/slides/slide1.xml", "xl/worksheets/sheet1.xml"].forEach(function (part) {
    h.assert(corpus.indexOf('"' + part) !== -1, "corpus must carry " + part);
  });
  contains(README, "how much structural fidelity survived?", "comparator scope is documented");
});

h.staticTest("#2457 fidelity comparator and results-derived coverage matrix ship as offline test tooling (no published dashboard claimed)", function () {
  // Non-vacuity: the comparator renders per-feature differences and a score.
  contains(README, "OOXML round-trip feature comparator (#2457)", "comparator is shipped");
  ["| DOCX | paragraphs, tables,", "| PPTX | slides, shapes, tables,",
    "| XLSX | sheets, rows, cells,"].forEach(function (row) {
      contains(README, row, "feature schema row is shipped");
    });
  contains(REPORT, '"# Office round-trip fidelity report (#2457)"', "report renderer ships");
  contains(REPORT, '"## Feature differences"', "per-feature diff section ships");
  contains(REPORT, '"| Path | Feature | Type | Original | Round-tripped | Score | Status |"',
    "per-feature diff table ships");

  // The test-only matrix is DERIVED from recorded results, not hand-written.
  contains(MATRIX, "function buildMatrix(results)", "matrix builder ships");
  contains(MATRIX, "features.FEATURES[kind]", "matrix iterates the fixed feature schema");
  contains(MATRIX,
    'status = bucket.fail ? "failing" : bucket.degraded ? "degraded" : bucket.pass ? "supported" : "untested";',
    "status vocabulary is derived from evidence");
  contains(MATRIX, "h.setCoverage(buildMatrix(h.results()));", "builder consumes recorded results");
  contains(HARNESS, "function printCoverage()", "harness prints the matrix");
  contains(HARNESS, '"FEATURE COVERAGE MATRIX (derived from PASS/DEGRADED/FAIL results)"',
    "matrix header declares evidence derivation");

  // Non-vacuity: the matrix source must be substantial, and other statuses
  // besides "supported" must remain reachable in its expressions.
  h.assert(h.readRepo(MATRIX).length > 1000, "matrix module must be non-trivial");
  ["bucket.fail", "bucket.degraded", "bucket.pass"].forEach(function (token) {
    h.assert(h.readRepo(MATRIX).indexOf(token) !== -1, "matrix must account for " + token);
  });
});

h.staticTest("#2457 importer fuzz entrypoint, seeded corpus and PR fallback ship (nightly/deep run still out of scope)", function () {
  // Non-vacuity: a real libfuzzer target calls the shipped importer.
  contains(FUZZ_TARGET, "#![no_main]", "libfuzzer binary is shipped");
  contains(FUZZ_TARGET, "fuzz_target!(|data: &[u8]| {", "fuzz entrypoint is shipped");
  contains(FUZZ_TARGET, "botsheet::storage::import::import_spreadsheet_bytes(data, name",
    "real importer is the fuzz target");
  contains(FUZZ_TARGET, 'for name in ["fuzz.xlsx", "fuzz.xls", "fuzz.ods", "fuzz.csv", "fuzz"] {',
    "all importer labels are fuzzed");
  contains(FUZZ_CARGO, 'name = "office_import"', "office target is registered");
  contains(FUZZ_CARGO, 'libfuzzer-sys = "0.4"', "libfuzzer dependency is shipped");
  contains(FUZZ_CARGO, "cargo-fuzz = true", "detached fuzz package is shipped");

  // Seeds on disk, plus the deterministic per-PR fallback.
  var seeds = h.fs.readdirSync(h.path.join(h.REPO_DIR, SEED_DIR)).filter(function (name) {
    return h.fs.statSync(h.path.join(h.REPO_DIR, SEED_DIR, name)).size > 0;
  });
  h.assert(seeds.length >= 2, "at least two non-empty fuzz seeds must ship, found " + seeds.length);
  h.assert(seeds.indexOf("sheet.csv") !== -1 && seeds.indexOf("zip-magic.xlsx") !== -1,
    "expected office fuzz seeds must ship, found " + seeds.join(" | "));
  contains(FUZZ_STABLE, "const ROUNDS: usize = 256;", "PR fallback runs seeded mutations");
  contains(FUZZ_STABLE, "office import: any byte sequence either imports or returns",
    "fail-closed importer property is documented");
});

// Criterion 6 is genuinely absent: there is no shipped cross-browser smoke
// matrix. The fn below ASSERTS the degraded shape; a fix that removes the
// "Not implemented" row or the harness gap note turns this into a FAIL,
// forcing an honest update instead of a silent PASS.
h.knownBug("#2457 four-browser smoke matrix is absent and documented as such", function () {
  var book = h.readRepo(BOOK);
  contains(BOOK,
    "| Cross-browser smoke matrix (Chrome, Firefox, Edge, Safari) for open/edit/save/export | Not implemented",
    "book must disclose the four-browser gap");
  contains(BOOK, "**That lab does not exist yet.**", "book must not claim the QA lab exists");
  contains(MATRIX, "four-browser smoke matrix", "harness must disclose the same gap");

  // No browser-project config ships inside the office harness itself.
  h.assert(!h.fileExistsRepo("playwright.config.js") &&
    !h.fileExistsRepo("playwright.config.ts"), "no repo-root playwright project config");
  var scanned = 0;
  var modules = h.fs.readdirSync(h.path.join(h.REPO_DIR,
    "botui/ui/suite/tests/webtest-office-fidelity"));
  modules.forEach(function (name) {
    if (name === "42_fidelity_coverage_matrix.js" || name.slice(-3) !== ".js") return;
    var src = h.fs.readFileSync(h.path.join(h.REPO_DIR,
      "botui/ui/suite/tests/webtest-office-fidelity", name), "utf8");
    scanned++;
    h.assert(!/firefox|webkit|msedge/i.test(src), name + " must not claim a cross-browser project");
  });
  h.assert(scanned > 20, "non-vacuity: must scan the shipped harness modules, scanned " + scanned);
});

h.note("GAP #2457: licensed 100+ files/format corpus, visual raster diff, PR/nightly dashboard and 24h fuzz run remain unimplemented; only narrow offline artifacts ship.");
