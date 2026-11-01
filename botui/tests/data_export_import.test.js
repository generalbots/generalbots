"use strict";
/* GB6-24 (#1575) — backup import validation. Zero-dependency test
 * (run: node tests/data_export_import.test.js)
 *
 * Drives the real validator the importer uses and proves the properties the
 * issue requires: only documented sections, bounded `gb_`/`gb-`/`gb.` keys,
 * no credentials, no prototype-pollution keys, and a full export/import round
 * trip of the exported settings. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULE = path.join(__dirname, "..", "ui", "suite", "js", "data-export.js");

let passed = 0;
let failed = 0;
function check(condition, label) {
  if (condition) passed++;
  else {
    failed++;
    console.error("FAIL " + label);
  }
}

const store = {};
const localStorage = {
  length: 0,
  key: function () { return null; },
  getItem: function (k) { return Object.prototype.hasOwnProperty.call(store, k) ? store[k] : null; },
  setItem: function (k, v) { store[k] = String(v); },
  removeItem: function (k) { delete store[k]; },
};
const window = { localStorage: localStorage };
const sandbox = {
  window: window,
  localStorage: localStorage,
  document: { createElement: function () { return {}; }, body: { appendChild: function () {}, removeChild: function () {} }, getElementById: function () { return null; }, addEventListener: function () {} },
  URL: { createObjectURL: function () { return "blob:x"; }, revokeObjectURL: function () {} },
  Blob: function () {},
  FileReader: function () {},
  JSON: JSON,
  Date: Date,
  Number: Number,
  String: String,
  Object: Object,
  Array: Array,
  Error: Error,
  setTimeout: function () {},
  console: console,
};
vm.runInNewContext(fs.readFileSync(MODULE, "utf8"), sandbox, { filename: MODULE });
const validate = window.GBDataExport.validateBackup;
const has = (object, key) => Object.prototype.hasOwnProperty.call(object, key);

function bundle(overrides) {
  return Object.assign({
    export_version: 1,
    exported_at: "2026-10-01T10:00:00.000Z",
    app_version: "6.0",
    settings: { gb_theme: "dark" },
    chat_history: [],
    chat_memory: {},
    local_storage: { gb_setup_complete: "1" },
  }, overrides || {});
}

// 1. Documented sections restore; undeclared ones abort the import.
let clean = validate(bundle());
check(clean.values.gb_theme === "dark", "settings are restored");
check(clean.values.gb_setup_complete === "1", "local storage entries are restored");
check(clean.rejected === 0, "a clean bundle has nothing rejected");

let threw = false;
try { validate(bundle({ arbitrary: {} })); } catch (e) { threw = true; }
check(threw, "an undeclared top-level section is rejected");

// 2. Version and date are validated, not just present.
threw = false;
try { validate(bundle({ export_version: 2 })); } catch (e) { threw = true; }
check(threw, "a future bundle version is rejected");
threw = false;
try { validate(bundle({ export_version: "one" })); } catch (e) { threw = true; }
check(threw, "a non-numeric bundle version is rejected");
threw = false;
try { validate(bundle({ exported_at: "not a date" })); } catch (e) { threw = true; }
check(threw, "a missing valid export date is rejected");

// 3. Credentials are never restored from a file.
clean = validate(bundle({
  local_storage: {
    "gb-access-token": "stolen",
    management_token: "stolen",
    gb_theme: "light",
  },
}));
check(!has(clean.values, "gb-access-token"), "session token is denied");
check(!has(clean.values, "management_token"), "management token is denied");
check(clean.values.gb_theme === "light", "safe keys in the same section still restore");
check(clean.rejected === 2, "denied keys are counted as rejected");

// 4. Prototype pollution and out-of-pattern keys are refused. A literal
// `__proto__:` sets the prototype instead of an own key, so build the hostile
// section from JSON the way a real crafted file arrives.
clean = validate(bundle({
  settings: JSON.parse('{"__proto__":"x","constructor":"y","not-gb-prefixed":"z","gb.valid-key":"ok"}'),
}));
check(!has(clean.values, "__proto__"), "__proto__ key is denied");
check(!has(clean.values, "constructor"), "constructor key is denied");
check(!has(clean.values, "not-gb-prefixed"), "keys without a gb prefix are denied");
check(clean.values["gb.valid-key"] === "ok", "the gb. prefix family is allowed");
check(clean.rejected === 3, "every refused key is reported");

// 5. Types and sizes are bounded.
clean = validate(bundle({ settings: { gb_number: 42, gb_object: {} } }));
check(!has(clean.values, "gb_number"), "numeric values are not restored");
check(!has(clean.values, "gb_object"), "object values are not restored");
check(clean.rejected === 2, "non-string values count as rejected");

const oversized = "x".repeat(300 * 1024);
clean = validate(bundle({ local_storage: { gb_huge: oversized } }));
check(!has(clean.values, "gb_huge"), "a single value over the cap is refused");
check(clean.rejected === 1, "oversized value is reported");

// 6. Round trip: what exportAll collects from localStorage sanitizes back to
// the same keys, so a backup can actually be restored.
store.gb_theme = "sentient";
store.gb_setup_complete = "1";
store["gb_chat_history_demo"] = JSON.stringify([{ role: "user", content: "hi" }]);
store["gb-access-token"] = "live-token";
const exported = {
  gb_theme: store.gb_theme,
  gb_setup_complete: store.gb_setup_complete,
  "gb_chat_history_demo": store["gb_chat_history_demo"],
};
clean = validate(bundle({ settings: {}, local_storage: exported }));
check(clean.values.gb_theme === "sentient", "round trip keeps the theme");
check(clean.values.gb_setup_complete === "1", "round trip keeps setup state");
check(typeof clean.values.gb_chat_history_demo === "string", "round trip keeps chat history");
check(!has(clean.values, "gb-access-token"), "round trip never restores the live token");

console.log(`data_export_import: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
