/**
 * RUNTIME gating assertions for the two shipped mobile IIFEs (#2515).
 *
 * Proves the touch model is installed only on a coarse pointer or a narrow
 * viewport, that desktop apps without the mobile root are inert, and that a
 * broken matchMedia degrades safely. Drawer wiring intentionally lives before
 * the gate in shipped code, so those handlers remain available on desktop.
 */
"use strict";

var h = require("./00_harness");
var f = require("./02_fixture");
var test = h.test;
var assert = h.assert;
var equal = h.assertEqual;
var create = f.create;

function desktop(apps) { return create({ apps: apps || ["docs", "slides"], pointer: "fine", width: 1440 }); }
function has(list, type) { return Object.prototype.hasOwnProperty.call(list || {}, type); }

test("RUNTIME #2515 desktop: docs installs no touch/caret listeners on a fine pointer", function () {
  var fx = desktop(["docs"]);
  var r = fx.refs.docs;
  assert(!has(r.app.listeners, "pointerup"), "no tap-to-caret listener");
  assert(!fx.hasDocListener("selectionchange"), "no caret-reveal listener");
  assert(!fx.hasWindowListener("orientationchange"), "no orientation reflow");
  equal(Object.keys(fx.vvListeners).length, 0, "no visualViewport listeners");
  assert(has(fx.doc._listeners, "keydown"), "drawer escape stays wired on desktop");
  r.toggle.dispatchEvent(fx.event({ type: "click" }));
  assert(r.main.classList.contains("sidebar-open"), "drawer still works on desktop");
});

test("RUNTIME #2515 desktop: slides installs no fit or gesture listeners on a fine pointer", function () {
  var fx = desktop(["slides"]);
  var r = fx.refs.slides;
  ["pointerdown", "pointermove", "pointerup", "pointercancel"].forEach(function (type) {
    assert(!has(r.scroll.listeners, type), "no " + type + " gesture listener");
  });
  assert(!fx.hasWindowListener("resize"), "no resize fit");
  assert(!fx.hasWindowListener("orientationchange"), "no orientation fit");
  fx.doc.dispatchEvent(fx.event({ type: "DOMContentLoaded" }));
  fx.flushTimers();
  equal(r.canvas.style.zoom, undefined, "canvas zoom untouched on desktop");
  equal(fx.win.SlideCanvas.scale, 1, "desktop hit-testing scale untouched");
  var toggle = fx.doc.getElementById("slidesSidebarToggle");
  toggle.dispatchEvent(fx.event({ type: "click" }));
  assert(r.main.classList.contains("sidebar-open"), "drawer still works on desktop");
});

test("RUNTIME #2515 coarse pointer at desktop width still installs the touch model", function () {
  var fx = create({ apps: ["docs", "slides"], pointer: "coarse", width: 1440 });
  assert(has(fx.refs.docs.app.listeners, "pointerup"), "docs tap-to-caret installed");
  assert(has(fx.refs.slides.scroll.listeners, "pointerdown"), "slides gestures installed");
});

test("RUNTIME #2515 narrow viewport with a fine pointer still installs the touch model", function () {
  var fx = create({ apps: ["docs", "slides"], pointer: "fine", width: 390 });
  assert(has(fx.refs.docs.app.listeners, "pointerup"), "docs width fallback");
  assert(has(fx.refs.slides.scroll.listeners, "pointerdown"), "slides width fallback");
});

test("RUNTIME #2515 matchMedia failure degrades to no touch model without throwing", function () {
  var fx = create({ apps: ["docs", "slides"], throwMatchMedia: true });
  assert(has(fx.doc._listeners, "keydown"), "drawer wiring still installed");
  assert(!has(fx.refs.docs.app.listeners, "pointerup"), "no touch caret listener");
  assert(!has(fx.refs.slides.scroll.listeners, "pointerdown"), "no gesture listener");
  fx.refs.docs.toggle.dispatchEvent(fx.event({ type: "click" }));
  assert(fx.refs.docs.main.classList.contains("sidebar-open"), "drawer usable after matchMedia error");
});

test("RUNTIME #2515 a fragment without #docs-app is inert and throws nothing", function () {
  var fx = create({ apps: [] });
  assert(!fx.hasDocListener("keydown"), "no drawer listener without the app root");
  assert(!fx.hasDocListener("selectionchange"), "no caret listener without the app root");
  equal(fx.doc.body.children.length, 0, "no app nodes were created");
});

test("RUNTIME #2515 a fragment without #slides-app creates no mobile controls", function () {
  var fx = create({ apps: [] });
  ["slidesSidebarToggle", "slidesSidebarClose", "slidesSidebarBackdrop"].forEach(function (id) {
    equal(fx.doc.getElementById(id), null, "no " + id);
  });
  assert(!fx.hasWindowListener("resize"), "no fit listener without the app root");
});

test("RUNTIME #2515 an already parsed document fits slides without DOMContentLoaded", function () {
  var fx = create({ apps: ["slides"], readyState: "complete" });
  assert(fx.timerDelays.indexOf(300) >= 0, "fit scheduled immediately");
  fx.flushTimers();
  equal(fx.refs.slides.canvas.style.zoom, "0.5", "fit still applied");
});
