"use strict";
/* #1853 — UI i18n audit.
 *
 * Guards the strings that were moved from hardcoded English into the i18n
 * catalogue for the desktop shell, launchers and Settings. It also prints the
 * remaining backlog for the rest of botui/ui/suite so the audit is visible in
 * CI without failing on unrelated legacy apps.
 *
 * run: node tests/i18n_ui_audit.test.js
 */
const fs = require("fs");
const path = require("path");

const ROOT = path.join(__dirname, "..", "..");
const SUITE = path.join(ROOT, "botui", "ui", "suite");

let passed = 0;
let failed = 0;
function check(condition, label) {
  if (condition) passed++;
  else {
    failed++;
    console.error("FAIL " + label);
  }
}

const PROTECTED = [
  "desktop.html",
  "js/window-manager/02_class_core.js",
  "js/window-manager/03_window_chrome.js",
  "js/window-manager/05_start_menu.js",
  "js/sidebar-apps.js",
  "js/spotlight.js",
  "js/command-palette.js",
  "js/gb-toasts.js",
  "settings/main/07_appearance.js",
  "settings/partials/02_language.html",
  "settings/partials/03_appearance_storage.html",
];

function walk(dir) {
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "vendor" || entry.name === "node_modules") continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walk(full));
    else if (/\.(html|js)$/.test(entry.name)) out.push(full);
  }
  return out;
}

function keysIn(file) {
  const text = fs.readFileSync(file, "utf8");
  const keys = new Set();
  for (const m of text.matchAll(/data-i18n(?:-html|-placeholder|-title|-aria-label|-value)?="([a-z0-9-]+)"/g)) {
    keys.add(m[1]);
  }
  // Generated chrome uses tr()/gbT()/i18n.t() rather than attributes.
  for (const m of text.matchAll(/(?:i18n\.t|i18n\.tOr|gbT|tr)\(\s*["']([a-z0-9-]+)["']/g)) {
    // Skip dynamically concatenated prefixes such as tr("category-" + cat).
    if (!m[1].endsWith("-")) keys.add(m[1]);
  }
  return keys;
}

const catalog = new Set();
for (const file of fs.readdirSync(path.join(ROOT, "botserver/crates/botcore/src/i18n/translations"))) {
  if (!/^translation_.*\.rs$/.test(file)) continue;
  const text = fs.readFileSync(path.join(ROOT, "botserver/crates/botcore/src/i18n/translations", file), "utf8");
  for (const m of text.matchAll(/"([a-z0-9-]+)"/g)) catalog.add(m[1]);
}

const english = new Set();
for (const file of fs.readdirSync(path.join(ROOT, "botlib/locales/en"))) {
  if (!file.endsWith(".ftl")) continue;
  const text = fs.readFileSync(path.join(ROOT, "botlib/locales/en", file), "utf8");
  for (const m of text.matchAll(/^([a-z0-9-]+)\s*=/gm)) english.add(m[1]);
}

for (const relative of PROTECTED) {
  const file = path.join(SUITE, relative);
  check(fs.existsSync(file), "protected file exists: " + relative);
  if (!fs.existsSync(file)) continue;
  const keys = keysIn(file);
  for (const key of keys) {
    check(catalog.has(key), relative + ": catalog exposes " + key);
    check(english.has(key), relative + ": English translates " + key);
  }
}

// Audit the whole suite for visibility (not a hard failure yet: legacy apps
// still carry ~300 keyed-but-untranslated strings tracked by #1853).
const all = new Set();
for (const file of walk(SUITE)) {
  for (const key of keysIn(file)) all.add(key);
}
const missing = [...all].filter((key) => !english.has(key)).sort();
// The API serves the Rust catalogue plus every key authored in English FTL,
// so a key is missing at runtime only when neither contains it.
const served = new Set([...catalog, ...english]);
const unserved = [...all].filter((key) => !served.has(key)).sort();
console.log(
  `i18n_ui_audit: ${all.size} UI keys, ${missing.length} without English text, ` +
    `${unserved.length} not served by the i18n API`,
);
if (missing.length) console.log("  backlog (English): " + missing.slice(0, 12).join(", "));
if (unserved.length) console.log("  backlog (unserved): " + unserved.slice(0, 12).join(", "));

console.log(`i18n_ui_audit: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
