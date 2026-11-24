/**
 * #2424 offline verification of docs/modules/75..79_layout_*.js.
 *
 * The five modules are IIFEs extending window.DocsLayout. They are loaded in
 * order into ONE vm sandbox with stubbed window/document/gbT/localStorage:
 * no module touches the DOM at load time, so every exported pure helper is
 * reachable as genuine runtime behaviour. Only genuinely private behaviour
 * (the "total comes from computed pages, not a stored counter" wiring) is
 * covered by clearly-labelled STATIC source assertions. No production logic is
 * reimplemented here; all fixtures feed the real helpers.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var readFile = h.readFile;
var loadModules = h.loadModules;
var deepEqual = h.deepEqual;

var MODULES = [
  "docs/modules/75_layout_state.js",
  "docs/modules/76_layout_geometry.js",
  "docs/modules/77_layout_rules.js",
  "docs/modules/78_layout_paginate.js",
  "docs/modules/79_layout_headers.js",
];
var SOURCE_PAGINATE = "docs/modules/78_layout_paginate.js";
var SOURCE_HEADERS = "docs/modules/79_layout_headers.js";

/** Fresh module instance with deterministic DOM/i18n/localStorage stubs. */
function fixture() {
  var store = {};
  var sandbox = {
    window: {
      gbT: function (key, fallback) { return fallback; },
      localStorage: {
        getItem: function (key) {
          return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
        },
        setItem: function (key, value) { store[key] = String(value); },
      },
    },
    document: {
      createElement: function () { return { setAttribute: function () {}, style: {} }; },
      getElementById: function () { return null; },
      querySelector: function () { return null; },
      addEventListener: function () {},
    },
    console: console,
  };
  var ctx = loadModules(MODULES, sandbox);
  return { NS: ctx.window.DocsLayout, ctx: ctx, store: store };
}

/** A fixed, synthetic line list: one independent paragraph per line. */
function fixedLines(count, prefix) {
  var tag = prefix || "p";
  var out = [];
  for (var i = 0; i < count; i += 1) {
    var id = tag + (i + 1);
    out.push({ id: id, kind: "paragraph", paraId: id, text: "Line " + (i + 1) });
  }
  return out;
}

function pageOf(NS, lines, geometry) {
  return {
    index: 0,
    geometry: geometry || NS.resolveGeometry(NS.defaultSetup(), {}, 1),
    lines: lines || [],
    footnotes: [],
    keepRulesFired: [],
  };
}

// ── Load / export surface ────────────────────────────────────────────

test("RUNTIME #2424: all five layout modules load and export the pure API", function () {
  var NS = fixture().NS;
  [
    "substitute", "esc", "focusMove", "defaultSetup", "defaultSections",
    "resolveGeometry", "lineMetrics", "buildSections", "linesFromBlocks",
    "paragraphRuns", "keepRuns", "sumHeights", "capacity", "used", "fits",
    "breakDecision", "applyWidowOrphan", "normalizeBoxes", "footnoteOf",
    "pageSignature", "projectInput", "paginate", "repaginate",
    "pageVariant", "variantValue", "resolveFields", "resolvePageChrome",
  ].forEach(function (name) {
    assert(typeof NS[name] === "function", name + " must be exported on DocsLayout");
  });
  assert(NS.PAPER.a4.width === 794 && NS.PAPER.letter.height === 1056,
    "paper table must expose deterministic A4/Letter sizes");
});

test("RUNTIME #2424: tr/substitute resolve placeholders deterministically", function () {
  var NS = fixture().NS;
  assert(NS.substitute("{$n}/{$n}", { n: 7 }) === "7/7", "substitute must replace all placeholders");
  assert(NS.substitute("{ $title } - { $author }", { title: "T", author: "A" }) === "T - A",
    "substitute must accept spaced named tokens");
  assert(NS.tr("docs-layout-section-label", "Section { $n }", { n: 2 }) === "Section 2",
    "tr with a fallback translator must still substitute params");
  assert(NS.esc('<a href="x">&\'') === "&lt;a href=&quot;x&quot;&gt;&amp;&#39;",
    "esc must escape markup characters");
});

test("RUNTIME #2424: focusMove is a pure focus-trap contract", function () {
  var NS = fixture().NS;
  var keys = ["a", "b", "c"];
  assert(NS.focusMove(keys, "b", { key: "Tab" }).key === "c", "Tab advances focus");
  assert(NS.focusMove(keys, "a", { key: "Tab", shiftKey: true }).key === "c", "Shift+Tab wraps back");
  assert(NS.focusMove(keys, "missing", { key: "Tab" }).index === 0, "escaped focus returns to first");
  assert(NS.focusMove(keys, "a", { key: "Escape" }).action === "close", "Escape closes");
  assert(NS.focusMove([], null, { key: "Tab" }).action === "none", "empty key set is a no-op");
});

// ── Geometry ─────────────────────────────────────────────────────────

test("RUNTIME #2424: page geometry is deterministic for a fixed section config", function () {
  var NS = fixture().NS;
  var a = NS.resolveGeometry(NS.defaultSetup(), {}, 1);
  var b = NS.resolveGeometry(NS.defaultSetup(), {}, 1);
  assert(deepEqual(a, b), "identical setup must produce identical geometry");
  assert(a.paper === "a4" && a.width === 794 && a.height === 1123, "A4 portrait size");
  assert(a.contentWidth === 602 && a.columnWidth === 602 && a.cols.length === 1, "single-column content width");
  assert(a.bodyTop === 128 && a.bodyBottom === 987 && a.bodyHeight === 859,
    "body box accounts for top/bottom margins plus header/footer");
});

test("RUNTIME #2424: orientation, columns, gutter and mirrored margins resolve", function () {
  var NS = fixture().NS;
  var land = NS.resolveGeometry(NS.defaultSetup(), { orientation: "landscape" }, 1);
  assert(land.width === 1123 && land.height === 794 && land.orientation === "landscape",
    "landscape swaps width/height");

  var two = NS.resolveGeometry(NS.defaultSetup(), { columns: 2 }, 1);
  assert(two.cols.length === 2 && two.columnWidth === 285 && two.columnGap === 32,
    "two columns split contentWidth minus the gap");
  assert(two.cols[0].left === 96 && two.cols[1].left === 413,
    "column origins advance by columnWidth + gap");
  assert(NS.capacity(two) === 1718, "capacity is the sum of column heights");

  var gutter = NS.resolveGeometry(NS.defaultSetup(), { gutter: 20 }, 1);
  assert(gutter.marginLeft === 116 && gutter.marginRight === 96 && gutter.gutter === 20,
    "gutter widens the left margin and is preserved in the resolved geometry");

  var mirrorOpts = { marginLeft: 72, marginRight: 120, mirrorMargins: true, gutter: 20 };
  var mOdd = NS.resolveGeometry(NS.defaultSetup(), mirrorOpts, 1);
  var mEven = NS.resolveGeometry(NS.defaultSetup(), mirrorOpts, 2);
  assert(mOdd.marginLeft !== mEven.marginLeft && mOdd.marginRight !== mEven.marginRight,
    "mirrored margins produce different inner/outer edges on odd and even pages");
  /* Regression (#2424): the gutter must sit on the inner edge. Odd page inner
   * is left (72+20=92); even page inner is right after the swap (72+20=92). */
  assert(mOdd.marginLeft === 92 && mOdd.marginRight === 120,
    "odd page gutter on inner left edge");
  assert(mEven.marginLeft === 120 && mEven.marginRight === 92,
    "even page gutter on inner right edge");
  var clamped = NS.resolveGeometry(NS.defaultSetup(), { columns: 9, columnGap: -5 }, 1);
  assert(clamped.cols.length === 3 && clamped.columnGap === 0, "columns clamp to 1..3 and gap to >= 0");
});

test("RUNTIME #2424: lineMetrics uses deterministic kind factors", function () {
  var NS = fixture().NS;
  var geo = NS.resolveGeometry(NS.defaultSetup(), {}, 1);
  assert(NS.lineMetrics({ kind: "heading1" }, geo).height > NS.lineMetrics({ kind: "paragraph" }, geo).height,
    "heading1 is taller than paragraph");
  assert(NS.lineMetrics({ kind: "footnote" }, geo).height < NS.lineMetrics({ kind: "paragraph" }, geo).height,
    "footnote is shorter than paragraph");
  assert(NS.lineMetrics({ kind: "image", imageAnchor: "float" }, geo).height === 0,
    "floating image reserves no vertical flow");
  assert(NS.lineMetrics({ kind: "paragraph", height: 99 }, geo).height === 99,
    "explicit height is honoured");
});

test("RUNTIME #2424: buildSections sorts by startLine and guarantees a section at 0", function () {
  var NS = fixture().NS;
  var built = NS.buildSections([
    { id: "later", startLine: 5 },
    { id: "first", startLine: 0 },
  ], []);
  assert(built[0].id === "first" && built[1].id === "later", "sections sorted by startLine");
  var injected = NS.buildSections([{ id: "late", startLine: 3 }], []);
  assert(injected.length === 2 && injected[0].startLine === 0, "a missing section at 0 is injected");
  assert(NS.buildSections([], []).length === 1, "empty sections yields the default single section");
});

test("RUNTIME #2424: linesFromBlocks maps typed blocks to line metadata", function () {
  var NS = fixture().NS;
  var lines = NS.linesFromBlocks([
    { id: "h", type: "heading", spans: [{ text: "Head" }] },
    { id: "img", type: "image", spans: [], attrs: { anchor: "float" } },
    { id: "pb", type: "page-break", spans: [] },
    { id: "sb", type: "section-break", spans: [] },
  ]);
  assert(lines[0].kind === "heading1" && lines[0].keepWithNext === true, "heading maps to keep-with-next");
  assert(lines[1].imageAnchor === "float", "image anchor is preserved");
  assert(lines[2].forceBreak === true && lines[3].forceSection === true, "break blocks are flagged");
});

// ── Keep rules ───────────────────────────────────────────────────────

test("RUNTIME #2424: paragraph/keep runs group consecutive lines by key", function () {
  var NS = fixture().NS;
  var lines = [
    { id: "a", paraId: "p1" }, { id: "b", paraId: "p1" }, { id: "c", paraId: "p2" },
  ];
  assert(NS.paragraphRuns(lines).length === 2, "two paragraph runs");
  assert(NS.paragraphRuns(lines)[0].ids.join("") === "ab", "run collects member ids");
  assert(NS.keepRuns([{ id: "x", keepGroup: "g" }, { id: "y", keepGroup: "g" }])[0].end === 1,
    "keep run spans consecutive members");
});

test("RUNTIME #2424: used/fits agree with capacity", function () {
  var NS = fixture().NS;
  var geo = NS.resolveGeometry(NS.defaultSetup(), {}, 1);
  var page = pageOf(NS, [{ metrics: { height: 400 } }, { metrics: { height: 200 } }], geo);
  assert(NS.used(page) === 600, "used sums line heights");
  assert(NS.capacity(geo) === 859, "capacity from columns");
  assert(NS.fits(page, 259) === true && NS.fits(page, 260) === false, "fits respects remaining capacity");
});

test("RUNTIME #2424: breakDecision reports overflow and keep-rule firings", function () {
  var NS = fixture().NS;
  var geo = NS.resolveGeometry(NS.defaultSetup(), {}, 1);
  var empty = pageOf(NS, [], geo);
  assert(NS.breakDecision({ metrics: { height: 900 } }, null, empty, {}, {}).rule === "page-overflow",
    "an over-tall line breaks with page-overflow");
  var near = pageOf(NS, [{ metrics: { height: 840 } }], geo);
  assert(NS.breakDecision({ metrics: { height: 27 }, keepWithNext: true },
    { metrics: { height: 27 } }, near, {}, {}).rule === "keep-with-next",
    "keep-with-next combines both heights and names its rule");
  assert(NS.breakDecision({ metrics: { height: 27 }, tableRow: true },
    null, near, {}, {}).rule === "table-row-split",
    "unsplittable table rows name their rule");
  assert(NS.breakDecision({ metrics: { height: 27 }, keepGroup: "g" },
    null, near, { g: 100 }, {}).rule === "keep-lines-together",
    "keep groups name their rule and use group height");
  assert(NS.breakDecision({ metrics: { height: 27 } }, null, empty, {}, {}).break === false,
    "a fitting line does not break");
});

test("RUNTIME #2424: applyWidowOrphan moves a split paragraph and records firings", function () {
  var NS = fixture().NS;
  var geo = NS.resolveGeometry(NS.defaultSetup(), {}, 1);
  var a = pageOf(NS, [
    { id: "a1", paraId: "pa", metrics: { height: 27 } },
    { id: "a2", paraId: "pa", metrics: { height: 27 } },
  ], geo);
  var b = pageOf(NS, [{ id: "b1", paraId: "pa", metrics: { height: 27 } }], geo);
  b.index = 1;
  var firings = NS.applyWidowOrphan([a, b], { widows: 2, orphans: 2 });
  assert(a.lines.length === 1 && a.lines[0].id === "a1", "orphan tail stays with the head");
  assert(b.lines.length === 2 && b.lines[0].id === "a2", "widow line is pulled to the next page");
  assert(firings.some(function (f) { return f.rule === "widow-control"; }), "widow-control firing recorded");
  assert(NS.applyWidowOrphan([pageOf(NS, [], geo), b], {}).length === 0, "empty page is safe");
});

test("RUNTIME #2424: normalizeBoxes assigns columns/y and handles floats", function () {
  var NS = fixture().NS;
  var geo = NS.resolveGeometry(NS.defaultSetup(), { columns: 2 }, 1);
  var page = pageOf(NS, [
    { id: "x1", metrics: { height: 27 }, lineIndex: 0 },
    { id: "x2", metrics: { height: 27 }, lineIndex: 1 },
  ], geo);
  var boxes = NS.normalizeBoxes(page);
  assert(boxes.length === 2 && boxes[0].column === 0 && boxes[1].y === 155,
    "inline boxes stack in the first column");
  assert(page.lineCount === 2, "lineCount counts inline boxes");
  var floatPage = pageOf(NS, [
    { id: "fl", metrics: { height: 50, anchor: "float", kind: "image" }, lineIndex: 0 },
  ], geo);
  NS.normalizeBoxes(floatPage);
  assert(floatPage.boxes[0].anchor === "float" && floatPage.lineCount === 0,
    "floats are anchored but excluded from the line count");
  assert(NS.footnoteOf({ footnote: { id: "f" } }).id === "f" && NS.footnoteOf({}) === null,
    "footnoteOf returns the payload or null");
});

// ── Pagination ───────────────────────────────────────────────────────

function paginateFixture(NS, overrides) {
  var opts = overrides || {};
  return NS.paginate({
    lines: opts.lines || fixedLines(70),
    setup: opts.setup || NS.defaultSetup(),
    sections: opts.sections || NS.defaultSections(),
    fields: opts.fields || { title: "Title", author: "Author", date: "Date" },
  });
}

test("RUNTIME #2424: pagination splits a fixed block list without height overflow", function () {
  var NS = fixture().NS;
  var result = paginateFixture(NS);
  assert(result.totalPages === 3 && result.sectionCount === 1, "70 lines paginate into 3 pages");
  result.pages.forEach(function (page) {
    assert(NS.used(page) <= NS.capacity(page.geometry) + 0.5, "page " + page.index + " within capacity");
    page.boxes.forEach(function (box) {
      var col = page.geometry.cols[Math.min(box.column, page.geometry.cols.length - 1)];
      assert(box.anchor === "float" || box.y + box.height <= col.bottom + 0.5,
        "boxed line fits its column box");
    });
  });
});

test("RUNTIME #2424: section breaks start a new page and can restart numbering", function () {
  var NS = fixture().NS;
  var sections = [
    { id: "s1", startLine: 0, header: { odd: "S1H { $page }" }, footer: { odd: "S1F { $page } of { $total }" } },
    {
      id: "s2", startLine: 4, restartNumbering: true, startNumber: 1, firstPageDifferent: true,
      header: { first: "FIRST", odd: "S2H { $page }" }, footer: { odd: "S2F" },
    },
  ];
  var result = paginateFixture(NS, { lines: fixedLines(8, "q"), sections: sections });
  assert(result.totalPages === 2 && result.sectionCount === 2, "section boundary creates a second page");
  assert(result.pages[1].sectionId === "s2" && result.pages[1].isSectionFirst === true,
    "the section page is marked as a section first page");
  assert(result.pages[1].number === 1, "numbering restarts at the section start number");
  assert(result.pages[0].footerLines[0] === "S1F 1 of 2",
    "footer total is the computed page count, not a stored counter");
});

test("RUNTIME #2424: footnote references stay on the page that carries them", function () {
  var NS = fixture().NS;
  var lines = fixedLines(70);
  lines[40] = {
    id: "fn", kind: "footnote", paraId: "fn", text: "note", footnote: { id: "fn1", text: "note" },
  };
  var result = paginateFixture(NS, { lines: lines });
  var notes = [];
  result.pages.forEach(function (page) { notes = notes.concat(page.footnotes); });
  assert(notes.length === 1, "exactly one footnote resolved");
  var carrier = result.pages.filter(function (page) {
    return page.lines.some(function (line) { return line.id === "fn"; });
  })[0];
  assert(notes[0].page === carrier.index, "footnote page matches its reference page");
});

test("RUNTIME #2424: identical input repaginates deterministically", function () {
  var NS = fixture().NS;
  var a = paginateFixture(NS);
  var b = paginateFixture(NS);
  assert(a.pages.map(function (p) { return p.signature; }).join("|") ===
    b.pages.map(function (p) { return p.signature; }).join("|"),
    "page signatures are identical across runs");
  assert(deepEqual(a.keepFirings, b.keepFirings), "keep firings are deterministic");
  assert(a.pages[0].signature === NS.pageSignature(a.pages[0]), "signature helper is stable");
});

test("RUNTIME #2424: repaginate reuses the unchanged page-box prefix", function () {
  var NS = fixture().NS;
  var prev = paginateFixture(NS);
  var edited = fixedLines(70);
  edited[66].text = "Edited on last page";
  var next = NS.repaginate(prev, {
    lines: edited, setup: NS.defaultSetup(), sections: NS.defaultSections(), fields: { title: "Title" },
  });
  assert(next.reusedCount === 2 && next.changedFrom === 2, "first two pages reused");
  assert(next.pages[0] === prev.pages[0] && next.pages[1] === prev.pages[1],
    "reused pages keep object identity");
  assert(next.changedPages.length === 1 && next.changedPages[0] === 2, "only the changed page is rebuilt");
});

test("RUNTIME #2424: empty and missing inputs are safe", function () {
  var NS = fixture().NS;
  assert(NS.paginate().totalPages === 1, "no-argument paginate yields one empty page");
  assert(paginateFixture(NS, { lines: [] }).pages[0].lines.length === 0, "empty line list is safe");
  assert(NS.used({}) === 0 && NS.capacity({}) === 0 && NS.fits({}, 0) === true, "empty boxes are safe");
  assert(NS.resolveFields(null, {}) === "" && NS.footnoteOf(null) === null, "null inputs are safe");
});

// ── Headers / footers ────────────────────────────────────────────────

test("RUNTIME #2424: pageVariant resolves first/even/odd variants", function () {
  var NS = fixture().NS;
  assert(NS.pageVariant({ number: 2, isSectionFirst: false }, {}) === "even", "even page");
  assert(NS.pageVariant({ number: 3, isSectionFirst: false }, {}) === "odd", "odd page");
  assert(NS.pageVariant({ number: 1, isSectionFirst: true }, { firstPageDifferent: true }) === "first",
    "different first page of a section");
  assert(NS.pageVariant({ number: 2, isSectionFirst: true }, { firstPageDifferent: false }) === "even",
    "isSectionFirst alone does not force the first variant");
});

test("RUNTIME #2424: variantValue falls back from first to odd to default", function () {
  var NS = fixture().NS;
  assert(NS.variantValue({ odd: "O", even: "E" }, "first") === "O", "missing first falls back to odd");
  assert(NS.variantValue({ odd: "O" }, "even") === "O", "missing even falls back to odd");
  assert(NS.variantValue(null, "odd", { odd: "D" }) === "D", "null bucket falls back to default bucket");
  assert(NS.variantValue({}, "odd") === "", "empty bucket yields an empty string");
});

test("[UAT:DOC-08] RUNTIME #2424: resolveFields substitutes all supported fields", function () {
  var NS = fixture().NS;
  var out = NS.resolveFields("P{$page}/{$total} { $title } { $author } { $date } { $nope }",
    { page: 3, total: 9, title: "Ti", author: "Au", date: "Da" });
  assert(out === "P3/9 Ti Au Da { $nope }", "supported fields resolve; unknown tokens are untouched");
  assert(NS.resolveFields(null, {}) === "", "null template is safe");
});

test("[UAT:DOC-08] RUNTIME #2424: resolvePageChrome resolves per-page header/footer/watermark", function () {
  var NS = fixture().NS;
  var section = {
    firstPageDifferent: true,
    header: { first: "FH { $page }", odd: "OH { $page }", even: "EH { $page }" },
    footer: { first: "FF", odd: "OF { $page } of { $total }", even: "EF" },
    watermark: "WM { $title }",
  };
  var page = { number: 2, isSectionFirst: true };
  NS.resolvePageChrome(page, section, { title: "T", author: "A", date: "D" }, 8);
  assert(page.variant === "first" && page.headerLines[0] === "FH 2" && page.footerLines[0] === "FF",
    "first-page chrome is selected");
  assert(page.watermark === "WM T", "watermark fields resolve");
  var odd = { number: 3, isSectionFirst: false };
  NS.resolvePageChrome(odd, section, {}, 8);
  assert(odd.headerLines[0] === "OH 3" && odd.footerLines[0] === "OF 3 of 8",
    "odd-page chrome uses computed page number and total");
});

// ── STATIC guards (private wiring not observable through pure helpers) ─

test("STATIC #2424: pagination derives totals from computed pages and reuses signatures", function () {
  var src = readFile(SOURCE_PAGINATE);
  var chrome = src.indexOf("NS.resolvePageChrome(item, sections[item.sectionIndex], fields, pages.length)");
  var signature = src.indexOf("item.signature = NS.pageSignature(item)");
  var reuse = src.indexOf("prior[reused].signature === fresh.pages[reused].signature");
  assert(chrome >= 0 && signature >= 0 && reuse >= 0,
    "page chrome, signature and prefix-reuse wiring must be present in the shipped paginator");
  assert(chrome < signature && signature < reuse,
    "chrome resolves before signatures are stamped, then signatures drive reuse");
});

test("STATIC #2424: header/footer chrome is computed, never a stored counter", function () {
  var src = readFile(SOURCE_HEADERS);
  var total = src.indexOf("total: total");
  var variant = src.indexOf("page.variant = variant");
  var fieldCount = (src.match(/NS\.resolveFields\(/g) || []).length;
  assert(total >= 0 && variant >= 0, "chrome resolves a computed total and a resolved variant");
  assert(fieldCount >= 3, "header, footer and watermark all pass through resolveFields");
});
