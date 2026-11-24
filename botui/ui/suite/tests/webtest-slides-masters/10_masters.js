/**
 * Slides masters / layouts / theme tokens / brand kit — pure-helper invariants (#2429).
 * Runtime: real shipped helpers in a VM sandbox. DOM-bound dialog/panel helpers
 * are covered by labelled STATIC source checks at the bottom (never executed).
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var gb = h.loadModules();
var M = gb.masters;

function has(obj, key) { return !!obj && Object.prototype.hasOwnProperty.call(obj, key); }
function freshState() { return gb.resetState(); }
function track(state, idx, opts) {
  var s = { themeId: null, layoutId: null, overrides: {} };
  Object.keys(opts || {}).forEach(function (key) { s[key] = opts[key]; });
  state.slides[String(idx)] = s;
  return s;
}
function addAltTheme(state, accent) {
  var flat = M.defaultFlat();
  flat["color.accent1"] = accent;
  state.themes.theme_alt = { id: "theme_alt", name: "T2", flat: flat };
  return state.themes.theme_alt;
}
function hasChange(rows, key) {
  return rows.some(function (row) {
    return row.changes.some(function (c) { return c.key === key; });
  });
}
// ── API surface ────────────────────────────────────────────────────────
test("all five shipped modules bind one namespace and expose their helpers", function () {
  assert(M && typeof M.tr === "function", "master namespace missing");
  ["fresh", "load", "init", "save", "clone", "deckId", "storeKey", "defaultFlat",
    "mutate", "setMasterToken", "undo", "redo", "canUndo", "canRedo", "theme",
    "layout", "slide", "slideKey"
  ].forEach(function (name) { assert(typeof M[name] === "function", "M." + name + " missing"); });
  ["themeFor", "layoutFor", "resolve", "effectiveStyle", "setOverride", "clearOverride",
    "diffApply", "applyTheme", "applyLayout", "addLayout", "duplicateLayout",
    "renameLayout", "deleteLayout", "setLayoutField", "setPlaceholder", "fontAllowed",
    "colorAllowed", "checkInsert", "saveBrand", "saveTemplate", "deleteTemplate",
    "restoreTemplate"
  ].forEach(function (name) { assert(typeof M[name] === "function", "M." + name + " missing"); });
  ["el", "labelled", "button", "textField", "selectField", "status", "openDialog"
  ].forEach(function (name) { assert(typeof M[name] === "function", "M." + name + " missing"); });
  ["tokenLabelKey", "sourceLabel", "buildThemePanel", "buildInheritPanel", "buildApplyPanel"
  ].forEach(function (name) { assert(typeof M[name] === "function", "M." + name + " missing"); });
  ["buildLayoutPanel", "buildBrandPanel", "buildTemplatesPanel"
  ].forEach(function (name) { assert(typeof M[name] === "function", "M." + name + " missing"); });
});

test("fresh master state is deterministic and has one complete default theme", function () {
  var a = M.fresh();
  var b = M.fresh();
  assert(JSON.stringify(a) === JSON.stringify(b), "fresh() is not deterministic");
  assert(a.version === 1 && a.master && typeof a.master.name === "string", "master record malformed");
  assert(Object.keys(a.themes).length === 1, "expected exactly one default theme");
  assert(a.layouts.length === 3 && Object.keys(a.slides).length === 0, "wrong layout/slide counts");
  var flat = a.themes[a.master.defaultThemeId].flat;
  M.TOKEN_KEYS.forEach(function (key) { assert(has(flat, key), "token missing: " + key); });
  assert(a.brand.locked === false && a.brand.mode === "warn", "brand defaults wrong");
  assert(Array.isArray(a.templates) && a.templates.length === 0, "template library not empty");
});
// ── Master create / update / delete (single local master record) ───────
test("setMasterToken updates the theme and reports inheriting slides", function () {
  var s = freshState();
  var theme = M.theme("theme_default");
  var before = theme.flat["color.accent1"];
  var affected = M.setMasterToken("theme_default", "color.accent1", "#abcabc");
  assert(affected.length === 0, "unexpected affected slides on an empty deck");
  assert(theme.flat["color.accent1"] === "#abcabc", "master token was not written");
  assert(before !== theme.flat["color.accent1"], "token value did not change");
  assert(M.canUndo() === true, "mutation did not checkpoint");
  track(s, 0, {});
  assert(JSON.stringify(M.setMasterToken("theme_default", "font.minor", "M")) === '["0"]',
    "inheriting slide not reported");
});

test("an unknown theme token write is a no-op with an empty affected list", function () {
  var s = freshState();
  var snap = JSON.stringify(s);
  assert(M.setMasterToken("__missing__", "color.accent1", "#000000").length === 0,
    "unknown theme reported affected slides");
  assert(JSON.stringify(s) === snap, "unknown theme mutated state");
  assert(M.setMasterToken("theme_default", "__missing__", "x").length === 0,
    "an unset token should not report affected slides");
});

test("layout create, duplicate, rename and field edit are deterministic", function () {
  var s = freshState();
  var created = M.addLayout("Appendix");
  assert(created.id === "layout_1" && s.layouts.length === 4, "create ids/counts wrong");
  var source = M.layout("layout_title");
  var copy = M.duplicateLayout("layout_title");
  assert(copy.id === "layout_2" && copy.name !== source.name, "duplicate metadata wrong");
  assert(JSON.stringify(copy.placeholders) === JSON.stringify(source.placeholders),
    "duplicate lost placeholder geometry");
  assert(copy.placeholders !== source.placeholders, "duplicate reused the source geometry object");
  M.renameLayout(copy.id, "Keynote");
  assert(M.layout(copy.id).name === "Keynote" && source.name !== "Keynote",
    "rename leaked to the source layout");
  M.setLayoutField(copy.id, "background", "#010203");
  assert(M.layout(copy.id).background === "#010203", "layout field edit failed");
  assert(M.deleteLayout(copy.id) === true && s.layouts.length === 4, "delete failed");
});

test("layout deletion guards the last item and clears slide references", function () {
  var s = freshState();
  track(s, 0, { layoutId: "layout_body" });
  assert(M.deleteLayout("layout_body") === true, "referenced layout was not deleted");
  assert(s.slides["0"].layoutId === null, "deleted layout reference was not cleared");
  s.layouts.slice(0, -1).forEach(function (l) {
    assert(M.deleteLayout(l.id) === true, "layout deletion failed");
  });
  assert(s.layouts.length === 1, "expected one remaining layout");
  assert(M.deleteLayout(s.layouts[0].id) === false, "last layout was deletable");
});

test("setPlaceholder stores a detached geometry on one layout only", function () {
  var s = freshState();
  var geo = { x: 7, y: 8, w: 9, h: 10 };
  M.setPlaceholder("layout_title", "title", geo);
  geo.x = 999;
  assert(M.layout("layout_title").placeholders.title.x === 7, "geometry was not cloned");
  assert(!has(M.layout("layout_body").placeholders, "custom"), "unexpected placeholder leaked");
  M.setPlaceholder("layout_body", "custom", { x: 1, y: 2, w: 3, h: 4 });
  assert(has(M.layout("layout_body").placeholders, "custom"), "new placeholder missing");
  assert(!has(M.layout("layout_title").placeholders, "custom"), "placeholder leaked across layouts");
});
// ── Placeholder roles and per-layout resolution ───────────────────────
test("all nine placeholder roles exist and bind to real theme tokens", function () {
  assert(M.ROLES.length === 9, "expected nine placeholder roles");
  M.ROLES.forEach(function (role) {
    assert(has(M.ROLE_BINDINGS, role), "role has no token binding: " + role);
    var binding = M.ROLE_BINDINGS[role];
    Object.keys(binding).forEach(function (prop) {
      assert(M.TOKEN_KEYS.indexOf(binding[prop]) >= 0,
        "role " + role + " binds an unknown token: " + binding[prop]);
    });
  });
  assert(M.EFFECTS.length > 0, "no effect styles declared");
});

test("fresh layouts carry role-specific geometry and no unknown roles", function () {
  var s = freshState();
  var byId = {};
  s.layouts.forEach(function (l) { byId[l.id] = l; });
  assert(has(byId.layout_title.placeholders, "title") &&
    has(byId.layout_title.placeholders, "subtitle"), "title layout roles missing");
  assert(!has(byId.layout_title.placeholders, "body"), "title layout has a body role");
  assert(has(byId.layout_body.placeholders, "body") &&
    !has(byId.layout_body.placeholders, "subtitle"), "body layout roles wrong");
  s.layouts.forEach(function (l) {
    Object.keys(l.placeholders).forEach(function (role) {
      assert(M.ROLES.indexOf(role) >= 0, "unknown role in " + l.id + ": " + role);
      var g = l.placeholders[role];
      ["x", "y", "w", "h"].forEach(function (axis) {
        assert(typeof g[axis] === "number" && isFinite(g[axis]), "non-finite geometry");
      });
      assert(g.w > 0 && g.h > 0, "empty placeholder geometry");
    });
  });
});

test("layout theme overrides resolve per layout, below slide overrides", function () {
  var s = freshState();
  M.setLayoutField("layout_title", "themeOverrides", { "color.accent2": "#123456" });
  track(s, 0, { layoutId: "layout_title" });
  track(s, 1, { layoutId: "layout_body" });
  var inherited = M.resolve(s, "0", "color.accent2");
  assert(inherited.value === "#123456" && inherited.source === "layout" &&
    inherited.explicit === false, "layout value did not inherit");
  assert(M.resolve(s, "1", "color.accent2").source === "master", "layout value leaked");
  M.setOverride("0", "color.accent2", "#654321");
  var over = M.resolve(s, "0", "color.accent2");
  assert(over.value === "#654321" && over.source === "slide" && over.explicit === true,
    "slide override did not win");
  M.clearOverride("0", "color.accent2");
  assert(M.resolve(s, "0", "color.accent2").source === "layout", "clear did not fall back");
});
// ── Cascade and deterministic merging ─────────────────────────────────
test("resolve follows slide > layout > master > none and reports the source", function () {
  var s = freshState();
  addAltTheme(s, "#111111");
  M.setLayoutField("layout_title", "themeOverrides", { "color.accent3": "#aaaaaa" });
  track(s, 0, { layoutId: "layout_title", themeId: "theme_alt", overrides: {
    "color.accent3": "#bbbbbb", "color.accent1": "#cccccc"
  } });
  assert(M.resolve(s, "0", "color.accent3").source === "slide", "override tier wrong");
  assert(M.resolve(s, "0", "color.accent1").source === "slide", "override before theme wrong");
  assert(M.resolve(s, "0", "color.accent4").source === "master", "theme tier wrong");
  var missing = M.resolve(s, "0", "__missing__");
  assert(missing.value === null && missing.source === "none" && missing.explicit === false,
    "missing token did not fall through");
  track(s, 1, { layoutId: "layout_body", themeId: "theme_alt" });
  assert(M.resolve(s, "1", "color.accent3").value === M.defaultFlat()["color.accent3"],
    "foreign theme was not used");
  assert(M.themeFor(s, { themeId: "__missing__" }) === null, "bad theme id should resolve to null");
  assert(M.layoutFor(s, { layoutId: "__missing__" }) === null, "bad layout id should resolve to null");
  assert(M.themeFor(s, null) === s.themes[s.master.defaultThemeId], "default theme fallback broken");
});

test("effectiveStyle covers every token with value, source and explicit flags", function () {
  var s = freshState();
  track(s, 0, {});
  var eff = M.effectiveStyle(s, "0");
  assert(Object.keys(eff).length === M.TOKEN_KEYS.length, "wrong effective token count");
  M.TOKEN_KEYS.forEach(function (key) {
    assert(has(eff, key) && has(eff[key], "value") && has(eff[key], "source") &&
      has(eff[key], "explicit"), "malformed effective entry for " + key);
    assert(eff[key].source === "master" && eff[key].explicit === false,
      "unexpected source for " + key);
  });
  assert(JSON.stringify(eff) === JSON.stringify(M.effectiveStyle(s, "0")),
    "effectiveStyle is not deterministic");
});

test("a master token change on a 40-slide deck touches only inheriting slides", function () {
  var s = freshState();
  addAltTheme(s, "#111111");
  for (var i = 0; i < 35; i++) track(s, i, {});
  for (var j = 35; j < 39; j++) {
    track(s, j, { overrides: { "color.accent1": "#eeeeee" } });
  }
  track(s, 39, { themeId: "theme_alt" });
  assert(Object.keys(s.slides).length === 40, "deck is not 40 slides");
  var affected = M.setMasterToken("theme_default", "color.accent1", "#222222");
  assert(affected.length === 35, "expected 35 inheriting slides, got " + affected.length);
  assert(affected[0] === "0" && affected[34] === "34", "affected slide keys wrong");
  assert(M.resolve(s, "0", "color.accent1").value === "#222222", "inheriting slide not recoloured");
  for (var k = 35; k < 39; k++) {
    assert(M.resolve(s, String(k), "color.accent1").value === "#eeeeee", "override was overwritten");
  }
  assert(M.resolve(s, "39", "color.accent1").value === "#111111", "foreign theme was recoloured");
});

test("clone produces detached state and tr substitutes both parameter forms", function () {
  var s = freshState();
  var copy = M.clone(s);
  copy.layouts[0].name = "changed";
  assert(s.layouts[0].name !== "changed", "clone shared a live layout object");
  assert(M.tr("k", "A {n} B { $n }", { n: 7 }) === "A 7 B 7", "parameter substitution failed");
});
// ── Apply-to-selection/all with before/after diff ─────────────────────
test("diffApply is pure and restricts the preview to selected slides", function () {
  var s = freshState();
  addAltTheme(s, "#111111");
  track(s, 0, {});
  track(s, 1, {});
  track(s, 2, { overrides: { "color.accent1": "#eeeeee" } });
  var snap = JSON.stringify(s);
  var rows = M.diffApply(s, { slideKeys: ["1"], themeId: "theme_alt" });
  assert(JSON.stringify(s) === snap, "diffApply mutated the state it inspected");
  assert(rows.length === 1 && rows[0].slideKey === "1", "selection was not respected");
  assert(hasChange(rows, "color.accent1"), "token change missing from the diff");
  assert(M.diffApply(s, { slideKeys: ["2"], themeId: "theme_alt" }).length === 0,
    "an override slide should not change on a theme token");
});

test("applyTheme on a selection changes only that slide and preserves overrides", function () {
  var s = freshState();
  addAltTheme(s, "#111111");
  track(s, 0, {});
  track(s, 1, { overrides: { "color.accent1": "#eeeeee" } });
  var rows = M.applyTheme({ slideKeys: ["0"], themeId: "theme_alt" });
  assert(rows.length === 1 && rows[0].slideKey === "0", "wrong affected slide count");
  assert(s.slides["0"].themeId === "theme_alt", "selected slide not switched");
  assert(s.slides["1"].themeId === null, "unselected slide was switched");
  assert(M.resolve(s, "0", "color.accent1").value === "#111111", "new theme not applied");
  assert(M.resolve(s, "1", "color.accent1").value === "#eeeeee", "override was lost");
});

test("[UAT:SLIDE-03] applyTheme on all slides preserves every explicit override", function () {
  var s = freshState();
  addAltTheme(s, "#111111");
  for (var i = 0; i < 6; i++) {
    track(s, i, i % 2 ? { overrides: { "color.accent1": "#eeeeee" } } : {});
  }
  M.applyTheme({ themeId: "theme_alt" });
  for (var j = 0; j < 6; j++) {
    assert(s.slides[String(j)].themeId === "theme_alt", "slide " + j + " not switched");
    assert(M.resolve(s, String(j), "color.accent1").value ===
      (j % 2 ? "#eeeeee" : "#111111"), "slide " + j + " merge wrong");
  }
});

test("applyLayout previews and applies a layout without dropping overrides", function () {
  var s = freshState();
  M.setLayoutField("layout_title", "themeOverrides", { "color.accent4": "#222222" });
  track(s, 0, {});
  track(s, 1, { overrides: { "color.accent4": "#333333" } });
  var rows = M.diffApply(s, { slideKeys: ["0", "1"], layoutId: "layout_title" });
  assert(rows.length === 2, "both layout changes should be previewed");
  assert(hasChange(rows, "layout") && hasChange(rows, "color.accent4"), "diff rows incomplete");
  M.applyLayout({ slideKeys: ["0"], layoutId: "layout_title" });
  assert(s.slides["0"].layoutId === "layout_title", "layout not applied");
  assert(s.slides["1"].layoutId === null, "unselected layout changed");
  assert(M.resolve(s, "1", "color.accent4").value === "#333333", "override was dropped");
  assert(M.resolve(s, "0", "color.accent4").value === "#222222", "layout token not inherited");
});
// ── Brand kit ──────────────────────────────────────────────────────────
test("font and colour checks are case-insensitive and allow empty values", function () {
  var s = freshState();
  var brand = s.brand;
  assert(M.fontAllowed(brand, "") === true && M.colorAllowed(brand, "") === true,
    "empty insertion should be allowed");
  assert(M.fontAllowed(brand, brand.fonts[0].toUpperCase()) === true, "case-insensitive font failed");
  assert(M.colorAllowed(brand, brand.palette[0].toUpperCase()) === true, "case-insensitive colour failed");
  assert(M.fontAllowed(brand, "__offbrand__") === false, "off-brand font was allowed");
  assert(M.colorAllowed(brand, "#010203") === false, "off-brand colour was allowed");
});

test("checkInsert honours warn and block modes and reports both reasons", function () {
  var s = freshState();
  var off = { font: "__offbrand__", color: "#010203" };
  var unlocked = M.checkInsert(s, off);
  assert(unlocked.allowed === true && unlocked.mode === "none" && unlocked.reasons.length === 0,
    "unlocked brand should not inspect insertions");
  s.brand.locked = true;
  s.brand.mode = "warn";
  var warn = M.checkInsert(s, off);
  assert(warn.allowed === true && warn.mode === "warn" && warn.reasons.length === 2,
    "warn mode result wrong");
  s.brand.mode = "block";
  var block = M.checkInsert(s, off);
  assert(block.allowed === false && block.mode === "block" && block.reasons.length === 2,
    "block mode result wrong");
  assert(block.reasons[0].kind === "font" && block.reasons[1].kind === "color",
    "reason kinds not reported by category");
  assert(M.checkInsert(s, { font: s.brand.fonts[0], color: s.brand.palette[0] }).allowed === true,
    "on-brand insertion was blocked");
});

test("brand kit and template snapshot round-trip restore the lock rules", function () {
  var s = freshState();
  var brand = M.clone(s.brand);
  brand.locked = true;
  brand.mode = "block";
  brand.palette = ["#010203"];
  M.saveBrand(brand);
  assert(JSON.stringify(s.brand) === JSON.stringify(brand) && s.brand !== brand,
    "saveBrand did not store a detached copy");
  var entry = M.saveTemplate(" kit ");
  assert(typeof entry.versionId === "string" && entry.versionId.length > 0, "no version id issued");
  assert(has(entry.snapshot, "brand") && has(entry.snapshot, "layouts"), "snapshot incomplete");
  var saved = M.clone(s.brand);
  var changed = M.clone(saved);
  changed.locked = false;
  changed.palette = [];
  M.saveBrand(changed);
  assert(M.checkInsert(M.init(), { color: "#010203" }).allowed === true, "unlocked state wrong");
  assert(M.restoreTemplate(entry.versionId) === true, "template was not restored");
  assert(JSON.stringify(s.brand) === JSON.stringify(saved), "brand kit did not round-trip");
  assert(M.checkInsert(M.init(), { color: "#ffffff" }).allowed === false,
    "restored brand rules were not enforced");
  assert(M.restoreTemplate("__missing__") === false, "unknown version reported as restored");
});

test("template snapshots explicitly exclude slide overrides", function () {
  var s = freshState();
  track(s, 0, {});
  var entry = M.saveTemplate("snap");
  assert(!has(entry.snapshot, "slides"), "snapshot unexpectedly includes slides");
  M.setOverride("0", "color.accent1", "#aaaaaa");
  M.restoreTemplate(entry.versionId);
  assert(has(s.slides["0"].overrides, "color.accent1"), "restore overwrote a slide override");
  assert(M.deleteTemplate(entry.versionId) === undefined && s.templates.length === 0,
    "template deletion failed");
});
// ── History / persistence ──────────────────────────────────────────────
test("[UAT:SLIDE-03] undo and redo restore master state and report availability honestly", function () {
  var s = freshState();
  var theme = M.theme("theme_default");
  var before = theme.flat["color.accent1"];
  M.setMasterToken("theme_default", "color.accent1", "#010203");
  assert(M.canUndo() === true && M.canRedo() === false, "history flags wrong after mutation");
  // undo/redo replace M.state with a clone, so re-read through the accessor.
  assert(M.undo() === true && M.theme("theme_default").flat["color.accent1"] === before,
    "undo did not restore");
  assert(M.canRedo() === true && M.redo() === true &&
    M.theme("theme_default").flat["color.accent1"] === "#010203", "redo did not reapply");
  gb.resetState();
  assert(M.undo() === false && M.redo() === false, "empty history reported as available");
});

test("mutations persist to the deck-scoped localStorage stub", function () {
  var s = freshState();
  assert(M.storeKey().indexOf("masters-offline-deck") >= 0, "deck scope missing from store key");
  M.setMasterToken("theme_default", "color.accent1", "#0a0b0c");
  var raw = gb.storage.getItem(M.storeKey());
  assert(!!raw, "nothing was persisted");
  assert(JSON.parse(raw).themes.theme_default.flat["color.accent1"] === "#0a0b0c",
    "persisted state is stale");
});
// ── STATIC source assertions (DOM-only helpers, never executed) ────────
test("STATIC: dialog shell declares a11y roles and uses no native prompts", function () {
  var src = gb.source("slides/modules/62_masters_dialog.js");
  ['"role", "dialog"', '"aria-modal"', '"aria-labelledby"', '"role", "status"',
    '"aria-live", "polite"', '"Escape"', '"Tab"', "previous.focus"
  ].forEach(function (needle) {
    assert(src.indexOf(needle) >= 0, "dialog a11y marker missing: " + needle);
  });
  assert(!/\b(alert|confirm|prompt)\s*\(/.test(src), "dialog uses a native prompt");
});

test("STATIC: panel modules resolve labels through gbT and data-i18n", function () {
  ["slides/modules/63_masters_panels_a.js", "slides/modules/64_masters_panels_b.js"
  ].forEach(function (rel) {
    var src = gb.source(rel);
    assert(src.indexOf("M.section(") >= 0 && src.indexOf("M.labelled(") >= 0,
      rel + " does not use the labelled helpers");
    assert(src.indexOf("data-i18n") >= 0 && src.indexOf("data-i18n-aria-label") >= 0,
      rel + " misses data-i18n attributes");
    assert(/\bt\(/.test(src), rel + " does not call gbT through t()");
  });
});

test("STATIC: no CDN, eval, network or raw innerHTML writes in shipped modules", function () {
  h.MODULE_RELS.forEach(function (rel) {
    var src = gb.source(rel);
    assert(!/unpkg|jsdelivr|cdnjs|https:\/\//i.test(src), rel + " references a CDN URL");
    assert(src.indexOf("eval(") < 0 && src.indexOf("fetch(") < 0 &&
      src.indexOf("XMLHttpRequest") < 0, rel + " uses eval or a network call");
    (src.match(/innerHTML\s*=\s*[^;]+;/g) || []).forEach(function (assignment) {
      assert(/innerHTML\s*=\s*""\s*;/.test(assignment),
        rel + " writes non-empty innerHTML: " + assignment.trim());
    });
  });
});

test("STATIC: cascade helper module is DOM-free and mutates through checkpoints", function () {
  var src = gb.source("slides/modules/61_masters_cascade.js");
  assert(src.indexOf("document") < 0 && src.indexOf("localStorage") < 0,
    "cascade helpers perform DOM or storage work");
  assert((src.match(/M\.mutate\(/g) || []).length >= 5,
    "mutating helpers do not route through M.mutate");
  assert(src.indexOf("M.clone(state)") >= 0, "diff preview does not work on a cloned state");
});
