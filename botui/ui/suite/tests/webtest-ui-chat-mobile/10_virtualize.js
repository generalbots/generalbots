/**
 * RUNTIME proofs for chat/messages/15_virtualize.js (#2229/#2516).
 * Every assertion below executes the real shipped module in the vm sandbox.
 */
"use strict";

var h = require("./00_harness");
var D = require("./05_dom");

function transcript(count, height) {
  var E = D.fresh();
  var container = E.dom.messages;
  var els = [];
  for (var i = 0; i < count; i++) {
    var el = D.makeMessage(E, i, height === undefined ? 60 : height, i % 2 ? "bot" : "user");
    container.appendChild(el);
    els.push(el);
  }
  var V = E.window.GBChatVirtual;
  V.registerAll(container);
  return { E: E, V: V, c: container, els: els };
}
function nearBottom(t) { t.c.scrollTop = t.c.scrollHeight - t.c.clientHeight; }
function run(t) {
  t.V.maybeEnable();
  t.V.forceUpdate();
  t.E.flushRaf();
}

h.test("windowing stays off below ENABLE_AT(120) and engages at 120", function () {
  var below = transcript(119, 60);
  below.V.maybeEnable();
  h.equal(below.V.isEnabled(), false, "119 messages must not window");
  h.equal(below.V.count(), 119, "all 119 tracked");
  var at = transcript(120, 60);
  at.V.maybeEnable();
  h.equal(at.V.isEnabled(), true, "120 messages must window");
  h.equal(at.V.count(), 120, "all 120 tracked");
});

h.test("offscreen messages become height-preserving, indexed placeholders", function () {
  var t = transcript(200, 60);
  nearBottom(t);
  run(t);
  h.equal(t.V.isEnabled(), true, "enabled at 200");
  h.equal(t.V.count(), 200, "no message lost from tracking");
  h.equal(t.c.childNodes.filter(function (node) {
    return node.classList.contains("message") || node.classList.contains("gb-vplaceholder");
  }).length, 200, "one DOM slot per message after replacement");
  h.assert(t.V.liveCount() <= 140, "MAX_LIVE 140 respected: " + t.V.liveCount());
  h.assert(t.V.liveCount() < 200 && t.V.liveCount() > 0, "a real window was applied");
  var placeholders = t.c.querySelectorAll(".gb-vplaceholder");
  h.equal(placeholders.length, 200 - t.V.liveCount(), "one placeholder per offscreen item");
  placeholders.forEach(function (ph) {
    var index = parseInt(ph.getAttribute("data-gb-vi"), 10);
    h.assert(index >= 0 && index < 200, "placeholder index in range: " + index);
    h.equal(ph.getAttribute("aria-hidden"), "true", "placeholder hidden from AT");
    h.equal(ph.style.height, "60px", "placeholder preserves measured 60px height");
    h.equal(t.V._items()[index].live, false, "tracked item marked offscreen");
  });
  h.equal(t.V._items()[199].live, true, "latest message stays live at tail");
  h.equal(t.els[199].isConnected, true, "latest node still connected");
});

h.test("pinned and streaming nodes stay live even outside the window", function () {
  var t = transcript(200, 60);
  t.els[5].setAttribute("data-gb-vpin", "1");
  t.els[10].id = "streaming-77";
  t.E.ChatState.streamingMessageId = "streaming-77";
  nearBottom(t);
  run(t);
  h.equal(t.V._items()[5].live, true, "data-gb-vpin keeps item 5 live");
  h.equal(t.els[5].isConnected, true, "pinned node connected");
  h.equal(t.V._items()[10].live, true, "streaming id keeps item 10 live");
  h.equal(t.els[10].isConnected, true, "streaming node connected");
  h.equal(t.c.querySelectorAll(".gb-vplaceholder").length, 200 - t.V.liveCount(),
    "pins are excluded from the placeholder set");
});

h.test("anchor compensation restores reading position after a layout shift", function () {
  /* 20px messages are inflated to the 24px placeholder floor: the 100 items
   * above the anchor grow by 4px each. findRange assumed 36px each, so the
   * anchor moves 400px; the shipped compensation must add that delta back. */
  var t = transcript(200, 20);
  nearBottom(t);
  var before = t.c.scrollTop;
  var anchor = null;
  var containerTop = t.c.getBoundingClientRect().top;
  for (var i = 0; i < t.els.length; i++) {
    var rect = t.els[i].getBoundingClientRect();
    if (rect.bottom > containerTop + 1) { anchor = t.els[i]; break; }
  }
  h.assert(anchor !== null, "an anchor exists in the live set");
  var anchorTopBefore = anchor.getBoundingClientRect().top;
  run(t);
  h.equal(t.c.scrollTop, before + 400, "compensation applied the 400px delta once");
  var anchorTopAfter = anchor.getBoundingClientRect().top;
  h.assert(Math.abs(anchorTopAfter - anchorTopBefore) <= 1,
    "anchor top restored: " + anchorTopBefore + " -> " + anchorTopAfter);
  h.equal(t.c.querySelectorAll(".gb-vplaceholder")[0].style.height, "24px",
    "placeholder floor is observable (see KNOWN BUG)");
});

h.test("ensureVisible / ensureById rematerialize a detached message", function () {
  var t = transcript(200, 60);
  nearBottom(t);
  run(t);
  h.equal(t.els[5].isConnected, false, "item 5 is offscreen before the probe");
  t.V.ensureVisible(t.els[5]);
  h.equal(t.els[5].isConnected, true, "ensureVisible reattached the node");
  h.equal(t.V._items()[5].live, true, "item 5 marked live again");
  run(t);
  var found = t.V.ensureById("msg-5");
  h.equal(found, t.els[5], "ensureById resolves tracked detached nodes");
  h.equal(found.isConnected, true, "ensureById materializes before returning");
});

h.test("clear() materializes before reset; remove() drops placeholder and item", function () {
  var t = transcript(200, 60);
  nearBottom(t);
  run(t);
  t.V.clear();
  h.equal(t.V.isEnabled(), false, "clear disables windowing");
  h.equal(t.V.count(), 0, "clear empties tracking");
  h.equal(t.c.querySelectorAll(":scope > .message").length, 200, "all messages restored on clear");
  h.equal(t.c.querySelectorAll(".gb-vplaceholder").length, 0, "no placeholder survives clear");

  var u = transcript(200, 60);
  nearBottom(u);
  run(u);
  var placeholdersBefore = u.c.querySelectorAll(".gb-vplaceholder").length;
  u.V.remove(u.els[10]);
  h.equal(u.V.count(), 199, "remove drops one tracked item");
  h.equal(u.c.querySelectorAll(".gb-vplaceholder").length, placeholdersBefore - 1,
    "remove deletes the item's placeholder");
});

h.test("suppressPersist and appendTarget guard mid-batch registration", function () {
  var t = transcript(200, 60);
  t.E.ChatState.suppressPersist = true;
  t.V.maybeEnable();
  h.equal(t.V.isEnabled(), false, "no windowing during history replay");
  t.E.ChatState.suppressPersist = false;
  t.V.maybeEnable();
  h.equal(t.V.isEnabled(), true, "windowing resumes after replay");

  var frag = t.E.sandbox.document.createDocumentFragment();
  var extra = D.makeMessage(t.E, 900, 60, "bot");
  frag.appendChild(extra);
  t.E.ChatState.appendTarget = frag;
  var before = t.V.count();
  t.V.register(extra);
  h.equal(t.V.count(), before, "register skips nodes still in appendTarget");
  t.E.ChatState.appendTarget = null;
});

h.test("registerAll and afterHistoryRender rebuild tracking from a changed transcript", function () {
  var t = transcript(130, 60);
  nearBottom(t);
  run(t);
  t.V.clear();
  for (var i = 0; i < 20; i++) t.c.removeChild(t.els[i]);
  t.V.registerAll(t.c);
  h.equal(t.V.count(), 110, "tracking reflects external removals");
  h.equal(t.V.isEnabled(), false, "registerAll resets enablement for a final settle");
  t.V.afterHistoryRender();
  h.equal(t.V.count(), 110, "afterHistoryRender registers the rebuilt transcript");
  h.equal(t.V.isEnabled(), false, "110 messages stay below the enable threshold");
  for (var k = 0; k < 15; k++) {
    var added = D.makeMessage(t.E, 900 + k, 60, "bot");
    t.c.appendChild(added);
  }
  t.V.afterHistoryRender();
  h.equal(t.V.count(), 125, "rebuilt transcript now exceeds the threshold");
  h.equal(t.V.isEnabled(), true, "afterHistoryRender enables windowing at 125");
});
