/**
 * #2425 RUNTIME verification of the shipped docs review modules 49..55.
 *
 * Every assertion below executes the real production IIFE loaded by the
 * harness into a shared vm sandbox. No production logic is reimplemented;
 * fixtures only supply deterministic inputs. English-looking literals appear
 * only as function arguments (never assigned to a DOM text slot), so the i18n
 * hardcoded-literal scanner is unaffected.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var assertEqual = h.assertEqual;
var fixture = h.fixture;
var deepEqual = h.deepEqual;

// ── Load surface / deterministic primitives ───────────────────────────

test("RUNTIME #2425: all review modules load and export the pure API", function () {
  var NS = fixture().NS;
  [
    "substitute", "tr", "esc", "uid", "docId", "docKey", "readStore", "writeStore",
    "defaultState", "loadState", "persist", "setCurrentUser", "me", "focusMove",
    "undo", "addChange", "getChange", "acceptChange", "rejectChange",
    "acceptByAuthor", "rejectByAuthor", "changeAuthors", "adjustAnchor",
    "mentions", "addThread", "getThread", "reply", "resolveThread", "reopenThread",
    "filterThreads", "computeRedline", "hunks", "acceptHunk", "rejectHunk",
    "mergeResult", "computeDashboard", "setSignOff", "modeLabel", "applyModeClass",
    "setViewMode", "toggleSuggestionMode", "toggleProtectMode", "proposeEdit",
    "typeLabel", "richText", "renderMarkup", "renderCommentPanel", "renderCompare",
    "renderDashboard", "can", "renderPermissions", "renderWorkspace", "openPanel",
  ].forEach(function (name) {
    assert(typeof NS[name] === "function", name + " must be exported on DocsReview");
  });
  assert(deepEqual(NS.VIEW_MODES, ["all", "simple", "none", "original", "final"]),
    "the five shipped view modes must be present");
  assert(deepEqual(NS.ROLES, ["owner", "editor", "commenter", "viewer"]),
    "the four shipped roles must be present");
});

test("RUNTIME #2425: substitute/esc/tr are deterministic", function () {
  var NS = fixture().NS;
  assertEqual(NS.substitute("{$n}/{$n}", { n: 7 }), "7/7", "substitute all occurrences");
  assertEqual(NS.substitute("{ $a } - { $b }", { a: "<x>", b: "<y>" }), "<x> - <y>",
    "substitute spaced named tokens");
  assertEqual(NS.esc("<a href='x'>&\""), "&lt;a href=&#39;x&#39;&gt;&amp;&quot;",
    "esc escapes markup characters");
  assertEqual(NS.tr("k", "<fallback>"), "<fallback>", "tr falls back when no translator value");
  assertEqual(NS.tr("k", "M: { $m }", { m: "<v>" }), "M: <v>", "tr substitutes fallback params");
});

test("RUNTIME #2425: document-scoped storage keys and loadState validation", function () {
  var fx = fixture();
  var NS = fx.NS;
  assertEqual(NS.docId(), "current", "docId falls back to current with no article");
  assertEqual(NS.docKey("state"), "docs2425:current:state", "docKey prefixes doc id");
  assertEqual(NS.writeStore("probe", { a: 1 }), true, "writeStore succeeds");
  assert(deepEqual(NS.readStore("probe", null), { a: 1 }), "readStore round-trips JSON");
  assertEqual(NS.readStore("absent", "<d>"), "<d>", "readStore returns fallback when missing");
  fx.store["docs2425:current:state"] = JSON.stringify({
    viewMode: "bad-mode", role: "admin-bogus", changes: "not-array",
    suggestionMode: true, signOff: 7, currentUser: "",
  });
  var s = NS.loadState();
  assertEqual(s.viewMode, "all", "invalid viewMode falls back");
  assertEqual(s.role, "editor", "invalid role falls back");
  assert(Array.isArray(s.changes) && s.changes.length === 0, "non-array changes fall back");
  assertEqual(s.suggestionMode, true, "valid boolean preserved");
  assertEqual(s.signOff, "pending", "non-string signOff falls back");
  assertEqual(s.currentUser, "You", "empty currentUser falls back");
});

test("RUNTIME #2425: focusMove is a pure focus-trap contract", function () {
  var NS = fixture().NS;
  var keys = ["a", "b", "c"];
  assertEqual(NS.focusMove(keys, "b", { key: "Tab" }).key, "c", "Tab advances");
  assertEqual(NS.focusMove(keys, "a", { key: "Tab", shiftKey: true }).key, "c", "Shift+Tab wraps");
  assertEqual(NS.focusMove(keys, "c", { key: "Tab" }).index, 0, "Tab wraps forward");
  assertEqual(NS.focusMove(keys, "a", { key: "Escape" }).action, "close", "Escape closes");
  assertEqual(NS.focusMove(keys, "a", { key: "Enter" }).action, "none", "other keys ignored");
  assertEqual(NS.focusMove([], "a", { key: "Tab" }).action, "none", "empty focus list ignored");
});

// ── Change model: accept/reject, isolation, undo ──────────────────────

test("RUNTIME #2425: addChange normalizes fields and defaults", function () {
  var NS = fixture().NS;
  var c = NS.addChange({ type: "bogus", before: 12, after: null, suggestOnly: true });
  assertEqual(c.status, "open", "new change starts open");
  assertEqual(c.type, "insert", "unknown type normalized to insert");
  assertEqual(c.before, "12", "before coerced to string");
  assertEqual(c.after, "", "null after becomes empty string");
  assertEqual(c.suggestOnly, true, "suggestOnly preserved");
  assertEqual(c.offset, 0, "default offset is zero");
  assert(NS.getChange(c.id) === c, "getChange finds the same object");
});

test("RUNTIME #2425: accepting one change leaves neighbours untouched; undo restores", function () {
  var NS = fixture().NS;
  var a = NS.addChange({ id: "<a>", author: "alice" });
  var b = NS.addChange({ id: "<b>", author: "alice" });
  var c = NS.addChange({ id: "<c>", author: "bob" });
  assertEqual(NS.acceptChange("<a>"), true, "accept succeeds on open change");
  assertEqual(a.status, "accepted", "target accepted");
  assertEqual(b.status, "open", "neighbour b untouched");
  assertEqual(c.status, "open", "neighbour c untouched");
  assertEqual(NS.acceptChange("<a>"), false, "cannot accept twice");
  assertEqual(NS.rejectChange("<b>"), true, "reject succeeds");
  assertEqual(NS.undo(), true, "undo pops last action");
  assertEqual(b.status, "open", "undo restores rejected neighbour");
  assertEqual(NS.undo(), true, "undo pops first action");
  assertEqual(a.status, "open", "undo restores accepted change");
  assertEqual(NS.undo(), false, "empty undo stack returns false");
});

test("RUNTIME #2425: 200 changes / 3 authors reach zero open, undo restores all", function () {
  var NS = fixture().NS;
  var authors = ["alice", "bob", "carol"];
  for (var i = 0; i < 200; i += 1) {
    NS.addChange({ author: authors[i % 3] });
  }
  assertEqual(NS.state.changes.length, 200, "all 200 changes recorded");
  assert(deepEqual(NS.changeAuthors(), ["alice", "bob", "carol"]), "authors sorted/deduped");
  authors.forEach(function (author) { assert(NS.acceptByAuthor(author) > 0, "bulk accept " + author); });
  assertEqual(NS.state.changes.filter(function (c) { return c.status === "open"; }).length, 0,
    "zero open changes after bulk accept");
  for (var u = 0; u < 3; u += 1) NS.undo();
  assertEqual(NS.state.changes.filter(function (c) { return c.status === "open"; }).length, 200,
    "three undos restore all 200 changes");
  assertEqual(NS.rejectByAuthor("nobody"), 0, "bulk reject with no match returns zero");
});

// ── Anchors, mentions and threads ─────────────────────────────────────

test("RUNTIME #2425: a stable anchor survives many edits above it", function () {
  var NS = fixture().NS;
  var anchor = { anchorId: "<p1>", offset: 50 };
  var edits = [];
  for (var i = 0; i < 20; i += 1) edits.push({ anchorId: "<p0>", at: 0, delta: 3 });
  var afterAbove = NS.adjustAnchor(anchor, edits);
  assertEqual(afterAbove.offset, 50, "20 edits in another anchor cannot detach this anchor");
  var shifted = NS.adjustAnchor(afterAbove, [{ anchorId: "<p1>", at: 10, delta: -5 }]);
  assertEqual(shifted.offset, 45, "same-anchor edit before offset shifts it by delta");
  assertEqual(NS.adjustAnchor(shifted, [{ anchorId: "<p1>", at: 90, delta: 9 }]).offset, 45,
    "edit after the offset never shifts it");
  assertEqual(NS.adjustAnchor(shifted, [{ anchorId: "<p1>", at: 45, delta: -100 }]).offset, 0,
    "offset is clamped at zero");
});

test("RUNTIME #2425: mentions are extracted and deduplicated", function () {
  var NS = fixture().NS;
  assert(deepEqual(NS.mentions("@alice @bob @alice hello"), ["alice", "bob"]),
    "mentions dedupe while preserving order");
  assert(deepEqual(NS.mentions("no mentions here"), []), "no mentions yields empty array");
  assert(deepEqual(NS.mentions("@a.b @a-b @a.b"), ["a.b", "a-b"]), "dots and dashes are allowed");
});

test("RUNTIME #2425: threaded comments, replies, resolve/reopen and filters", function () {
  var NS = fixture().NS;
  NS.setCurrentUser("alice");
  var t1 = NS.addThread({ anchorId: "<p1>", offset: 12, text: "@bob look", quote: "<q>" });
  var t2 = NS.addThread({ anchorId: "<p2>", text: "@alice ping" });
  assert(deepEqual(t1.mentions, ["bob"]), "thread mentions parsed");
  assertEqual(NS.reply(t1.id, { author: "carol", text: "@alice agreed" }).mentions[0], "alice",
    "reply mentions parsed");
  assertEqual(NS.getThread(t1.id).replies.length, 1, "reply stored on the thread");
  assertEqual(NS.filterThreads("open").length, 2, "open filter returns both threads");
  assert(NS.filterThreads("mentions").some(function (t) { return t.id === t2.id; }),
    "mentions filter finds a thread mentioning me");
  assert(NS.filterThreads("mentions").some(function (t) { return t.id === t1.id; }),
    "mentions filter also finds a thread whose reply mentions me");
  assert(NS.filterThreads("mine").some(function (t) { return t.id === t2.id; }),
    "mine filter finds a thread I authored");
  assertEqual(NS.resolveThread(t1.id), true, "resolve succeeds");
  assertEqual(NS.resolveThread(t1.id), false, "double resolve is a no-op");
  assertEqual(NS.filterThreads("open").length, 1, "resolve removes it from open filter");
  assertEqual(NS.reopenThread(t1.id), true, "reopen succeeds");
  assertEqual(NS.filterThreads("open").length, 2, "reopen restores it to open filter");
  assertEqual(NS.reply("<missing>", { text: "x" }), null, "reply to missing thread returns null");
});

// ── Compare / merge ───────────────────────────────────────────────────

test("RUNTIME #2425: computeRedline produces ordered hunks; merge honours decisions", function () {
  var NS = fixture().NS;
  var ops = NS.computeRedline("a\nb\nc", "a\nx\nc");
  assertEqual(ops[0].type, "equal", "leading equal line preserved");
  var hs = NS.hunks();
  assertEqual(hs.length, 2, "one delete and one insert hunk");
  assertEqual(hs[0].type, "delete", "delete hunk first");
  assertEqual(hs[1].type, "insert", "insert hunk second");
  assert(deepEqual(hs.map(function (x) { return x.status; }), ["open", "open"]),
    "hunks start open");
  hs[0].status = "rejected";
  hs[1].status = "accepted";
  assertEqual(NS.mergeResult(), "a\nb\nx\nc", "reject delete + accept insert keeps both texts");
  hs[0].status = "accepted";
  hs[1].status = "rejected";
  assertEqual(NS.mergeResult(), "a\nc", "accept delete + reject insert applies only the deletion");
  assertEqual(NS.acceptHunk(hs[0].id), false, "cannot accept a non-open hunk");
});

test("RUNTIME #2425: oversized compare collapses to a bounded two-hunk redline", function () {
  var NS = fixture().NS;
  var left = [];
  var right = [];
  for (var i = 0; i < 401; i += 1) {
    left.push("L" + i);
    right.push("R" + i);
  }
  var ops = NS.computeRedline(left.join("\n"), right.join("\n"));
  assertEqual(ops.length, 2, "bounded path emits exactly two ops");
  assertEqual(ops[0].type, "delete", "first bounded op is delete");
  assertEqual(ops[1].type, "insert", "second bounded op is insert");
  assertEqual(NS.hunks().length, 2, "bounded ops are addressable hunks");
});

// ── Dashboard, sign-off and permissions ───────────────────────────────

test("RUNTIME #2425: dashboard counts and sign-off are derived, undoable state", function () {
  var NS = fixture().NS;
  NS.addChange({ author: "alice" });
  NS.addChange({ author: "alice" });
  var c = NS.addChange({ author: "bob" });
  NS.acceptChange(c.id);
  NS.addThread({ text: "<x>" });
  NS.resolveThread(NS.state.threads[0].id);
  NS.state.reviewStartedAt = NS.state.reviewStartedAt - 5 * 60000;
  var d = NS.computeDashboard();
  assertEqual(d.byAuthor.alice.open, 2, "alice has two open changes");
  assertEqual(d.byAuthor.bob.accepted, 1, "bob has one accepted change");
  assertEqual(d.totalOpen, 2, "total open changes derived from state");
  assertEqual(d.unresolvedThreads, 0, "resolved thread excluded from unresolved count");
  assertEqual(d.timeInReviewMinutes, 5, "elapsed minutes use the deterministic clock");
  assertEqual(d.signOff, "pending", "default sign-off is pending");
  assertEqual(NS.setSignOff("approved"), true, "setSignOff accepts a valid status");
  assertEqual(NS.setSignOff("bogus"), false, "invalid sign-off rejected");
  assertEqual(NS.computeDashboard().signOff, "approved", "dashboard reflects sign-off");
  assertEqual(NS.undo(), true, "undo pops sign-off action");
  assertEqual(NS.computeDashboard().signOff, "pending", "undo restores previous sign-off");
});

test("RUNTIME #2425: permission matrix can() matches the shipped role/capability rows", function () {
  var NS = fixture().NS;
  assertEqual(NS.can("owner", "manage"), true, "owner manages permissions");
  assertEqual(NS.can("owner", "signoff"), true, "owner signs off");
  assertEqual(NS.can("editor", "review"), true, "editor accepts/rejects");
  assertEqual(NS.can("editor", "signoff"), false, "editor cannot sign off");
  assertEqual(NS.can("commenter", "suggest"), true, "commenter can suggest");
  assertEqual(NS.can("commenter", "edit"), false, "commenter cannot edit content");
  assertEqual(NS.can("commenter", "review"), false, "commenter cannot accept/reject");
  assertEqual(NS.can("viewer", "comment"), true, "viewer can comment");
  assertEqual(NS.can("viewer", "edit"), false, "viewer cannot edit");
  assertEqual(NS.can("viewer", "suggest"), false, "viewer cannot suggest");
  assertEqual(NS.can("nobody", "comment"), false, "unknown role denies every capability");
});
