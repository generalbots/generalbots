/**
 * Shared harness for the offline boot-budget static audit (#2495).
 *
 * The scanner is deliberately lexical and mirrors scripts/check_boot_budget.py
 * so the numbers it prints are comparable with the shipped count-only guard.
 * It reads files only: it never imports or executes production JS, and it
 * never writes a baseline/ratchet file (that guard is owned by another change).
 */
"use strict";

var fs = require("fs");
var path = require("path");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var UI_DIR = path.resolve(SUITE_DIR, "..");
var TESTS_DIR = path.resolve(__dirname, "..");

var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var ALL = null;

function log(message) {
  console.log(message);
}

function test(name, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ name: name, status: "PASS" });
  } catch (e) {
    FAIL++;
    RESULTS.push({ name: name, status: "FAIL", error: e && e.message ? e.message : String(e) });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function equal(actual, expected, label) {
  assert(actual === expected, (label || "value") + ": expected " + expected + ", got " + actual);
}

function readFile(relPath) {
  return fs.readFileSync(path.join(UI_DIR, relPath), "utf8");
}

function fileExists(relPath) {
  return fs.existsSync(path.join(UI_DIR, relPath));
}

function statBytes(relPath) {
  try {
    return fs.statSync(path.join(UI_DIR, relPath)).size;
  } catch (e) {
    return 0;
  }
}

// ── Lexical HTML scanner ────────────────────────────────────────────────

var SCRIPT_TAG_RE = /<script\b([^>]*)>/gi;
var LINK_TAG_RE = /<link\b([^>]*)>/gi;
var ATTR_RE = /([A-Za-z_:][-A-Za-z0-9_:.]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/g;
var BOOL_RE = /(?:^|\s)(defer|async)(?=\s|=|$)/gi;
var HASHED_CSS_RE = /\.[0-9a-f]{8,}(?:\.[0-9a-f]{8,})?\.css$/i;
var COUNT_KEYS = [
  "scripts", "scriptSrc", "inlineScript", "deferAsync", "blockingScriptSrc",
  "stylesheet", "mediaLink", "mediaScript", "hashedCss", "versionedCss",
];

function parseAttrs(tagText) {
  var attrs = {};
  var m;
  ATTR_RE.lastIndex = 0;
  while ((m = ATTR_RE.exec(tagText)) !== null) {
    attrs[m[1].toLowerCase()] = m[2] || m[3] || m[4] || "";
  }
  BOOL_RE.lastIndex = 0;
  while ((m = BOOL_RE.exec(tagText)) !== null) {
    var key = m[1].toLowerCase();
    if (!Object.prototype.hasOwnProperty.call(attrs, key)) attrs[key] = "";
  }
  return attrs;
}

function isStylesheet(attrs) {
  return String(attrs.rel || "").toLowerCase().split(/\s+/).indexOf("stylesheet") >= 0;
}

function emptyCounts() {
  var c = {};
  COUNT_KEYS.forEach(function (k) { c[k] = 0; });
  c.scriptSrcs = [];
  c.stylesheetHrefs = [];
  c.mediaAttrs = [];
  return c;
}

function measureHtml(absPath) {
  var c = emptyCounts();
  var text;
  try {
    text = fs.readFileSync(absPath, "utf8");
  } catch (e) {
    return c;
  }
  var m;
  SCRIPT_TAG_RE.lastIndex = 0;
  while ((m = SCRIPT_TAG_RE.exec(text)) !== null) {
    var sa = parseAttrs(m[1]);
    c.scripts++;
    if (Object.prototype.hasOwnProperty.call(sa, "media")) {
      c.mediaScript++;
      c.mediaAttrs.push(sa.media || "");
    }
    if (!Object.prototype.hasOwnProperty.call(sa, "src")) {
      c.inlineScript++;
      continue;
    }
    c.scriptSrc++;
    c.scriptSrcs.push(sa.src);
    if (Object.prototype.hasOwnProperty.call(sa, "defer") ||
        Object.prototype.hasOwnProperty.call(sa, "async")) {
      c.deferAsync++;
    } else {
      c.blockingScriptSrc++;
    }
  }
  LINK_TAG_RE.lastIndex = 0;
  while ((m = LINK_TAG_RE.exec(text)) !== null) {
    var la = parseAttrs(m[1]);
    if (!isStylesheet(la)) continue;
    c.stylesheet++;
    var href = la.href || "";
    c.stylesheetHrefs.push(href);
    if (Object.prototype.hasOwnProperty.call(la, "media")) {
      c.mediaLink++;
      c.mediaAttrs.push(la.media || "");
    }
    var base = href.split("?")[0].split("#")[0];
    if (HASHED_CSS_RE.test(base)) c.hashedCss++;
    if (href.indexOf("?v=") >= 0) c.versionedCss++;
  }
  return c;
}

function listHtmlFiles(dir, relPrefix) {
  dir = dir || SUITE_DIR;
  relPrefix = relPrefix || "";
  var out = [];
  var entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch (e) {
    return out;
  }
  entries.forEach(function (entry) {
    var rel = relPrefix ? relPrefix + "/" + entry.name : entry.name;
    if (entry.isDirectory()) {
      if (entry.name === "tests") return; // out of scope per the issue
      out = out.concat(listHtmlFiles(path.join(dir, entry.name), rel));
    } else if (entry.isFile() && /\.html$/i.test(entry.name)) {
      out.push(rel);
    }
  });
  return out.sort();
}

function measureAll() {
  var files = listHtmlFiles();
  var total = emptyCounts();
  var perFile = {};
  files.forEach(function (rel) {
    var c = measureHtml(path.join(SUITE_DIR, rel));
    perFile[rel] = c;
    COUNT_KEYS.forEach(function (k) { total[k] += c[k]; });
    total.scriptSrcs = total.scriptSrcs.concat(c.scriptSrcs);
    total.stylesheetHrefs = total.stylesheetHrefs.concat(c.stylesheetHrefs);
    total.mediaAttrs = total.mediaAttrs.concat(c.mediaAttrs);
  });
  return { files: files, perFile: perFile, total: total };
}

function all() {
  if (ALL === null) ALL = measureAll();
  return ALL;
}

function measure(rel) {
  var c = all().perFile[rel];
  assert(c, "HTML file not scanned: " + rel);
  return c;
}

function report() {
  var a = all();
  var d = a.perFile["desktop.html"] || emptyCounts();
  var c = a.perFile["partials/chat.html"] || emptyCounts();
  var p = a.perFile["chat/projector.html"] || emptyCounts();
  log("");
  log("=".repeat(64));
  log("UI BOOT BUDGET OFFLINE STATIC AUDIT (#2495)");
  log("=".repeat(64));
  log("Scanned shipped HTML files (tests/ excluded): " + a.files.length);
  log("desktop.html     scripts=" + d.scripts + " src=" + d.scriptSrc +
      " blocking=" + d.blockingScriptSrc + " inline=" + d.inlineScript +
      " defer/async=" + d.deferAsync + " stylesheets=" + d.stylesheet +
      " mediaLinks=" + d.mediaLink);
  log("partials/chat    scripts=" + c.scripts + " src=" + c.scriptSrc +
      " blocking=" + c.blockingScriptSrc + " inline=" + c.inlineScript +
      " defer/async=" + c.deferAsync + " stylesheets=" + c.stylesheet +
      " mediaLinks=" + c.mediaLink);
  log("chat/projector   scripts=" + p.scripts + " src=" + p.scriptSrc +
      " blocking=" + p.blockingScriptSrc + " inline=" + p.inlineScript +
      " defer/async=" + p.deferAsync + " stylesheets=" + p.stylesheet +
      " mediaLinks=" + p.mediaLink);
  log("suite total      scripts=" + a.total.scripts + " src=" + a.total.scriptSrc +
      " blocking=" + a.total.blockingScriptSrc + " inline=" + a.total.inlineScript +
      " defer/async=" + a.total.deferAsync + " stylesheets=" + a.total.stylesheet +
      " mediaLinks=" + a.total.mediaLink + " hashedCss=" + a.total.hashedCss);
  log("-".repeat(64));
  RESULTS.forEach(function (r) {
    log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
  });
  log("=".repeat(64));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(64));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  SUITE_DIR, UI_DIR, TESTS_DIR, COUNT_KEYS,
  log, test, assert, equal, readFile, fileExists, statBytes,
  parseAttrs, isStylesheet, emptyCounts, measureHtml, listHtmlFiles,
  measureAll, all, measure, finish,
};
