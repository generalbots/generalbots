/**
 * #2426 offline verification of the shipped docs structure helpers
 * (modules 42_structure_core, 43_structure_numbering, 44_structure_fields,
 * 45_structure_captions). Pure helpers are called directly; DOM-bound helpers
 * are run against the harness fake DOM and observed through the real data
 * attributes / label nodes they write. STATIC checks document deferred scope
 * (index generation, TOC page numbers, CSL citations) without fabricating a
 * runtime result for behaviour that is not shipped in these four modules.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var readFile = h.readFile;
var loadStructure = h.loadStructure;

// ── Fixture builders (plain fake-DOM nodes, no production logic) ──────

function heading(doc, tag, text, id) {
  var node = doc.createElement(tag);
  node.textContent = text;
  if (id) node.setAttribute("id", id);
  return node;
}

function captionNode(doc, kind, id, title) {
  var node = doc.createElement("p");
  node.setAttribute("data-gb-caption", "1");
  node.setAttribute("data-gb-caption-kind", kind);
  node.setAttribute("data-gb-caption-id", id);
  node.setAttribute("data-gb-num", "9");
  node.setAttribute("data-gb-caption-title", title);
  var prefix = doc.createElement("span");
  prefix.className = "gb-caption-prefix";
  prefix.textContent = "STALE:";
  node.appendChild(prefix);
  return node;
}

function makeField(doc, targetId, kind, format) {
  var field = doc.createElement("span");
  field.setAttribute("data-gb-field", "xref");
  field.setAttribute("data-gb-field-target", targetId);
  field.setAttribute("data-gb-field-kind", kind);
  field.setAttribute("data-gb-field-format", format);
  var label = doc.createElement("span");
  label.className = "gb-field-label";
  field.appendChild(label);
  return field;
}

// ── Load / export surface ─────────────────────────────────────────────

test("RUNTIME #2426: four real modules load in order and export the pure API", function () {
  var f = loadStructure();
  assert(f.NS && f.NS.LEVELS === 6, "DocsStructure.LEVELS must be 6");
  ["defaultConfig", "config", "esc", "uid", "formatLevel", "computeNumbering",
    "headingLevel", "label", "fieldHtml", "findTarget", "formats", "captionLabel",
    "captionHtml", "captionKind", "renderNumbering", "targetsFrom", "updateIn",
    "renumberCaptions", "listHtml"].forEach(function (name) {
    assert(typeof f.NS[name] === "function", name + " must be exported");
  });
});

// ── Core pure helpers (42) ────────────────────────────────────────────

test("RUNTIME #2426: defaultConfig has six decimal levels and restart on", function () {
  var cfg = loadStructure().NS.defaultConfig();
  assert(cfg.levels.length === 6, "six levels expected");
  assert(cfg.levels.every(function (row) {
    return row.scheme === "decimal" && row.continuePrevious === false;
  }), "every default level must be decimal / no continue");
  assert(cfg.restartPerSection === true, "restartPerSection defaults to true");
});

test("RUNTIME #2426: esc escapes all five HTML-sensitive characters", function () {
  var out = loadStructure().NS.esc("a&<>\"'");
  assert(out === "a&amp;&lt;&gt;&quot;&#39;", "unexpected escape output: " + out);
});

test("RUNTIME #2426: uid is deterministic with a stubbed crypto source", function () {
  var NS = loadStructure().NS;
  assert(NS.uid("fig") === "fig-12345678", "prefix + first UUID segment expected");
  assert(NS.uid("tbl") === "tbl-12345678", "prefix must be preserved");
});

test("RUNTIME #2426: tr prefers gbT translation and falls back only when absent", function () {
  var f = loadStructure();
  f.window.gbT = function () { return "TRANSLATED"; };
  assert(f.NS.tr("k", "fb") === "TRANSLATED", "gbT value must win");
  f.window.gbT = function () { return undefined; };
  assert(f.NS.tr("k", "fb") === undefined, "a present gbT result is returned verbatim");
  f.window.gbT = undefined;
  assert(f.NS.tr("k", "fb") === "fb", "fallback is used when no translator is present");
  assert(f.NS.tr("k") === "k", "key is the last-resort fallback");
});

test("RUNTIME #2426: config normalises stored values and rejects malformed state", function () {
  var f = loadStructure();
  var stored = null;
  f.window.DocsNav = { readStore: function () { return stored; } };
  assert(f.NS.config().levels.length === 6, "null store must yield defaults");
  stored = { levels: [{ level: 1 }] };
  assert(f.NS.config().levels.length === 6, "short level list must yield defaults");
  stored = f.NS.defaultConfig();
  stored.levels[2].scheme = "legal";
  stored.levels[3].continuePrevious = true;
  stored.restartPerSection = false;
  var cfg = f.NS.config();
  assert(cfg.levels[2].scheme === "legal", "legal scheme must survive normalisation");
  assert(cfg.levels[3].continuePrevious === true, "continuePrevious must survive");
  assert(cfg.restartPerSection === false, "restartPerSection false must survive");
});

test("RUNTIME #2426: setConfig delegates to the persistence store", function () {
  var f = loadStructure();
  var written = null;
  f.window.DocsNav = { writeStore: function (key, value) { written = { key: key, value: value }; return true; } };
  var cfg = f.NS.defaultConfig();
  assert(f.NS.setConfig(cfg) === true, "writeStore success must propagate");
  assert(written.key === "structureConfig" && written.value === cfg, "config must be stored under structureConfig");
});

// ── Multilevel numbering (43) ────────────────────────────────────────

test("RUNTIME #2426: formatLevel renders decimal and legal labels", function () {
  var NS = loadStructure().NS;
  assert(NS.formatLevel({ 1: 1, 2: 2, 3: 3 }, 3, "decimal") === "1.2.3", "decimal 3 levels");
  assert(NS.formatLevel({ 1: 1, 2: 2, 3: 3 }, 3, "legal") === "1.2.3.", "legal 3 levels");
  assert(NS.formatLevel({ 1: 4 }, 1, "decimal") === "4", "single level");
  assert(NS.formatLevel({}, 2, "decimal") === "0.0", "missing counters default to zero");
});

test("RUNTIME #2426: computeNumbering produces the canonical 1, 1.1, 1.1.1", function () {
  var NS = loadStructure().NS;
  var labels = NS.computeNumbering([{ level: 1 }, { level: 2 }, { level: 3 }]);
  assert(labels.join("|") === "1|1.1|1.1.1", "unexpected labels: " + labels.join("|"));
});

test("RUNTIME #2426: sibling headings increment the trailing component", function () {
  var NS = loadStructure().NS;
  var labels = NS.computeNumbering([
    { level: 1 }, { level: 2 }, { level: 3 }, { level: 2 }, { level: 3 },
  ]);
  assert(labels.join("|") === "1|1.1|1.1.1|1.2|1.2.1", "unexpected labels: " + labels.join("|"));
});

test("RUNTIME #2426: a new section resets all deeper counters", function () {
  var NS = loadStructure().NS;
  var labels = NS.computeNumbering([
    { level: 1 }, { level: 2 }, { level: 1 }, { level: 2 },
  ]);
  assert(labels.join("|") === "1|1.1|2|2.1", "unexpected labels: " + labels.join("|"));
});

test("RUNTIME #2426: inserting a section in the middle renumbers deterministically", function () {
  var NS = loadStructure().NS;
  var before = NS.computeNumbering([
    { level: 1 }, { level: 2 }, { level: 2 }, { level: 1 }, { level: 2 },
  ]);
  assert(before.join("|") === "1|1.1|1.2|2|2.1", "baseline labels changed");
  var after = NS.computeNumbering([
    { level: 1 }, { level: 2 }, { level: 1 }, { level: 2 }, { level: 1 }, { level: 2 },
  ]);
  assert(after.join("|") === "1|1.1|2|2.1|3|3.1", "insert must shift later sections: " + after.join("|"));
});

test("RUNTIME #2426: deleting a heading renumbers deterministically", function () {
  var NS = loadStructure().NS;
  var after = NS.computeNumbering([{ level: 1 }, { level: 2 }, { level: 2 }]);
  assert(after.join("|") === "1|1.1|1.2", "delete must close the gap: " + after.join("|"));
});

test("RUNTIME #2426: an explicit restart marker restarts the subtree at that heading", function () {
  var NS = loadStructure().NS;
  var labels = NS.computeNumbering([
    { level: 1, restart: true }, { level: 2 }, { level: 1, restart: true }, { level: 2 },
  ]);
  assert(labels.join("|") === "1|1.1|1|1.1", "restart did not reset: " + labels.join("|"));
});

test("RUNTIME #2426: legal scheme appends a trailing dot at every depth", function () {
  var f = loadStructure();
  var cfg = f.NS.defaultConfig();
  cfg.levels.forEach(function (row) { row.scheme = "legal"; });
  var labels = f.NS.computeNumbering([{ level: 1 }, { level: 2 }, { level: 3 }], cfg);
  assert(labels.join("|") === "1.|1.1.|1.1.1.", "unexpected legal labels: " + labels.join("|"));
});

test("RUNTIME #2426: continue-previous level resumes its original parent prefix", function () {
  var f = loadStructure();
  var cfg = f.NS.defaultConfig();
  cfg.levels[1].continuePrevious = true;
  var labels = f.NS.computeNumbering([
    { level: 1 }, { level: 2 }, { level: 1 }, { level: 2 },
  ], cfg);
  assert(labels.join("|") === "1|1.1|2|1.2", "continue-previous labels: " + labels.join("|"));
});

test("RUNTIME #2426: restart on a continued level rebinds its parent prefix", function () {
  var f = loadStructure();
  var cfg = f.NS.defaultConfig();
  cfg.levels[1].continuePrevious = true;
  var labels = f.NS.computeNumbering([
    { level: 1 }, { level: 2 }, { level: 1 }, { level: 2, restart: true }, { level: 2 },
  ], cfg);
  assert(labels.join("|") === "1|1.1|2|2.1|2.2", "restart must rebind prefix: " + labels.join("|"));
});

test("RUNTIME #2426: headingLevel parses h1..h6 and ignores other tags", function () {
  var NS = loadStructure().NS;
  assert(NS.headingLevel({ tagName: "H3" }) === 3, "uppercase H3 -> 3");
  assert(NS.headingLevel({ tagName: "h6" }) === 6, "lowercase h6 -> 6");
  assert(NS.headingLevel({ tagName: "P" }) === 0, "non-heading -> 0");
  assert(NS.headingLevel(null) === 0, "null -> 0");
});

// ── Cross-reference fields (44) ──────────────────────────────────────

var SCOPE = { id: "sec-scope", kind: "heading", number: "4.2", text: "Scope", prefix: "" };
var FIGURE = { id: "fig-3", kind: "figure", number: "3", text: "Architecture", prefix: "Figure" };

test("RUNTIME #2426: label honours number/text/full formats", function () {
  var NS = loadStructure().NS;
  assert(NS.label(SCOPE, "full") === "4.2 Scope", "full heading label");
  assert(NS.label(SCOPE, "number") === "4.2", "number-only label");
  assert(NS.label(SCOPE, "text") === "Scope", "text-only label");
  assert(NS.label(FIGURE, "full") === "Figure 3: Architecture", "full figure label");
  assert(NS.label(FIGURE, "number") === "Figure 3", "number figure label");
  assert(NS.label(null, "full") === "", "null target yields empty label");
});

test("RUNTIME #2426: a field label tracks its target after renumbering", function () {
  var NS = loadStructure().NS;
  assert(NS.label(SCOPE, "full") === "4.2 Scope", "before renumber");
  var renumbered = { id: SCOPE.id, kind: SCOPE.kind, number: "5.1", text: SCOPE.text, prefix: "" };
  assert(NS.label(renumbered, "full") === "5.1 Scope", "label must follow the new number");
  assert(NS.label(renumbered, "number") === "5.1", "number format must follow too");
});

test("RUNTIME #2426: no page-number format is offered and page resolves empty", function () {
  var NS = loadStructure().NS;
  assert(NS.formats().join("|") === "number|text|full", "unexpected format list");
  assert(NS.formats().indexOf("page") === -1, "page must not be a selectable format");
  assert(NS.label(SCOPE, "page") === "", "page format must never fabricate a value");
});

test("RUNTIME #2426: fieldHtml carries target/kind/format and escapes the id", function () {
  var NS = loadStructure().NS;
  var html = NS.fieldHtml({ id: "a&<b", kind: "heading", number: "1", text: "T", prefix: "" }, "number");
  assert(html.indexOf('data-gb-field="xref"') >= 0, "must be an xref field");
  assert(html.indexOf('data-gb-field-target="a&amp;&lt;b"') >= 0, "target id must be escaped");
  assert(html.indexOf('data-gb-field-format="number"') >= 0, "format attribute expected");
  assert(html.indexOf('<span class="gb-field-label">1</span>') >= 0, "rendered label expected");
});

test("RUNTIME #2426: findTarget matches id and optional kind, first match wins", function () {
  var NS = loadStructure().NS;
  var list = [SCOPE, FIGURE, { id: "sec-scope", kind: "figure", number: "9", text: "F", prefix: "Figure" }];
  assert(NS.findTarget(list, "sec-scope", "heading") === SCOPE, "heading match expected");
  assert(NS.findTarget(list, "sec-scope", "figure") === list[2], "figure match expected");
  assert(NS.findTarget(list, "sec-scope") === SCOPE, "first id match expected");
  assert(NS.findTarget(list, "missing", "heading") === null, "missing target -> null");
});

test("RUNTIME #2426: kindLabel and captionLabel use the English fallback in Node", function () {
  var NS = loadStructure().NS;
  assert(NS.kindLabel("figure") === "Figure", "figure fallback");
  assert(NS.kindLabel("table") === "Table", "table fallback");
  assert(NS.captionLabel("figure", "3") === "Figure 3", "figure caption label");
  assert(NS.captionLabel("table", "2") === "Table 2", "table caption label");
});

// ── Captions (45) ────────────────────────────────────────────────────

test("RUNTIME #2426: captionKind defaults to figure and honours table", function () {
  var NS = loadStructure().NS;
  assert(NS.captionKind({ getAttribute: function () { return "table"; } }) === "table", "explicit table");
  assert(NS.captionKind({ getAttribute: function () { return null; } }) === "figure", "implicit figure");
});

test("RUNTIME #2426: captionHtml records kind/number/id and escapes the title", function () {
  var NS = loadStructure().NS;
  var html = NS.captionHtml("table", "a<b", "2", "tbl-1");
  assert(html.indexOf('data-gb-caption-kind="table"') >= 0, "kind attribute expected");
  assert(html.indexOf('data-gb-caption-id="tbl-1"') >= 0, "id attribute expected");
  assert(html.indexOf('data-gb-num="2"') >= 0, "number attribute expected");
  assert(html.indexOf('data-gb-caption-title="a&lt;b"') >= 0, "title must be escaped");
  assert(html.indexOf("Table 2:") >= 0, "prefix label expected");
});

// ── DOM-bound helpers exercised through the real shipped code ───────

test("RUNTIME #2426: renderNumbering writes deterministic labels and spans", function () {
  var f = loadStructure();
  var root = f.doc.createElement("article");
  root.appendChild(heading(f.doc, "h1", "Introduction", "sec-intro"));
  root.appendChild(heading(f.doc, "h2", "Scope", "sec-scope"));
  root.appendChild(heading(f.doc, "h3", "Details", "sec-details"));
  var labels = f.NS.renderNumbering(root);
  assert(labels.join("|") === "1|1.1|1.1.1", "unexpected render labels: " + labels.join("|"));
  var nodes = root.querySelectorAll("h1,h2,h3");
  assert(nodes[1].getAttribute("data-gb-num") === "1.1", "data-gb-num must be written");
  assert(nodes[2].getAttribute("data-gb-title") === "Details", "clean title must be stored");
  var span = nodes[0].querySelector(".gb-num-label");
  assert(span && span.textContent.indexOf("1") === 0, "label span must be inserted and non-editable");
  assert(span.getAttribute("contenteditable") === "false", "label span must be non-editable");
});

test("RUNTIME #2426: renderNumbering renumbers in place after an insert", function () {
  var f = loadStructure();
  var root = f.doc.createElement("article");
  var intro = heading(f.doc, "h1", "Introduction", "sec-intro");
  var scope = heading(f.doc, "h2", "Scope", "sec-scope");
  var details = heading(f.doc, "h3", "Details", "sec-details");
  root.appendChild(intro); root.appendChild(scope); root.appendChild(details);
  f.NS.renderNumbering(root);
  root.insertBefore(heading(f.doc, "h2", "Method", "sec-method"), details);
  var after = f.NS.renderNumbering(root);
  assert(after.join("|") === "1|1.1|1.2|1.2.1", "insert must renumber: " + after.join("|"));
  assert(details.getAttribute("data-gb-num") === "1.2.1", "existing heading must get the new number");
});

test("RUNTIME #2426: renderNumbering closes gaps after a delete", function () {
  var f = loadStructure();
  var root = f.doc.createElement("article");
  var h1 = heading(f.doc, "h1", "One", "s1");
  var h2 = heading(f.doc, "h2", "Two", "s2");
  var h3 = heading(f.doc, "h3", "Three", "s3");
  var last = heading(f.doc, "h2", "Four", "s4");
  [h1, h2, h3, last].forEach(function (n) { root.appendChild(n); });
  root.removeChild(h3);
  var labels = f.NS.renderNumbering(root);
  assert(labels.join("|") === "1|1.1|1.2", "delete must renumber: " + labels.join("|"));
});

test("RUNTIME #2426: targetsFrom enumerates headings, bookmarks and captions by kind", function () {
  var f = loadStructure();
  var root = f.doc.createElement("article");
  var h = heading(f.doc, "h2", "Scope", "sec-scope");
  h.setAttribute("data-gb-num", "4.2"); h.setAttribute("data-gb-title", "Scope");
  root.appendChild(h);
  var bookmark = f.doc.createElement("span");
  bookmark.setAttribute("id", "bm-appendix");
  bookmark.setAttribute("data-gb-bookmark", "appendix");
  root.appendChild(bookmark);
  root.appendChild(captionNode(f.doc, "figure", "fig-1", "Architecture"));
  root.appendChild(captionNode(f.doc, "table", "tbl-1", "Costs"));
  var targets = f.NS.targetsFrom(root);
  assert(targets.map(function (t) { return t.id; }).join("|") === "sec-scope|bm-appendix|fig-1|tbl-1",
    "unexpected target order: " + targets.map(function (t) { return t.id; }).join("|"));
  assert(targets[0].kind === "heading" && targets[0].number === "4.2", "heading target data");
  assert(targets[1].kind === "bookmark", "bookmark target kind");
  assert(targets[2].kind === "figure" && targets[2].prefix === "Figure", "figure target data");
  assert(targets[3].kind === "table" && targets[3].prefix === "Table", "table target data");
});

test("RUNTIME #2426: updateIn resolves a field and refreshes it after renumbering", function () {
  var f = loadStructure();
  var root = f.doc.createElement("article");
  var intro = heading(f.doc, "h1", "Introduction", "sec-intro");
  var scope = heading(f.doc, "h2", "Scope", "sec-scope");
  root.appendChild(intro); root.appendChild(scope);
  f.NS.renderNumbering(root);
  var field = makeField(f.doc, "sec-scope", "heading", "full");
  root.appendChild(field);
  f.NS.updateIn(root);
  var label = field.querySelector(".gb-field-label");
  assert(label.textContent === "1.1 Scope", "initial field label: " + label.textContent);
  root.insertBefore(heading(f.doc, "h1", "Two", "sec-two"), scope);
  f.NS.renderNumbering(root);
  f.NS.updateIn(root);
  assert(label.textContent === "2.1 Scope", "field must follow renumber: " + label.textContent);
});

test("RUNTIME #2426: renumberCaptions counts figures and tables independently", function () {
  var f = loadStructure();
  var root = f.doc.createElement("article");
  root.appendChild(captionNode(f.doc, "figure", "fig-a", "Alpha"));
  root.appendChild(captionNode(f.doc, "table", "tbl-a", "Beta"));
  root.appendChild(captionNode(f.doc, "figure", "fig-b", "Gamma"));
  f.NS.renumberCaptions(root);
  var caps = root.querySelectorAll("[data-gb-caption]");
  assert(caps.map(function (n) { return n.getAttribute("data-gb-num"); }).join("|") === "1|1|2",
    "per-kind numbering expected 1|1|2");
  var prefixes = caps.map(function (n) { return n.querySelector(".gb-caption-prefix").textContent; });
  assert(prefixes.join("|") === "Figure 1:|Table 1:|Figure 2:", "prefixes: " + prefixes.join("|"));
});

test("RUNTIME #2426: listHtml builds a TOF from caption fields", function () {
  var f = loadStructure();
  var root = f.doc.createElement("article");
  root.appendChild(captionNode(f.doc, "figure", "fig-a", "Alpha"));
  root.appendChild(captionNode(f.doc, "figure", "fig-b", "Gamma"));
  var html = f.NS.listHtml(root, "figure");
  assert(html.indexOf('data-gb-structure-list="figure"') >= 0, "TOF wrapper expected");
  assert((html.match(/gb-structure-list-entry/g) || []).length === 2, "two entries expected");
  assert((html.match(/data-gb-field="xref"/g) || []).length === 2, "entries must be real xref fields");
  assert(html.indexOf("Alpha") >= 0 && html.indexOf("Gamma") >= 0, "titles must appear");
});

// ── Clearly-labelled STATIC guards for deferred scope ────────────────

test("STATIC #2426: structure modules neither fabricate page numbers nor generate page fields", function () {
  var fields = readFile("docs/modules/44_structure_fields.js");
  assert(fields.indexOf('if (!target || format === "page") return "";') >= 0,
    "page format must be an explicit empty branch");
  assert(fields.indexOf('var FORMATS = ["number", "text", "full"];') >= 0,
    "FORMATS must not contain a page entry");
  assert(fields.indexOf('data-gb-field="page"') === -1, "no page field may be generated");
  assert(fields.indexOf("there is no page model") >= 0, "the no-page-model rationale must remain documented");
});

test("STATIC #2426: index generation, TOC pagination and CSL are deferred, not faked", function () {
  h.STRUCTURE_MODULES.forEach(function (rel) {
    var src = readFile(rel);
    assert(src.indexOf(".sort(") === -1, rel + " must not claim index sorting");
    assert(src.indexOf("data-gb-index") === -1, rel + " must not fabricate index entries");
    assert(src.indexOf("pageNumber") === -1, rel + " must not fabricate page numbers");
    assert(src.indexOf("csl-") === -1, rel + " must not claim CSL citation styles");
  });
});
