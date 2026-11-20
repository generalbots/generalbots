/**
 * RUNTIME proofs for chat/messages/10_scroll.js (#2014) and its integration
 * with the shipped virtualizer. All functions are the real shipped globals.
 */
"use strict";

var h = require("./00_harness");
var D = require("./05_dom");

function scrollScenario(count, height) {
  var E = D.fresh();
  var c = E.dom.messages;
  var els = [];
  for (var i = 0; i < (count || 0); i++) {
    var el = D.makeMessage(E, i, height || 60, i % 2 ? "bot" : "user");
    c.appendChild(el);
    els.push(el);
  }
  E.window.GBChatVirtual.registerAll(c);
  return { E: E, c: c, els: els, S: E.sandbox, V: E.window.GBChatVirtual, ChatState: E.ChatState };
}

h.test("scrollToBottom / snapToBottom pin to the live edge", function () {
  var t = scrollScenario(10, 60);
  t.c.scrollTop = 0;
  t.S.scrollToBottom(false);
  h.equal(t.c.scrollTop, t.c.scrollHeight, "instant pin sets scrollTop to scrollHeight");
  t.c.scrollTop = 12;
  t.S.snapToBottom();
  h.equal(t.c.scrollTop, t.c.scrollHeight, "snapToBottom pins again");
  t.c.scrollTop = 0;
  t.S.scrollToBottom(true);
  h.equal(t.c.scrollTop, 0, "animated path delegates to scrollTo, not scrollTop");
  h.equal(t.c._scrollCalls.length, 1, "one scrollTo call recorded");
  h.equal(t.c._scrollCalls[0].behavior, "smooth", "animated path requests smooth scrolling");
  h.equal(t.c._scrollCalls[0].top, t.c.scrollHeight, "animated target is the live edge");
});

h.test("stickToBottom never fights a reader already away from the bottom", function () {
  var t = scrollScenario(5, 60);
  t.ChatState.isUserAtBottom = false;
  h.equal(t.S.gbStickTimer, null, "no timer armed before the call");
  t.S.stickToBottom(1000);
  h.equal(t.S.gbStickTimer, null, "isUserAtBottom === false short-circuits");
  h.equal(t.E.activeTimers(), 0, "no interval left running");
});

h.test("stickToBottom keeps a single interval and releases on reader intent", function () {
  var t = scrollScenario(5, 60);
  t.ChatState.isUserAtBottom = true;
  t.S.gbPinCancelAt = 0;
  t.S.stickToBottom(5000);
  var first = t.S.gbStickTimer;
  t.S.stickToBottom(5000);
  h.equal(t.E.activeTimers(), 1, "re-arm does not stack intervals");
  h.assert(first !== t.S.gbStickTimer, "existing timer was replaced");
  t.S.gbAttachReaderIntent(t.c);
  t.c.scrollTop = 123;
  t.E.setNow(t.E.fixedNow + 10);
  t.c.dispatch("wheel");
  h.assert(t.S.gbPinCancelAt > 0, "wheel marks reader intent with the fixed clock");
  t.E.advance(121);
  h.equal(t.S.gbStickTimer, null, "interval cleared once intent is detected");
  h.equal(t.c.scrollTop, 123, "transcript was not yanked to the bottom");
});

h.test("stickToBottom pins only while the reader stays at the live edge", function () {
  var t = scrollScenario(5, 60);
  t.ChatState.isUserAtBottom = true;
  t.S.gbPinCancelAt = 0;
  t.c.scrollTop = 0;
  t.S.stickToBottom(200);
  t.E.advance(121);
  h.equal(t.c.scrollTop, t.c.scrollHeight, "interval pins to the live edge");
  t.E.advance(121);
  h.equal(t.S.gbStickTimer, null, "interval self-releases at the end of the window");
});

h.test("gbAttachReaderIntent observes all touch/pointer input once", function () {
  var t = scrollScenario(5, 60);
  t.S.gbAttachReaderIntent(t.c);
  var types = t.c.listenerTypes();
  ["wheel", "touchstart", "pointerdown", "mousedown", "keydown", "scroll"].forEach(function (type) {
    h.assert(types.indexOf(type) >= 0, type + " listener attached");
  });
  h.equal(t.c.dataset.gbReaderIntent, "1", "attachment is idempotent by dataset flag");
  var before = t.c.listenerTypes().length;
  t.S.gbAttachReaderIntent(t.c);
  h.equal(t.c.listenerTypes().length, before, "second attach adds no duplicate listeners");
});

h.test("followLiveEdge cancels on touchstart and leaves the viewport alone", function () {
  var t = scrollScenario(5, 60);
  t.ChatState.isUserAtBottom = true;
  t.S.gbPinCancelAt = 0;
  t.c.scrollTop = 42;
  t.S.followLiveEdge(500);
  h.assert(t.E.activeRaf() > 0, "follow step queued through requestAnimationFrame");
  t.c.dispatch("touchstart");
  t.E.flushRaf();
  h.equal(t.E.activeRaf(), 0, "cancelled follow schedules no further frames");
  h.equal(t.c.scrollTop, 42, "reader position unchanged");
});

h.test("scroll-to-bottom button follows distance from the live edge", function () {
  var t = scrollScenario(200, 60);
  var btn = t.E.sandbox.document.getElementById("scrollToBottom");
  h.assert(btn !== null, "button exists in the stub DOM");
  t.c.scrollTop = t.c.scrollHeight - t.c.clientHeight;
  t.S.updateScrollButton();
  h.equal(btn.classList.contains("visible"), false, "button hidden near the bottom");
  t.c.scrollTop = 0;
  t.S.updateScrollButton();
  h.equal(btn.classList.contains("visible"), true, "button visible when far from bottom");
});

h.test("scrollbar markers cover detached virtualized turns and jump rematerializes", function () {
  var t = scrollScenario(200, 60);
  t.c.scrollTop = t.c.scrollHeight - t.c.clientHeight;
  t.V.maybeEnable();
  t.V.forceUpdate();
  t.E.flushRaf();
  t.S.gbRefreshScrollbarMarkers();
  var users = t.V._items().filter(function (it) { return it.el.classList.contains("user"); });
  h.assert(users.length > 0, "user turns exist in the tracked set");
  h.assert(users.some(function (it) { return !it.live; }), "some user turns are offscreen");
  var overlay = t.c.querySelector(".scrollbar-markers");
  h.assert(overlay !== null, "marker overlay created");
  var markers = overlay.querySelectorAll(".scrollbar-marker.user-marker");
  h.equal(markers.length, users.length, "one marker per user turn, including detached ones");
  var detached = users.filter(function (it) { return !it.live; })[0];
  h.equal(detached.el.isConnected, false, "chosen target is a placeholder-backed node");
  /* Markers are created in item order; the first detached user turn is the first
   * matching marker with a placeholder sibling in the transcript. */
  var clicked = false;
  markers.forEach(function (marker) {
    if (clicked) return;
    marker.dispatch("click");
    if (detached.el.isConnected) clicked = true;
  });
  h.equal(clicked, true, "a marker click rematerialized the detached target");
  h.assert(t.c._scrollCalls.length > 0, "marker click issued a smooth scrollTo");
});

h.test("thinking indicators delegate to the turn state machine", function () {
  var t = scrollScenario(0, 60);
  t.S.showThinkingIndicator({});
  t.S.hideThinkingIndicator({});
  h.assert(t.E.turnCalls.indexOf("start") >= 0, "start reached GBTurn");
  h.assert(t.E.turnCalls.indexOf("forceHide") >= 0, "forceHide reached GBTurn");
});

h.test("marker labels truncate long turns and use relative time", function () {
  var t = scrollScenario(0, 60);
  var msg = D.makeMessage(t.E, 1, 60, "bot");
  msg.setAttribute("data-ts", String(t.E.fixedNow - 65000));
  msg.querySelector(".message-content").textContent = "x".repeat(90);
  var label = t.S.gbMarkerLabel(msg);
  h.assert(label.length <= 80, "label is truncated: " + label.length);
  h.assert(label.indexOf("\u2026") >= 0, "ellipsis marks truncation");
  h.assert(label.indexOf("1m ago") >= 0, "relative time from the fixed clock");
});

h.test("scroll drift during a pending stream is not treated as reader intent", function () {
  var t = scrollScenario(200, 60);
  t.c.scrollTop = t.c.scrollHeight - t.c.clientHeight;
  t.V.maybeEnable();
  t.V.forceUpdate();
  t.E.flushRaf();
  t.S.gbAttachReaderIntent(t.c);
  t.S.gbPinCancelAt = 0;
  t.ChatState.isStreaming = true;
  t.ChatState.pendingResponseAt = t.E.fixedNow;
  t.ChatState.isUserAtBottom = false;
  t.c.scrollTop = 3000;
  t.V.forceUpdate();
  h.equal(t.c.scrollTop, 3000, "windowing does not yank a reader scrolling up mid-answer");
  t.c.dispatch("scroll");
  h.equal(t.S.gbPinCancelAt, 0, "pending stream suppresses intent marking");
  t.S.stickToBottom(1000);
  h.equal(t.S.gbStickTimer, null, "stick-to-bottom stays disabled mid-stream");
});
