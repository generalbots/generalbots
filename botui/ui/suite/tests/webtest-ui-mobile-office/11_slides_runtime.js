/**
 * RUNTIME assertions for the REAL slides/modules/99_mobile.js IIFE (#2515).
 *
 * The drawer DOM is built by the shipped code itself. Fit-to-width maths and
 * the two-finger pan/pinch guards are exercised by dispatching pointer events
 * into the sandbox and reading canvas zoom, scroll offsets and synthetic
 * window events. No production logic is reimplemented here.
 */
"use strict";

var h = require("./00_harness");
var f = require("./02_fixture");
var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;
var create = f.create;

function phone(opts) {
  var options = { apps: ["slides"], readyState: "loading" };
  Object.keys(opts || {}).forEach(function (key) { options[key] = opts[key]; });
  var fx = create(options);
  fx.doc.dispatchEvent(fx.event({ type: "DOMContentLoaded" }));
  fx.flushTimers();
  return fx;
}

function click(node, fx) { node.dispatchEvent(fx.event({ type: "click" })); }
function down(fx, id, x, y, type) {
  fx.refs.slides.scroll.dispatchEvent(fx.event({
    type: "pointerdown", pointerType: type || "touch", pointerId: id,
    clientX: x, clientY: y, target: fx.refs.slides.canvas,
  }));
}
function move(fx, id, x, y) {
  var event = fx.event({ type: "pointermove", pointerType: "touch", pointerId: id,
    clientX: x, clientY: y, target: fx.refs.slides.canvas });
  fx.refs.slides.scroll.dispatchEvent(event);
  return event;
}
function end(fx, type, id) {
  fx.refs.slides.scroll.dispatchEvent(fx.event({
    type: type, pointerType: "touch", pointerId: id, target: fx.refs.slides.canvas,
  }));
}
function twoFingers(fx) { down(fx, 1, 100, 100); down(fx, 2, 200, 100); }

test("RUNTIME #2515 slides: shipped code creates toggle, backdrop and close with catalog keys", function () {
  var fx = phone();
  var toggle = fx.doc.getElementById("slidesSidebarToggle");
  var close = fx.doc.getElementById("slidesSidebarClose");
  var backdrop = fx.doc.getElementById("slidesSidebarBackdrop");
  assert(toggle && close && backdrop, "all three controls exist");
  equal(toggle.getAttribute("data-i18n-aria-label"), "paper-toggle-sidebar", "toggle uses a real catalog key");
  equal(close.getAttribute("data-i18n-aria-label"), "action-close", "close uses a real catalog key");
  equal(toggle.getAttribute("aria-label"), "Toggle sidebar", "toggle label from gbT fallback");
  equal(close.getAttribute("aria-label"), "Close", "close label from gbT fallback");
  assert(fx.gbTCalls.some(function (call) { return call.key === "paper-toggle-sidebar"; }), "gbT consulted for toggle");
});

test("RUNTIME #2515 slides: controls are inserted at the head of their hosts", function () {
  var fx = phone();
  var r = fx.refs.slides;
  equal(r.toolbarLeft.children[0], fx.doc.getElementById("slidesSidebarToggle"), "toggle first in toolbar-left");
  equal(r.main.children[0], fx.doc.getElementById("slidesSidebarBackdrop"), "backdrop first in main");
  equal(r.tabs.children[0], fx.doc.getElementById("slidesSidebarClose"), "close first in sidebar tabs");
});

test("RUNTIME #2515 slides: drawer opens and closes via toggle with aria state", function () {
  var fx = phone();
  var r = fx.refs.slides;
  var toggle = fx.doc.getElementById("slidesSidebarToggle");
  equal(toggle.getAttribute("aria-expanded"), "false", "initially collapsed");
  click(toggle, fx);
  assert(r.main.classList.contains("sidebar-open"), "drawer opened");
  equal(toggle.getAttribute("aria-expanded"), "true", "expanded published");
  click(toggle, fx);
  assert(!r.main.classList.contains("sidebar-open"), "drawer closed");
  equal(toggle.getAttribute("aria-expanded"), "false", "collapsed published");
});

test("RUNTIME #2515 slides: close, backdrop and Escape all dismiss the drawer", function () {
  var fx = phone();
  var r = fx.refs.slides;
  var toggle = fx.doc.getElementById("slidesSidebarToggle");
  click(toggle, fx);
  click(fx.doc.getElementById("slidesSidebarClose"), fx);
  assert(!r.main.classList.contains("sidebar-open"), "close button dismissed");
  click(toggle, fx);
  click(fx.doc.getElementById("slidesSidebarBackdrop"), fx);
  assert(!r.main.classList.contains("sidebar-open"), "backdrop dismissed");
  click(toggle, fx);
  fx.doc.dispatchEvent(fx.event({ type: "keydown", key: "Escape" }));
  assert(!r.main.classList.contains("sidebar-open"), "Escape dismissed");
});

test("RUNTIME #2515 slides: fit-to-width zooms the 960px canvas at 300ms after boot", function () {
  var fx = phone();
  assert(fx.timerDelays.indexOf(300) >= 0, "boot scheduled a 300ms fit");
  equal(fx.refs.slides.canvas.style.zoom, "0.5", "504px host -> 480px available -> 0.5 zoom");
  equal(fx.win.SlideCanvas.scale, 0.5, "hit-testing scale kept in sync");
});

test("RUNTIME #2515 slides: fit clamps to the 0.25 floor on a narrow host", function () {
  var fx = phone({ scrollWidth: 100 });
  equal(fx.refs.slides.canvas.style.zoom, "0.25", "available floored at 240 -> 0.25 zoom");
});

test("RUNTIME #2515 slides: fit clamps to the 2x ceiling on a wide host", function () {
  var fx = phone({ scrollWidth: 4000 });
  equal(fx.refs.slides.canvas.style.zoom, "2", "scale capped at 2");
});

test("RUNTIME #2515 slides: a non-finite host width falls back to zoom 1", function () {
  var fx = phone({ scrollWidth: NaN });
  equal(fx.refs.slides.canvas.style.zoom, "1", "guarded against NaN scale");
  equal(fx.win.SlideCanvas.scale, 1, "guarded scale published");
});

test("RUNTIME #2515 slides: window resize refits the canvas synchronously", function () {
  var fx = phone();
  fx.refs.slides.scroll.clientWidth = 1224;
  fx.win.dispatchEvent(fx.event({ type: "resize" }));
  equal(fx.refs.slides.canvas.style.zoom, "1.25", "resize recomputed fit");
});

test("RUNTIME #2515 slides: orientationchange refits after 300ms", function () {
  var fx = phone();
  fx.refs.slides.scroll.clientWidth = 1224;
  fx.win.dispatchEvent(fx.event({ type: "orientationchange" }));
  fx.flushTimers();
  equal(fx.refs.slides.canvas.style.zoom, "1.25", "rotated viewport refitted");
});

test("RUNTIME #2515 slides: fit still sets CSS zoom when SlideCanvas is absent", function () {
  var fx = phone({ slideCanvas: false });
  equal(fx.refs.slides.canvas.style.zoom, "0.5", "zoom applied without the global scale sink");
  equal(fx.win.SlideCanvas, undefined, "no SlideCanvas global was fabricated");
});

test("RUNTIME #2515 slides: mouse pointerdown is ignored by the gesture model", function () {
  var fx = phone();
  down(fx, 1, 100, 100, "mouse");
  down(fx, 2, 200, 100);
  equal(fx.windowDispatched.length, 0, "one real touch pointer did not arm two-finger mode");
  var event = move(fx, 2, 260, 100);
  equal(event.preventDefaultCalls, 0, "no pan without a second real touch");
});

test("RUNTIME #2515 slides: second finger ends object drag via a bubbling window pointerup", function () {
  var fx = phone();
  var seen = [];
  fx.win.addEventListener("pointerup", function (event) { seen.push(event); });
  down(fx, 1, 100, 100);
  var second = fx.event({ type: "pointerdown", pointerType: "touch", pointerId: 2,
    clientX: 200, clientY: 100, target: fx.refs.slides.canvas });
  fx.refs.slides.scroll.dispatchEvent(second);
  equal(fx.windowDispatched.length, 1, "one synthetic pointerup dispatched");
  equal(fx.windowDispatched[0].type, "pointerup", "synthetic event type");
  equal(fx.windowDispatched[0].bubbles, true, "synthetic event bubbles to window drag handlers");
  equal(seen.length, 1, "window-level pointerup listener received it");
  equal(second.stopPropagationCalls, 1, "second touch did not start an object drag");
});

test("RUNTIME #2515 slides: two-finger pan scrolls by the midpoint delta and refits", function () {
  var fx = phone();
  twoFingers(fx);
  var event = move(fx, 2, 220, 100);
  equal(event.preventDefaultCalls, 1, "pan prevented the browser default");
  equal(fx.refs.slides.scroll.scrollLeft, -10, "midpoint moved +10 -> scrollLeft -10");
  equal(fx.refs.slides.canvas.style.zoom, "0.6", "pinch ratio 120/100 rescaled the canvas");
});

test("RUNTIME #2515 slides: pinch zoom is clamped so the canvas never vanishes", function () {
  var fx = phone();
  twoFingers(fx);
  move(fx, 2, 220, 100);
  move(fx, 2, 101, 100);
  equal(fx.refs.slides.canvas.style.zoom, "0.25", "userZoom clamped to 0.5 -> 0.5 * 0.5 fit");
});

test("RUNTIME #2515 slides: lifting one finger ends pan mode", function () {
  var fx = phone();
  twoFingers(fx);
  end(fx, "pointerup", 2);
  var event = move(fx, 1, 180, 100);
  equal(event.preventDefaultCalls, 0, "single-finger move is left to the object drag modules");
  equal(fx.refs.slides.scroll.scrollLeft, 0, "no pan after one finger lifted");
});

test("RUNTIME #2515 slides: pointercancel also ends pan mode", function () {
  var fx = phone();
  twoFingers(fx);
  end(fx, "pointercancel", 1);
  var event = move(fx, 2, 260, 100);
  equal(event.preventDefaultCalls, 0, "cancelled pointer did not leave a live pan");
});

test("RUNTIME #2515 slides: a third finger is guarded, not a new drag", function () {
  var fx = phone();
  twoFingers(fx);
  var before = fx.windowDispatched.length;
  var third = fx.event({ type: "pointerdown", pointerType: "touch", pointerId: 3,
    clientX: 300, clientY: 100, target: fx.refs.slides.canvas });
  fx.refs.slides.scroll.dispatchEvent(third);
  equal(fx.windowDispatched.length, before, "no extra synthetic pointerup for the third finger");
  equal(third.stopPropagationCalls, 1, "third pointer was swallowed while panning");
});

test("RUNTIME #2515 slides: fit retries at 200ms until the canvas host appears", function () {
  var fx = create({ apps: ["slides"], canvas: false, readyState: "loading" });
  fx.doc.dispatchEvent(fx.event({ type: "DOMContentLoaded" }));
  fx.flushOne();
  assert(fx.timerDelays.indexOf(200) >= 0, "missing canvas scheduled a 200ms retry");
  equal(fx.refs.slides.canvas, null, "canvas still absent after first fit");
  var canvas = fx.doc.createElement("div");
  canvas.className = "sl-canvas";
  fx.refs.slides.scroll.appendChild(canvas);
  fx.refs.slides.canvas = canvas;
  fx.flushOne();
  equal(fx.refs.slides.canvas.style.zoom, "0.5", "retry applied fit once canvas arrived");
});
