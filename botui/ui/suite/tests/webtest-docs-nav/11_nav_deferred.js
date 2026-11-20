/**
 * #2485 NAV slice: remaining runtime coverage, clearly-labelled [STATIC]
 * source assertions for backend-deferred aspects, and KNOWN BUG checks that
 * assert the observed shipped behaviour without patching production.
 *
 * STATIC checks read source text only. They never claim a runtime outcome for
 * org-scoped RBAC, server versioning, cross-document insert or chat autotext:
 * those are explicitly deferred to the backend by the shipped modules.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var assertEqual = h.assertEqual;
var fixture = h.fixture;
var article = h.article;
var byClass = h.byClass;
var readFile = h.readFile;
var deepEqual = h.deepEqual;

var CORE = "docs/modules/19_nav_core.js";
var PANE = "docs/modules/20_nav_pane.js";
var BOOKMARKS = "docs/modules/22_bookmarks.js";
var QUICKPARTS = "docs/modules/23_quickparts.js";
var INIT = "docs/modules/24_nav_init.js";
var LOCALES = "../../../botlib/locales/en/docs-nav.ftl";

// ── Additional runtime coverage ─────────────────────────────────────────

test("RUNTIME #2485: searchScopes indexes body, headers, footers, footnotes, comments and images", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<scopes>");
  function add(tag, attrs, text) {
    var node = fx.doc.createElement(tag);
    Object.keys(attrs || {}).forEach(function (key) { node.setAttribute(key, attrs[key]); });
    node.textContent = text;
    art.appendChild(node);
    return node;
  }
  add("p", {}, "<body-text>");
  add("header", { "data-gb-header": "1" }, "<header-text>");
  add("footer", { "data-gb-footer": "1" }, "<footer-text>");
  add("span", { "data-gb-footnote": "1" }, "<footnote-text>");
  add("div", { "data-gb-comment": "1" }, "<comment-text>");
  add("img", { alt: "<alt-text>" }, "");
  var scopes = NS.searchScopes(art);
  var names = scopes.map(function (scope) { return scope.scope; });
  ["body", "headers", "footers", "footnotes", "comments", "embedded"].forEach(function (name) {
    assert(names.indexOf(name) >= 0, name + " scope indexed");
  });
  assertEqual(scopes.length, 6, "one searchable entry per non-empty scope");
  assertEqual(NS.searchScopes(null).length, 0, "missing article yields no scopes");
});

test("RUNTIME #2485: validateHtml rejects active content before any bulk replacement", function () {
  var NS = fixture().NS;
  assertEqual(NS.validateHtml('<p><script>bad()</script></p>').ok, false, "script tag rejected");
  assertEqual(NS.validateHtml('<p onclick="bad()">x</p>').ok, false, "event attribute rejected");
  assertEqual(NS.validateHtml('<a href="javascript:bad()">x</a>').ok, false, "javascript href rejected");
  assertEqual(NS.validateHtml("", "<original>").ok, false, "blanking non-blank content rejected");
  assertEqual(NS.validateHtml("<p>fine</p>").ok, true, "benign markup accepted");
});

test("RUNTIME #2485: openPane and scroll-to-offset wire the desktop shell deterministically", function () {
  var fx = fixture();
  var NS = fx.NS;
  var tab = fx.addHost("panel-tab");
  tab.setAttribute("data-sidebar-tab", "panels");
  var tabClicks = 0;
  tab.click = function () { tabClicks += 1; };
  var sidebar = fx.addHost("sidebar-panels");
  var files = fx.addHost("sidebar-files");
  NS.openPane();
  assertEqual(sidebar.style.display, "flex", "panels sidebar shown");
  assertEqual(files.style.display, "none", "files sidebar hidden");
  assertEqual(tabClicks, 1, "panels tab activated through the shell");
  var art = article(fx, "<scroll>");
  var host = fx.addHost("docs-content");
  host.scrollTo = null;
  art.appendChild(host);
  NS.jumpToOffset(50);
  assertEqual(host.scrollTop, 50, "fixed offset applied when scrollTo is absent");
  host.scrollTop = 0;
  host.scrollTo = function (options) { host._lastScrollTo = options; };
  assert(NS.jumpToOffset(120) === art, "jumpToOffset returns the active article");
  deepEqual(host._lastScrollTo, { top: 120, behavior: "smooth" }, "smooth scroll with positive offset");
  host._lastScrollTo = null;
  NS.jumpToOffset(-10);
  assertEqual(host._lastScrollTo.top, 0, "negative offset clamped to zero");
});

test("RUNTIME #2485: boot wiring defers on DOM ready and refreshes on content swaps", function () {
  var fx = fixture({ readyState: "loading" });
  var NS = fx.NS;
  var navBtn = fx.addHost("navPaneBtn");
  var bmBtn = fx.addHost("bookmarkBtn");
  var qpBtn = fx.addHost("quickPartsBtn");
  assertEqual((navBtn.listeners.click || []).length, 0, "no handler bound before DOMContentLoaded");
  fx.doc.dispatch("DOMContentLoaded", {});
  assertEqual((navBtn.listeners.click || []).length, 1, "navigation button bound on boot");
  assertEqual((bmBtn.listeners.click || []).length, 1, "bookmark button bound on boot");
  assertEqual((qpBtn.listeners.click || []).length, 1, "quick-parts button bound on boot");
  var called = 0;
  var original = NS.refreshPane;
  NS.refreshPane = function () { called += 1; };
  fx.doc.dispatch("htmx:afterSwap", { target: { id: "docs-content" } });
  fx.doc.dispatch("htmx:afterSwap", { target: { id: "unrelated" } });
  fx.doc.dispatch("gb:deep-link", {});
  NS.refreshPane = original;
  assertEqual(called, 2, "only content swaps and deep links refresh the pane");
});

test("RUNTIME #2485: undo shortcut stays gated on an open find dialog", function () {
  var fx = fixture();
  var NS = fx.NS;
  var prevented = 0;
  function keydown(key) {
    fx.doc.dispatch("keydown", {
      key: key, ctrlKey: true,
      preventDefault: function () { prevented += 1; },
      stopPropagation: function () {},
    });
  }
  keydown("z");
  assertEqual(prevented, 0, "no dialog means the shortcut is inert");
  var dialog = fx.addHost("fr-dialog");
  dialog.className = "gb-fr-dialog";
  var restored = 0;
  NS.undoReplace = function () { restored += 1; return true; };
  keydown("Z");
  assertEqual(prevented, 1, "uppercase Z with an open dialog is consumed");
  assertEqual(restored, 1, "find-slice undo delegate invoked exactly once");
});

test("RUNTIME #2485: in-app dialog traps focus, restores focus and resolves on submit", function () {
  var fx = fixture();
  var NS = fx.NS;
  var host = fx.addHost("docs-modal-container");
  var restore = fx.addHost("restore-target");
  restore.focus();
  var value = "<unset>";
  NS.dialogPrompt("<message>", "<prefill>").then(function (result) { value = result; });
  var dialog = host.querySelector('[role="dialog"]');
  assertEqual(dialog.getAttribute("aria-modal"), "true", "dialog is modal");
  assertEqual(host.querySelector("#gb-ask-input").value, "<prefill>", "prompt prefill applied");
  fx.doc.dispatch("keydown", { key: "Tab", preventDefault: function () {} });
  assertEqual(fx.doc.activeElement.getAttribute("id"), "gb-ask-cancel", "Tab moves to the next control");
  fx.doc.dispatch("keydown", { key: "Tab", shiftKey: true, preventDefault: function () {} });
  assertEqual(fx.doc.activeElement.getAttribute("id"), "gb-ask-input", "Shift+Tab wraps to the input");
  host.querySelector("#gb-ask-input").value = "<answer>";
  host.querySelector("#gb-ask-ok").dispatch("click", {});
  assertEqual(value, "<answer>", "OK resolves the entered value");
  assertEqual(host.children.length, 0, "dialog removed after submit");
  assertEqual(fx.doc.activeElement, restore, "focus returned to the invoking control");
  var confirmed = "<unset>";
  NS.dialogConfirm("<message>").then(function (result) { confirmed = result; });
  host.querySelector("#gb-ask-cancel").dispatch("click", {});
  assertEqual(confirmed, false, "Cancel resolves false");
  NS.dialogConfirm("<message>").then(function (result) { confirmed = result; });
  fx.doc.dispatch("keydown", { key: "Escape", preventDefault: function () {} });
  assertEqual(confirmed, false, "Escape resolves false");
  assertEqual(host.children.length, 0, "Escape removes the dialog");
});

test("RUNTIME #2485: content library delegates variable entry, copy and link insertion", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<library>");
  var modal = fx.addHost("docs-modal-container");
  NS.openContentLibrary();
  assertEqual(modal.querySelectorAll(".gb-qp-row").length, 2, "both built-ins listed");
  assert(modal.querySelector('[role="dialog"]').getAttribute("aria-modal") === "true", "library is modal");
  var link = modal.querySelector("[data-qp-link]");
  modal.dispatch("click", { target: link, preventDefault: function () {} });
  assert(modal.querySelector("#gb-ask-input"), "link insertion asks for the template variable");
  modal.querySelector("#gb-ask-input").value = "7500";
  modal.querySelector("#gb-ask-ok").dispatch("click", {});
  var linked = art.querySelector(".gb-quickpart-linked");
  assert(linked, "linked instance inserted after variable entry");
  assert(linked.innerHTML.indexOf("7500") >= 0, "entered value reached the inserted block");
  NS.openContentLibrary();
  modal.dispatch("click", { target: modal.querySelector("[data-qp-copy]"), preventDefault: function () {} });
  modal.querySelector("#gb-ask-ok").dispatch("click", {});
  assertEqual(art.querySelectorAll(".gb-quickpart").length, 2, "copy inserted alongside the linked block");
  assertEqual(art.querySelectorAll(".gb-quickpart-linked").length, 1, "only one linked binding created");
});

// ── [STATIC] source assertions (backend-deferred / wiring only) ─────────

test("[STATIC] #2485: org-scoped RBAC and versioning are explicitly backend-deferred", function () {
  var parts = readFile(QUICKPARTS);
  assert(parts.indexOf("Org-scoped RBAC/versioning") >= 0 && parts.indexOf("backend concern") >= 0,
    "quick-parts module labels RBAC/versioning as backend work");
  [CORE, PANE, BOOKMARKS, QUICKPARTS, INIT].forEach(function (file) {
    var src = readFile(file);
    assert(src.indexOf("fetch(") < 0, file + " performs no server round-trip");
    assert(src.indexOf("XMLHttpRequest") < 0, file + " performs no XHR");
    assert(src.indexOf("resolve_user_role") < 0, file + " does not invent a client-side role gate");
  });
  assert(parts.indexOf("api/") < 0, "no pretend org-scoped API endpoint in the frontend slice");
});

test("[STATIC] #2485: no native dialogs; accessible roles and status regions are shipped", function () {
  [CORE, PANE, BOOKMARKS, QUICKPARTS, INIT].forEach(function (file) {
    var src = readFile(file);
    assert(src.indexOf("window.prompt(") < 0 && src.indexOf("window.confirm(") < 0,
      file + " never calls a blocking native dialog");
  });
  assert(readFile(CORE).indexOf("NS.dialogPrompt") >= 0 && readFile(CORE).indexOf("NS.dialogConfirm") >= 0,
    "shared in-app prompt/confirm exported from the core");
  assert(readFile(PANE).indexOf('role="status" aria-live="polite"') >= 0, "nav pane status region is polite");
  [CORE, BOOKMARKS, QUICKPARTS].forEach(function (file) {
    assert(readFile(file).indexOf('aria-modal="true"') >= 0, file + " ships a modal dialog contract");
  });
});

test("[STATIC] #2485: every catalog key used by the slice exists in the shipped locale", function () {
  [CORE, PANE, BOOKMARKS, QUICKPARTS, INIT].forEach(function (file) {
    var src = readFile(file);
    assert(src.indexOf("auto-") < 0, file + " invents no auto-* key (#2542)");
  });
  var used = {};
  [CORE, PANE, BOOKMARKS, QUICKPARTS, INIT].forEach(function (file) {
    var matches = readFile(file).match(/(?:tr|docsT)\("([a-z0-9-]+)"/g) || [];
    matches.forEach(function (entry) {
      var key = entry.replace(/^(?:tr|docsT)\("/, "").replace(/"$/, "");
      used[key] = true;
      assert(key.indexOf("docs-") === 0, key + " uses the catalog namespace");
    });
  });
  var keys = Object.keys(used);
  assert(keys.length > 20, keys.length + " catalog keys exercised by this slice");
  var catalog = readFile(LOCALES);
  keys.forEach(function (key) {
    assert(catalog.indexOf(key + " =") >= 0, key + " defined in en/docs-nav.ftl");
  });
});

test("[STATIC] #2485: shipped loader order matches the shared-sandbox order", function () {
  var html = readFile("docs/docs.html");
  var order = ["19_nav_core.js", "20_nav_pane.js", "22_bookmarks.js", "23_quickparts.js", "24_nav_init.js"];
  var previous = -1;
  order.forEach(function (name) {
    var index = html.indexOf(name);
    assert(index > previous, name + " loaded in production order");
    previous = index;
  });
  assert(readFile("tests/webtest-docs-nav/00_harness.js").indexOf('"docs/modules/19_nav_core.js"') >= 0,
    "harness loads the same core module first");
});

// ── KNOWN BUGS: assert observed shipped behaviour, report only ───────────

test("KNOWN BUG #2485: reconcile skips the second adjacent event attribute", function () {
  var NS = fixture().NS;
  var markup = '<p style="color:red" onclick="a()" onmouseover="b()"><span style="x">t</span></p>';
  var result = NS.reconcileQuickPartStyles(markup);
  assert(result.indexOf("style=") < 0, "style attributes removed from descendants");
  assert(result.indexOf('onclick="a()"') < 0, "first event attribute removed");
  assert(result.indexOf('onmouseover="b()"') >= 0,
    "observed: adjacent second event attribute survives the live NamedNodeMap iteration");
});

test("KNOWN BUG #2485: propagation replaces filled variables with raw placeholders", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<vars>");
  NS.insertQuickPart("qp-liability", true, { amount: "5000" });
  var linked = art.querySelector('[data-gb-quickpart="qp-liability"]');
  assert(linked.innerHTML.indexOf("5000") >= 0, "linked insert starts with the filled value");
  NS.updateQuickPart("qp-liability", "<p>Cap is {{amount}}.</p>", true);
  assert(linked.innerHTML.indexOf("{{amount}}") >= 0,
    "observed: propagation re-renders with empty values, exposing the raw placeholder");
  assert(linked.innerHTML.indexOf("5000") < 0, "observed: the per-insert value is lost");
});

test("KNOWN BUG #2485: drag-down inserts the moved section inside the target section", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<split>");
  function section(id, text) {
    var heading = fx.doc.createElement("h1");
    heading.setAttribute("id", id);
    heading.textContent = text;
    var body = fx.doc.createElement("p");
    body.textContent = text + " body";
    art.appendChild(heading);
    art.appendChild(body);
  }
  section("alpha", "<alpha>");
  section("beta", "<beta>");
  var pane = fx.addHost("docs-nav-pane");
  NS.refreshPane();
  var rows = pane.querySelectorAll(".gb-nav-heading");
  function event() { return { preventDefault: function () {}, dataTransfer: { effectAllowed: "" } }; }
  rows[0].dispatch("dragstart", event());
  rows[1].dispatch("drop", event());
  deepEqual(art.children.map(function (node) { return node.textContent; }),
    ["<beta>", "<alpha>", "<alpha> body", "<beta> body"],
    "observed: target heading and its body are separated by the moved section");
});
