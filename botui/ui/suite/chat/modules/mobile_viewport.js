"use strict";
/* #2516 — chat-scoped visualViewport bridge: visible height for the composer
   cap and transcript; re-pin only while typing, 10_scroll.js intent wins. */
(function () {
  var app, input, messages, height = 0, published = 0, frame = 0;
  function nodes() {
    app = app || document.getElementById("chat-app");
    input = input || document.getElementById("messageInput");
    messages = messages || document.getElementById("messages");
  }
  function visibleHeight() {
    var vv = window.visualViewport;
    var value = vv && typeof vv.height === "number" ? vv.height : window.innerHeight;
    return typeof value === "number" && isFinite(value) && value > 0 ? value : 0;
  }
  window.GBChatVisibleHeight = visibleHeight;
  function apply() {
    frame = 0;
    nodes();
    var next = visibleHeight();
    if (!next) return;
    var rounded = Math.round(next);
    if (app && rounded !== published) {
      app.style.setProperty("--gb-chat-visible-height", rounded + "px");
      published = rounded;
    }
    if (Math.abs(next - height) < 1) return;
    var closing = height > 0 && next < height;
    height = next;
    if (!input) return;
    if (typeof Event === "function") input.dispatchEvent(new Event("gb-chat-viewport"));
    if (!closing || !messages || document.activeElement !== input) return;
    var intentAt = typeof gbPinCancelAt === "number" ? gbPinCancelAt : 0;
    if (Date.now() - intentAt < 800) return;
    if (typeof ChatState !== "undefined" && ChatState.isUserAtBottom === false) return;
    messages.scrollTop = messages.scrollHeight;
  }
  function schedule() {
    if (!frame) frame = window.requestAnimationFrame(apply);
  }
  var vv = window.visualViewport;
  if (vv && typeof vv.addEventListener === "function") {
    vv.addEventListener("resize", schedule);
    vv.addEventListener("scroll", schedule);
  }
  window.addEventListener("resize", schedule);
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", schedule);
  else schedule();
})();
