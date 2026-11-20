/**
 * [STATIC] source assertions and KNOWN BUG observations (#2490).
 *
 * XLSX-native chart parts, server named ranges, templates, axis formats,
 * vector/print export are NOT shipped: the eight charting modules declare
 * themselves local-only. Those criteria are asserted against SOURCE ONLY and
 * are never reported as runtime PASS. KNOWN BUG tests execute shipped code and
 * pin observed behaviour for the driver to report; they never patch production.
 */
"use strict";

var h = require("./00_harness");
var e = h.engine;
var test = h.test;
var staticTest = h.staticTest;
var knownBug = h.knownBug;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;
var note = h.note;
var source = h.source;

function textValues(parts) {
  return parts.filter(function (p) { return p.indexOf("<text ") === 0; }).map(function (p) {
    var open = p.indexOf(">");
    var close = p.lastIndexOf("</text>");
    return close > open ? p.slice(open + 1, close) : "";
  });
}

// ── [STATIC] XLSX-native chart round-trip is explicitly out of scope ────
staticTest("shipped modules declare themselves local-only for XLSX", function () {
  var src = source("36_charting_render.js");
  assert(/no XLSX chart-part round trip/.test(src), "local-only disclaimer missing");
  assert(/def\.localOnly = true/.test(src), "addLocal does not mark charts local-only");
  assert(/sheet-charting-local-only/.test(src), "local-only badge key missing");
});
staticTest("no shipped charting module emits native XLSX chart XML", function () {
  h.ORDER.forEach(function (fileName) {
    h.sourceLacks(fileName, "c:chart|xl/charts|chartSpace|chartPart|drawing\\.xml|workbook\\.xml",
      fileName + " contains native chart XML handling");
  });
});
staticTest("no shipped charting module persists a chart to the server", function () {
  h.ORDER.forEach(function (fileName) {
    h.sourceLacks(fileName, "fetch\\(|XMLHttpRequest|/api/sheet/chart|localStorage\\.setItem",
      fileName + " contains a server/storage write path");
  });
});
staticTest("alt text and a data-table fallback are shipped locally", function () {
  var svgSrc = source("33_charting_svg.js");
  var renderSrc = source("36_charting_render.js");
  assert(/role="img"/.test(svgSrc) && /aria-label=/.test(svgSrc), "SVG root not accessible");
  assert(/sheet-charting-alt/.test(renderSrc), "alt key missing");
  assert(/sheet-charting-data-table/.test(renderSrc), "data-table key missing");
  assert(/createElement\("table"\)/.test(renderSrc), "data table not built");
});

// ── [STATIC] acceptance criteria deliberately not shipped ──────────────
staticTest("server-side dynamic named ranges and table columns are deferred", function () {
  h.ORDER.forEach(function (fileName) {
    h.sourceLacks(fileName, "definedName|defined_names|namedRange|tableColumn",
      fileName + " resolves a server named range or table column");
  });
  h.sourceHas("31_charting_data.js", "function bindRange\\(text, options\\)",
    "local range binder missing");
});
staticTest("chart templates and board styling are deferred", function () {
  h.ORDER.forEach(function (fileName) {
    h.sourceLacks(fileName, "applyTemplate|chartTemplate|boardChart",
      fileName + " applies a chart template");
  });
  h.sourceHas("32_charting_palette.js", "const CVD = \\[", "colour-blind palette missing");
});
staticTest("number/date axis formats are deferred", function () {
  h.ORDER.forEach(function (fileName) {
    h.sourceLacks(fileName, "numFmt|axisFormat|dateAxis|tickFormat",
      fileName + " implements an axis number/date format");
  });
});
staticTest("vector PDF, chosen-DPI PNG and print parity are deferred", function () {
  h.ORDER.forEach(function (fileName) {
    h.sourceLacks(fileName, "toDataURL|jsPDF|dpi\\s*=|page-for-page|printParity",
      fileName + " implements vector/raster export or print parity");
  });
});

// ── KNOWN BUG: report only, never patched ──────────────────────────────
knownBug("null data points become zero bars and inflate error bars", function () {
  var def = e.seriesDef([{ label: "s", data: [10, null, 30] }], ["a", "b", "c"]);
  def.analysis = { errorBars: true };
  var ctx = e.ctxFor(def);
  deep(ctx.model.series[0].data, [10, 0, 30], "model silently coerced null to zero");
  var stats = e.data.stats([10, null, 30]);
  equal(stats.count, 3);
  assert(Math.abs(stats.sd - 12.47219128924647) < 1e-9, "unexpected null-inflated sd");
  var parts = e.overlays.apply(ctx, [], e.mapFor(ctx));
  equal(parts.filter(function (p) { return p.indexOf("<line ") === 0; }).length, 9,
    "expected three error-bar segments for all three points");
  note("31_charting_data.js:58 numbers() coerces null via Number(null)=0; " +
    "35_charting_overlays.js:51 null guard never sees a null after model() — report only");
});
knownBug("a zero variance target is rejected as falsy", function () {
  var def = e.seriesDef([{ label: "s", data: [-5, 5] }], ["a", "b"]);
  def.analysis = { variance: true, referenceLines: [{ value: 0 }] };
  var ctx = e.ctxFor(def);
  var parts = e.overlays.apply(ctx, [], e.mapFor(ctx));
  var callouts = textValues(parts).filter(function (value) { return value.indexOf("%") !== -1; });
  equal(callouts.length, 0, "unexpected variance callout for a valid zero target");
  assert(textValues(parts).indexOf("0") !== -1, "reference line itself should still render");
  note("35_charting_overlays.js:106 `|| !target` suppresses variance callouts at target=0 — report only");
});
