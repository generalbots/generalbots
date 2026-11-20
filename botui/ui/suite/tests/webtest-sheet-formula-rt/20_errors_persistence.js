/**
 * Error semantics, preview settings persistence, deferred Rust/CI criteria
 * ([STATIC] only) and observed defects (KNOWN BUG, report-only).
 */
"use strict";

var h = require("./00_harness");
var e = h.engine;
var E = e.evaluator;
var P = e.parser;
var S = e.settings;
var F = e.financial;
var D = e.date;
var assert = h.assert;
var approx = h.approx;
var errCode = h.errCode;
var assertError = h.assertError;

h.test("numeric helpers reject text but treat blanks as zero", function () {
  assertError(e.errors.num("3"), "#VALUE!", "text was coerced to a number");
  assert(e.errors.num(undefined) === 0 && e.errors.num(null) === 0,
    "blank cell is not zero");
  assert(e.errors.num(true) === 1, "boolean coercion changed");
  assertError(e.errors.num(Infinity), "#NUM!", "non-finite number was accepted");
});

h.test("error propagation is sticky through nested values and arithmetic", function () {
  var nested = e.errors.firstError([1, [2, e.errors.err("#N/A")]]);
  assertError(nested, "#N/A", "firstError missed a nested error");
  assertError(e.errors.divide(1, 0), "#DIV/0!", "division by zero changed");
  assertError(e.errors.op("+", e.errors.err("#REF!"), 1), "#REF!", "operator dropped an error");
  assertError(E.evaluate("=SUM([1,1/0])"), "#DIV/0!", "SUM swallowed an array error");
  assertError(F.PMT([e.errors.err("#VALUE!"), 12, 1000]), "#VALUE!",
    "financial family swallowed an error argument");
});

h.test("IFERROR / IFNA / ISERROR / ISNA keep Excel error semantics", function () {
  assert(E.evaluate("=IFERROR(1/0,9)") === 9, "IFERROR did not consume #DIV/0!");
  assert(E.evaluate("=IFNA(#N/A,9)") === 9, "IFNA did not consume #N/A");
  assertError(E.evaluate("=IFNA(1/0,9)"), "#DIV/0!", "IFNA consumed a non-#N/A error");
  assert(E.evaluate("=ISERROR(1/0)") === true, "ISERROR on #DIV/0! changed");
  assert(E.evaluate("=ISNA(#N/A)") === true, "ISNA on #N/A changed");
  assert(E.evaluate("=ISNA(1/0)") === false, "ISNA on #DIV/0! changed");
  assert(E.evaluate("=IF(TRUE,1,1/0)") === 1, "unused IF branch was evaluated");
});

h.test("round-trip settings keys mirror the preview wire shape", function () {
  var defaults = S.defaults();
  ["version", "localOnly", "mode", "iterative", "precisionAsDisplayed",
    "dateSystem", "locale", "separators"].forEach(function (key) {
    assert(Object.prototype.hasOwnProperty.call(defaults, key), "missing default key " + key);
  });
  assert(defaults.localOnly === true, "preview must stay local-only");
  assert(defaults.iterative.maxIterations === 100 && defaults.iterative.tolerance === 1e-7,
    "iterative defaults changed");
  assert(S.storageKey() === "gb-sheet-functions:" + e.fixedSheetId,
    "preview storage key is not workbook-scoped by the fixed uid");
});

h.test("applyForm -> localStorage -> state restores every calculation setting", function () {
  var before = S.state();
  try {
    var form = {
      mode: "manual",
      iterative: true,
      maxIterations: 250,
      tolerance: 1e-9,
      precisionAsDisplayed: true,
      dateSystem: "1904",
      locale: "pt-BR",
      separators: { decimal: ",", group: "." },
    };
    var applied = S.applyForm(form);
    assert(applied, "applyForm failed to persist");
    var restored = S.state();
    assert(restored.mode === "manual" && restored.precisionAsDisplayed === true,
      "mode or precision was not restored");
    assert(restored.dateSystem === "1904", "date system was not restored");
    assert(restored.locale === "pt-BR" && restored.separators.decimal === "," &&
      restored.separators.group === ".", "locale separators were not restored");
    assert(restored.iterative.enabled === true && restored.iterative.maxIterations === 250 &&
      restored.iterative.tolerance === 1e-9, "iterative settings were not restored");
    var raw = JSON.parse(e.sandbox.localStorage.getItem(S.storageKey()));
    assert(raw.dateSystem === "1904" && raw.iterative.maxIterations === 250,
      "raw localStorage payload does not hold the settings");
  } finally {
    S.save(before);
  }
});

h.test("normalize clamps iterative bounds and rejects invalid separators", function () {
  var normalized = S.normalize({
    iterative: { enabled: true, maxIterations: 5000, tolerance: -1 },
    separators: { decimal: ",", group: "," },
    locale: "pt-BR",
  });
  assert(normalized.iterative.maxIterations === 1000, "maxIterations was not clamped");
  assert(normalized.iterative.tolerance === 1e-7, "tolerance was not defaulted");
  assert(normalized.separators.decimal === "," && normalized.separators.group === ".",
    "equal separator marks were not replaced with the locale pair");
  assert(S.normalize({ locale: "xx" }).locale === S.defaults().locale,
    "unknown locale was accepted");
});

h.test("setLocale updates separators, emits an event and reloads i18n", function () {
  var before = S.state();
  try {
    assert(S.setLocale("pt-BR") === true, "setLocale rejected a supported locale");
    assert(S.state().separators.decimal === ",", "setLocale kept the old decimal mark");
    var last = e.events[e.events.length - 1];
    assert(last && last.type === "gb-sheet-functions-locale" && last.detail.locale === "pt-BR",
      "locale change event was not emitted");
    assert(e.i18nLoads[e.i18nLoads.length - 1] === "pt-BR", "i18n reload was not requested");
    assert(S.setLocale("xx") === false && S.state().locale === "pt-BR",
      "unknown locale changed the state");
  } finally {
    S.save(before);
    e.window.SheetI18n.load("en");
  }
});

h.test("sheet id falls back to the loaded workbook when the core id is empty", function () {
  var core = e.window.SheetCore;
  var original = core.currentSheetId;
  e.window.__LOADED_SHEET = { id: "fallback-id" };
  core.currentSheetId = function () { return ""; };
  try {
    assert(S.sheetId() === "fallback-id", "workbook fallback id was ignored");
  } finally {
    core.currentSheetId = original;
    delete e.window.__LOADED_SHEET;
  }
});

h.test("date functions read persisted settings on every call", function () {
  var a = e.withPersistedSettings({ dateSystem: "1900", locale: "en" }, function () {
    return D.serial("2026-01-01");
  });
  var b = e.withPersistedSettings({ dateSystem: "1904", locale: "en" }, function () {
    return D.serial("2026-01-01");
  });
  assert(a - b === 1462, "persisted date system was not re-read per call");
  assert(S.state().dateSystem === S.defaults().dateSystem, "settings were not restored");
});

h.staticTest("preview settings stay browser-local (no server round-trip)", function () {
  h.sourceHas("79_functions_settings.js", "localStorage", "settings module lost localStorage");
  h.sourceLacks("79_functions_settings.js", "fetch\\(", "preview unexpectedly calls fetch");
  h.sourceLacks("79_functions_settings.js", "/api/", "preview unexpectedly calls an API");
});

h.staticTest("server settings type mirrors the preview wire shape", function () {
  var src = h.repoFile("botserver/crates/botsheet-core/src/types/workbook_settings.rs");
  ["pub struct WorkbookSettings", "pub enum FormulaLocale", "argument_separator",
    "settings_from_workbook_json", "embed_settings_in_workbook_json", "local_only",
  ].forEach(function (token) {
    assert(src.indexOf(token) >= 0, "workbook_settings.rs is missing " + token);
  });
});

h.staticTest("server settings store is keyed per workbook", function () {
  var src = h.repoFile("botserver/crates/botsheet-core/src/types/workbook_settings_store.rs");
  assert(src.indexOf("pub fn set_workbook_settings") >= 0, "store has no writer");
  assert(src.indexOf("pub fn workbook_settings") >= 0, "store has no reader");
  assert(src.indexOf("HashMap<String, WorkbookSettings>") >= 0, "store is not per-workbook");
});

h.staticTest("workbook save/load hooks embed and restore settings", function () {
  var src = h.repoFile("botserver/crates/botsheet-core/src/state/persistence.rs");
  assert(src.indexOf("embed_settings_in_workbook_json") >= 0, "save hook does not embed settings");
  assert(src.indexOf("settings_from_workbook_bytes") >= 0, "load hook does not restore settings");
});

h.staticTest("XLSX export canonicalizes formulas with workbook settings", function () {
  var src = h.repoFile("botserver/crates/botsheet/src/export/xlsx.rs");
  assert(src.indexOf("to_xlsx_formula") >= 0, "export does not canonicalize formulas");
  assert(src.indexOf("workbook_settings") >= 0, "export ignores stored locale settings");
});

h.staticTest("Rust codec corpus covers errors, epochs and iteration", function () {
  var src = h.repoFile("botserver/crates/botsheet-core/src/engine/xlsx_codec/tests.rs");
  ["test_error_literals_are_canonicalized_and_sticky",
    "test_both_epochs_round_trip_with_1900_leap_quirk_and_fixed_offset",
    "test_iterative_settings_clamp_and_recurrence_converges",
  ].forEach(function (name) {
    assert(src.indexOf(name) >= 0, "codec corpus is missing " + name);
  });
  var differential = h.repoFile("botserver/crates/botsheet-core/src/formulas/differential_tests.rs");
  assert(differential.indexOf("#[test]") >= 0, "differential corpus has no tests");
});

h.staticTest("DEFERRED GAP: XLSX writer does not apply the 1904 date system", function () {
  var files = [
    "botserver/crates/botsheet/src/export/xlsx.rs",
    "botserver/crates/botsheet/src/storage/xlsx_write.rs",
  ];
  files.forEach(function (file) {
    assert(h.repoFile(file).indexOf("set_date1904") < 0,
      file + " now applies the 1904 flag; re-check this deferred criterion");
  });
  h.note("source-level gap only: no runtime claim is made here");
});

h.staticTest("DEFERRED GAP: botsheet-core unit tests are absent from the CI matrix", function () {
  var ci = h.repoFile(".github/workflows/ci.yml");
  assert(ci.indexOf("botsheet-core") < 0,
    "botsheet-core entered the CI matrix; re-check this deferred criterion");
  h.note("cargo test -p botsheet-core remains a CI-only acceptance criterion");
});

h.knownBug("semicolon argument separator is not tokenized", function () {
  var ast = P.parse("=SOMA(1;2)");
  var value = E.evaluate("=SOMA(1;2)", { locale: "pt-BR" });
  assert(ast === null, "separator was unexpectedly accepted; defect may be fixed");
  assertError(value, "#VALUE!", "separator defect changed its observable error");
  h.note("observe: pt-BR alias SOMA resolves, but ';' yields #VALUE! " +
    "(78a_functions_parser.js:60 token set; cf. FormulaLocale::argument_separator)");
});

h.knownBug("decimal-comma literal is silently split into extra arguments", function () {
  var formatted = E.format(1.5, { settings: { separators: { decimal: ",", group: "." } } });
  assert(formatted === "1,5", "pt-BR format of 1.5 changed");
  var ast = P.parse("=ROUND(1,5,2)");
  assert(ast && ast.t === "call" && ast.args.length === 3,
    "comma was not treated as an argument separator; defect may be fixed");
  assert(E.evaluate("=ROUND(1,5,2)") === 1, "wrong-value result changed");
  h.note("observe: '=ROUND(1,5,2)' returns 1 instead of 1.5; no #VALUE! is raised");
});

h.knownBug("grouped format output cannot be parsed back", function () {
  var text = E.format(1234.5);
  assert(text === "1,234.5", "default formatted output changed");
  assert(P.parse(text) === null, "grouped output was unexpectedly parseable");
  assertError(E.evaluate(text), "#VALUE!", "grouped output defect changed its observable error");
  h.note("observe: format() emits ',' grouping that tokenize() reads as an argument separator");
});

h.knownBug("iterative zero seed aborts reciprocal recurrences", function () {
  var result = E.evaluateCircular({ A1: "=(A1+2/A1)/2" }, {
    settings: { iterative: { enabled: true, maxIterations: 100, tolerance: 1e-9 } },
  });
  assertError(result.error, "#DIV/0!", "reciprocal recurrence defect changed");
  assert(result.converged === false && result.iterations === 1,
    "reciprocal recurrence unexpectedly converged; defect may be fixed");
  h.note("observe: cache seeds each cell with 0, so 2/A1 divides by zero on pass 1 " +
    "(78_functions_evaluator.js:235); Babylonian sqrt never converges");
});

h.knownBug("absolute references with $ are not tokenized", function () {
  ["=$A$1+1", "=$A1+1", "=A$1+1"].forEach(function (formula) {
    assert(P.parse(formula) === null, "absolute reference was unexpectedly parsed");
    assertError(E.evaluate(formula, { cells: { A1: 2 } }), "#VALUE!",
      "absolute reference defect changed its observable error");
  });
  assert(E.evaluate("=A1+1", { cells: { A1: 2 } }) === 3, "relative reference regressed");
  h.note("observe: CELL_RE and columnOf/rowOf strip '$', but tokenize() rejects '$' " +
    "(78a_functions_parser.js:60) — every $-qualified ref becomes #VALUE!");
});
