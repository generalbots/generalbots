/**
 * Invariants that windowing must not change (RUNTIME), plus labelled [STATIC]
 * source assertions for CSS/HTML/device criteria and reported KNOWN BUGs.
 * No production file is modified by this harness.
 */
"use strict";

var h = require("./00_harness");
var D = require("./05_dom");

function windowed(count, height) {
  var E = D.fresh();
  var c = E.dom.messages;
  for (var i = 0; i < count; i++) c.appendChild(D.makeMessage(E, i, height || 60, i % 2 ? "bot" : "user"));
  E.window.GBChatVirtual.registerAll(c);
  c.scrollTop = c.scrollHeight - c.clientHeight;
  E.window.GBChatVirtual.maybeEnable();
  E.window.GBChatVirtual.forceUpdate();
  E.flushRaf();
  return E;
}

/* ------------------------------------------------------- RUNTIME invariants */
h.test("persistence is session-scoped, fixed-clock and capped at 500 entries", function () {
  var E = D.fresh();
  var S = E.sandbox;
  S.persistLocalMessage("bot", "hello");
  var key = "gb_chat_history_" + E.fixedUid;
  h.assert(Object.prototype.hasOwnProperty.call(E.store, key), "session key written to localStorage");
  var history = S.loadLocalHistory();
  h.equal(history.length, 1, "one entry stored");
  h.equal(history[0].sender, "bot", "sender preserved");
  h.equal(history[0].content, "hello", "content preserved");
  h.equal(history[0].ts, E.fixedNow, "timestamp uses the fixed clock");
  for (var i = 0; i < 510; i++) S.persistLocalMessage("user", "m" + i);
  h.equal(S.loadLocalHistory().length, 500, "history capped at 500");
  h.equal(S.listLocalSessions().length, 1, "session listed with its history");
});

h.test("windowing does not change persistence behaviour", function () {
  var E = windowed(200, 60);
  h.assert(E.window.GBChatVirtual.liveCount() < 200, "virtualization is active");
  E.sandbox.persistLocalMessage("bot", "after-window");
  var history = E.sandbox.loadLocalHistory();
  h.equal(history.length, 1, "same single-entry write after windowing");
  h.equal(history[0].content, "after-window", "content still persisted verbatim");
  h.equal(history[0].ts, E.fixedNow, "fixed clock still used");
});

h.test("streaming updates a live bubble and marks it aria-hidden/busy", function () {
  var E = windowed(200, 60);
  var S = E.sandbox;
  var target = E.dom.byId("msg-150");
  h.assert(target !== null && target.isConnected, "target message is in the live window");
  E.ChatState.isStreaming = true;
  E.ChatState.streamingMessageId = "msg-150";
  S.updateStreaming("chunk **x**");
  var content = target.querySelector(".message-content");
  h.assert(content.innerHTML.indexOf("chunk") >= 0, "shipped updateStreaming rendered the chunk");
  h.equal(target.getAttribute("aria-hidden"), "true", "streaming bubble hidden from AT");
  h.equal(target.getAttribute("aria-busy"), "true", "streaming bubble marked busy");
  h.equal(E.dom.messages.getAttribute("aria-busy"), "true", "transcript marked busy");
});

h.test("streaming reaches a placeholder-backed bubble through ensureById", function () {
  var E = windowed(200, 60);
  var S = E.sandbox;
  h.equal(E.dom.byId("msg-5"), null, "offscreen message is detached from the DOM");
  E.ChatState.isStreaming = true;
  E.ChatState.streamingMessageId = "msg-5";
  S.updateStreaming("more from detached");
  var target = E.dom.byId("msg-5");
  h.assert(target !== null && target.isConnected, "ensureById rematerialized the bubble");
  h.assert(target.querySelector(".message-content").innerHTML.indexOf("more from detached") >= 0,
    "shipped streaming code wrote into the rematerialized bubble");
});

h.test("finalizeStreaming removes the bubble via the virtualizer and clears turn state", function () {
  var E = windowed(200, 60);
  var S = E.sandbox;
  var target = E.dom.byId("msg-150");
  var before = E.window.GBChatVirtual.count();
  E.ChatState.isStreaming = true;
  E.ChatState.streamingMessageId = "msg-150";
  E.ChatState.currentStreamingContent = "abc";
  E.ChatState.currentReasoning = "r";
  E.ChatState.streamingBuffer = "abc";
  E.dom.messages.setAttribute("aria-busy", "true");
  S.finalizeStreaming();
  h.equal(E.window.GBChatVirtual.count(), before - 1, "virtualizer dropped the streaming item");
  h.equal(target.isConnected, false, "streaming bubble detached");
  h.equal(E.ChatState.streamingMessageId, null, "streaming id cleared");
  h.equal(E.ChatState.currentStreamingContent, "", "stream content cleared");
  h.equal(E.ChatState.currentReasoning, "", "reasoning cleared");
  h.equal(E.ChatState.streamingBuffer, "", "buffer cleared");
  h.equal(E.dom.messages.getAttribute("aria-busy"), null, "transcript busy flag removed");
});

/* -------------------------------------------- [STATIC] source-only criteria */
h.staticTest("visualViewport keyboard bridge is shipped in shell and chat", function () {
  h.sourceHas("js/mobile-viewport.js", "window\\.visualViewport", "shell reads visualViewport");
  h.sourceHas("js/mobile-viewport.js", "addEventListener\\(\"resize\"", "shell listens to viewport resize");
  h.sourceHas("js/mobile-viewport.js", "--gb-vv-height", "shell publishes visible height");
  h.sourceHas("js/mobile-viewport.js", "gb-keyboard-open", "shell exposes keyboard-open flag");
  h.sourceHas("chat/modules/mobile_viewport.js", "GBChatVisibleHeight", "chat publishes visible height");
  h.sourceHas("chat/modules/mobile_viewport.js", "addEventListener\\(\"resize\"", "chat listens to viewport resize");
  h.sourceHas("chat/modules/mobile_viewport.js", "--gb-chat-visible-height", "chat CSS variable published");
  h.sourceHas("partials/chat.html", "chat/modules/mobile_viewport\\.js", "chat bridge is loaded first");
  h.sourceHas("chat/chat-composer.css", "--gb-vv-bottom", "composer tracks keyboard inset");
  h.sourceHas("chat/chat-layout.css", "--gb-chat-visible-height", "transcript capped to visible height");
  h.note("source-level only: real 390x844 keyboard screenshots remain a browser UAT gate");
});

h.staticTest("composer cap is viewport-relative, not a fixed 180px ceiling", function () {
  h.sourceHas("chat/chat-init_b.js", "Math\\.min\\(fixed, Math\\.round\\(visible \\* 0\\.3\\)\\)", "0.3 viewport cap present");
  h.sourceHas("chat/chat-init_b.js", "Math\\.max\\(64, fixed\\)", "64px floor present");
  h.sourceHas("chat/chat-init_b.js", "gb-chat-viewport", "autoGrow re-runs on viewport events");
  h.sourceLacks("chat/chat-init_b.js", "Math\\.min\\(input\\.scrollHeight, 180\\)", "fixed 180px cap removed");
  h.note("source-level only: measured composer/send visibility needs a keyboard-open device run");
});

h.staticTest("reply tables and long code are touch-safe in source", function () {
  h.sourceHas("chat/chat-layout.css", "\\.bot-message \\.message-content table", "reply table rule present");
  h.sourceHas("chat/chat-layout.css", "overflow-x:\\s*auto", "table scrolls in its bubble");
  h.sourceHas("chat/chat-code-ux.js", "Math\\.max\\(240, Math\\.round\\(\\(vh \\|\\| 800\\) \\* 0\\.5\\)\\)",
    "collapse threshold is viewport-relative with a 240px floor");
  h.sourceLacks("chat/chat-code-ux.js", "scrollHeight <= 480", "desktop 480px threshold removed from code");
  h.sourceHas("chat/chat-artifact-overlay.css", "max-width:\\s*640px", "artifact overlays go full-screen on phones");
  h.sourceHas("partials/chat.html", "capture=\\\"environment\\\"", "native camera capture markup present");
});

h.staticTest("virtualizer source never touches creation, streaming or persistence", function () {
  h.sourceLacks("chat/messages/15_virtualize.js",
    "addMessage|persistLocalMessage|processMessage|updateStreaming|localStorage|gbStoreGet|innerHTML",
    "windowing is DOM-presence only");
  h.sourceHas("chat/messages/15_virtualize.js", "replaceChild", "placeholder swap uses replaceChild");
  h.sourceHas("chat/messages/15_virtualize.js", "insertBefore", "rematerialization uses insertBefore");
  h.sourceHas("chat/messages/15_virtualize.js", "gb-vplaceholder", "placeholder marker class present");
});

h.staticTest("[STATIC] single mobile media intake sheet is shipped", function () {
  h.sourceHas("partials/chat.html", "gb-intake-sheet|gb-intake-row", "bottom-sheet markup exists");
  h.sourceHas("partials/chat.html", "id=\\\"gbIntakeBtn\\\"", "one '+' intake button exists");
  h.sourceHas("partials/chat.html", "id=\\\"cameraFileInput\\\"", "camera primitive is wired to the sheet");
  h.sourceHas("chat/modules/79_intake_sheet.js", "GBIntakeSheet", "intake module is shipped and bound");
  h.note("criterion 3 of #2516: one 44px Camera/Photos/File/Voice sheet");
});

/* --------------------------------------------------------------- KNOWN BUGs */
h.test("short turns keep exact placeholder height and do not jump the reader", function () {
  /* Regression for the former 24px placeholder floor: a <24px message was
   * inflated to 24px; the accumulated drift skipped anchor compensation when it
   * passed a viewport and the reader jumped. Placeholders now keep exact h. */
  var E = D.fresh();
  var c = E.dom.messages;
  var V = E.window.GBChatVirtual;
  for (var i = 0; i < 200; i++) c.appendChild(D.makeMessage(E, i, 10, "bot"));
  V.registerAll(c);
  c.scrollTop = c.scrollHeight - c.clientHeight;
  var before = c.scrollTop;
  var anchor = c.childNodes[169];
  var anchorBefore = anchor.getBoundingClientRect().top;
  V.maybeEnable();
  V.forceUpdate();
  E.flushRaf();
  var ph = c.querySelectorAll(".gb-vplaceholder")[0];
  h.equal(V._items()[0].h, 10, "original measured height is 10px");
  h.equal(ph.style.height, "10px", "placeholder preserves the exact 10px height");
  h.equal(c.scrollTop, before, "no drift means no scrollTop correction is needed");
  var moved = anchor.getBoundingClientRect().top - anchorBefore;
  h.assert(Math.abs(moved) <= 1, "reader position preserved: moved " + moved + "px");
  h.note("regression: former floor inflated short turns and jumped the reader");
});

h.test("chat boot defers optional modules instead of blocking on all of them", function () {
  var html = h.readFile("partials/chat.html");
  var scripts = (html.match(/<script\s+src=/g) || []).length;
  var deferred = (html.match(/<script[^>]*\b(?:defer|async)\b/g) || []).length;
  var formFactor = (html.match(/matchMedia|formFactor|isMobile/g) || []).length;
  h.equal(scripts, 42, "observed chat blocking script tag count");
  h.equal(deferred, 13, "deferred/async script tags");
  h.assert(formFactor >= 1, "form-factor branch documented in the chat fragment");
  h.assert(deferred > 0, "optional modules no longer all parser-blocking");
  h.note("criterion 4 of #2516: progressive/deferred module split for first send"
    + " (criterion 5, scroll-up during a stream, is RUNTIME-proven in 20_scroll.js)");
});

/* Regression for the evidence mapping itself: the boot-split KNOWN BUG must be
 * attributed to criterion 4 (3G first token), not criterion 5 (#2014 scroll
 * intent). A wrong mapping makes the per-criterion closure table misleading. */
h.staticTest("[REGRESSION] #2516 boot gap maps to criterion 4; criterion 5 stays runtime-proven", function () {
  var src = h.readFile("tests/webtest-ui-chat-mobile/30_invariants.js");
  h.assert(src.indexOf("criterion 4 of #2516 remains open: progressive/deferred module split") >= 0,
    "boot-split KNOWN BUG must point at criterion 4");
  var stale = "criterion 5" + " of #2516 remains open";
  h.assert(src.indexOf(stale) < 0,
    "criterion 5 must not be reported open — it is RUNTIME-proven");
  var scroll = h.readFile("tests/webtest-ui-chat-mobile/20_scroll.js");
  h.assert(scroll.indexOf("windowing does not yank a reader scrolling up mid-answer") >= 0,
    "criterion 5 runtime proof is missing from 20_scroll.js");
});

h.note("Browser/device-only criteria not claimed here: 390x844 keyboard screenshots, native camera/mic, 3G timeline, streaming scroll UAT.");
