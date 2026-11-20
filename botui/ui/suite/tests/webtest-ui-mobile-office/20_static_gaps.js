/**
 * #2515 labelled [STATIC] source assertions and one KNOWN BUG characterisation.
 *
 * STATIC checks read shipped source only: CSS media queries, touch targets and
 * deliberately absent features cannot be executed in a Node vm, so they never
 * produce a runtime PASS. The KNOWN BUG test executes the shipped Slides IIFE
 * under the real fragment load order and asserts the observed defect; the fix
 * belongs to production, not to this harness.
 */
"use strict";

var h = require("./00_harness");
var f = require("./02_fixture");
var test = h.test;
var staticTest = h.staticTest;
var knownBug = h.knownBug;
var assert = h.assert;
var equal = h.assertEqual;
var read = h.readFile;
var has = h.sourceHas;
var lacks = h.sourceLacks;

var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var DOCS_MOBILE = "docs/modules/19_mobile.js";
var SLIDES_MOBILE = "slides/modules/99_mobile.js";
var SHEET_MOBILE = "sheet/sheet-inline/05_mobile.js";

staticTest("#2515: both real mobile IIFEs exist and are wired last in their fragments", function () {
  assert(h.fileExists(DOCS_MOBILE), "docs mobile IIFE exists");
  assert(h.fileExists(SLIDES_MOBILE), "slides mobile IIFE exists");
  var docsHtml = read("docs/docs.html");
  var slidesHtml = read("slides/slides.html");
  has(DOCS_MOBILE, "#2515", "docs module is tagged with the issue");
  has(SLIDES_MOBILE, "#2515", "slides module is tagged with the issue");
  assert(docsHtml.indexOf("/suite/docs/modules/19_mobile.js") >= 0, "docs.html loads the module");
  assert(slidesHtml.indexOf("/suite/slides/modules/99_mobile.js") >= 0, "slides.html loads the module");
  assert(docsHtml.indexOf("19_mobile.js") > docsHtml.indexOf("99_init.js"), "docs module loads after init");
});

staticTest("#2515: Sheets ships a drawer, tap edit, long-press menu and pinch zoom", function () {
  has(SHEET_MOBILE, '"sheet-app"', "sheet mobile module targets the real app root");
  has(SHEET_MOBILE, "sheetSidebarToggle", "drawer toggle is wired");
  has(SHEET_MOBILE, "sidebar-open", "drawer state class is reused");
  has(SHEET_MOBILE, 'pointerType !== "touch"', "touch-only gating is present");
  has(SHEET_MOBILE, "contextmenu", "long-press opens the context menu");
  has(SHEET_MOBILE, "--sheet-touch-zoom", "pinch writes a CSS zoom variable");
  has("sheet/sheet.html", "/suite/sheet/sheet-inline/05_mobile.js", "sheet.html loads it");
});

staticTest("#2515: the old 240px column and hidden formatting toolbar are gone in CSS source", function () {
  var menus = "sheet/css/04_context_menus.css";
  has(menus, "@media \\(max-width: 760px\\)", "phone layout breakpoint exists");
  has(menus, "#sheet-app \\.sheet-sidebar", "sidebar is id-qualified to win specificity");
  has(menus, "position: absolute !important", "sidebar leaves the flex flow");
  has(menus, "width: min\\(86vw, 320px\\) !important", "fixed 240px width is overridden");
  has(menus, "@media \\(max-width: 768px\\)", "toolbar breakpoint exists");
  has(menus, "overflow-x: auto", "formatting controls scroll instead of disappearing");
  lacks(menus, "\\.toolbar-center\\s*\\{[^}]*display:\\s*none", "no toolbar-center display:none remains");
  has("sheet/css/08_a11y.css", "display: none !important", "the later a11y rule still exists");
  has(menus, "display: flex !important", "id-qualified rule beats the later a11y rule");
  has("sheet/css/02_grid.css", "@media \\(max-width: 720px\\)", "formula-bar compact guard exists");
  has("sheet/css/02_grid.css", "font-size: var\\(--text-md\\) !important", "iOS zoom is defeated at 16px");
});

staticTest("#2515: Docs and Slides use drawers and keyboard-aware layout, not 34% strips", function () {
  var docs = read("docs/docs-mobile.css");
  var slides = read("slides/slides.css");
  assert(docs.indexOf("@media (max-width: 760px)") >= 0, "docs phone breakpoint");
  assert(slides.indexOf("@media (max-width: 760px)") >= 0, "slides phone breakpoint");
  assert(docs.indexOf(".docs-main.sidebar-open .docs-sidebar { transform: translateX(0); }") >= 0,
    "docs sidebar slides in");
  assert(slides.indexOf(".slides-main.sidebar-open > .slides-sidebar { transform: translateY(0); }") >= 0,
    "slides outline is a bottom drawer");
  assert(slides.indexOf("height: min(62vh, 420px)") >= 0, "drawer height is bounded");
  assert(!/height:\s*34%/.test(slides), "the old 34% canvas-squeezing strip is absent");
  assert(docs.indexOf("overflow-x: hidden") >= 0 && slides.indexOf("overflow-x: hidden") >= 0 &&
    read("sheet/css/04_context_menus.css").indexOf("overflow-x: hidden") >= 0,
    "all three app roots suppress page-level horizontal scroll");
  var viewport = read("js/mobile-viewport.js");
  assert(viewport.indexOf("--gb-vv-bottom") >= 0, "visualViewport publishes the keyboard inset");
  assert(docs.indexOf("var(--gb-vv-bottom, 0px)") >= 0 && slides.indexOf("var(--gb-vv-bottom, 0px)") >= 0 &&
    read("sheet/css/04_context_menus.css").indexOf("var(--gb-vv-bottom, 0px)") >= 0,
    "all three apps reserve the keyboard inset");
});

staticTest("#2515: 44px touch targets and 24px handles are present in CSS source", function () {
  var docs = read("docs/docs-mobile.css");
  var slides = read("slides/slides.css");
  var sheet = read("sheet/css/04_context_menus.css");
  [docs, slides, sheet].forEach(function (css, index) {
    var name = ["docs", "slides", "sheet"][index];
    assert(/min-width:\s*44px/.test(css), name + " has 44px minimum widths");
    assert(/min-height:\s*44px/.test(css), name + " has 44px minimum heights");
  });
  assert(slides.indexOf("width: 24px !important") >= 0 && slides.indexOf("height: 24px !important") >= 0,
    "slides object handles are 24px");
  assert(slides.indexOf(".sl-handle::after") >= 0 && slides.indexOf(".sl-rotator::after") >= 0,
    "handles gain a larger touch ring");
  assert(slides.indexOf(".sl-element { touch-action: none; }") >= 0, "object drags keep their touch default disabled");
  assert(sheet.indexOf(".ss-fill-handle::after") >= 0 && sheet.indexOf("inset: -18px") >= 0,
    "sheet fill handle has a 45px grab ring");
  assert(docs.indexOf("min-height: 48px") >= 0, "docs toolbar rows are 48px tall");
});

staticTest("#2515: features requested by the issue that are still not shipped", function () {
  var docs = read(DOCS_MOBILE) + read("docs/docs-mobile.css");
  var slidesSrc = read(SLIDES_MOBILE) + read("slides/slides.css") +
    read("slides/modules/02_handles.js") + read("slides/modules/03_canvas.js");
  assert(!/longpress|long-press|selection-handle/i.test(docs),
    "Docs has no custom long-press selection handles; native contenteditable owns them");
  assert(docs.indexOf("-webkit-touch-callout: default") >= 0, "native callout is explicitly preserved");
  assert(!/gesturestart|gesturechange|deviceorientation/i.test(slidesSrc),
    "Slides has no rotation gesture recogniser");
  assert(slidesSrc.indexOf("makeRotateHandler") >= 0 && read("slides/slides.css").indexOf("#rotationInput") >= 0,
    "rotation is offered by an explicit control instead");
  var sheetMobile = read(SHEET_MOBILE);
  assert(!/freeze/i.test(sheetMobile), "the Sheet mobile module does not auto-freeze panes while scrolling");
  has("sheet/js/modules-sheet-advanced/15_freeze.js", "freezeTopRow", "freezing remains an opt-in command");
  h.note("DEFERRED (browser/hardware only): 390x844 CDP tap/edit/overflow screenshots, keyboard coverage of the active cell/caret, real-device pan/pinch/rotation and 1440x900 no-regression comparison. No runtime PASS is claimed for these.");
});

staticTest("#2515: mobile control keys exist in all eight locale catalogs", function () {
  var checks = [
    { key: "paper-toggle-sidebar", file: "ui-productivity.ftl" },
    { key: "action-close", file: "common.ftl" },
  ];
  checks.forEach(function (check) {
    LOCALES.forEach(function (locale) {
      var text = h.readRepo("botlib/locales/" + locale + "/" + check.file);
      assert(new RegExp("^" + check.key + "\\s*=", "m").test(text),
        locale + " missing " + check.key);
    });
  });
  var src = read(SLIDES_MOBILE);
  assert(src.indexOf('"paper-toggle-sidebar"') >= 0 && src.indexOf('"action-close"') >= 0,
    "shipped code uses literal keys, not generated ones");
  assert(src.indexOf("auto-") < 0, "no machine-generated keys");
});

knownBug("#2515 KNOWN BUG: Slides mobile fit and gestures target a host no shipped fragment renders", function () {
  /* Source facts: 99_mobile.js captures `.sl-canvas-scroll` once, fit() returns
     early when that capture is null and the gesture listeners live inside
     `if (scroll)`. slides.html ships an empty #slides-content, and the server
     presentation-view renders `.sl-canvas-container > .sl-canvas` — the queried
     class exists only in CSS and in the two querying modules. */
  has(SLIDES_MOBILE, "var scroll = main\\.querySelector\\(\"\\.sl-canvas-scroll\"\\);", "host captured once");
  has(SLIDES_MOBILE, "if \\(!scroll\\) return;", "fit returns early without the host");
  has(SLIDES_MOBILE, "if \\(scroll\\) \\{", "gesture listeners are host-gated");
  assert(read("slides/slides.html").indexOf("sl-canvas-scroll") < 0,
    "the fragment does not contain the host at script time");
  has("slides/modules/99_init.js", "presentation-view", "a later htmx swap populates #slides-content");
  var fragment = h.readRepo("botserver/crates/botslides/src/ui_fragments/sidebars.rs");
  assert(fragment.indexOf("sl-canvas-container") >= 0,
    "the server fragment uses sl-canvas-container");
  assert(fragment.indexOf("sl-canvas-scroll") < 0,
    "no shipped server fragment ever renders the queried host");

  /* Execute the real IIFE at the real moment (no host), then render the real
     server fragment shape and fire every wake-up the module offers. */
  var fx = f.create({ apps: ["slides"], scroll: false, readyState: "complete" });
  var r = fx.refs.slides;
  var container = fx.doc.createElement("div");
  container.className = "sl-canvas-container";
  container.clientWidth = 504;
  r.main.appendChild(container);
  var canvas = fx.doc.createElement("div");
  canvas.className = "sl-canvas";
  container.appendChild(canvas);

  fx.flushTimers();
  fx.win.dispatchEvent(fx.event({ type: "resize" }));
  fx.win.dispatchEvent(fx.event({ type: "orientationchange" }));
  fx.flushTimers();
  down(container, fx, 1, 100, 100);
  down(container, fx, 2, 200, 100);
  var moved = fx.event({ type: "pointermove", pointerType: "touch", pointerId: 2,
    clientX: 260, clientY: 100, target: canvas });
  container.dispatchEvent(moved);

  equal(canvas.style.zoom, undefined, "OBSERVED: no fit after the fragment rendered");
  equal(fx.win.SlideCanvas.scale, 1, "OBSERVED: scale sink never updated");
  equal(container.scrollLeft, 0, "OBSERVED: two-finger pan never scrolled");
  equal(moved.preventDefaultCalls, 0, "OBSERVED: no gesture handler ran");
  equal(fx.windowDispatched.filter(function (event) { return event.type === "pointerup"; }).length,
    0, "OBSERVED: no synthetic pointerup ended any drag");
  h.note("KNOWN BUG (reported, not patched): slides/modules/99_mobile.js:67 queries `.sl-canvas-scroll`, but no shipped fragment renders that class — slides.html has an empty #slides-content and botslides render_presentation_view emits `.sl-canvas-container > .sl-canvas`. `scroll` stays null, fit() returns at its first guard and the `if (scroll)` gesture block never installs, so fit-to-width and two-finger pan/pinch are permanently inert on a phone. The 200ms retry only covers a missing .sl-canvas inside an already-captured host. Re-query or defer installation until the slide host is populated in production.");
});

function down(scroll, fx, id, x, y) {
  scroll.dispatchEvent(fx.event({ type: "pointerdown", pointerType: "touch", pointerId: id,
    clientX: x, clientY: y, target: scroll.children[0] }));
}
