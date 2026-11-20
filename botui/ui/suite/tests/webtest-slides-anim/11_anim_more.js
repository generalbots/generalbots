/**
 * Slides animation/transition timeline — engine, transitions and STATIC tests (#2431).
 * Continuation of 10_anim.js: runtime checks exercise the REAL exported helpers
 * in a vm sandbox; DOM-only helpers are covered by labelled STATIC assertions.
 */
"use strict";

var h = require("./00_harness");
var test = h.test, assert = h.assert;
var gb = h.loadModules();
var A = gb.A;

function effect(id, elementId, extra) {
  var result = {
    id: id, elementId: elementId || null, effect: "fade", category: "entrance",
    duration: 500, delay: 0, easing: "ease", start: "click", direction: "left",
    path: { x1: 0, y1: 0, cx: 0.5, cy: 0, x2: 1, y2: 0 }, levels: []
  };
  Object.keys(extra || {}).forEach(function (key) { result[key] = extra[key]; });
  return result;
}

/* Canvas stub: elementFor() resolves effects through canvas.querySelector. */
function stage(env, ids) {
  var elements = {};
  ids.forEach(function (id) {
    var node = env.node("div");
    node.dataset.id = id;
    node.setAttribute("style", "");
    elements[id] = node;
  });
  env.window.SlideCanvas = {
    canvas: {
      querySelector: function (selector) {
        var match = /data-id="([^"]+)"/.exec(selector);
        return match && elements[match[1]] ? elements[match[1]] : null;
      },
      querySelectorAll: function () { return ids.map(function (id) { return elements[id]; }); }
    }
  };
  return elements;
}

function paragraphsOn(node, count) {
  var kids = [];
  for (var i = 0; i < count; i++) kids.push({ style: {} });
  node.querySelectorAll = function () { return kids; };
  return kids;
}

// ── Engine observable effects ─────────────────────────────────────────
test("seek applies initial, progress and final states on real stub elements", function () {
  var env = h.loadModules();
  var nodes = stage(env, ["enter", "exit"]);
  var list = [
    effect("fade", "enter", { effect: "fade", category: "entrance", duration: 200 }),
    effect("fade-out", "exit", { effect: "fade-out", category: "exit", duration: 200, start: "after" })
  ];
  assert(env.E.totalOf(list) === 400, "two-step build total wrong");
  env.E.seek(100, list);
  assert(env.E.time === 100 && env.E.total === 400, "seek did not publish clock/total");
  assert(Number(nodes.enter.style.opacity) > 0 && Number(nodes.enter.style.opacity) < 1,
    "entrance was not mid-progress");
  assert(nodes.exit.style.opacity === undefined, "a not-yet-started effect must not be forced hidden");
  env.E.seek(250, list);
  assert(Number(nodes.exit.style.opacity) > 0 && Number(nodes.exit.style.opacity) < 1,
    "exit was not mid-progress");
  env.E.seek(999, list);
  assert(env.E.time === 400 && nodes.exit.style.opacity === "0", "exit final state not settled");
  assert(env.E.failed.length === 0, "unexpected failed effect id");
});

test("motion path point generation is deterministic and settles at the endpoint", function () {
  var env = h.loadModules();
  var nodes = stage(env, ["mover"]);
  var list = [effect("line", "mover", {
    effect: "line", category: "motion", duration: 400,
    path: { x1: 0, y1: 0, cx: 0.5, cy: 0, x2: 1, y2: 0 }
  })];
  env.E.seek(200, list);
  var first = nodes.mover.style.transform;
  env.E.seek(200, list);
  assert(first === nodes.mover.style.transform, "path point differed between identical seeks");
  assert(first === "translate(50px,0px)", "quadratic midpoint wrong: " + first);
  env.E.revealAll(list);
  assert(nodes.mover.style.transform === "translate(100px,0px)", "path endpoint not applied");
});

test("a broken effect is reported while other effects still reach their state", function () {
  var env = h.loadModules();
  var nodes = stage(env, ["bad", "good"]);
  nodes.bad.setAttribute("style", "opacity: 1;");
  // The engine resets inline styles before every frame; keep the injected
  // failure on the live style object across that reset.
  var setAttribute = nodes.bad.setAttribute;
  nodes.bad.setAttribute = function (key, value) {
    return key === "style" ? undefined : setAttribute.call(this, key, value);
  };
  Object.defineProperty(nodes.bad.style, "opacity", {
    configurable: true,
    get: function () { return "1"; },
    set: function () { throw new Error("stubbed style failure"); }
  });
  var list = [
    effect("broken", "bad", { effect: "fade", category: "entrance", duration: 100, start: "click" }),
    effect("healthy", "good", { effect: "fade", category: "entrance", duration: 100, start: "with" })
  ];
  env.E.seek(50, list);
  assert(env.E.failed.indexOf("broken") >= 0, "broken effect id not recorded");
  assert(Number(nodes.good.style.opacity) > 0, "healthy effect was blocked by the broken one");
  env.E.revealAll(list);
  assert(nodes.bad.style.opacity === "1", "broken element was left hidden instead of restored");
});

test("revealAll shows every paragraph of a build effect immediately", function () {
  var env = h.loadModules();
  var nodes = stage(env, ["build"]);
  var kids = paragraphsOn(nodes.build, 3);
  var list = [effect("paragraph", "build", { effect: "paragraph", category: "build", duration: 300 })];
  env.E.seek(0, list);
  assert(kids.every(function (kid) { return kid.style.opacity === "0"; }), "build did not start hidden");
  env.E.revealAll(list);
  assert(kids.every(function (kid) { return kid.style.opacity === "1"; }), "build did not reveal all levels");
});

// ── Preview and reduce motion ─────────────────────────────────────────
test("reduced play skips the frame loop and settles final states immediately", function () {
  var env = h.loadModules();
  var nodes = stage(env, ["x"]);
  env.A.setReduceMotion(true);
  env.window.requestAnimationFrame = function () { throw new Error("frame scheduled in reduced mode"); };
  var list = [effect("fade-out", "x", { effect: "fade-out", category: "exit", duration: 500 })];
  var promise = env.E.play(list, { reduced: true });
  assert(promise && typeof promise.then === "function", "play did not return a promise");
  assert(nodes.x.style.opacity === "0", "reduced mode did not show the final state");
});

test("normal play advances the scrubber through frames to the final state", function () {
  var env = h.loadModules();
  var nodes = stage(env, ["y"]);
  var frames = [];
  env.window.requestAnimationFrame = function (cb) { frames.push(cb); return frames.length; };
  var list = [effect("fade-out", "y", { effect: "fade-out", category: "exit", duration: 100 })];
  env.E.play(list);
  assert(frames.length === 1, "play did not request a frame");
  frames.shift()(0);
  assert(frames.length === 1 && env.E.time === 0, "first tick did not schedule a follow-up");
  frames.shift()(200);
  assert(nodes.y.style.opacity === "0", "play did not settle at the end");
  assert(env.E.playing === false, "play left the engine running");
});

test("reduce-motion flag and media query are both honoured and serialized", function () {
  assert(A.reduced() === false, "default reduce-motion state should be off");
  assert(h.loadModules({ matchMediaMatches: true }).A.reduced() === true,
    "prefers-reduced-motion was ignored");
  var env = h.loadModules();
  env.A.setReduceMotion(true);
  assert(env.A.reduced() === true, "deck flag did not enable reduced motion");
  assert(env.A.load().reduceMotion === true, "reduce flag did not round-trip through storage");
  env.A.setReduceMotion(false);
  assert(env.A.reduced() === false, "deck flag did not disable reduced motion");
});

// ── Transitions ───────────────────────────────────────────────────────
test("default transition is well-formed and supports per-slide override", function () {
  var env = h.loadModules();
  var api = env.A;
  var def = api.defaultTransition();
  assert(def.type === "fade" && def.duration === 500 && def.advance === "click", "bad default transition");
  api.setTransition({ type: "wipe", duration: 900, direction: "right", advance: "time", seconds: 7 }, 0, false);
  assert(api.transition(0).type === "wipe" && api.transition(0).duration === 900, "override not applied");
  assert(api.transition(1).type === "fade", "unrelated slide lost the inherited transition");
  assert(api.transition(1, false) === null, "inherit=false should not synthesize a transition");
});

test("apply-to-all resets per-slide overrides and survives serialization", function () {
  var env = h.loadModules();
  var api = env.A;
  api.setTransition({ type: "wipe", duration: 700 }, 0, false);
  api.setTransition({ type: "push", duration: 100 }, 2, true);
  assert(api.transition(0).type === "push", "apply-to-all did not update the default");
  assert(Object.keys(api.state.slides).length === 0, "apply-to-all kept stale slide overrides");
  assert(api.load().defaultTransition.type === "push", "transition did not round-trip through storage");
});

test("playTransition never throws and is a no-op when reduced or unsupported", function () {
  var env = h.loadModules();
  var calls = 0;
  var target = { animate: function () { calls++; } };
  env.A.setReduceMotion(true);
  env.E.playTransition(target, { type: "fade", duration: 100 });
  assert(calls === 0, "transition animated in reduced mode");
  env.A.setReduceMotion(false);
  env.E.playTransition(null, { type: "fade", duration: 100 });
  env.E.playTransition(target, { type: "none", duration: 100 });
  assert(calls === 0, "none transition should not animate");
  env.E.playTransition(target, { type: "fade", duration: 100 });
  assert(calls === 1, "supported transition did not animate");
  env.E.playTransition({ animate: function () { throw new Error("stubbed animate failure"); } },
    { type: "fade", duration: 100 });
});

// ── Determinism ───────────────────────────────────────────────────────
test("identical authoring sequences serialize to identical deck JSON", function () {
  function build(env) {
    var api = env.A;
    api.setEffects([], 1);
    api.addEffect("fade", "o1", 1);
    api.addEffect("fly", "o2", 1);
    api.moveEffect(api.effects(1)[1].id, -1, 1);
    api.setTransition({ type: "wipe", duration: 700 }, 1, false);
    api.setReduceMotion(true);
    return JSON.stringify(api.state);
  }
  var first = build(h.loadModules());
  assert(first === build(h.loadModules()), "deck serialization was not reproducible");
  var parsed = JSON.parse(first);
  assert(parsed.slides["1"].effects.length === 2, "serialized timeline lost effects");
  assert(parsed.slides["1"].effects[0].start === "click", "reordered effect was not serialized");
  assert(parsed.reduceMotion === true, "reduce flag missing from serialized deck");
});

// ── STATIC source assertions (DOM-only helpers, never executed) ──────
test("STATIC: motion editor snaps points to guides and clamps to the canvas", function () {
  var source = gb.source("slides/modules/16b_anim_motion.js");
  assert(source.indexOf("var GUIDE = 0.25") >= 0, "guide step missing");
  assert(source.indexOf("Math.round(v / GUIDE) * GUIDE") >= 0, "snap does not use the guide step");
  assert(source.indexOf("Math.max(0, Math.min(1, v))") >= 0, "clamp to 0..1 missing");
});

test("motion editor exposes editable points and snaps keyboard edits to guides", function () {
  var env = h.loadModules();
  var fx = env.A.freshEffect("line", { elementId: "m" });
  var changes = [];
  var wrap = env.M.editor(fx, function (points) {
    changes.push(JSON.parse(JSON.stringify(points)));
  });
  assert(wrap && wrap._points === fx.path, "editor did not expose its working path");
  var dot = env.findNode(wrap, function (node) {
    return node.dataset && node.dataset.point === "control";
  });
  assert(dot && dot._listeners.keydown && dot._listeners.keydown.length === 1,
    "control point has no keyboard handler");
  dot._listeners.keydown[0]({ key: "ArrowRight", shiftKey: true, preventDefault: function () {} });
  assert(fx.path.cx === 0.75, "snapped keyboard step wrong: " + fx.path.cx);
  assert(changes.length === 1 && changes[0].cx === 0.75, "onChange did not report the snapped point");
  dot._listeners.keydown[0]({ key: "ArrowRight", shiftKey: true, preventDefault: function () {} });
  assert(fx.path.cx === 1, "point was not clamped at the canvas edge");
});

test("STATIC: editor catalogs expose all three trigger modes and DOM-bound fields", function () {
  var source = gb.source("slides/modules/16c_anim_editor.js");
  ["slides-anim-start-click", "slides-anim-start-with", "slides-anim-start-after"
  ].forEach(function (key) {
    assert(source.indexOf('"' + key + '"') >= 0, "trigger option missing: " + key);
  });
  ["slides-anim-duration", "slides-anim-delay", "slides-anim-easing", "slides-anim-direction"
  ].forEach(function (key) {
    assert(source.indexOf('"' + key + '"') >= 0, "editor field missing: " + key);
  });
  assert(source.indexOf("M.editor(fx") >= 0, "motion editor is not reachable from the effect editor");
  assert(source.indexOf("E.play([fx])") >= 0, "single-effect preview is not reachable");
});

test("STATIC: timeline and transition panels keep A11Y markers and real API calls", function () {
  var panel = gb.source("slides/modules/16_anim_timeline.js");
  assert(panel.indexOf('role", "dialog"') >= 0, "timeline panel has no dialog role");
  assert(panel.indexOf('role", "status"') >= 0 && panel.indexOf('aria-live", "polite"') >= 0,
    "timeline status region is not announced");
  assert(panel.indexOf("A.moveEffect") >= 0 && panel.indexOf("A.setEffects(reordered)") >= 0,
    "reorder controls are not wired to the real state API");
  var gallery = gb.source("slides/modules/17_anim_transitions.js");
  assert(gallery.indexOf("A.TRANSITIONS") >= 0, "gallery is not driven by the transition catalog");
  assert(gallery.indexOf("E.playTransition") >= 0, "preview is not wired to the real engine");
  assert(gallery.indexOf('role", "status"') >= 0 && gallery.indexOf('aria-live", "polite"') >= 0,
    "transition status region is not announced");
  assert(gallery.indexOf("confirm(") < 0 && gallery.indexOf("prompt(") < 0 && gallery.indexOf("alert(") < 0,
    "panel uses a blocking native dialog");
});

test("STATIC: engine applies inline states with guarded, non-blocking frames", function () {
  var source = gb.source("slides/modules/15_anim_engine.js");
  assert(source.indexOf("applyPatch(element") >= 0, "frames are not applied as inline styles");
  assert(source.indexOf("failedIds.push") >= 0, "effect failures are not recorded");
  assert(source.indexOf("revealAll(currentList)") >= 0, "final states are not guaranteed after a failure");
  assert(source.indexOf("catch (e)") >= 0, "effect application is not wrapped against errors");
});

test("STATIC: shipped animation modules have no CDN, eval or raw-HTML usage", function () {
  h.MODULE_RELS.forEach(function (rel) {
    var source = gb.source(rel);
    assert(!/unpkg|jsdelivr|cdnjs|https:\/\//i.test(source), rel + " references an external URL");
    assert(source.indexOf("eval(") < 0 && source.indexOf("innerHTML") < 0,
      rel + " uses eval or innerHTML");
    var urls = source.match(/http:\/\/[^"'\s]+/g) || [];
    urls.forEach(function (url) {
      assert(url === "http://www.w3.org/2000/svg", rel + " has a non-SVG URL: " + url);
    });
  });
});
