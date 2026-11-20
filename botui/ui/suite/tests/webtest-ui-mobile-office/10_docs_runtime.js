/**
 * RUNTIME assertions for the REAL docs/modules/19_mobile.js IIFE (#2515).
 *
 * Each test executes shipped code in a fresh fixture. Nothing here reimplements
 * the drawer, tap-to-caret or revealCaret logic; fixtures only feed events and
 * observe the resulting classes, attributes, focus calls and scroll offsets.
 */
"use strict";

var h = require("./00_harness");
var f = require("./02_fixture");
var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;
var create = f.create;

function phone() { return create({ apps: ["docs"] }); }

function click(node, fx) { node.dispatchEvent(fx.event({ type: "click" })); }
function escape(fx) { fx.doc.dispatchEvent(fx.event({ type: "keydown", key: "Escape" })); }
function selectionChange(fx) { fx.doc.dispatchEvent(fx.event({ type: "selectionchange" })); }
function tap(fx, target) {
  fx.refs.docs.app.dispatchEvent(fx.event({ type: "pointerup", pointerType: "touch", target: target }));
}

test("RUNTIME #2515 docs: drawer opens via toggle and publishes aria-expanded", function () {
  var fx = phone();
  var r = fx.refs.docs;
  click(r.toggle, fx);
  assert(r.main.classList.contains("sidebar-open"), "main gained sidebar-open");
  equal(r.toggle.getAttribute("aria-expanded"), "true", "toggle aria-expanded");
});

test("RUNTIME #2515 docs: second toggle click closes the drawer", function () {
  var fx = phone();
  var r = fx.refs.docs;
  click(r.toggle, fx);
  click(r.toggle, fx);
  assert(!r.main.classList.contains("sidebar-open"), "sidebar-open removed");
  equal(r.toggle.getAttribute("aria-expanded"), "false", "aria-expanded reset");
});

test("RUNTIME #2515 docs: close button closes an open drawer", function () {
  var fx = phone();
  var r = fx.refs.docs;
  click(r.toggle, fx);
  click(r.close, fx);
  assert(!r.main.classList.contains("sidebar-open"), "close button removed sidebar-open");
  equal(r.toggle.getAttribute("aria-expanded"), "false", "close button reset aria");
});

test("RUNTIME #2515 docs: backdrop click closes an open drawer", function () {
  var fx = phone();
  var r = fx.refs.docs;
  click(r.toggle, fx);
  click(r.backdrop, fx);
  assert(!r.main.classList.contains("sidebar-open"), "backdrop removed sidebar-open");
});

test("RUNTIME #2515 docs: Escape closes an open drawer and is inert when closed", function () {
  var fx = phone();
  var r = fx.refs.docs;
  click(r.toggle, fx);
  escape(fx);
  assert(!r.main.classList.contains("sidebar-open"), "Escape closed the drawer");
  escape(fx);
  assert(!r.main.classList.contains("sidebar-open"), "second Escape is a no-op");
});

test("RUNTIME #2515 docs: touch tap on the paper focuses the article with preventScroll", function () {
  var fx = phone();
  var r = fx.refs.docs;
  var text = fx.doc.createTextNode("x");
  r.article.appendChild(text);
  tap(fx, text);
  equal(r.article.focusCalls, 1, "article focused once");
  equal(fx.doc.activeElement, r.article, "activeElement is the article");
  equal(r.article.lastFocusOptions.preventScroll, true, "focus requested preventScroll");
});

test("RUNTIME #2515 docs: mouse pointerup never places the caret", function () {
  var fx = phone();
  var text = fx.doc.createTextNode("x");
  fx.refs.docs.article.appendChild(text);
  fx.refs.docs.app.dispatchEvent(fx.event({ type: "pointerup", pointerType: "mouse", target: text }));
  equal(fx.refs.docs.article.focusCalls, 0, "mouse pointer did not focus");
});

test("RUNTIME #2515 docs: touch tap outside the article is ignored", function () {
  var fx = phone();
  tap(fx, fx.refs.docs.app);
  equal(fx.refs.docs.article.focusCalls, 0, "outside tap did not focus");
});

test("RUNTIME #2515 docs: an existing text selection is preserved on tap", function () {
  var fx = phone();
  var text = fx.doc.createTextNode("x");
  fx.refs.docs.article.appendChild(text);
  fx.setSelection("selected word", { top: 200, bottom: 220 });
  tap(fx, text);
  equal(fx.refs.docs.article.focusCalls, 0, "non-empty selection prevented refocus");
});

test("RUNTIME #2515 docs: tapping an already focused article is a no-op", function () {
  var fx = phone();
  var text = fx.doc.createTextNode("x");
  fx.refs.docs.article.appendChild(text);
  fx.refs.docs.article.focus();
  tap(fx, text);
  equal(fx.refs.docs.article.focusCalls, 1, "focus called only by the test setup");
});

function caretBelow(fx) {
  var r = fx.refs.docs;
  r.scroller.rect = { top: 100, bottom: 700 };
  r.article.focus();
  fx.setSelection("", { top: 380, bottom: 396 });
  selectionChange(fx);
  fx.flushTimers();
  return r.scroller.scrollTop;
}

test("RUNTIME #2515 docs: caret below the keyboard-safe band scrolls the content down", function () {
  equal(caretBelow(phone()), 40, "scrolled by caret overflow past safeBottom 356");
});

test("RUNTIME #2515 docs: caret above the viewport scrolls the content up", function () {
  var fx = phone();
  var r = fx.refs.docs;
  r.scroller.rect = { top: 100, bottom: 700 };
  r.scroller.scrollTop = 100;
  r.article.focus();
  fx.setSelection("", { top: 88, bottom: 108 });
  selectionChange(fx);
  fx.flushTimers();
  equal(r.scroller.scrollTop, 80, "scrolled up by the 20px top overshoot");
});

test("RUNTIME #2515 docs: a caret inside the safe band leaves scroll alone", function () {
  var fx = phone();
  var r = fx.refs.docs;
  r.scroller.rect = { top: 100, bottom: 700 };
  r.scroller.scrollTop = 55;
  r.article.focus();
  fx.setSelection("", { top: 300, bottom: 320 });
  selectionChange(fx);
  fx.flushTimers();
  equal(r.scroller.scrollTop, 55, "no scroll adjustment");
});

test("RUNTIME #2515 docs: revealCaret is inert when the article is not focused", function () {
  var fx = phone();
  var r = fx.refs.docs;
  r.scroller.rect = { top: 100, bottom: 700 };
  fx.setSelection("", { top: 380, bottom: 396 });
  selectionChange(fx);
  fx.flushTimers();
  equal(r.scroller.scrollTop, 0, "unfocused article did not scroll");
});

test("RUNTIME #2515 docs: the visualViewport keyboard height changes the safe band", function () {
  var small = phone();
  small.keyboard(0);
  var large = phone();
  large.keyboard(320);
  equal(caretBelow(small), 0, "with no keyboard the caret is already visible");
  equal(caretBelow(large), 40, "with a keyboard the caret is pushed into view");
});

test("RUNTIME #2515 docs: visualViewport resize and scroll both reflow the caret", function () {
  var fx = phone();
  var r = fx.refs.docs;
  r.scroller.rect = { top: 100, bottom: 700 };
  r.article.focus();
  fx.setSelection("", { top: 380, bottom: 396 });
  fx.win.visualViewport.dispatch("resize", fx.event({ type: "resize" }));
  fx.win.visualViewport.dispatch("scroll", fx.event({ type: "scroll" }));
  assert(fx.timerDelays.indexOf(60) >= 0, "both handlers schedule a 60ms reveal");
  fx.flushTimers();
  equal(r.scroller.scrollTop, 80, "two reveals each applied the 40px overshoot");
});

test("RUNTIME #2515 docs: orientationchange reflows the caret after 300ms", function () {
  var fx = phone();
  fx.refs.docs.scroller.rect = { top: 100, bottom: 700 };
  fx.refs.docs.article.focus();
  fx.setSelection("", { top: 380, bottom: 396 });
  fx.win.dispatchEvent(fx.event({ type: "orientationchange" }));
  assert(fx.timerDelays.indexOf(300) >= 0, "orientationchange uses a 300ms timer");
  fx.flushTimers();
  equal(fx.refs.docs.scroller.scrollTop, 40, "caret reflowed after rotation");
});
