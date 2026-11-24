/**
 * Suite UI a11y static audit — WCAG 2.2 AA source-level checks (#2454).
 * Run with: node botui/ui/suite/tests/webtest-ui-aria.js
 *
 * Deterministic OFFLINE proxies for criteria the issue calls out: focus
 * visibility (2.4.7/2.4.11), target size (2.5.8), accessible names/roles/states
 * (4.1.2), polite status announcements (4.1.3) and i18n key parity across all
 * eight locale catalogs (3.1.1 / AGENTS.md i18n rule).
 *
 * These are source-level checks only: computed contrast, the live AX tree and
 * axe-core results still need a browser (see the AT-SPOTCHECK documented gap).
 * No key is created here; the checks assert keys already shipped in botlib.
 */
"use strict";

var h = require("./00_harness");

function has(text, needle, msg) {
  h.assert(text.indexOf(needle) !== -1, msg);
}

/** Parse FTL message identifiers from "key = value" lines; ignore # comments. */
function ftlKeys(text) {
  var out = {};
  text.split(/\r?\n/).forEach(function (line) {
    var match = /^([A-Za-z0-9][A-Za-z0-9._-]*)\s*=/.exec(line);
    if (match) out[match[1]] = true;
  });
  return Object.keys(out).sort();
}

function sameKeys(a, b) {
  if (a.length !== b.length) return false;
  return a.every(function (key, i) { return key === b[i]; });
}

h.test("[STATIC][WCAG-2.4.7] global :focus-visible outline is shipped", function () {
  var css = h.readSource("css/app/06_focus_visible.css");
  has(css, "*:focus-visible", "no global focus-visible rule");
  has(css, "outline: 2px solid", "focus indicator has no visible outline");
  has(css, "outline-offset", "focus indicator has no outline offset");
  has(css, "prefers-reduced-motion", "no reduced-motion accommodation");
});

h.test("[STATIC][WCAG-2.5.8] coarse-pointer controls get a 44px target floor", function () {
  var css = h.readSource("css/touch-targets.css");
  has(css, "@media (pointer: coarse)", "no coarse-pointer media query");
  has(css, "min-width: 44px", "no 44px minimum width");
  has(css, "min-height: 44px", "no 44px minimum height");
  has(css, "#gb-skip-nav", "skip link is not covered by the target floor");
  has(css, '[role="button"]', "role=button controls are not covered");
});

h.test("[STATIC][WCAG-4.1.2] Sheets grid exposes names, roles and states", function () {
  var js = h.readSource("sheet/js/constants/03a_grid_a11y.js");
  has(js, 'setAttribute("role", "grid")', "grid role missing");
  has(js, 'setAttribute("aria-multiselectable", "true")', "multiselect state missing");
  has(js, 'setAttribute("aria-label", t("sheet-a11y-grid-label"', "grid has no accessible name");
  has(js, 'setAttribute("aria-selected"', "cell selected state missing");
  has(js, 'setAttribute("aria-readonly"', "readonly state missing");
  has(js, "node.tabIndex = isActive ? 0 : -1", "roving tab stop missing");
  has(js, 'setAttribute("aria-controls", gridId)', "formula bar is not associated with the grid");
  has(js, 'setAttribute("aria-label", t("sheet-a11y-name-box-label"', "name box has no accessible name");
  has(js, 'setAttribute("aria-label", t("sheet-a11y-formula-bar-label"', "formula bar has no accessible name");
});

h.test("[STATIC][WCAG-4.1.3] Sheets grid declares a localized polite status region", function () {
  var js = h.readSource("sheet/js/constants/03a_grid_a11y.js");
  has(js, 'this.a11yLive.className = "sheet-a11y-live"', "live region node missing");
  has(js, 'setAttribute("role", "status")', "no status role on the live region");
  has(js, 'setAttribute("aria-live", "polite")', "no polite live value");
  has(js, 'setAttribute("aria-atomic", "true")', "live region is not atomic");
  has(js, 't("sheet-a11y-live-selection"', "selection is not announced via a catalog key");
  has(js, 't("sheet-a11y-live-edited"', "edits are not announced via a catalog key");
});

h.test("[STATIC][WCAG-3.1.1] a11y catalog keys are present in all eight locales", function () {
  ["sheet-a11y", "forms-a11y"].forEach(function (catalog) {
    var base = ftlKeys(h.readRepo("botlib/locales/en/" + catalog + ".ftl"));
    h.assert(base.length > 0, "source catalog " + catalog + " has no keys");
    h.LOCALES.forEach(function (locale) {
      var text;
      try {
        text = h.readRepo("botlib/locales/" + locale + "/" + catalog + ".ftl");
      } catch (e) {
        throw new Error("missing locale catalog " + locale + "/" + catalog + ".ftl");
      }
      var actual = ftlKeys(text);
      h.assert(sameKeys(base, actual), locale + "/" + catalog + ".ftl key set differs from en (" +
        base.length + " vs " + actual.length + ")");
    });
  });
});
