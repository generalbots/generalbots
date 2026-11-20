/**
 * #2515 labelled [STATIC] source assertions and a late-swap REGRESSION.
 *
 * STATIC checks read shipped source only: CSS media queries, touch targets and
 * deliberately absent features cannot be executed in a Node vm, so they never
 * produce a runtime PASS. The REGRESSION executes the shipped Slides IIFE with
 * no canvas host at load time, renders the real fragment shape afterwards and
 * asserts fit-to-width and two-finger pan install against the late host.
 */
"use strict";

var h = require("./00_harness");
var f = require("./02_fixture");
var test = h.test;
var staticTest = h.staticTest;
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

staticTest("#2515 REGRESSION: phone grid is hit-testable — toolbar is one row, format row is in the overflow", function () {
  var menus = read("sheet/css/04_context_menus.css");
  var phone = menus.slice(menus.indexOf("@media (max-width: 760px)"),
    menus.indexOf("@media (max-width: 480px)"));
  /* Root cause of the CDP FAIL: a 305px main row + 56px format row left
     .sheet-content 0px tall, so the first cell's centre was covered by the
     toolbar and pointerdown landed on the Bold button. */
  has("sheet/css/08_a11y.css", "\\.sheet-app \\.sheet-toolbar-row \\{\\s*flex-wrap: wrap",
    "the late a11y rule that wrapped both rows still exists");
  assert(/#sheet-app \.sheet-toolbar-row-main \{ flex-wrap: nowrap; \}/.test(phone),
    "phone main row defeats the late wrap rule");
  assert(/#sheet-app \.sheet-toolbar-row-format \{ display: none !important; \}/.test(phone),
    "format row leaves the phone layout so it cannot intercept a cell tap");
  assert(/#sheet-app \.sheet-main,\s*#sheet-app \.sheet-content \{ min-height: 0; \}/.test(phone),
    "grid host keeps a measurable height inside the flex column");
  var overflow = read("sheet/sheet-inline/04_toolbar_overflow.js");
  assert(overflow.indexOf("sheet-toolbar-row-format") >= 0,
    "overflow script pulls the hidden format commands into its menu");
  assert(overflow.indexOf('menuSources(container).length > 1') >= 0,
    "overflow affordance stays visible when only the format row is hidden");
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

test("#2515 REGRESSION: Slides fit and gestures install after the late canvas-host swap", function () {
  /* Root cause: 99_mobile.js used to capture `.sl-canvas-scroll` once at load
     time. slides.html ships an empty #slides-content and the server swaps in
     `.sl-canvas-container > .sl-canvas`, so the capture stayed null, fit()
     returned at its first guard and the gesture block never installed. */
  has(SLIDES_MOBILE, "scroll = main\\.querySelector\\(\"\\.sl-canvas-scroll\"\\) \\|\\| main\\.querySelector\\(\"\\.sl-canvas-container\"\\)",
    "the host is re-queried for both shipped shapes");
  has(SLIDES_MOBILE, "if \\(!scroll\\) \\{ setTimeout\\(fit, 200\\); return; \\}",
    "fit retries until the htmx swap supplies a host");
  has(SLIDES_MOBILE, "main\\.addEventListener\\(\"pointerdown\"",
    "gestures bind to the stable .slides-main host");
  assert(read("slides/slides.html").indexOf("sl-canvas-scroll") < 0,
    "the fragment still has no host at script time");
  has("slides/modules/99_init.js", "presentation-view", "a later htmx swap populates #slides-content");
  var fragment = h.readRepo("botserver/crates/botslides/src/ui_fragments/sidebars.rs");
  assert(fragment.indexOf("sl-canvas-container") >= 0,
    "the server fragment uses sl-canvas-container");
  assert(fragment.indexOf("sl-canvas-scroll") < 0,
    "no shipped server fragment ever renders the old queried host");

  /* Execute the real IIFE at the real moment (no host), then render the real
     server fragment shape and prove fit + pan work against the late host. */
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
  equal(canvas.style.zoom, "0.5", "late-rendered canvas fitted to width");
  equal(fx.win.SlideCanvas.scale, 0.5, "hit-testing scale updated after the swap");

  down(r.main, fx, 1, 100, 100, canvas);
  down(r.main, fx, 2, 200, 100, canvas);
  equal(fx.windowDispatched.filter(function (event) { return event.type === "pointerup"; }).length,
    1, "second finger ended any object drag");
  var moved = fx.event({ type: "pointermove", pointerType: "touch", pointerId: 2,
    clientX: 260, clientY: 100, target: canvas });
  r.main.dispatchEvent(moved);
  equal(moved.preventDefaultCalls, 1, "two-finger pan handler installed after the swap");
  equal(container.scrollLeft, -30, "late host scrolled by the midpoint delta");
  var userZoomApplied = canvas.style.zoom;

  /* A slide change swaps #slides-content innerHTML again; pan must follow the
     current host, not the container captured on the first render. */
  [1, 2].forEach(function (id) {
    r.main.dispatchEvent(fx.event({ type: "pointerup", pointerType: "touch", pointerId: id, target: canvas }));
  });
  r.main.removeChild(container);
  var next = fx.doc.createElement("div");
  next.className = "sl-canvas-container";
  next.clientWidth = 504;
  r.main.appendChild(next);
  var nextCanvas = fx.doc.createElement("div");
  nextCanvas.className = "sl-canvas";
  next.appendChild(nextCanvas);
  fx.win.dispatchEvent(fx.event({ type: "resize" }));
  equal(nextCanvas.style.zoom, userZoomApplied, "a later slide swap is refitted at the user zoom");
  down(r.main, fx, 3, 100, 100, nextCanvas);
  down(r.main, fx, 4, 200, 100, nextCanvas);
  var moved2 = fx.event({ type: "pointermove", pointerType: "touch", pointerId: 4,
    clientX: 220, clientY: 100, target: nextCanvas });
  r.main.dispatchEvent(moved2);
  equal(next.scrollLeft, -10, "pan follows the current host after a slide swap");
});

function down(host, fx, id, x, y, target) {
  host.dispatchEvent(fx.event({ type: "pointerdown", pointerType: "touch", pointerId: id,
    clientX: x, clientY: y, target: target }));
}
