/**
 * #2427 offline verification of the shipped docs forms state, template and
 * content-control modules (58_forms_state, 59_forms_templates,
 * 60_forms_controls). Pure helpers are called directly; DOM-bound helpers run
 * against the shared fake DOM and are read back through their real data-*
 * attributes. All DOM sentinels use code-like "<...>" tokens so no hardcoded
 * user-facing literal is introduced.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var loadForms = h.loadForms;
var makeArticle = h.makeArticle;

function controlElement(f, descriptor) {
  var node = f.doc.createElement("span");
  node.setAttribute("data-gb-form-control", "1");
  node.setAttribute("data-form-id", descriptor.id);
  node.setAttribute("data-form-type", descriptor.type);
  node.setAttribute("data-form-label", descriptor.label);
  node.setAttribute("data-form-options", descriptor.options.join(","));
  node.setAttribute("data-form-default", descriptor.value);
  node.setAttribute("data-form-required", descriptor.required ? "true" : "false");
  node.setAttribute("data-form-placeholder", descriptor.placeholder);
  return node;
}

// ── Load / export surface ──────────────────────────────────────────────

test("RUNTIME #2427: ten real modules load in order and export the namespace", function () {
  var f = loadForms();
  var NS = f.NS;
  assert(NS && NS.PREFIX === "docs2427:", "DocsForms.PREFIX must be docs2427:");
  ["substitute", "tr", "esc", "uid", "readStore", "writeStore", "defaultState",
    "loadState", "persist", "ensureStatus", "status", "statusKey", "focusMove",
    "getArticle", "docId", "docKey", "localNote", "previewNote"].forEach(function (name) {
    assert(typeof NS[name] === "function", "NS." + name + " must be exported");
  });
  ["templates", "controls", "merge", "dialog", "dialogs", "security", "protection"].forEach(function (name) {
    assert(NS[name] && typeof NS[name] === "object", "NS." + name + " must be exported");
  });
});

// ── State core (58) ────────────────────────────────────────────────────

test("RUNTIME #2427: defaultState is a complete, deterministic local snapshot", function () {
  var a = loadForms().NS.defaultState();
  var b = loadForms().NS.defaultState();
  assert(JSON.stringify(a) === JSON.stringify(b), "defaultState must be deterministic");
  assert(a.selectedTemplate === "" && a.csvText === "" && a.merged === "", "string fields start empty");
  assert(a.controls.length === 0 && a.redactions.length === 0, "array fields start empty");
  assert(a.mapping && Object.keys(a.mapping).length === 0, "mapping starts empty");
  assert(a.recordIndex === 0 && a.signature === null, "navigation/signature start empty");
  assert(a.protect.enabled === false && a.protect.passwordPreview === "", "protection starts off");
});

test("RUNTIME #2427: esc escapes all five HTML-sensitive characters", function () {
  assert(loadForms().NS.esc("a&<>\"'") === "a&amp;&lt;&gt;&quot;&#39;", "unexpected escape output");
});

test("RUNTIME #2427: substitute handles $name, bare name and whitespace forms", function () {
  var s = loadForms().NS.substitute;
  assert(s("{ $label }:{name}", { label: "<L>", name: "<N>" }) === "<L>:<N>", "both placeholder forms");
  assert(s("{  $x  }", { x: "7" }) === "7", "inner whitespace must be tolerated");
  assert(s("{missing}", { other: "1" }) === "{missing}", "absent key is left untouched");
  assert(s("{x}", null) === "{x}", "null params are a no-op");
});

test("RUNTIME #2427: tr prefers gbT and applies params to the fallback", function () {
  var f = loadForms();
  f.window.gbT = function () { return "<translated>"; };
  assert(f.NS.tr("k", "fb") === "<translated>", "gbT result must win");
  f.window.gbT = function (key, fallback) { return fallback; };
  assert(f.NS.tr("k", "V={ $v }", { v: "<x>" }) === "V=<x>", "fallback params must be substituted");
  f.window.gbT = undefined;
  assert(f.NS.tr("k", "fb") === "fb", "fallback used when no translator exists");
  assert(f.NS.tr("only-key") === "only-key", "key is last-resort fallback");
});

test("RUNTIME #2427: uid is deterministic with a stubbed random source", function () {
  var f = loadForms();
  assert(f.NS.uid("ctl") === "ctl-aaaaaaaa", "crypto path uses first uuid segment");
  assert(f.NS.uid("red") === "red-aaaaaaaa", "prefix must be preserved");
  var fallback = loadForms({ crypto: false }).NS;
  assert(fallback.uid("ctl") === fallback.uid("ctl"), "no-crypto path must still be deterministic");
  assert(fallback.uid("ctl").indexOf("ctl-") === 0, "no-crypto path keeps the prefix");
});

test("RUNTIME #2427: docId and docKey resolve article, title and fallback scope", function () {
  var f = loadForms();
  assert(f.NS.docId() === "current", "no article/title falls back to current");
  assert(f.NS.docKey("state") === "docs2427:current:state", "doc key prefix");
  var article = makeArticle(f, "doc-7");
  assert(f.NS.docId() === "doc-7", "article dataset.docId wins");
  assert(f.NS.docKey("state") === "docs2427:doc-7:state", "doc key uses article id");
  var title = f.doc.createElement("input");
  title.id = "docTitle";
  title.value = "<title-value>";
  f.setArticle(null);
  f.doc.body.removeChild(article);
  assert(f.NS.docId() === "<title-value>", "docTitle value is the next fallback");
  assert(article.parentNode === null, "fixture article detached before the fallback check");
});

test("RUNTIME #2427: localNote and previewNote resolve through the translator", function () {
  var NS = loadForms().NS;
  assert(NS.localNote().indexOf("Local only") === 0, "local note fallback");
  assert(NS.previewNote().indexOf("Preview only") === 0, "preview note fallback");
});

test("RUNTIME #2427: readStore/writeStore round-trip in a doc-scoped key", function () {
  var f = loadForms();
  makeArticle(f, "doc-A");
  assert(f.NS.writeStore("x", { a: 1 }) === true, "write must report success");
  assert(f.window.localStorage.getItem("docs2427:doc-A:x") === JSON.stringify({ a: 1 }), "stored under scoped key");
  var value = f.NS.readStore("x", null);
  assert(value && value.a === 1, "round-trip value");
  f.window.localStorage.setItem(f.NS.docKey("x"), "{malformed");
  assert(f.NS.readStore("x", "<fallback>") === "<fallback>", "malformed JSON falls back");
});

test("RUNTIME #2427: loadState normalises malformed stored fields", function () {
  var f = loadForms();
  makeArticle(f, "doc-N");
  f.window.localStorage.setItem(f.NS.docKey("state"), JSON.stringify({
    selectedTemplate: 42,
    controls: "not-an-array",
    csvText: null,
    mapping: [],
    recordIndex: "x",
    merged: 7,
    signature: "signed",
    redactions: {},
    protect: { enabled: "yes", passwordPreview: 5 },
  }));
  var s = f.NS.loadState();
  assert(s.selectedTemplate === "" && s.csvText === "" && s.merged === "", "bad strings -> defaults");
  assert(s.controls.length === 0 && s.redactions.length === 0, "bad arrays -> defaults");
  assert(Object.keys(s.mapping).length === 0 && s.recordIndex === 0, "bad mapping/index -> defaults");
  assert(s.signature === null, "bad signature -> null");
  assert(s.protect.enabled === false && s.protect.passwordPreview === "", "strict boolean coercion");
  f.window.localStorage.setItem(f.NS.docKey("state"), JSON.stringify({
    selectedTemplate: "letter", controls: [{ id: "c1" }], recordIndex: 3,
    protect: { enabled: true, passwordPreview: "set" },
  }));
  var ok = f.NS.loadState();
  assert(ok.selectedTemplate === "letter" && ok.controls.length === 1, "valid fields survive");
  assert(ok.recordIndex === 3 && ok.protect.enabled === true, "valid index/protection survive");
});

test("RUNTIME #2427: focusMove closes on Escape and wraps Tab in both directions", function () {
  var move = loadForms().NS.focusMove;
  var keys = ["a", "b", "c"];
  assert(move(keys, "a", { key: "Escape" }).action === "close", "Escape closes");
  assert(move(keys, "b", { key: "Tab" }).index === 2, "Tab forward");
  assert(move(keys, "a", { key: "Tab", shiftKey: true }).index === 2, "Shift+Tab wraps backwards");
  assert(move(keys, "unknown", { key: "Tab" }).index === 1, "unknown current wraps from the first slot");
  assert(move([], "a", { key: "Tab" }).action === "none", "empty key list is a no-op");
  assert(move(keys, "a", { key: "Enter" }).action === "none", "other keys are ignored");
});

// ── Templates (59) ─────────────────────────────────────────────────────

test("RUNTIME #2427: template gallery enumerates and searches localized names", function () {
  var T = loadForms().NS.templates;
  var all = T.all();
  assert(all.length === 5, "five built-in templates expected");
  assert(all.map(function (t) { return t.id; }).join("|") === "blank|letter|meeting|report|invoice", "stable order");
  assert(T.name(all[0]).indexOf("Blank") === 0, "name uses the fallback in Node");
  assert(T.search("").length === 5, "empty query returns all templates");
  assert(T.search("LETTER").map(function (t) { return t.id; }).join("") === "letter", "case-insensitive search");
  assert(T.search("<none>").length === 0, "no match returns empty");
  assert(all !== T.all(), "all() returns a defensive copy");
});

test("RUNTIME #2427: buildHtml produces structural placeholders, never fabricated data", function () {
  var NS = loadForms().NS;
  assert(NS.templates.buildHtml(null) === "<p><br></p>", "blank template placeholder");
  var letter = NS.templates.all()[1];
  var html = NS.templates.buildHtml(letter);
  assert(html.indexOf("<h2>") >= 0, "heading block rendered as h2");
  assert((html.match(/data-gb-form-control="1"/g) || []).length === 2, "two field blocks in the letter template");
  assert(html.indexOf("<p>") >= 0, "paragraph block rendered");
});

test("RUNTIME #2427: apply inserts placeholder HTML and records the selection", function () {
  var f = loadForms();
  var article = makeArticle(f, "doc-9");
  var letter = f.NS.templates.all()[1];
  assert(f.NS.templates.apply(letter) === true, "apply with an article succeeds");
  assert(article._inserted && article._inserted[0].position === "beforeend", "insertAdjacentHTML used");
  assert(article._inserted[0].html === f.NS.templates.buildHtml(letter), "exact shipped HTML inserted");
  assert(f.NS.state.selectedTemplate === "letter", "selection stored in state");
  var stored = JSON.parse(f.window.localStorage.getItem(f.NS.docKey("state")));
  assert(stored.selectedTemplate === "letter", "selection persisted");
  assert(loadForms().NS.templates.apply(letter) === false, "apply without an article is a no-op");
});

// ── Content controls (60) ──────────────────────────────────────────────

test("RUNTIME #2427: control type catalogue and typeName fallback", function () {
  var C = loadForms().NS.controls;
  assert(C.TYPES.map(function (t) { return t.id; }).join("|") === "text|dropdown|date|checkbox|repeat", "five control types");
  assert(C.typeName("text") === "Plain text", "localized fallback for text");
  assert(C.typeName("repeat") === "Repeating section", "localized fallback for repeat");
  assert(C.typeName("<unknown>") === "<unknown>", "unknown type returned verbatim");
});

test("RUNTIME #2427: create normalizes options and keeps explicit fields", function () {
  var C = loadForms().NS.controls;
  var d = C.create("dropdown", {
    id: "d1", label: "<lbl>", options: "A, B\nC", value: "B", required: true, placeholder: "<ph>",
  });
  assert(d.id === "d1" && d.type === "dropdown", "explicit id/type");
  assert(d.options.join("|") === "A|B|C", "comma/newline string split into options");
  assert(d.value === "B" && d.required === true && d.placeholder === "<ph>", "explicit fields kept");
  var plain = C.create();
  assert(plain.id.indexOf("ctl-") === 0 && plain.type === "text", "default id/type");
  assert(plain.options.length === 0 && plain.required === false && plain.value === "", "default fields");
});

test("RUNTIME #2427: attrs/render carry data-form-* metadata and escape labels", function () {
  var C = loadForms().NS.controls;
  var d = C.create("dropdown", { id: "d1", label: "<lbl>", options: ["A", "B"], value: "B", required: true });
  var attrs = C.attrs(d);
  ["data-gb-form-control=\"1\"", "data-form-id=\"d1\"", "data-form-type=\"dropdown\"",
    "data-form-label=\"&lt;lbl&gt;\"", "data-form-options=\"A,B\"", "data-form-default=\"B\"",
    "data-form-required=\"true\""].forEach(function (token) {
    assert(attrs.indexOf(token) >= 0, "missing attribute token: " + token);
  });
  var html = C.render(d);
  assert(html.indexOf("gb-forms-control-dropdown") >= 0, "type-specific class");
  assert(html.indexOf("<span class=\"gb-forms-control-label\">") >= 0, "label span");
  assert(html.indexOf("<span class=\"gb-forms-control-value\">B</span>") >= 0, "value span uses display()");
});

test("RUNTIME #2427: display picks a deterministic sentinel for each control kind", function () {
  var C = loadForms().NS.controls;
  assert(C.render(C.create("checkbox", { value: "true" })).indexOf(">[x]</span>") >= 0, "checked box");
  assert(C.render(C.create("checkbox", { value: "false" })).indexOf(">[ ]</span>") >= 0, "unchecked box");
  assert(C.render(C.create("repeat")).indexOf(">\u2026</span>") >= 0, "repeat sentinel");
  assert(C.render(C.create("text", { placeholder: "<ph>" })).indexOf(">&lt;ph&gt;</span>") >= 0, "placeholder sentinel is escaped");
});

test("RUNTIME #2427: parseFromElement round-trips a rendered control", function () {
  var f = loadForms();
  var C = f.NS.controls;
  var d = C.create("date", { id: "c-9", label: "<date>", value: "2026-01-01", required: true, placeholder: "<ph>" });
  var parsed = C.parseFromElement(controlElement(f, d));
  assert(parsed.id === "c-9" && parsed.type === "date", "id/type round-trip");
  assert(parsed.label === "<date>" && parsed.value === "2026-01-01", "label/value round-trip");
  assert(parsed.required === true && parsed.placeholder === "<ph>", "flags round-trip");
});

test("RUNTIME #2427: validate rejects empty required fields and accepts filled ones", function () {
  var C = loadForms().NS.controls;
  var required = C.create("text", { id: "r1", label: "<name>", required: true });
  var empty = C.validate(required, "   ");
  assert(empty.ok === false && empty.key === "docs-forms-control-missing-required", "empty required fails with key");
  assert(empty.params.label === "<name>", "label passed to the message params");
  assert(C.validate(required, "<filled>").ok === true, "filled required passes");
  var box = C.create("checkbox", { id: "b1", required: true });
  assert(C.validate(box, "false").ok === false, "unchecked box fails");
  assert(C.validate(box, "true").ok === true, "checked box passes");
  assert(C.validate(C.create("text"), "  ").ok === true, "optional empty passes");
  var errors = C.validateAll([required, box], { r1: "", b1: false });
  assert(errors.length === 2, "validateAll collects every failure");
});

test("RUNTIME #2427: syncFromDom reads the live control nodes in document order", function () {
  var f = loadForms();
  var C = f.NS.controls;
  var host = f.doc.createElement("div");
  host.appendChild(controlElement(f, C.create("text", { id: "c1", label: "<one>" })));
  host.appendChild(controlElement(f, C.create("dropdown", { id: "c2", label: "<two>" })));
  var found = C.syncFromDom(host);
  assert(found.length === 2 && f.NS.state.controls.length === 2, "both controls synced");
  assert(found[0].id === "c1" && found[1].id === "c2", "document order preserved");
});

test("RUNTIME #2427: insert records control HTML and persists the new state", function () {
  var f = loadForms();
  var article = makeArticle(f, "doc-ctl");
  var descriptor = f.NS.controls.insert("date", article);
  assert(descriptor && descriptor.type === "date", "descriptor returned");
  assert(article._inserted && article._inserted[0].html.indexOf('data-form-type="date"') >= 0, "control HTML inserted");
  assert(f.window.localStorage.getItem(f.NS.docKey("state")) !== null, "state persisted");
  assert(loadForms().NS.controls.insert("text", null) === null, "insert without a host is a no-op");
});

test("RUNTIME #2427: updateInDom patches attributes through the shipped render path", function () {
  var f = loadForms();
  var C = f.NS.controls;
  var host = f.doc.createElement("div");
  var node = controlElement(f, C.create("dropdown", { id: "u1", label: "<old>", options: ["A"] }));
  host.appendChild(node);
  assert(C.updateInDom("u1", { label: "<x>", options: ["A", "B"] }, host) === true, "patch applied");
  assert(node.getAttribute("data-form-label") === "<x>", "label attribute updated via outerHTML");
  assert(f.NS.state.controls[0].options.join("|") === "A|B", "options re-synced after patch");
  assert(C.updateInDom("missing", { label: "<y>" }, host) === false, "unknown id returns false");
});
