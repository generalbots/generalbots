"use strict";
/* Wizard i18n regression test (#1625).
 *
 * Loads the real dictionary files with a minimal window shim and asserts that
 * every locale defines every English key. A missing key would silently fall
 * back to English in the UI, which is exactly the regression this guards.
 *
 * run: node tests/wizard_i18n.test.js
 */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const ROOT = path.join(__dirname, "..", "..");
const WIZARD = path.join("botui", "ui", "wizard");
const LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

let passed = 0;
let failed = 0;
function check(condition, label) {
  if (condition) passed++;
  else { failed++; console.error("FAIL " + label); }
}

const sandbox = {
  window: { dispatchEvent() {} },
  document: { documentElement: { setAttribute() {} } },
  console,
  navigator: { language: "en" },
  CustomEvent: function CustomEvent(type, init) { this.type = type; this.detail = init && init.detail; },
};
vm.createContext(sandbox);
for (const locale of LOCALES) {
  const file = path.join(ROOT, WIZARD, "i18n", locale + ".js");
  check(fs.existsSync(file), "dictionary exists: " + locale);
  vm.runInContext(fs.readFileSync(file, "utf8"), sandbox, { filename: file });
}
vm.runInContext(fs.readFileSync(path.join(ROOT, WIZARD, "wizard-i18n.js"), "utf8"), sandbox, {
  filename: "wizard-i18n.js",
});

const dictionaries = sandbox.window.GBWizardLocales;
const i18n = sandbox.window.WizardI18n;
check(!!dictionaries, "dictionaries registered on window");
check(!!i18n, "WizardI18n exported");

const englishKeys = Object.keys(dictionaries.en || {});
check(englishKeys.length > 50, "English dictionary is populated (" + englishKeys.length + " keys)");

for (const locale of LOCALES) {
  const missing = englishKeys.filter((key) => !dictionaries[locale] || dictionaries[locale][key] == null);
  check(missing.length === 0, locale + " defines every English key" + (missing.length ? ": " + missing.join(", ") : ""));
}

check(i18n.LOCALES.length === LOCALES.length, "selector lists all supported locales");
check(i18n.normalize("pt") === "pt-BR", "short locale codes resolve to a full locale");
check(i18n.normalize("xx") === "en", "unknown locale falls back to English");

i18n.setLocale("en");
check(i18n.t("wizard.next") === "Next", "English key resolves");
i18n.setLocale("pt-BR");
check(i18n.t("wizard.next") === "Avançar", "Portuguese key resolves");
check(i18n.t("wizard.foot-step", { current: 2, total: 7 }) === "Etapa 2 de 7", "parameters are substituted");

console.log(`wizard_i18n: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
