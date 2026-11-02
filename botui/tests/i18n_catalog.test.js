"use strict";
/* Translation catalogue coverage (#1625).
 *
 * The /api/i18n/:locale endpoint only returns keys listed in
 * botcore/src/i18n/translations/*.rs. A key in that list with no entry in the
 * English FTL bundle reaches the UI as "[key]". This test keeps English at
 * 100% and every other locale either translated or covered by the English
 * fallback.
 *
 * run: node tests/i18n_catalog.test.js
 */
const fs = require("fs");
const path = require("path");

const ROOT = path.join(__dirname, "..", "..");
let passed = 0;
let failed = 0;
function check(condition, label) {
  if (condition) passed++;
  else { failed++; console.error("FAIL " + label); }
}

const catalogDir = path.join(ROOT, "botserver/crates/botcore/src/i18n/translations");
const catalogFiles = fs.readdirSync(catalogDir)
  .filter((name) => name.startsWith("translation_") && name.endsWith(".rs"))
  .map((name) => path.join(catalogDir, name));
check(catalogFiles.length > 0, "translation catalogue files are present");
const keys = new Set();
for (const file of catalogFiles) {
  const text = fs.readFileSync(file, "utf8");
  for (const m of text.matchAll(/"([a-z0-9-]+)"/g)) keys.add(m[1]);
}
check(keys.size > 300, "catalogue declares the translation keys (" + keys.size + ")");

function loadLocale(locale) {
  const dir = path.join(ROOT, "botlib", "locales", locale);
  const out = new Set();
  for (const file of fs.readdirSync(dir)) {
    if (!file.endsWith(".ftl")) continue;
    const text = fs.readFileSync(path.join(dir, file), "utf8");
    for (const m of text.matchAll(/^([a-z0-9-]+)\s*=/gm)) out.add(m[1]);
  }
  return out;
}

const en = loadLocale("en");
const missingEn = [...keys].filter((key) => !en.has(key));
check(missingEn.length === 0, "English defines every catalog key" + (missingEn.length ? ": " + missingEn.slice(0, 8).join(", ") : ""));

for (const locale of ["pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"]) {
  const dict = loadLocale(locale);
  // Every catalog key is either translated or resolved by the English fallback.
  const uncovered = [...keys].filter((key) => !dict.has(key) && !en.has(key));
  check(uncovered.length === 0, locale + " has no uncovered key");
}

// Chat keys added for the composer must exist in every locale (these are
// visible in the first screen, so an English fallback would be jarring).
const chatKeys = ["chat-format", "chat-attach", "chat-web-on", "chat-web-off", "chat-private-signin"];
for (const locale of ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"]) {
  const dict = loadLocale(locale);
  const missing = chatKeys.filter((key) => !dict.has(key));
  check(missing.length === 0, locale + " translates the chat composer keys");
}

console.log(`i18n_catalog: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
