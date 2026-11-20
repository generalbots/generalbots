/**
 * #2425 RUNTIME DOM smoke + clearly-labelled STATIC source assertions.
 *
 * Runtime checks here execute the shipped render/boot IIFEs against the
 * harness FakeNode DOM. STATIC checks cover only private wiring or behaviour
 * the shipped modules explicitly defer to the backend (403 enforcement,
 * DOCX w:ins/w:del, colour PDF export, @mention notifications, native prompt
 * removal); they read source text and never fabricate a runtime outcome.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var assertEqual = h.assertEqual;
var fixture = h.fixture;
var byClass = h.byClass;
var readFile = h.readFile;

// ── More runtime: view modes, proposal gate, DOM renders ───────────────

test("RUNTIME #2425: mode/type labels, view-mode class and proposal gate", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = h.article(fx, "<d1>");
  assertEqual(NS.modeLabel("simple"), NS.MODE_LABELS.simple[1], "modeLabel uses shipped label");
  assert(typeof NS.typeLabel("insert") === "string" && NS.typeLabel("insert").length > 0,
    "typeLabel returns a string");
  assertEqual(NS.setViewMode("simple"), true, "setViewMode accepts a valid mode");
  assert(art.classList.contains("gb-review-mode-simple"), "article receives the mode class");
  assertEqual(art._classes.filter(function (c) { return c.indexOf("gb-review-mode-") === 0; }).length, 1,
    "exactly one mode class is active");
  assertEqual(NS.setViewMode("bogus"), false, "invalid mode rejected");

  NS.toggleSuggestionMode(true);
  assertEqual(NS.state.suggestionMode, true, "suggestion mode on");
  NS.toggleProtectMode(true);
  assertEqual(NS.state.protectMode, true, "protect mode on");
  assertEqual(NS.state.suggestionMode, false, "protect mode disables suggestion mode");
  NS.toggleSuggestionMode(true);
  assertEqual(NS.state.protectMode, false, "suggestion mode disables protect mode");

  NS.toggleProtectMode(true);
  var blocked = NS.proposeEdit({ before: "<old>", after: "<new>" });
  assertEqual(blocked.blocked, true, "protect mode blocks edits");
  assertEqual(blocked.recorded, false, "blocked edit is not recorded");
  NS.toggleProtectMode(false);
  NS.toggleSuggestionMode(true);
  var suggested = NS.proposeEdit({ before: "<old>", after: "<new>" });
  assertEqual(suggested.recorded, true, "suggestion mode records the edit");
  assertEqual(suggested.applied, false, "suggestion mode does not apply the edit");
  assertEqual(suggested.change.suggestOnly, true, "recorded change is suggestion-only");
  NS.toggleSuggestionMode(false);
  var applied = NS.proposeEdit({});
  assertEqual(applied.applied, true, "normal mode applies the edit directly");
  assertEqual(NS.state.changes.length, 1, "no extra change recorded in normal mode");
});

test("RUNTIME #2425: renderMarkup builds a margin balloon and clears it on decision", function () {
  var fx = fixture();
  var NS = fx.NS;
  var art = h.article(fx, "<d1>");
  var span = fx.doc.createElement("span");
  span.dataset.reviewAnchor = "<a1>";
  art.appendChild(span);
  var change = NS.addChange({ anchorId: "<a1>", author: "alice", before: "<old>", after: "<new>" });
  NS.renderMarkup(art);
  assertEqual(span.getAttribute("data-review-change"), change.id, "anchor span marked with change id");
  var margin = fx.doc.body.querySelector(".gb-review-margin");
  assert(margin, "margin aside created");
  assertEqual(margin.getAttribute("data-i18n-aria-label"), "docs-review-changes", "margin aria key set");
  var balloons = byClass(margin, "gb-review-balloon");
  assertEqual(balloons.length, 1, "one balloon for the open change");
  assertEqual(balloons[0].getAttribute("data-review-balloon"), change.id, "balloon carries change id");
  NS.acceptChange(change.id);
  NS.renderMarkup(art);
  assertEqual(byClass(fx.doc.body, "gb-review-balloon").length, 0, "decided change loses its balloon");
  assertEqual(fx.doc.body.querySelectorAll(".gb-review-margin").length, 1, "old margin replaced, not stacked");
});

test("RUNTIME #2425: richText renders @mention chips as non-editable tokens", function () {
  var NS = fixture().NS;
  var wrap = NS.richText("hi @alice and @alice");
  var chips = byClass(wrap, "gb-review-mention");
  assertEqual(chips.length, 2, "two mention chips rendered");
  assertEqual(chips[0].getAttribute("data-review-mention"), "alice", "chip exposes the mentioned name");
  assert(wrap.textContent.indexOf("@alice") >= 0, "chip text still shows the @token");
});

test("RUNTIME #2425: renderCommentPanel renders only open threads and reacts to filters", function () {
  var fx = fixture();
  var NS = fx.NS;
  NS.setCurrentUser("alice");
  NS.addThread({ anchorId: "<p>", text: "@bob please" });
  var host = fx.doc.createElement("div");
  fx.doc.body.appendChild(host);
  NS.renderCommentPanel(host);
  assertEqual(byClass(host, "gb-review-thread").length, 1, "open thread is rendered");
  var select = host.querySelector("select");
  assert(select, "filter select rendered");
  select.value = "mentions";
  select.dispatch("change", {});
  assertEqual(byClass(host, "gb-review-thread").length, 0, "mentions filter hides non-matching thread");
});

test("RUNTIME #2425: renderCompare builds hunks, runs redline and merges on click", function () {
  var fx = fixture();
  var NS = fx.NS;
  var host = fx.doc.createElement("div");
  fx.doc.body.appendChild(host);
  NS.renderCompare(host);
  assertEqual(host.querySelectorAll("textarea").length, 2, "two compare textareas rendered");
  var run = byClass(host, "gb-review-btn-primary")[0];
  assert(run, "run button rendered");
  var areas = host.querySelectorAll("textarea");
  areas[0].value = "a\nb";
  areas[1].value = "a\nc";
  run.dispatch("click", {});
  assertEqual(NS.hunks().length, 2, "click computes a redline");
  assertEqual(byClass(host, "gb-review-hunk").length, 2, "one row per hunk");
  var merge = host.querySelector(".gb-review-merge-text");
  assert(merge && merge.textContent.length > 0, "merge preview rendered");
  var accept = byClass(host, "gb-review-hunk")[0].querySelectorAll("button")[0];
  accept.dispatch("click", {});
  assertEqual(NS.hunks()[0].status, "accepted", "per-hunk accept updates model");
});

test("RUNTIME #2425: renderDashboard reflects pure counts into the DOM", function () {
  var fx = fixture();
  var NS = fx.NS;
  NS.addChange({ author: "alice" });
  var accepted = NS.addChange({ author: "bob" });
  NS.acceptChange(accepted.id);
  var host = fx.doc.createElement("div");
  fx.doc.body.appendChild(host);
  NS.renderDashboard(host);
  var values = host.querySelectorAll(".gb-review-metric-value");
  assertEqual(values.length, 4, "four dashboard metrics rendered");
  assertEqual(values[0].textContent, "1", "total-open metric derives from state");
  assertEqual(host.querySelectorAll(".gb-review-author-row").length, 2, "one author row per author");
});

test("RUNTIME #2425: renderPermissions renders the matrix and reacts to role change", function () {
  var fx = fixture();
  var NS = fx.NS;
  var host = fx.doc.createElement("div");
  fx.doc.body.appendChild(host);
  NS.renderPermissions(host);
  var select = host.querySelector("select");
  assert(select, "role select rendered");
  assertEqual(byClass(host, "gb-review-allow").length, 4, "editor allows four capabilities");
  select.value = "viewer";
  select.dispatch("change", {});
  assertEqual(byClass(host, "gb-review-allow").length, 1, "viewer allows only commenting");
});

test("RUNTIME #2425: dialog shell is focus-trapped and closes on Escape", function () {
  var fx = fixture();
  var NS = fx.NS;
  var shell = NS.dialog.shell({ titleKey: "docs-review-title", titleFallback: "<x>" });
  assertEqual(shell.root.getAttribute("role"), "dialog", "dialog role set");
  assertEqual(shell.root.getAttribute("aria-modal"), "true", "aria-modal set");
  var status = shell.root.querySelector(".gb-review-status");
  assertEqual(status.getAttribute("role"), "status", "status region role set");
  assertEqual(status.getAttribute("aria-live"), "polite", "status region is polite");
  assert(fx.doc.body.querySelector(".gb-review-backdrop"), "backdrop appended to host");
  assert(fx.doc.activeElement, "first focusable receives focus");
  fx.doc.dispatch("keydown", { key: "Tab", preventDefault: function () {} });
  fx.doc.dispatch("keydown", { key: "Escape", preventDefault: function () {} });
  assertEqual(fx.doc.body.querySelectorAll(".gb-review-backdrop").length, 0, "Escape removes dialog");
});

test("RUNTIME #2425: boot wiring hydrates and delegates the suggest shortcut", function () {
  var fx = fixture();
  var NS = fx.NS;
  var button = fx.doc.createElement("button");
  button.setAttribute("id", "docsReviewSuggestBtn");
  fx.doc.body.appendChild(button);
  fx.doc.dispatch("DOMContentLoaded", {});
  assertEqual(NS.state.suggestionMode, false, "hydrate loaded default state");
  fx.doc.dispatch("click", {
    target: button, preventDefault: function () {}, stopPropagation: function () {},
  });
  assertEqual(NS.state.suggestionMode, true, "delegated click toggles suggestion mode");
  assertEqual(button.getAttribute("aria-pressed"), "true", "aria-pressed follows state");
  fx.doc.dispatch("keydown", {
    key: "r", ctrlKey: true, altKey: true, preventDefault: function () {},
  });
  assert(fx.doc.body.querySelector(".gb-review-backdrop"), "Ctrl+Alt+R opens review panel");
});

test("RUNTIME #2425: complementary server-history module exports undo/redo handles", function () {
  var fx = fixture();
  var history = fx.sandbox.window.GBDocsServerHistory;
  assert(history, "GBDocsServerHistory global installed by shipped module");
  assertEqual(typeof history.undo, "function", "undo handle exported");
  assertEqual(typeof history.redo, "function", "redo handle exported");
});

// ── STATIC source assertions (private wiring / explicitly deferred only) ──

var S_STATE = "docs/modules/49_review_state.js";
var S_MODEL = "docs/modules/50_review_model.js";
var S_COMMENTS = "docs/modules/52_review_comments.js";
var S_COMPARE = "docs/modules/53_review_compare.js";
var S_PERMS = "docs/modules/55_review_permissions.js";

test("STATIC #2425: local-only review slice performs no server/export round-trip", function () {
  [S_STATE, S_MODEL, S_COMPARE].forEach(function (file) {
    var src = readFile(file);
    assert(src.indexOf("fetch(") < 0, file + " must not call fetch");
    assert(src.indexOf("XMLHttpRequest") < 0, file + " must not call XHR");
  });
  assert(readFile(S_STATE).indexOf("Local only") >= 0,
    "49 labels the surface as local-only so reviewers are not misled");
});

test("STATIC #2425: permission enforcement and DOCX/PDF/notification work is deferred", function () {
  var perms = readFile(S_PERMS);
  assert(perms.indexOf("server-side enforcement") >= 0,
    "permissions dialog states server enforcement is not implemented yet");
  assert(perms.indexOf("fetch(") < 0, "permissions preview does not call the server");
  var comments = readFile(S_COMMENTS);
  assert(comments.indexOf("no notification is sent") >= 0,
    "comment module states @mention notifications are deferred");
  var model = readFile(S_MODEL);
  assert(model.indexOf("w:ins") < 0 && model.indexOf("w:del") < 0,
    "DOCX w:ins/w:del persistence is deferred, not claimed");
  var compare = readFile(S_COMPARE);
  assert(compare.indexOf("deferred") >= 0, "compare module labels DOCX/PDF export as deferred");
  assert(compare.indexOf("colour-coded") >= 0, "colour-coded author PDF is documented as deferred");
});

test("STATIC #2425: reply composer is a focus-trapped dialog, never native prompt()", function () {
  var comments = readFile(S_COMMENTS);
  assert(comments.indexOf("prompt(") < 0 && comments.indexOf("confirm(") < 0,
    "no native prompt/confirm for comment replies (#2299)");
  assert(comments.indexOf("NS.dialog.open(") >= 0, "replies use the shared dialog");
  var state = readFile(S_STATE);
  assert(state.indexOf('ev.key === "Escape"') >= 0 && state.indexOf("ev.shiftKey") >= 0,
    "focusMove contract handles Escape and Shift+Tab wrap");
});

test("STATIC #2425: earlier comments/history modules stay distinct server-backed paths", function () {
  var comments = readFile("docs/modules/08_comments.js");
  assert(comments.indexOf('"docs:range"') >= 0,
    "range comments use the shared collab backend resource type");
  var history = readFile("docs/modules/11_history_server.js");
  assert(history.indexOf("/api/docs/history") >= 0,
    "server history fallback posts to /api/docs/history");
});
