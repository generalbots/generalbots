/**
 * Slides animation/transition timeline — pure-helper invariant tests, part 1 (#2431).
 * Runtime checks exercise the REAL exported helpers in a vm sandbox; private or
 * DOM-bound helpers are reached only through observable exported effects.
 * Part 2 (engine, transitions, STATIC assertions) lives in 11_anim_more.js.
 */
"use strict";

var h = require("./00_harness");
var test = h.test, assert = h.assert;
var gb = h.loadModules();
var A = gb.A, E = gb.E, M = gb.M, Editor = gb.Editor;
var Timeline = gb.Timeline, Transitions = gb.Transitions;

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

// ── API surface ───────────────────────────────────────────────────────
test("all six shipped modules bind their expected namespaces", function () {
  assert(A && typeof A.addEffect === "function", "SlidesAnimation missing");
  assert(E && typeof E.timeline === "function", "SlidesAnimationEngine missing");
  assert(M && typeof M.editor === "function" && M.GUIDE === 0.25, "SlidesAnimationMotion missing");
  assert(Editor && typeof Editor.render === "function", "SlidesAnimationEditor missing");
  assert(Timeline && typeof Timeline.open === "function", "SlidesAnimationTimeline missing");
  assert(Transitions && typeof Transitions.open === "function", "SlidesAnimationTransitions missing");
});

test("exported pure helpers are genuine functions on the real namespaces", function () {
  ["saT", "effectDef", "effectName", "transitionName", "freshEffect", "defaultTransition",
    "load", "effects", "setEffects", "addEffect", "updateEffect", "removeEffect",
    "moveEffect", "transition", "setTransition", "setReduceMotion", "reduced"
  ].forEach(function (name) {
    assert(typeof A[name] === "function", "SlidesAnimation." + name + " missing");
  });
  ["timeline", "totalOf", "applyFrame", "revealAll", "seek", "play", "playTransition"
  ].forEach(function (name) {
    assert(typeof E[name] === "function", "SlidesAnimationEngine." + name + " missing");
  });
});

// ── Catalogs and i18n ─────────────────────────────────────────────────
test("effect catalog covers entrance, emphasis, exit, motion and build", function () {
  assert(A.EFFECTS.length > 0, "effect catalog is empty");
  var cats = {};
  A.EFFECTS.forEach(function (def) {
    cats[def.cat] = true;
    assert(typeof def.id === "string" && def.id.length > 0, "effect id missing");
    assert(typeof def.key === "string" && def.key.indexOf("slides-anim-") === 0,
      "effect label key missing for " + def.id);
    assert(typeof def.fallback === "string" && def.fallback.length > 0,
      "effect fallback missing for " + def.id);
    assert(A.effectDef(def.id).id === def.id, "effectDef failed for " + def.id);
  });
  ["entrance", "emphasis", "exit", "motion", "build"].forEach(function (cat) {
    assert(cats[cat], "missing effect category " + cat);
  });
  assert(A.effectDef("__unknown__") === null, "unknown effect should have no descriptor");
  assert(A.CATEGORIES.length >= 5, "category catalog is too small");
});

test("transition catalog is non-empty and names resolve through gbT", function () {
  assert(A.TRANSITIONS.length > 0, "transition catalog is empty");
  var seen = {};
  A.TRANSITIONS.forEach(function (def) {
    assert(!seen[def.id], "duplicate transition id " + def.id);
    seen[def.id] = true;
    assert(typeof def.key === "string" && def.key.indexOf("slides-trans-") === 0,
      "transition label key missing for " + def.id);
    assert(A.transitionName(def.id) === def.fallback, "transitionName fallback failed");
  });
  var keyed = h.loadModules({ gbT: function (key) { return key; } });
  assert(keyed.A.effectName({ effect: "fade" }) === "slides-anim-fx-fade", "effectName ignored its key");
  assert(keyed.A.transitionName("fade") === "slides-trans-fade", "transitionName ignored its key");
});

test("freshEffect applies deterministic catalog defaults and a stable id", function () {
  var first = A.freshEffect("line", { elementId: "m" });
  var second = A.freshEffect("line", { elementId: "m" });
  var replay = h.loadModules().A.freshEffect("line", { elementId: "m" });
  assert(JSON.stringify(first) === JSON.stringify(replay), "freshEffect id was not reproducible");
  assert(first.id !== second.id, "two effects in one timeline must get distinct ids");
  assert(first.category === "motion" && first.duration === 800, "motion defaults wrong");
  assert(first.start === "click" && first.delay === 0 && first.easing === "ease", "trigger defaults wrong");
  assert(first.path.x2 === 1 && first.path.y2 === 0 && first.path.cx === 0.5, "path defaults wrong");
  var fade = A.freshEffect("fade");
  assert(fade.category === "entrance" && fade.duration === 500 && fade.elementId === null,
    "entrance defaults wrong");
  assert(A.freshEffect("__unknown__").category === "entrance", "unknown effect category wrong");
});

// ── Effect list: add, patch, remove, reorder ──────────────────────────
test("addEffect appends a catalog-backed effect and effects() reflects it", function () {
  var env = h.loadModules();
  var api = env.A;
  api.setEffects([], 1);
  var one = api.addEffect("fade", "o1", 1);
  var two = api.addEffect("fly", "o2", 1);
  var three = api.addEffect("pulse", "o3", 1);
  assert(api.effects(1).length === 3, "addEffect did not append");
  assert(api.effects(1).map(function (fx) { return fx.id; }).join(",") ===
    [one.id, two.id, three.id].join(","), "append order changed");
  assert(api.effects(1)[1].category === "entrance" && api.effects(1)[2].category === "emphasis",
    "categories were not derived from the catalog");
});

test("updateEffect patches only the target and reports misses honestly", function () {
  var env = h.loadModules();
  var api = env.A;
  var first = api.addEffect("fade", "o1", 1);
  api.addEffect("fly", "o2", 1);
  var patched = api.updateEffect(first.id, { duration: 900, start: "after" }, 1);
  assert(patched && patched.duration === 900 && patched.start === "after", "patch not applied");
  assert(api.effects(1)[1].duration === 500, "patch leaked to a sibling effect");
  assert(api.updateEffect("__missing__", { duration: 1 }, 1) === null, "missing id reported as patched");
});

test("removeEffect deletes only its target and moveEffect clamps to list bounds", function () {
  var env = h.loadModules();
  var api = env.A;
  var one = api.addEffect("fade", "o1", 1);
  var two = api.addEffect("fly", "o2", 1);
  var three = api.addEffect("pulse", "o3", 1);
  assert(api.moveEffect(two.id, -1, 1) === true, "move up failed");
  assert(api.effects(1)[0].id === two.id, "move up did not reorder");
  assert(api.moveEffect(two.id, -5, 1) === false, "no-op move reported success");
  assert(api.moveEffect(two.id, 99, 1) === true, "clamped move down failed");
  assert(api.effects(1)[2].id === two.id, "clamped move landed in the wrong slot");
  assert(api.moveEffect("__missing__", 1, 1) === false, "missing id reported as moved");
  api.removeEffect(two.id, 1);
  assert(api.effects(1).length === 2, "removeEffect did not remove one row");
  assert(api.effects(1).every(function (fx) { return fx.id !== two.id; }), "removed row survived");
});

test("setEffects replaces the list and rejects non-arrays", function () {
  var env = h.loadModules();
  var api = env.A;
  api.addEffect("fade", "o1", 1);
  api.setEffects("not-an-array", 1);
  assert(api.effects(1).length === 0, "non-array was accepted as a timeline");
  api.setEffects([{ id: "kept" }], 1);
  assert(api.effects(1).length === 1 && api.effects(1)[0].id === "kept", "replacement failed");
});

// ── Trigger resolution (the pure timeline) ────────────────────────────
test("on-click effects open a group at their own start plus delay", function () {
  var list = [
    effect("a", "a", { duration: 300, delay: 50, start: "click" }),
    effect("b", "b", { duration: 200, delay: 10, start: "click" })
  ];
  var items = E.timeline(list);
  assert(items[0].start === 50 && items[0].end === 350, "first click group resolved wrong");
  assert(items[1].start === 360 && items[1].end === 560, "second click group did not follow the first");
  assert(E.totalOf(list) === 560, "totalOf is not the maximum end");
  assert(E.totalOf([]) === 0, "empty timeline total must be zero");
});

test("with-previous shares the previous start and after-previous follows its end", function () {
  var list = [
    effect("a", "a", { duration: 200, delay: 10, start: "click" }),
    effect("b", "b", { duration: 100, delay: 20, start: "with" }),
    effect("c", "c", { duration: 150, delay: 30, start: "after" })
  ];
  var items = E.timeline(list);
  assert(items[0].start === 10 && items[1].start === 30 && items[2].start === 160,
    "trigger starts wrong: " + items.map(function (it) { return it.start; }).join(","));
  assert(items[0].end === 210 && items[1].end === 130 && items[2].end === 310,
    "trigger ends wrong: " + items.map(function (it) { return it.end; }).join(","));
  assert(E.totalOf(list) === 310, "totalOf ignored the after-previous tail");
});

test("timeline clamps negative values and is deterministic for identical input", function () {
  var list = [effect("a", "a", { duration: -5, delay: -3 })];
  var item = E.timeline(list)[0];
  assert(item.start === 0 && item.duration === 0 && item.end === 0, "negative values not clamped");
  assert(JSON.stringify(E.timeline(list)) === JSON.stringify(E.timeline(list)),
    "timeline differed between calls");
  assert(E.timeline([{}])[0].start === 0, "malformed effect should resolve to zero");
});

