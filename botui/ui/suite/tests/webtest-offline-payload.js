/**
 * Offline static guard for #2441 — offline payload manifest truthfulness.
 * Run with: node botui/ui/suite/tests/webtest-offline-payload.js
 *
 * Asserts every [[payload]] in desktop/os/provision/offline-payload.toml
 * resolves to a real [section.key] entry in botserver/3rdparty.toml with a
 * matching filename + sha256 (empty pins allowed only where the source has
 * no pin either), and that the manifest carries no http(s):// URL and no
 * secret-looking value. Pure Node, no network, no browser, no CI claim.
 */
"use strict";

var fs = require("fs");
var path = require("path");

var ROOT = path.resolve(__dirname, "..", "..", "..", "..");
var MANIFEST = path.resolve(ROOT, "desktop/os/provision/offline-payload.toml");
var THIRDPARTY = path.resolve(ROOT, "botserver/3rdparty.toml");
var PASS = 0;
var FAIL = 0;

function check(name, condition, detail) {
  if (condition) {
    PASS++;
    console.log("[PASS] " + name);
  } else {
    FAIL++;
    console.log("[FAIL] " + name + (detail ? " — " + detail : ""));
  }
}

// Minimal TOML reader for the flat key/value tables used by both files.
// Returns { tables: { "components.vault": { filename, sha256, ... } },
//           arrays: { payload: [ { key, section, ... }, ... ] } }.
function parseToml(text) {
  var tables = {};
  var arrays = {};
  var current = null;
  text.split(/\r?\n/).forEach(function (line) {
    var t = line.trim();
    if (!t || t[0] === "#") return;
    var m = t.match(/^\[\[([^\]]+)\]\]$/);
    if (m) {
      if (!arrays[m[1]]) arrays[m[1]] = [];
      current = {};
      arrays[m[1]].push(current);
      return;
    }
    m = t.match(/^\[([^\]]+)\]$/);
    if (m) {
      current = tables[m[1]] || (tables[m[1]] = {});
      return;
    }
    var eq = t.indexOf("=");
    if (eq <= 0 || !current) return;
    var key = t.slice(0, eq).trim();
    var raw = t.slice(eq + 1).trim();
    var hash = raw.indexOf("#");
    if (hash >= 0) raw = raw.slice(0, hash).trim();
    if (raw.length >= 2 &&
        ((raw[0] === '"' && raw[raw.length - 1] === '"') ||
         (raw[0] === "'" && raw[raw.length - 1] === "'"))) {
      raw = raw.slice(1, -1);
    }
    current[key] = raw;
  });
  return { tables: tables, arrays: arrays };
}

var manifestText = fs.readFileSync(MANIFEST, "utf8");
var thirdpartyText = fs.readFileSync(THIRDPARTY, "utf8");
var manifest = parseToml(manifestText);
var thirdparty = parseToml(thirdpartyText);
var payloads = manifest.arrays.payload || [];

check("manifest has payload entries", payloads.length > 0);

// 1. No http(s):// URL anywhere outside comments (comments document provenance).
var uncommented = manifestText.split(/\r?\n/).map(function (line) {
  var t = line.trim();
  if (!t || t[0] === "#") return "";
  var hash = t.indexOf("#");
  return hash >= 0 ? t.slice(0, hash) : t;
}).join("\n");
check("manifest contains no http(s):// URL",
  !/\bhttps?:\/\//i.test(uncommented));

// 2. No secret-looking key or value in the manifest.
var SECRET_KEY = /(?:pass(?:word)?|secret|token|api[_-]?key|access[_-]?key|private[_-]?key|credential|auth)/i;
var SECRET_VALUE = /(?:gh[pousr]_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{10,}|eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,})/;
var secretHits = [];
function scanSecrets(obj, where) {
  Object.keys(obj).forEach(function (key) {
    var val = obj[key];
    if (SECRET_KEY.test(key)) secretHits.push(where + "." + key + " (key)");
    if (SECRET_VALUE.test(val)) secretHits.push(where + "." + key + " (value)");
  });
}
Object.keys(manifest.tables).forEach(function (name) {
  scanSecrets(manifest.tables[name], name);
});
payloads.forEach(function (p, i) { scanSecrets(p, "payload[" + i + "]"); });
check("manifest contains no secret-looking value", secretHits.length === 0,
  secretHits.join(", "));

// 3. Every key/section pair resolves with matching filename and sha256.
payloads.forEach(function (p, i) {
  var id = p.id || ("payload[" + i + "]");
  var required = ["key", "section", "filename", "sha256"];
  var missing = required.filter(function (f) { return !p[f] && p[f] !== ""; });
  check(id + " declares " + required.join("/"), missing.length === 0,
    "missing: " + missing.join(", "));
  if (missing.length) return;

  var table = p.section + "." + p.key;
  var entry = thirdparty.tables[table];
  check(id + " resolves to 3rdparty.toml [" + table + "]", !!entry);
  if (!entry) return;

  check(id + " filename matches", entry.filename === p.filename,
    "manifest=" + p.filename + " 3rdparty=" + entry.filename);

  var sourcePin = entry.sha256 || "";
  var manifestPin = p.sha256 || "";
  if (manifestPin) {
    check(id + " sha256 matches", manifestPin === sourcePin,
      "manifest=" + manifestPin + " 3rdparty=" + sourcePin);
  } else {
    // Empty pins are allowed only where the source has no pin (e.g. the
    // minio sidecar and the digest-free pythonbox releases).
    check(id + " empty pin is documented by source", sourcePin === "",
      "3rdparty has pin " + sourcePin);
  }
});

// 4. #2441 wizard wording: the per-component source label must exist in every
// locale catalog and be used by the installer module (no hardcoded strings).
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var I18N_DIR = path.resolve(ROOT, "botui/ui/wizard/i18n");
var INSTALL_JS = path.resolve(ROOT, "botui/ui/wizard/modules/20_install.js");
var WORDING_KEYS = ["install.from-medium", "install.downloading"];
var englishValues = {};

function quotedValue(text, key) {
  var pattern = new RegExp('"' + key.replace(/\./g, "\\.") + '"\\s*:\\s*"([^"]*)"');
  var match = text.match(pattern);
  return match ? match[1] : null;
}

LOCALES.forEach(function (locale) {
  var file = path.join(I18N_DIR, locale + ".js");
  var text = fs.readFileSync(file, "utf8");
  WORDING_KEYS.forEach(function (key) {
    var value = quotedValue(text, key);
    check(locale + " defines " + key + " with {name}",
      !!value && value.indexOf("{name}") !== -1,
      value === null ? "key missing" : "no {name} placeholder");
    if (locale === "en") {
      englishValues[key] = value;
    } else {
      check(locale + " genuinely translates " + key,
        !!value && value !== englishValues[key],
        value === null ? "key missing" : "identical to English fallback");
    }
  });
  check(locale + " catalog contains no external URL", !/\bhttps?:\/\//i.test(text));
});

var installSrc = fs.readFileSync(INSTALL_JS, "utf8");
WORDING_KEYS.forEach(function (key) {
  var used = new RegExp('t\\(\\s*"' + key.replace(/\./g, "\\.") + '"').test(installSrc);
  check("20_install.js translates " + key + " via t()", used);
});
check("20_install.js contains no external URL", !/\bhttps?:\/\//i.test(installSrc));

var wizardSecretHits = [];
[INSTALL_JS].concat(LOCALES.map(function (locale) {
  return path.join(I18N_DIR, locale + ".js");
})).forEach(function (file) {
  var hit = fs.readFileSync(file, "utf8").match(SECRET_VALUE);
  if (hit) { wizardSecretHits.push(path.basename(file) + ": " + hit[0].slice(0, 8)); }
});
check("wizard wording files contain no secret-looking value",
  wizardSecretHits.length === 0, wizardSecretHits.join(", "));

console.log("=".repeat(64));
console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
console.log("=".repeat(64));
process.exitCode = FAIL === 0 ? 0 : 1;
