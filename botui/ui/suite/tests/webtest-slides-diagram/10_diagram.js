/**
 * Slides diagram/flowchart — pure-helper invariant tests (#2488).
 * Runtime: real shipped helpers in a VM sandbox. DOM-only render helpers are
 * covered by labelled STATIC source checks at the bottom (never executed).
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var gb = h.loadModules();

var presets = gb.presets;
var flow = gb.flow;
var org = gb.org;
var icons = gb.icons;

function node(id, extra) {
  var result = { id: id, text: id, icon: "", color: "", level: 0, parentId: null,
    x: 0, y: 0, w: presets.NODE_W, h: presets.NODE_H };
  Object.keys(extra || {}).forEach(function (key) { result[key] = extra[key]; });
  return result;
}
function nodes(count) {
  var out = [];
  for (var i = 0; i < count; i++) out.push(node("n" + i));
  return out;
}
function pointEquals(actual, expected, label) {
  // SDFlow.pathD/paths use [x, y] tuples; portPoint uses { x, y } objects.
  var point = Array.isArray(actual) ? { x: actual[0], y: actual[1] } : actual;
  assert(point && point.x === expected.x && point.y === expected.y,
    label + ": expected (" + expected.x + "," + expected.y + ") got " +
    JSON.stringify(actual));
}
function isFinitePoint(p) {
  return p && typeof p.x === "number" && typeof p.y === "number" &&
    isFinite(p.x) && isFinite(p.y);
}
function segmentsAreOrthogonal(points) {
  for (var i = 1; i < points.length; i++) {
    var a = points[i - 1];
    var b = points[i];
    assert(a[0] === b[0] || a[1] === b[1],
      "segment " + i + " is diagonal: " + JSON.stringify([a, b]));
  }
}
function allParentsAbove(diagram) {
  var byId = {};
  diagram.nodes.forEach(function (n) { byId[n.id] = n; });
  diagram.connectors.forEach(function (c) {
    var parent = byId[c.from];
    var child = byId[c.to];
    assert(parent && child, "connector references a missing node");
    assert(parent.y < child.y, "parent not above child for " + c.id);
  });
}

var ORG_CSV = "Name,Title,Reports To\nAda,CEO,\nBob,CTO,Ada\nCara,CFO,Ada\n" +
  "Dan,VP Eng,Bob\nEve,VP Ops,Bob\nFinn,Controller,Cara\nGia,Manager,Dan\n" +
  "Hal,Engineer,Gia\nIvy,Engineer,Gia\nJay,Analyst,Eve\nKim,Analyst,Eve\n" +
  "Lou,Assistant,Cara";
// ── API surface ───────────────────────────────────────────────────────
test("all four shipped modules bind their expected namespaces", function () {
  assert(presets && typeof presets.layout === "function", "SDPresets missing");
  assert(gb.presetDefs && Array.isArray(gb.presetDefs), "SD_PRESETS missing");
  assert(icons && typeof icons.search === "function", "SDIcons missing");
  assert(flow && typeof flow.recompute === "function", "SDFlow missing");
  assert(org && typeof org.build === "function", "SDOrg missing");
});

test("pure helper entry points are functions", function () {
  ["layout", "reflow", "create", "addNode", "removeNode", "promote", "demote"
  ].forEach(function (name) {
    assert(typeof presets[name] === "function", "SDPresets." + name + " missing");
  });
  ["portPoint", "orthogonal", "autoPorts", "recompute", "pathD"].forEach(function (name) {
    assert(typeof flow[name] === "function", "SDFlow." + name + " missing");
  });
  ["parseRows", "resolveParents", "build", "csvLine", "separatorOf"].forEach(function (name) {
    assert(typeof org[name] === "function", "SDOrg." + name + " missing");
  });
});
// ── Preset catalogue ──────────────────────────────────────────────────
test("preset list is non-empty and every preset has an id and label key", function () {
  assert(gb.presetDefs.length > 0, "preset catalogue is empty");
  gb.presetDefs.forEach(function (def) {
    assert(typeof def.id === "string" && def.id.length > 0, "preset id missing");
    assert(typeof def.key === "string" && def.key.indexOf("slides-diagrams-") === 0,
      "preset label key missing for " + def.id);
    assert(typeof def.fallback === "string" && def.fallback.length > 0,
      "preset fallback label missing for " + def.id);
  });
});

test("def() resolves every listed id and falls back for unknown ids", function () {
  gb.presetDefs.forEach(function (def) {
    assert(presets.def(def.id).id === def.id, "def() failed for " + def.id);
  });
  assert(presets.def("__unknown__").id === gb.presetDefs[0].id,
    "unknown preset did not fall back to the first entry");
});

test("name() returns a non-empty label for every preset", function () {
  gb.presetDefs.forEach(function (def) {
    var label = presets.name(def.id);
    assert(typeof label === "string" && label.length > 0,
      "empty label for " + def.id);
  });
});
// ── Linear process flow ───────────────────────────────────────────────
test("linear flow of three nodes yields increasing x at a constant y", function () {
  var ns = nodes(3);
  var connectors = presets.layout("process", ns);
  assert(ns[0].x < ns[1].x && ns[1].x < ns[2].x,
    "process nodes are not left-to-right: " + ns.map(function (n) { return n.x; }).join(","));
  assert(ns[0].y === ns[1].y && ns[1].y === ns[2].y, "process y is not constant");
  assert(connectors.length === 2, "expected 2 connectors, got " + connectors.length);
  assert(connectors[0].from === ns[0].id && connectors[0].to === ns[1].id &&
    connectors[1].from === ns[1].id && connectors[1].to === ns[2].id,
    "connectors do not chain the nodes in order");
  assert(connectors[0].fromPort === "e" && connectors[0].toPort === "w",
    "process connectors must use e/w ports");
});

test("create('process', 3) builds a complete deterministic diagram", function () {
  gb.resetUid();
  var d = presets.create("process", 3);
  assert(d.nodes.length === 3 && d.connectors.length === 2, "wrong element counts");
  assert(d.preset === "process" && d.flowMode === true, "wrong preset metadata");
  assert(d.slideIndex === 0, "stubbed slideIndex not honoured");
  var ids = d.nodes.map(function (n) { return n.id; });
  assert(new Set(ids).size === 3, "node ids are not unique");
});

test("process connector endpoints attach to box edges, not centres", function () {
  var ns = nodes(3);
  var connectors = presets.layout("process", ns);
  flow.recompute({ nodes: ns, connectors: connectors });
  var first = connectors[0].path;
  var last = connectors[1].path;
  pointEquals(first[0], { x: ns[0].x + ns[0].w, y: ns[0].y + ns[0].h / 2 }, "source endpoint");
  pointEquals(first[first.length - 1], { x: ns[1].x, y: ns[1].y + ns[1].h / 2 }, "target endpoint");
  pointEquals(last[0], { x: ns[1].x + ns[1].w, y: ns[1].y + ns[1].h / 2 }, "second source endpoint");
  pointEquals(last[last.length - 1], { x: ns[2].x, y: ns[2].y + ns[2].h / 2 }, "second target endpoint");
});
// ── Branching / hierarchy ─────────────────────────────────────────────
test("branching layout places siblings without overlap, parent above", function () {
  var ns = [node("r"), node("c1", { parentId: "r" }), node("c2", { parentId: "r" })];
  var connectors = presets.layout("org", ns);
  var c1 = presets.byId({ nodes: ns }, "c1");
  var c2 = presets.byId({ nodes: ns }, "c2");
  var root = presets.byId({ nodes: ns }, "r");
  assert(Math.abs(c1.x - c2.x) >= presets.NODE_W,
    "siblings overlap horizontally: " + c1.x + " vs " + c2.x);
  assert(root.y < c1.y && root.y < c2.y, "parent is not above its children");
  assert(connectors.length === 2, "expected one connector per child");
  connectors.forEach(function (c) {
    assert(c.fromPort === "s" && c.toPort === "n", "tree ports must be s/n");
  });
});

test("12-node org layout yields one box per node with finite positions", function () {
  var d = presets.create("org", 12);
  assert(d.nodes.length === 12, "expected 12 boxes");
  d.nodes.forEach(function (n) {
    assert(isFinitePoint(n), "non-finite position for " + n.id);
  });
  assert(d.connectors.length === 11, "expected one connector per non-root node");
  allParentsAbove(d);
});
// ── Data-driven org chart ─────────────────────────────────────────────
test("parseRows skips the header and blank lines", function () {
  var parsed = org.parseRows(ORG_CSV + "\n\n");
  assert(parsed.errors.length === 0, "unexpected parse errors");
  assert(parsed.rows.length === 12, "expected 12 data rows, got " + parsed.rows.length);
  assert(parsed.rows[0].name === "Ada" && parsed.rows[0].title === "CEO",
    "first row not parsed correctly");
  assert(parsed.rows[1].reportsTo === "Ada", "reports-to column not parsed");
});

test("separator detection and quoted CSV cells are handled", function () {
  assert(org.separatorOf("a\tb") === "\t", "tab separator not detected");
  assert(org.separatorOf("a;b") === ";", "semicolon separator not detected");
  assert(org.separatorOf("a,b") === ",", "comma separator not detected");
  assert(org.separatorOf("plain") === "|", "fallback separator not returned");
  var cells = org.csvLine('a,"b,c",d', ",");
  assert(cells.length === 3 && cells[1] === "b,c", "quoted comma cell broken");
  assert(org.csvLine('"a""b",c', ",")[0] === 'a"b', "escaped quote broken");
});

test("resolveParents matches reports-to names case-insensitively", function () {
  var ns = [node("a", { name: "Ada" }), node("b", { name: "Bob", reportsTo: "ada" }),
    node("c", { name: "Cara", reportsTo: "BOB" })];
  org.resolveParents(ns);
  assert(ns[0].parentId === null, "root should have no parent");
  assert(ns[1].parentId === "a", "Bob not attached to Ada");
  assert(ns[2].parentId === "b", "Cara not attached to Bob");
});

test("resolveParents breaks a reports-to cycle", function () {
  var ns = [node("a", { name: "A", reportsTo: "B" }),
    node("b", { name: "B", reportsTo: "A" })];
  org.resolveParents(ns);
  assert(ns[0].parentId === null || ns[1].parentId === null,
    "cycle was not broken");
});

test("build() from 12 rows produces one box per row and a valid hierarchy", function () {
  gb.resetUid();
  var result = org.build(ORG_CSV);
  assert(result.count === 12 && result.errors.length === 0, "build count/errors wrong");
  assert(result.diagram.nodes.length === 12, "expected one box per row");
  assert(result.diagram.nodes[0].name === "Ada", "node text not taken from data");
  assert(result.diagram.nodes[1].parentId === result.diagram.nodes[0].id,
    "reports-to relation not materialised");
  assert(result.diagram.connectors.length === 11, "wrong connector count");
  allParentsAbove(result.diagram);
});
// ── Connector geometry ────────────────────────────────────────────────
test("portPoint attaches exactly to each of the four box edges", function () {
  var n = node("n", { x: 100, y: 200, w: 150, h: 68 });
  pointEquals(flow.portPoint(n, "n"), { x: 175, y: 200 }, "north port");
  pointEquals(flow.portPoint(n, "e"), { x: 250, y: 234 }, "east port");
  pointEquals(flow.portPoint(n, "s"), { x: 175, y: 268 }, "south port");
  pointEquals(flow.portPoint(n, "w"), { x: 100, y: 234 }, "west port");
  var centre = { x: 175, y: 234 };
  ["n", "e", "s", "w"].forEach(function (p) {
    var pt = flow.portPoint(n, p);
    assert(pt.x !== centre.x || pt.y !== centre.y, p + " port is the box centre");
  });
});

test("portPoint is deterministic for identical input", function () {
  var n = node("n", { x: 13, y: 27, w: 150, h: 68 });
  assert(JSON.stringify(flow.portPoint(n, "e")) === JSON.stringify(flow.portPoint(n, "e")),
    "portPoint differed between calls");
});

test("orthogonal paths are axis-aligned and terminate on the chosen ports", function () {
  var a = node("a", { x: 100, y: 100 });
  var b = node("b", { x: 500, y: 100 });
  var horizontal = flow.orthogonal(a, b, "e", "w");
  segmentsAreOrthogonal(horizontal);
  pointEquals(horizontal[0], flow.portPoint(a, "e"), "horizontal start");
  pointEquals(horizontal[horizontal.length - 1], flow.portPoint(b, "w"), "horizontal end");
  var c = node("c", { x: 100, y: 300 });
  var vertical = flow.orthogonal(a, c, "s", "n");
  segmentsAreOrthogonal(vertical);
  pointEquals(vertical[0], flow.portPoint(a, "s"), "vertical start");
  pointEquals(vertical[vertical.length - 1], flow.portPoint(c, "n"), "vertical end");
  var mixed = flow.orthogonal(a, c, "e", "n");
  segmentsAreOrthogonal(mixed);
  pointEquals(mixed[0], flow.portPoint(a, "e"), "mixed start");
  pointEquals(mixed[mixed.length - 1], flow.portPoint(c, "n"), "mixed end");
});

test("autoPorts chooses the nearest opposing edge pair", function () {
  var a = node("a", { x: 100, y: 100 });
  assert(flow.autoPorts(a, node("b", { x: 500, y: 100 })).join(",") === "e,w",
    "rightwards pair wrong");
  assert(flow.autoPorts(a, node("b", { x: 100, y: 300 })).join(",") === "s,n",
    "downwards pair wrong");
  assert(flow.autoPorts(a, node("b", { x: 10, y: 100 })).join(",") === "w,e",
    "leftwards pair wrong");
});

test("recompute reattaches connectors when a box moves", function () {
  var a = node("a", { x: 100, y: 100 });
  var b = node("b", { x: 500, y: 100 });
  var connector = { id: "c1", from: "a", to: "b", fromPort: "", toPort: "", path: [] };
  var diagram = { nodes: [a, b], connectors: [connector] };
  flow.recompute(diagram);
  pointEquals(connector.path[0], flow.portPoint(a, "e"), "initial source edge");
  b.x = 100;
  b.y = 400;
  flow.recompute(diagram);
  pointEquals(connector.path[0], flow.portPoint(a, "s"), "source edge after move");
  pointEquals(connector.path[connector.path.length - 1], flow.portPoint(b, "n"), "target edge after move");
  segmentsAreOrthogonal(connector.path);
});

test("recompute clears dangling connectors and pathD serialises points", function () {
  var d = { nodes: [node("a")],
    connectors: [{ id: "dangling", from: "a", to: "gone", path: [[1, 2]] }] };
  flow.recompute(d);
  assert(d.connectors[0].path.length === 0, "dangling connector kept its path");
  assert(flow.pathD(d.connectors[0]) === "", "empty path should serialise to empty string");
  assert(flow.pathD({ path: [[1, 2], [3, 4]] }) === "M 1 2 L 3 4",
    "pathD output wrong");
  flow.recompute({ nodes: [], connectors: [] });
});
// ── Determinism ───────────────────────────────────────────────────────
test("layout() produces identical geometry for identical input", function () {
  var first = nodes(4);
  var second = JSON.parse(JSON.stringify(first));
  var c1 = presets.layout("process", first);
  var c2 = presets.layout("process", second);
  assert(JSON.stringify(first) === JSON.stringify(second), "node geometry diverged");
  assert(c1.map(function (c) { return c.from + ">" + c.to; }).join() ===
    c2.map(function (c) { return c.from + ">" + c.to; }).join(),
    "connector endpoints diverged");
});

test("create() produces identical JSON with a fresh id sequence", function () {
  gb.resetUid();
  var first = JSON.stringify(presets.create("matrix", 6));
  gb.resetUid();
  var second = JSON.stringify(presets.create("matrix", 6));
  assert(first === second, "create() was not reproducible");
});

test("build() produces identical JSON with a fresh id sequence", function () {
  gb.resetUid();
  var first = JSON.stringify(org.build(ORG_CSV));
  gb.resetUid();
  var second = JSON.stringify(org.build(ORG_CSV));
  assert(first === second, "build() was not reproducible");
});
// ── Empty / defensive input ───────────────────────────────────────────
test("empty node list lays out to an empty connector list for every preset", function () {
  gb.presetDefs.forEach(function (def) {
    var empty = [];
    var connectors = presets.layout(def.id, empty);
    assert(Array.isArray(connectors) && connectors.length === 0,
      "empty layout failed for " + def.id);
    assert(empty.length === 0, "empty input was mutated for " + def.id);
  });
});

test("reflow tolerates empty and malformed diagrams", function () {
  var d = { nodes: [], connectors: [], preset: "list" };
  assert(presets.reflow(d) === d && d.connectors.length === 0, "empty reflow failed");
  assert(presets.reflow(null) === null, "null reflow should be a no-op");
  assert(presets.reflow({}) && true, "malformed reflow threw");
});

test("empty org input yields zero rows without throwing", function () {
  var parsed = org.parseRows("");
  assert(parsed.rows.length === 0 && parsed.errors.length === 0, "empty parse not empty");
  var built = org.build("");
  assert(built.count === 0 && built.diagram.nodes.length === 0, "empty build not empty");
});
// ── Icon catalogue (pure catalogue helpers only) ──────────────────────
test("icon catalogue is populated with unique, well-formed entries", function () {
  var all = icons.all();
  assert(all.length >= 16, "icon catalogue is too small: " + all.length);
  var seen = {};
  all.forEach(function (icon) {
    assert(!seen[icon.id], "duplicate icon id: " + icon.id);
    seen[icon.id] = true;
    assert(typeof icon.key === "string" && icon.key.indexOf("slides-diagrams-icon-") === 0,
      "icon label key missing for " + icon.id);
    assert(typeof icon.fallback === "string" && icon.fallback.length > 0,
      "icon fallback label missing for " + icon.id);
    assert(typeof icon.d === "string" && icon.d.length > 0, "icon path data missing");
    assert(typeof icon.words === "string", "icon search words missing");
  });
});

test("icon find/label/search helpers return catalogue-backed values", function () {
  assert(icons.find("check").id === "check", "find() failed");
  assert(icons.find("__missing__") === null, "unknown icon should return null");
  assert(icons.label(icons.find("check")).length > 0, "label() returned nothing");
  assert(icons.label(null) === "", "label(null) should be empty");
  var found = icons.search("cloud");
  assert(found.some(function (i) { return i.id === "cloud"; }), "search by word failed");
  assert(icons.search("search").some(function (i) { return i.id === "search"; }),
    "search by id failed");
  assert(icons.search("").length === icons.all().length,
    "empty search should return the whole catalogue");
});

test("all() returns a defensive copy of the catalogue", function () {
  var first = icons.all();
  first.pop();
  assert(icons.all().length === first.length + 1, "all() exposed the live array");
});
// ── Editable reflow ───────────────────────────────────────────────────
test("addNode/removeNode reflow connectors and report success honestly", function () {
  gb.resetUid();
  var d = presets.create("process", 3);
  assert(presets.addNode(d) !== null, "addNode failed");
  assert(d.nodes.length === 4 && d.connectors.length === 3, "addNode did not reflow");
  assert(presets.removeNode(d, d.nodes[0].id) === true, "removeNode failed");
  assert(d.nodes.length === 3 && d.connectors.length === 2, "removeNode did not reflow");
  assert(presets.removeNode(d, "__missing__") === false, "unknown id reported as removed");
});

test("promote/demote reflow both ordered and hierarchical presets", function () {
  gb.resetUid();
  var flat = presets.create("process", 3);
  var before = flat.nodes[1].id;
  assert(presets.promote(flat, before) === true, "flat promote failed");
  assert(flat.nodes[0].id === before, "promote did not reorder the node");
  assert(presets.demote(flat, before) === true, "flat demote failed");
  assert(flat.nodes[1].id === before, "demote did not restore the node order");
  gb.resetUid();
  var tree = presets.create("org", 3);
  // demote() requires a following sibling to become the new parent, so use the
  // middle node (the trailing node is intentionally a no-op).
  var child = tree.nodes[1];
  var oldLevel = child.level;
  assert(presets.demote(tree, child.id) === true, "tree demote failed");
  assert(child.level > oldLevel, "tree demote did not lower the level");
  assert(presets.promote(tree, child.id) === true, "tree promote failed");
  assert(child.level <= oldLevel, "tree promote did not raise the level");
});
// ── STATIC source assertions (DOM-only render helpers, not executed) ──
test("STATIC: SDIcons.svg is vendored SVG with currentColor and a <title>", function () {
  var src = gb.source("slides/modules/32_diag_icons.js");
  assert(src.indexOf("createElementNS") >= 0, "no SVG namespace creation");
  assert(src.indexOf('"currentColor"') >= 0, "icon is not theme-recolourable");
  assert(src.indexOf('"title"') >= 0, "icon has no alt-text title element");
  assert(src.indexOf('"role", "img"') >= 0, "icon has no img role");
  assert(src.indexOf("fetch(") < 0 && src.indexOf("innerHTML") < 0,
    "icon renderer performs a network or raw-HTML operation");
});

test("STATIC: SDFlow SVG helpers render paths from computable geometry", function () {
  var src = gb.source("slides/modules/33_diag_flow.js");
  assert(src.indexOf("data-connector-id") >= 0, "no connector identity on rendered paths");
  assert(src.indexOf('setAttribute("d", SDFlow.pathD(connector))') >= 0,
    "rendered path is not derived from pathD()");
  assert(src.indexOf("createElementNS") >= 0 && src.indexOf('"viewBox"') >= 0,
    "SVG canvas construction missing");
});

test("STATIC: shipped diagram modules have no CDN, eval or raw-HTML usage", function () {
  h.MODULE_RELS.forEach(function (rel) {
    var src = gb.source(rel);
    assert(!/unpkg|jsdelivr|cdnjs|https:\/\//i.test(src),
      rel + " references an external/CDN URL");
    assert(src.indexOf("eval(") < 0 && src.indexOf("innerHTML") < 0,
      rel + " uses eval or innerHTML");
    var urls = src.match(/http:\/\/[^"'\s]+/g) || [];
    urls.forEach(function (url) {
      assert(url === "http://www.w3.org/2000/svg", rel + " has a non-SVG URL: " + url);
    });
  });
});
