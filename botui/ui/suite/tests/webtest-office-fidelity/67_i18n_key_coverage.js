/**
 * #2921 — deterministic static guard for the suite i18n key graph.
 *
 * Two cases, both source-only (no server, browser, cargo or network):
 *
 *  (A) Every user-facing key the suite asks for resolves in the canonical
 *      English catalogue. `data-i18n` attributes and the gbT() family
 *      (`gbT`/`t`/`tOr`/`tr`/`gbChatText`/`i18n.t`) are scanned; app-owned
 *      dictionaries exempt their own keys, exactly as scripts/i18n_lint.py
 *      does. Non-English locales resolve through the runtime English fallback
 *      (see js/i18n.js `englishFallback`), so English is the authority for
 *      "resolves" — the all-locales requirement is pinned in case (B).
 *
 *  (B) Every #2921 key is authored in ALL EIGHT locale catalogues. These are
 *      the keys reported missing on the Slides page (shell, sidebar, chat,
 *      notepad, billing, toasts). A key absent from even one locale fails with
 *      the key and the locale listed.
 *
 * Both cases assert non-zero counts so an empty scan can never pass vacuously.
 * The driver owns issue closure and browser re-UAT.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var h = require("./00_harness");

var SUITE = path.join(h.REPO_DIR, "botui", "ui", "suite");
var LOCALES_DIR = path.join(h.REPO_DIR, "botlib", "locales");
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

// The exact keys this issue adds to every catalogue (71 from the report and
// its follow-up comment, plus the remaining shipped keys the same scan found).
var ISSUE_KEYS = [
  "about-dependency-tree-missing",
  "app-events-desc", "app-events-name", "app-notepad-desc", "app-notepad-name",
  "app-preview-desc", "app-preview-name",
  "billing-col-due", "billing-col-number", "billing-export",
  "billing-filter-all", "billing-filter-cancelled", "billing-filter-draft",
  "billing-filter-overdue", "billing-filter-paid", "billing-filter-sent",
  "billing-period-all", "billing-period-month", "billing-period-quarter",
  "billing-period-year",
  "chat-confirm-title", "chat-confirm-unavailable", "chat-copy", "chat-delete",
  "chat-delete-message-confirm", "chat-delete-title", "chat-edit",
  "chat-edit-unavailable", "chat-message-actions", "chat-regenerate",
  "chat-regenerate-no-source", "chat-regenerate-unavailable",
  "chat-voice-capture-silent", "chat-voice-duplex-audio",
  "chat-voice-duplex-closed", "chat-voice-duplex-connect",
  "chat-voice-duplex-connected", "chat-voice-duplex-disabled",
  "chat-voice-duplex-error", "chat-voice-duplex-fallback", "chat-voice-duplex-live",
  "chat-voice-duplex-lost", "chat-voice-duplex-timeout",
  "chat-voice-duplex-unavailable", "chat-voice-mic-failed", "chat-web-title",
  "collab-typing-many", "collab-typing-one", "collab-typing-two",
  "palette-kind-tour", "palette-show-tour", "palette-show-tour-hint",
  "sidebar-app-hide", "sidebar-apps-count-off", "sidebar-apps-empty",
  "sidebar-apps-manage", "sidebar-apps-manage-all", "sidebar-apps-manage-close",
  "sidebar-apps-manage-empty", "sidebar-apps-manage-summary", "sidebar-apps-none",
  "timer-time-up", "toast-open",
  "voice-mode-connecting", "voice-mode-fallback-idle",
  "voice-mode-fallback-listening", "voice-mode-fallback-speaking",
  "voice-mode-fallback-thinking", "voice-mode-live-listening",
  "voice-mode-live-ready", "voice-mode-live-speaking",
];

var DATA_I18N_RE = /data-i18n(?:-[a-z-]+)?=\s*["']([^"']+)["']/g;
var CALL_RE = /\b(?:gbT|gbChatText|gbTOr|tOr|i18n\.t|t|tr)\(\s*["']([a-z0-9][a-z0-9._-]*)["']/g;
var DICT_ENTRY_RE = /(?:["']([A-Za-z0-9_.-]+)["']|\b([A-Za-z_][A-Za-z0-9_.-]*))\s*:\s*["'`]/g;
var FTL_KEY_RE = /^([A-Za-z0-9_.-]+)\s*=\s*(.*)$/;
// Only literal message ids can resolve from a Fluent catalogue. Filters out
// attributes holding a JS expression such as `+ column.labelKey +`.
var VALID_KEY_RE = /^[a-z0-9]+(-[a-z0-9]+)*$/;

/** Every .js/.html source under the suite, excluding third-party bundles. */
function sources(dir, out) {
  fs.readdirSync(dir, { withFileTypes: true }).forEach(function (entry) {
    var full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name !== "vendor") sources(full, out);
    } else if (/\.(js|html)$/.test(entry.name)) {
      out.push(full);
    }
  });
  return out;
}

/** The app scope of a suite file, mirroring i18n_lint.app_scope(). */
function scopeOf(file) {
  var rel = path.relative(SUITE, file).split(path.sep);
  return rel.length > 1 ? rel[0] : "";
}

/** Keys an app resolves from its own dictionary, not from the FTL catalogue. */
function localKeys(scope) {
  var keys = new Set();
  var base = scope ? path.join(SUITE, scope) : SUITE;
  if (!fs.existsSync(base)) return keys;
  sources(base, []).forEach(function (file) {
    var text = fs.readFileSync(file, "utf8");
    var match;
    DICT_ENTRY_RE.lastIndex = 0;
    while ((match = DICT_ENTRY_RE.exec(text)) !== null) {
      keys.add(match[1] || match[2]);
    }
  });
  return keys;
}

/** FTL message ids of one locale (all .ftl files in its directory). */
function catalogue(locale) {
  var keys = new Set();
  var dir = path.join(LOCALES_DIR, locale);
  fs.readdirSync(dir).filter(function (name) { return /\.ftl$/.test(name); })
    .sort().forEach(function (name) {
      fs.readFileSync(path.join(dir, name), "utf8").split(/\r?\n/)
        .forEach(function (line) {
          var match = FTL_KEY_RE.exec(line);
          if (match) keys.add(match[1]);
        });
    });
  return keys;
}

/** key -> Set of source files that use it, plus the counted call forms. */
function usedKeys(files) {
  var used = new Map();
  var dataCount = 0;
  var callCount = 0;
  function add(key, file) {
    if (!VALID_KEY_RE.test(key)) return;
    if (!used.has(key)) used.set(key, new Set());
    used.get(key).add(file);
  }
  files.forEach(function (file) {
    var text = fs.readFileSync(file, "utf8");
    var match;
    DATA_I18N_RE.lastIndex = 0;
    while ((match = DATA_I18N_RE.exec(text)) !== null) {
      add(match[1], file);
      dataCount++;
    }
    CALL_RE.lastIndex = 0;
    while ((match = CALL_RE.exec(text)) !== null) {
      add(match[1], file);
      callCount++;
    }
  });
  return { used: used, dataCount: dataCount, callCount: callCount };
}

h.staticTest("#2921 every data-i18n key and gbT-family call resolves in the English catalogue", function () {
  var files = sources(SUITE, []);
  h.assert(files.length > 100,
    "non-vacuity: expected a large suite source graph, found " + files.length);

  var scanned = usedKeys(files);
  h.assert(scanned.dataCount > 500,
    "non-vacuity: expected >500 data-i18n attributes, found " + scanned.dataCount);
  h.assert(scanned.callCount > 500,
    "non-vacuity: expected >500 translation calls, found " + scanned.callCount);
  h.assert(scanned.used.size > 1000,
    "non-vacuity: expected >1000 distinct keys, found " + scanned.used.size);

  var english = catalogue("en");
  h.assert(english.size > 1000,
    "non-vacuity: expected a populated English catalogue, found " + english.size);

  var locals = new Map();
  function local(scope) {
    if (!locals.has(scope)) locals.set(scope, localKeys(scope));
    return locals.get(scope);
  }

  var checked = 0;
  var missing = [];
  scanned.used.forEach(function (filesForKey, key) {
    // String-concatenation fragments ("category-" + id) and test-only keys are
    // not catalogue message ids.
    if (key.endsWith("-")) return;
    var onlyTests = Array.from(filesForKey).every(function (file) {
      return path.relative(SUITE, file).split(path.sep)[0] === "tests";
    });
    if (onlyTests) return;
    // An app-owned dictionary resolves the key locally, as i18n_lint does.
    var files = Array.from(filesForKey);
    if (files.every(function (file) { return local(scopeOf(file)).has(key); })) return;
    checked++;
    if (!english.has(key)) missing.push(key + " (" + files[0] + ")");
  });

  h.assert(checked > 1000,
    "non-vacuity: expected >1000 keys checked against the catalogue, found " + checked);
  h.assert(missing.length === 0,
    "keys used by the suite but absent from the English catalogue (" +
    checked + " checked):\n  " + missing.sort().join("\n  "));
});

h.staticTest("#2921 every fixed key is authored in all eight locale catalogues", function () {
  h.assert(LOCALES.length === 8, "non-vacuity: all eight locales must be checked");
  h.assert(ISSUE_KEYS.length > 30,
    "non-vacuity: expected the full #2921 key set, found " + ISSUE_KEYS.length);
  h.assert(new Set(ISSUE_KEYS).size === ISSUE_KEYS.length,
    "the pinned key list must not contain duplicates");

  var catalogs = new Map();
  LOCALES.forEach(function (locale) {
    var cat = catalogue(locale);
    h.assert(cat.size > 1000,
      "non-vacuity: " + locale + " catalogue must be populated, found " + cat.size);
    catalogs.set(locale, cat);
  });

  var missing = [];
  ISSUE_KEYS.forEach(function (key) {
    LOCALES.forEach(function (locale) {
      if (!catalogs.get(locale).has(key)) missing.push(locale + ": " + key);
    });
  });
  h.assert(missing.length === 0,
    "#2921 keys missing from a locale catalogue (" + ISSUE_KEYS.length +
    " keys x " + LOCALES.length + " locales):\n  " + missing.join("\n  "));
});

h.staticTest("#2921 a failed /api/product boot discards the unversioned translation cache", function () {
  var src = h.readRepo("botui/ui/suite/js/i18n.js");
  var start = src.indexOf("let productVersion = null;");
  var end = src.indexOf('const response = await fetch("/api/i18n/locales")', start);
  h.assert(start !== -1 && end > start, "locate the /api/product boot block in i18n.js");
  var block = src.slice(start, end);

  h.assert(block.indexOf("if (productVersion)") !== -1,
    "a resolved product version takes the normal, cached path");
  h.assert(block.indexOf("cacheBuild = productVersion;") !== -1,
    "the cache key still carries the real build version when it is known");
  h.assert(block.indexOf("clearCache();") !== -1,
    "a missing build version must invalidate the stale 'dev' cache");
  h.assert(block.indexOf('cacheBuild = "dev";') !== -1,
    "the fallback key remains 'dev' for offline boots");
  h.assert(src.indexOf("const CACHE_TTL_MS = 3600000;") !== -1,
    "normal versioned loads keep their one-hour cache TTL");
  h.assert(src.indexOf("if (age < CACHE_TTL_MS)") !== -1,
    "normal versioned loads still read the cache");
});
