/**
 * #2485 RUNTIME verification of the shipped NAV IIFEs (19, 20, 22, 23, 24).
 * Every assertion below executes the real production code loaded by the
 * harness. Fixtures only supply deterministic inputs; no production helper is
 * reimplemented. DOM sentinels are code-like tokens, never user-facing text.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var assertEqual = h.assertEqual;
var fixture = h.fixture;
var article = h.article;
var byClass = h.byClass;
var deepEqual = h.deepEqual;

var API = [
  "tr", "escapeHtml", "docKey", "readStore", "writeStore", "validateHtml",
  "searchScopes", "flash", "jumpTo", "save", "status", "collectSection",
  "headings", "ask", "dialogPrompt", "dialogConfirm", "getArticle",
  "openPane", "refreshPane", "navSetQuery", "jumpToOffset",
  "mintBookmarkId", "addBookmark", "deleteBookmark", "gotoBookmark",
  "insertBookmarkRef", "openBookmarkDialog", "quickParts",
  "quickPartVariables", "fillQuickPartVariables", "reconcileQuickPartStyles",
  "insertQuickPart", "updateQuickPart", "propagateLinked", "expandAutotext",
  "bindQuickPartKeys", "openContentLibrary",
];

test("RUNTIME #2485: the five shipped NAV IIFEs load into one shared sandbox", function () {
  var fx = fixture();
  var NS = fx.NS;
  assert(NS, "DocsNav global installed");
  API.forEach(function (name) {
    assertEqual(typeof NS[name], "function", name + " exported on DocsNav");
  });
  assertEqual(typeof NS.openFindReplace, "undefined",
    "find/replace module is not part of this slice (owned by webtest-docs-find)");
  assertEqual(typeof NS.undoReplace, "undefined", "undoReplace is loaded only by the find slice");
  assertEqual(typeof fx.sandbox.window.gbT, "function", "gbT stub present for the shipped tr fallback");
});

test("RUNTIME #2485: tr, escapeHtml and document-scoped storage are deterministic", function () {
  var fx = fixture();
  var NS = fx.NS;
  assertEqual(NS.tr("docs-nav-headings", "<fallback>", { n: 3 }), "<fallback>",
    "tr delegates to the shipped docsT");
  assertEqual(fx.trCalls[fx.trCalls.length - 1].key, "docs-nav-headings", "tr forwards the catalog key");
  deepEqual(fx.trCalls[fx.trCalls.length - 1].params, { n: 3 }, "tr forwards params");
  assertEqual(NS.escapeHtml("<a href='x'>&\""), "&lt;a href=&#39;x&#39;&gt;&amp;&quot;",
    "escapeHtml escapes markup characters");
  assertEqual(NS.docKey("bookmarks"), "docs2485:current:bookmarks", "docKey falls back to current");
  article(fx, "<d1>");
  assertEqual(NS.docKey("bookmarks"), "docs2485:<d1>:bookmarks", "docKey tracks the active doc id");
  assertEqual(NS.writeStore("probe", { a: 1 }), true, "writeStore succeeds");
  deepEqual(NS.readStore("probe", null), { a: 1 }, "readStore round-trips JSON");
  assertEqual(NS.readStore("absent", "<missing>"), "<missing>", "missing key returns the fallback");
  fx.store[NS.docKey("broken")] = "{not json";
  assertEqual(NS.readStore("broken", "<fallback>"), "<fallback>", "corrupt JSON returns the fallback");
});

test("RUNTIME #2485: getArticle prefers the active article, then the contenteditable query", function () {
  var fx = fixture({ noActiveArticle: true });
  var node = fx.doc.createElement("article");
  node.setAttribute("contenteditable", "true");
  fx.doc.body.appendChild(node);
  assertEqual(fx.NS.getArticle(), node, "falls back to article[contenteditable] query");
  var withActive = fixture();
  var other = withActive.doc.createElement("article");
  other.setAttribute("contenteditable", "true");
  withActive.doc.body.appendChild(other);
  withActive.setArticle(other);
  assertEqual(withActive.NS.getArticle(), other, "active article wins over the DOM query");
});

test("RUNTIME #2485: headings() and collectSection() model the heading outline", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<outline>");
  function add(tag, id, text) {
    var node = fx.doc.createElement(tag);
    node.setAttribute("id", id);
    node.textContent = text;
    art.appendChild(node);
    return node;
  }
  var h1 = add("h1", "s1", "One");
  add("p", "p1", "alpha");
  var h2 = add("h2", "s2", "Child");
  add("p", "p2", "beta");
  var h1b = add("h1", "s3", "Two");
  add("p", "p3", "gamma");
  assertEqual(NS.headings().length, 3, "three headings in document order");
  assertEqual(NS.headings()[1].getAttribute("id"), "s2", "outline keeps DOM order");
  assertEqual(NS.collectSection(h2).length, 2, "child section stops at the next peer heading");
  assertEqual(NS.collectSection(h1).length, 4, "parent section includes its child heading");
  assertEqual(NS.collectSection(h1b).length, 2, "last section stops at the document end");
  assertEqual(NS.collectSection(null).length, 0, "missing heading yields an empty section");
});

test("RUNTIME #2485: nav pane builds five accessible tabs and one polite status region", function () {
  var fx = fixture();
  var NS = fx.NS;
  article(fx, "<pane>");
  var pane = fx.addHost("docs-nav-pane");
  NS.refreshPane();
  assertEqual(pane.querySelectorAll(".gb-nav-tab").length, 5, "five navigation tabs");
  deepEqual(pane.querySelectorAll(".gb-nav-tab").map(function (tab) {
    return tab.getAttribute("data-nav-tab");
  }), ["headings", "pages", "results", "comments", "bookmarks"], "shipped tab order");
  assertEqual(pane.querySelectorAll(".gb-nav-section").length, 5, "one panel per tab");
  assertEqual(pane.querySelector('[role="tablist"]').getAttribute("role"), "tablist", "tablist role set");
  assertEqual(pane.querySelector(".gb-nav-status").getAttribute("role"), "status", "status role set");
  assertEqual(pane.querySelector(".gb-nav-status").getAttribute("aria-live"), "polite", "status is polite");
  var keys = fx.trCalls.map(function (call) { return call.key; });
  ["docs-nav-headings", "docs-nav-pages", "docs-nav-results", "docs-nav-comments",
    "docs-nav-bookmarks"].forEach(function (key) {
    assert(keys.indexOf(key) >= 0, key + " resolved through the catalog");
  });
  assertEqual(pane.dataset.built, "1", "build runs once per pane");
  NS.refreshPane();
  assertEqual(pane.querySelectorAll(".gb-nav-tab").length, 5, "refresh does not duplicate tabs");
});

test("RUNTIME #2485: tab activation swaps sections and renders pages/results", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<tabs>");
  art.dataset.pageH = "100";
  art.scrollHeight = 250;
  var pane = fx.addHost("docs-nav-pane");
  var heading = fx.doc.createElement("h1");
  heading.setAttribute("id", "s1");
  heading.textContent = "<intro>";
  art.appendChild(heading);
  NS.refreshPane();
  var tabs = pane.querySelectorAll(".gb-nav-tab");
  var sections = pane.querySelectorAll(".gb-nav-section");
  function activate(index) {
    pane.dispatch("click", { target: tabs[index], preventDefault: function () {} });
  }
  assertEqual(tabs[0].classList.contains("is-active"), true, "headings starts active");
  assertEqual(sections[0].hidden, false, "headings panel visible");
  activate(1);
  assertEqual(tabs[1].classList.contains("is-active"), true, "pages active after click");
  assertEqual(sections[0].hidden, true, "headings panel hidden");
  var offsets = sections[1].querySelectorAll("[data-nav-offset]").map(function (row) {
    return row.getAttribute("data-nav-offset");
  });
  deepEqual(offsets, ["0", "100", "200"], "pages derive from scrollHeight and pageH");
  NS.findMatches = function () {
    return [
      { scope: "body", text: "first hit", id: "s1", headingIndex: 0 },
      { scope: "comments", text: "second hit" },
    ];
  };
  NS.navSetQuery("<needle>");
  assertEqual(tabs[2].classList.contains("is-active"), true, "navSetQuery activates results");
  assertEqual(sections[2].querySelectorAll("[data-nav-jump]").length, 2, "results rows rendered");
  assert(sections[2].innerHTML.indexOf("[comments] second hit") >= 0, "scope label shown for comments");
});

test("RUNTIME #2485: heading rows carry indentation, stable targets and click-to-jump", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<rows>");
  function heading(tag, id, text) {
    var node = fx.doc.createElement(tag);
    if (id) node.setAttribute("id", id);
    node.textContent = text;
    art.appendChild(node);
    return node;
  }
  heading("h1", "s1", "Parent");
  heading("h2", null, "Child");
  var pane = fx.addHost("docs-nav-pane");
  NS.refreshPane();
  var rows = pane.querySelectorAll(".gb-nav-heading");
  assertEqual(rows.length, 2, "one row per heading");
  assertEqual(rows[0].getAttribute("data-nav-target"), "s1", "explicit id used as target");
  assertEqual(rows[1].getAttribute("data-nav-target"), "sec-1", "missing id gets a stable index target");
  assert(rows[0].getAttribute("style").indexOf("padding-left:8px") >= 0, "h1 indentation");
  assert(rows[1].getAttribute("style").indexOf("padding-left:20px") >= 0, "h2 indentation");
  var jumped = [];
  fx.sandbox.window.docsScrollToHeading = function (node) { jumped.push(node.textContent); };
  pane.dispatch("click", { target: rows[0], preventDefault: function () {} });
  deepEqual(jumped, ["<intro>"], "heading click delegates to the heading scroller");
});

test("RUNTIME #2485: drag-to-reorder moves a heading with its section and saves once", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<drag>");
  function section(id, text) {
    var heading = fx.doc.createElement("h1");
    heading.setAttribute("id", id);
    heading.textContent = text;
    var body = fx.doc.createElement("p");
    body.textContent = text + " body";
    art.appendChild(heading);
    art.appendChild(body);
  }
  section("one", "One");
  section("two", "Two");
  var pane = fx.addHost("docs-nav-pane");
  NS.refreshPane();
  var rows = pane.querySelectorAll(".gb-nav-heading");
  var saves = 0;
  var originalSave = NS.save;
  NS.save = function () { saves += 1; };
  function dragEvent() { return { preventDefault: function () {}, dataTransfer: { effectAllowed: "" } }; }
  rows[0].dispatch("dragstart", dragEvent());
  rows[1].dispatch("dragover", dragEvent());
  assertEqual(rows[1].classList.contains("gb-nav-drop"), true, "dragover marks the drop target");
  rows[1].dispatch("drop", dragEvent());
  NS.save = originalSave;
  deepEqual(art.children.map(function (node) { return node.textContent; }),
    ["Two", "Two body", "One", "One body"],
    "target section stays intact ahead of the moved section; nodes never duplicated");
  assertEqual(art.children.length, 4, "reorder moves nodes, never duplicates them");
  assertEqual(saves, 1, "exactly one save per reorder");
  assert(fx.trCalls.some(function (call) { return call.key === "docs-nav-reordered"; }),
    "reorder reports through the catalog");
});

test("RUNTIME #2485: bookmark CRUD persists state, inserts an anchor and gates on its marker", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<bm>");
  var pane = fx.addHost("docs-nav-pane");
  var id = NS.addBookmark("Place", 7);
  assertEqual(id, "bm-00000001-0000-4000-8000-000000000000", "bookmark id is deterministic");
  var stored = JSON.parse(fx.store[NS.docKey("bookmarks")]);
  assertEqual(stored.length, 1, "one bookmark stored");
  assertEqual(stored[0].offset, 7, "explicit caret offset preserved");
  var marker = art.querySelector('[id="' + id + '"]');
  assert(marker, "real anchor marker inserted in the article");
  assertEqual(marker.getAttribute("data-gb-bookmark"), "Place", "marker carries the bookmark name");
  assertEqual(marker.contentEditable, "false", "marker is not editable");
  assert(fx.trCalls.some(function (call) { return call.key === "docs-bm-created"; }), "creation status keyed");
  assertEqual(NS.gotoBookmark(id), true, "go-to succeeds when the marker exists");
  assertEqual(NS.insertBookmarkRef(id), true, "cross-reference succeeds when the marker exists");
  var link = art.querySelector("[data-gb-xref]");
  assertEqual(link.getAttribute("href"), "#" + id, "cross-reference links to the marker");
  assertEqual(link.textContent, "Place", "cross-reference uses the bookmark name");
  assertEqual(NS.gotoBookmark("<missing>"), false, "go-to is gated on marker existence");
  assertEqual(NS.insertBookmarkRef("<missing>"), false, "cross-reference is gated on marker existence");
  NS.deleteBookmark(id);
  deepEqual(JSON.parse(fx.store[NS.docKey("bookmarks")]), [], "delete clears stored state");
  assertEqual(art.querySelector('[id="' + id + '"]'), null, "delete removes the marker");
  assertEqual(art.querySelectorAll(".gb-nav-bookmark").length, 0, "bookmarks pane reflects deletion");
});

test("RUNTIME #2485: bookmark storage is isolated per document id", function () {
  var fx = fixture();
  var NS = fx.NS;
  var first = article(fx, "<doc-a>");
  NS.addBookmark("A", 1);
  var second = fx.doc.createElement("article");
  second.setAttribute("contenteditable", "true");
  second.dataset.docId = "<doc-b>";
  fx.doc.body.appendChild(second);
  fx.setArticle(second);
  NS.addBookmark("B", 2);
  assertEqual(JSON.parse(fx.store["docs2485:<doc-a>:bookmarks"]).length, 1, "first document untouched");
  assertEqual(JSON.parse(fx.store["docs2485:<doc-b>:bookmarks"]).length, 1, "second document has its own bookmark");
  assertEqual(first.querySelectorAll(".gb-bookmark-marker").length, 1, "first marker still present");
  assertEqual(second.querySelectorAll(".gb-bookmark-marker").length, 1, "second marker present");
});

test("RUNTIME #2485: quick-part variables are discovered, filled without escaping and isolated", function () {
  var fx = fixture();
  var NS = fx.NS;
  var parts = NS.quickParts();
  assertEqual(parts.length, 2, "two built-ins shipped");
  assertEqual(parts[0].id, "qp-liability", "liability part present");
  assertEqual(parts[0].shortcut, "/liability", "liability shortcut present");
  assertEqual(parts[0].version, 1, "built-ins start at version one");
  deepEqual(NS.quickPartVariables("<p>{{a}} and {{a}} and {{ b }}</p>"), ["a", "b"],
    "variables are deduplicated and whitespace tolerant");
  assertEqual(NS.fillQuickPartVariables("{{a}}/{{missing}}", { a: "<value>" }), "<value>/{{missing}}",
    "known variables filled, unknown left intact");
  parts[0].content = "<changed>";
  assert(NS.quickParts()[0].content !== "<changed>", "callers cannot mutate the shipped library");
});

test("RUNTIME #2485: insert-as-copy and insert-and-link produce distinct DOM contracts", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<qp>");
  assertEqual(NS.insertQuickPart("qp-liability", false, { amount: "5000" }), true, "copy insert succeeds");
  var copy = art.querySelector(".gb-quickpart");
  assert(copy, "copy node inserted");
  assertEqual(copy.className, "gb-quickpart", "copy has no linked class");
  assertEqual(copy.getAttribute("data-gb-quickpart"), null, "copy has no library binding");
  assert(copy.innerHTML.indexOf("limited to 5000.") >= 0, "copy has the filled variable value");
  assertEqual(NS.insertQuickPart("qp-liability", true, { amount: "5000" }), true, "linked insert succeeds");
  var linked = art.querySelector('[data-gb-quickpart="qp-liability"]');
  assert(linked, "linked node inserted");
  assert(linked.classList.contains("gb-quickpart-linked"), "linked class marks propagation target");
  assertEqual(linked.getAttribute("data-gb-version"), "1", "linked node records its version");
  assert(fx.trCalls.some(function (call) { return call.key === "docs-qp-inserted-copy"; }), "copy status keyed");
  assert(fx.trCalls.some(function (call) { return call.key === "docs-qp-inserted-linked"; }), "link status keyed");
  assertEqual(NS.insertQuickPart("<unknown>", false, {}), false, "insert is gated on a known part");
});

test("RUNTIME #2485: version bump propagates only to linked instances, once", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<prop>");
  NS.insertQuickPart("qp-liability", false, { amount: "5000" });
  NS.insertQuickPart("qp-liability", true, { amount: "5000" });
  var copy = art.querySelectorAll(".gb-quickpart").filter(function (node) {
    return !node.classList.contains("gb-quickpart-linked");
  })[0];
  assertEqual(NS.updateQuickPart("qp-liability", "<p>First {{amount}}.</p>", true), true, "update succeeds");
  var linked = art.querySelector('[data-gb-quickpart="qp-liability"]');
  assertEqual(linked.getAttribute("data-gb-version"), "2", "linked version follows the library");
  assert(linked.innerHTML.indexOf("First") >= 0, "linked content replaced");
  assert(copy.innerHTML.indexOf("limited to 5000.") >= 0, "copy is deliberately untouched");
  assertEqual(NS.updateQuickPart("qp-liability", "<p>Second {{amount}}.</p>", false), true, "non-propagating update");
  assert(linked.innerHTML.indexOf("First") >= 0 && linked.innerHTML.indexOf("Second") < 0,
    "linked instance stays stale without propagation");
  assertEqual(NS.propagateLinked(), 1, "explicit propagation finds the stale instance");
  assert(linked.innerHTML.indexOf("Second") >= 0, "stale instance catches up");
  assertEqual(NS.propagateLinked(), 0, "second propagation is a no-op");
  assertEqual(NS.updateQuickPart("<unknown>", "<x>", true), false, "update is gated on a known part");
});

test("RUNTIME #2485: autotext expands a trailing shortcut and ignores unknown tokens", function () {
  var fx = fixture();
  var NS = fx.NS;
  var node = fx.doc.createTextNode("please see /liability");
  assertEqual(NS.expandAutotext("please see /liability", node, 21), true, "known shortcut expands");
  assert(node.data.indexOf("liability shall be limited to") >= 0, "approved clause text inserted");
  assert(node.data.indexOf("/liability") < 0, "shortcut token removed");
  var before = node.data;
  assertEqual(NS.expandAutotext("please see /unknown", node, 19), false, "unknown shortcut is a no-op");
  assertEqual(node.data, before, "unknown token leaves the text node unchanged");
  assertEqual(NS.expandAutotext("no shortcut here", node, 16), false, "plain text is a no-op");
});

test("RUNTIME #2485: keyboard autotext is bound once and guarded by Space/Enter", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = article(fx, "<keys>");
  art.isContentEditable = true;
  var node = fx.doc.createTextNode("run /signature");
  art.appendChild(node);
  var before = (fx.doc._listeners.keydown || []).length;
  NS.bindQuickPartKeys();
  assertEqual((fx.doc._listeners.keydown || []).length, before, "binding is idempotent");
  fx.setCaret(node, node.data.length);
  var prevented = 0;
  fx.doc.dispatch("keydown", {
    key: " ", target: art, preventDefault: function () { prevented += 1; },
  });
  assertEqual(prevented, 1, "expansion consumes the Space key");
  assert(node.data.indexOf("Sincerely,") >= 0, "signature shortcut expanded from the keyboard");
  fx.doc.dispatch("keydown", { key: "ArrowLeft", target: art, preventDefault: function () {} });
  assertEqual(prevented, 1, "non-triggering keys are ignored");
});
