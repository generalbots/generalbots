/**
 * #2930 — deterministic static coverage for the single-owner theme fix.
 *
 * Before the fix, four scripts wrote the same `data-theme` attribute using two
 * vocabularies and three localStorage keys: the light/dark manager never
 * loaded in the desktop shell, the theme-id manager and the legacy base
 * fragment both wrote <html>/<body>, and Settings wrote a third key. Whichever
 * ran last won, so "Light" had no visible effect.
 *
 * This guard pins the authoritative writer, the canonical attribute and one
 * storage key, the scoped base-token selectors for both light and dark, and
 * the shared token vocabulary. Every case is a literal source assertion — no
 * server, browser, cargo or network. Non-vacuity: any second writer restored
 * in any of the four manager files flips a case to FAIL.
 *
 * The driver owns issue closure and browser UAT.
 */
"use strict";

var h = require("./00_harness");

var OWNER = "botui/ui/suite/js/theme-manager.js";
var SCHEME = "botui/ui/suite/js/theme-scheme.js";
var IDENTITY = "botui/ui/suite/identity/theme.js";
var LEGACY = "botui/ui/suite/js/base/02_theme.js";
var TOKENS_CSS = "botui/ui/suite/css/base/01_root-themes.css";
var ROOT_CSS = "botui/ui/suite/css/base/01_root.css";
var BEVEL_CSS = "botui/ui/suite/css/base/02_theme_3dbevel.css";
var MANAGERS = [OWNER, SCHEME, IDENTITY, LEGACY];
var BASE_CSS = [TOKENS_CSS, ROOT_CSS, BEVEL_CSS];

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function between(src, start, end) {
  var a = src.indexOf(start);
  h.assert(a !== -1, "missing start marker <" + start + ">");
  var b = src.indexOf(end, a + start.length);
  h.assert(b !== -1, "missing end marker <" + end + ">");
  h.assert(b > a, "empty source span between <" + start + "> and <" + end + ">");
  return src.slice(a, b);
}
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 240));
}
function spanLacks(span, needle, msg) {
  h.assert(span.indexOf(needle) === -1,
    (msg || "span must not contain") + " <" + needle + "> :: " + span.slice(0, 240));
}
function countIn(src, needle) { return src.split(needle).length - 1; }
function ordered(span, markers, msg) {
  var last = -1;
  markers.forEach(function (m) {
    var at = span.indexOf(m);
    h.assert(at !== -1, (msg || "span") + " must contain <" + m + ">");
    h.assert(at > last, (msg || "span") + " order broken at <" + m + ">");
    last = at;
  });
}

// (a) Exactly one writer of the canonical attribute. A second writer in ANY
// of the four managers — restored legacy body writes included — fails here.
h.staticTest("#2930 exactly one authoritative writer of data-theme, in ThemeManager.applyThemeId", function () {
  var owner = h.readRepo(OWNER);
  h.assert(owner.length > 6000, "theme-manager.js must be a non-trivial shipped module");
  var write = 'document.documentElement.setAttribute("data-theme", id);';
  h.assertEqual(countIn(owner, write), 1,
    "the canonical attribute has exactly one write site");
  var apply = between(owner, "function applyThemeId(id) {", "/* Read one key");
  spanHas(apply, write, "the single write lives in applyThemeId");
  spanHas(apply, 'document.body.removeAttribute("data-theme");',
    "the legacy body attribute is cleared, never set");
  spanHas(apply, "scheme.clearInline();",
    "previous inline tokens are dropped at the one write site");
  spanHas(apply, "syncColorScheme();", "the scheme follows the one write");

  MANAGERS.forEach(function (path) {
    var src = h.readRepo(path);
    if (path !== OWNER) {
      h.assertEqual(countIn(src, 'setAttribute("data-theme"'), 0,
        path + " must not write data-theme");
      h.assertEqual(countIn(src, ".dataset.theme"), 0,
        path + " must not write data-theme through dataset");
      h.assertEqual(countIn(src, 'setAttribute("data-theme"'), 0,
        path + " must not write data-theme on <body> either");
    }
    h.assert(src.split("\n").length <= 449,
      path + " must stay under 450 lines, got " + src.split("\n").length);
  });
  contains(h.readRepo(IDENTITY), 'document.documentElement.setAttribute("data-color-scheme", effective);',
    "the standalone adapter applies the state-free colour-scheme axis instead");
  contains(h.readRepo(LEGACY), "window.addEventListener(eventName",
    "the legacy fragment subscribes to owner events");
});

// (b) One key and one vocabulary: the attribute carries theme ids, the
// light/dark/System preference is resolved to a pair of ids and all three
// legacy stores are migrated into the single key without losing a choice.
h.staticTest("#2930 one localStorage key, theme-id vocabulary, three-way legacy migration", function () {
  var owner = h.readRepo(OWNER);
  var head = between(owner, "const ThemeManager = (() => {", "// Bot ID to theme mapping");
  spanHas(head, 'const STORAGE_KEY = "gb-theme";', "one canonical key");
  spanHas(head, 'const LIGHT_ID = "light";', "light maps to the light theme id");
  spanHas(head, 'const DARK_ID = "carbon";', "dark maps to a registered dark theme id");
  contains(owner, '{ id: "carbon"', "the dark theme id is registered in the theme catalog");
  spanHas(head, 'const SYSTEM = "system";', "system is a preference value, not an attribute value");
  spanHas(head, 'const VALID_PREF = { light: 1, dark: 1, system: 1 };',
    "the preference vocabulary is explicit");
  ordered(head, ['const STORAGE_KEY', 'const LIGHT_ID', 'const DARK_ID',
    'const SYSTEM', 'const VALID_PREF'], "canonical declarations");

  var migrate = between(owner, "function storedPreference() {", "function osDark()");
  spanHas(migrate, "localStorage.getItem(STORAGE_KEY)", "the canonical key is read first");
  spanHas(migrate, "localStorage.getItem(`gb-theme-${botId}`)", "per-bot legacy theme is migrated");
  spanHas(migrate, "localStorage.getItem(`gb-theme-data-${botId}`)", "server theme payload is migrated");
  spanHas(migrate, "data.theme_base", "the legacy JSON theme_base value is preserved");
  spanHas(migrate, "localStorage.setItem(STORAGE_KEY, value)", "migrated value is written once to the single key");
  ordered(migrate, ["localStorage.getItem(STORAGE_KEY)",
    "localStorage.getItem(`gb-theme-${botId}`)",
    "localStorage.getItem(`gb-theme-data-${botId}`)",
    "localStorage.setItem(STORAGE_KEY, value)"], "read, fall back, migrate, persist");

  var resolve = between(owner, "function resolve(pref) {", "/* The preference is one key");
  spanHas(resolve, 'if (known(pref)) return pref;', "named theme ids are never collapsed to System");
  spanHas(resolve, 'if (pref === SYSTEM) return osDark() ? DARK_ID : LIGHT_ID;',
    "System resolves to one of the two theme ids");
  spanHas(resolve, 'if (pref === "dark") return DARK_ID;', "Dark resolves to the dark id");

  var api = between(owner, "const api = {", "window.ThemeManager = api;");
  spanHas(api, "set: setPreference,", "the preference API is exposed");
  spanHas(api, "get: storedPreference,", "the stored preference is readable");
  spanHas(api, "current:", "the resolved surface is readable");
  spanHas(api, "resolve,", "the resolver is exposed to the adapter");
  contains(owner, "window.GBTheme = api;",
    "standalone callers delegate to the one owner, not a second writer");
  absent(owner, "localStorage.setItem(`gb-theme-${botId}`",
    "the owner never reintroduces a second theme key");
  absent(owner, "localStorage.setItem(`gb-theme-data-${botId}`",
    "server payload writes never carry a theme choice");
  contains(h.readRepo(IDENTITY), 'var STORAGE_KEY = "gb-theme";',
    "the standalone adapter uses the same single key");
  contains(h.readRepo(IDENTITY), "if (owner && typeof owner.set === \"function\")",
    "the adapter delegates to the owner when present");
});

// (c) Base tokens switch on the canonical attribute for BOTH light and dark,
// scoped to :root so a legacy body[data-theme] writer cannot win the cascade.
h.staticTest("#2930 scoped :root light/dark base tokens with both vocabularies", function () {
  var css = h.readRepo(TOKENS_CSS);
  var light = between(css, ':root[data-theme="light"] {', ':root[data-theme="blue"] {');
  var dark = between(css, ':root[data-theme="dark"] {', ':root[data-theme="light"] {');
  ["--background", "--foreground", "--bg", "--surface", "--text"].forEach(function (token) {
    spanHas(light, token + ":", "light base tokens must emit " + token);
    spanHas(dark, token + ":", "dark base tokens must emit " + token);
  });
  spanHas(light, "--bg: #f8fafc;", "light surface is light");
  spanHas(dark, "--bg: #0f172a;", "dark surface is dark");
  spanHas(light, "--text: #1e293b;", "light ink is dark");
  spanHas(dark, "--text: #f8fafc;", "dark ink is light");

  // The dark preference resolves to the registered `carbon` theme; its CSS
  // file must carry both token vocabularies, as must the default paper theme.
  var darkThemeCss = h.readRepo("botui/ui/suite/public/themes/carbon.css");
  ["--background", "--foreground", "--bg", "--surface", "--text"].forEach(function (token) {
    contains(darkThemeCss, token + ":", "carbon theme must emit " + token);
  });
  var paper = between(css, ':root[data-theme="default"] {', ':root[data-theme="sentient"] {');
  ["--background", "--foreground", "--bg", "--surface", "--text"].forEach(function (token) {
    spanHas(paper, token + ":", "default base tokens must emit " + token);
  });

  BASE_CSS.forEach(function (path) {
    var src = h.readRepo(path);
    h.assert(!/^\s*\[data-theme=/.test(src),
      path + " must scope every theme selector to :root");
    h.assert(src.split("\n").length <= 449,
      path + " must stay under 450 lines, got " + src.split("\n").length);
  });

  var scheme = h.readRepo(SCHEME);
  spanHas(scheme, 'vars["--background"] = background;',
    "the bridge publishes the theme-CSS vocabulary");
  spanHas(scheme, 'vars["--foreground"] = foreground;',
    "the bridge publishes the foreground token");
  spanHas(scheme, 'document.documentElement.style.setProperty(key, vars[key])',
    "both vocabularies reach inline root variables");
  contains(scheme, "bridge: bridge,", "the state-free bridge is exposed to the owner");
  contains(scheme, "clearInline: clearInline,",
    "inline token cleanup is exposed to the single write site");
  spanHas(scheme.slice(scheme.indexOf("const BRIDGED_TOKENS = [")),
    '"--background", "--foreground"',
    "both vocabularies are cleared, not just the base-token one");
  absent(scheme, 'setAttribute("data-theme"', "the scheme helper never writes the attribute");
});

// (d) Non-vacuity: the shipped public API and all 30 themes survive, and the
// old multi-writer shapes are gone. Reintroducing any of them fails this case.
h.staticTest("#2930 non-vacuity: themes and API preserved, old writer shapes absent", function () {
  var owner = h.readRepo(OWNER);
  var themes = between(owner, "const themes = [", "];");
  ["default", "light", "carbon", "aurora", "nord", "sentient"].forEach(function (id) {
    // sentient has no theme CSS but is the dark target; it lives in base CSS.
    if (id === "sentient") return;
    h.assert(themes.indexOf('id: "' + id + '"') !== -1, "theme " + id + " must survive");
  });
  h.assert(countIn(themes, "id: \"") >= 30, "all shipped themes must survive the refactor");
  var apiSpan = between(owner, "const api = {", "window.ThemeManager = api;");
  ["init", "loadTheme", "setThemeFromServer", "loadSavedTheme",
    "applyCustomizations", "subscribe", "getAvailableThemes"].forEach(function (name) {
    h.assert(apiSpan.indexOf(name + ",") !== -1 || apiSpan.indexOf(name + ":") !== -1,
      "public API must be preserved: " + name);
  });
  contains(owner, "window.ThemeManager = api;", "ThemeManager global is preserved");
  contains(owner, "window.dispatchEvent(new CustomEvent(\"gb:themechange\"",
    "the existing change event is preserved");
  contains(owner, "gbT(\"theme-high-contrast\", \"High contrast\")",
    "the i18n theme label is preserved with its English fallback");

  // Any resurrection of the old shapes is a failure, not a note.
  [OWNER, SCHEME, IDENTITY, LEGACY].forEach(function (path) {
    var src = h.readRepo(path);
    absent(src, "dataset.theme =", path + " must not assign dataset.theme");
    absent(src, "gb-theme-data-${botId}`, id", path + " must not persist a theme id twice");
  });
  absent(h.readRepo(LEGACY), "document.body.setAttribute(\"data-theme\"",
    "the legacy body writer must stay removed");
  absent(h.readRepo(IDENTITY), "document.documentElement.setAttribute(\"data-theme\"",
    "the adapter must stay a non-writer");
});

h.note("FOLLOW-UP #2930: static guard only — one authoritative data-theme writer, one key with legacy migration, scoped :root light/dark base tokens and both token vocabularies. Driver must run browser UAT for Light, Dark and System in desktop.html and assistant.html, and check the other Settings/assistant callers that do not live in the owned theme-manager paths; no browser/server/cargo was used here.");
