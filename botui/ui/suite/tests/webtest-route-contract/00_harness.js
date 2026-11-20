/**
 * Shared, read-only scanning harness for the #2546 offline route-contract audit.
 *
 * Rules: scan shipped source only, cache every read, never write a file, never
 * import production JS, never read or write a baseline/ratchet file and never
 * claim a runtime, browser or CI result this process cannot observe. The Rust
 * route parser mirrors scripts/check_route_contract.py semantics so its counts
 * are comparable with the shipped Python guard, but it is implemented here so
 * this harness stays a standalone `node <file>` audit.
 */
"use strict";

var fs = require("fs");
var path = require("path");

var REPO_DIR = path.resolve(__dirname, "..", "..", "..", "..", "..");
var BOTSERVER = "botserver";
var CATALOG_MOD = "botserver/src/core/bot/api_catalog/mod.rs";
var FIND_API = "botserver/src/core/bot/api_catalog/find_api_entries.rs";
var CI_WORKFLOW = ".github/workflows/ci.yml";
var ROUTE_ROOTS = ["botserver/src", "botserver/crates"];
var INVENTORY_ROOT = "botserver/src/core/bot";
var METHODS = ["get", "post", "put", "patch", "delete", "options", "head"];
var ANY_VERBS = ["GET", "POST", "PUT", "PATCH", "DELETE"];

var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var FINDINGS = [];
var OBSERVED = [];
var SRC_CACHE = Object.create(null);
var FILE_CACHE = Object.create(null);
var consts;
var CONST_CACHE = null;
var ROUTE_CACHE = null;
var INVENTORY_CACHE = null;
var STUB_CACHE = null;

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
    RESULTS.push({
      name: name,
      status: "FAIL",
      error: e && e.message ? e.message : String(e),
    });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function equal(actual, expected, label) {
  assert(
    actual === expected,
    (label || "value") + ": expected <" + expected + ">, got <" + actual + ">"
  );
}

/** Record an observed deviation with file:line evidence. Never patches code. */
function finding(name, detail) {
  FINDINGS.push({ name: name, detail: detail });
}

function observe(label, value) {
  OBSERVED.push(label + ": " + value);
}

function exists(relPath) {
  return fs.existsSync(path.join(REPO_DIR, relPath));
}

function readRepo(relPath) {
  return fs.readFileSync(path.join(REPO_DIR, relPath), "utf8");
}

function src(relPath) {
  if (!Object.prototype.hasOwnProperty.call(SRC_CACHE, relPath)) {
    SRC_CACHE[relPath] = exists(relPath) ? readRepo(relPath) : "";
  }
  return SRC_CACHE[relPath];
}

/** Cached sorted file list. `tests` is NOT skipped: Rust tests mount routes. */
function walk(relRoot, suffix) {
  var key = relRoot + "|" + suffix;
  if (Object.prototype.hasOwnProperty.call(FILE_CACHE, key)) return FILE_CACHE[key];
  var out = [];
  var absRoot = path.join(REPO_DIR, relRoot);
  var skipped = { target: true, node_modules: true, vendor: true, ".git": true };
  function visit(dir, prefix) {
    var entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch (e) {
      return;
    }
    entries.forEach(function (entry) {
      var rel = prefix ? prefix + "/" + entry.name : entry.name;
      if (entry.isDirectory()) {
        if (!skipped[entry.name]) visit(path.join(dir, entry.name), rel);
      } else if (entry.isFile() && (!suffix || entry.name.slice(-suffix.length) === suffix)) {
        out.push(relRoot + "/" + rel.split(path.sep).join("/"));
      }
    });
  }
  visit(absRoot, "");
  FILE_CACHE[key] = out.sort();
  return FILE_CACHE[key];
}

function rustFiles() {
  return ROUTE_ROOTS.reduce(function (all, root) {
    return all.concat(walk(root, ".rs"));
  }, []).sort();
}

function inventoryFiles() {
  return walk(INVENTORY_ROOT, ".rs").filter(function (rel) {
    var name = rel.slice(rel.lastIndexOf("/") + 1);
    if (rel.indexOf("/endpoint_inventory/") >= 0) return false;
    return /^endpoint_inventory[^/]*\.rs$/.test(name) ||
      /\/endpoint_inventory_[^/]+\/.*\.rs$/.test(rel);
  });
}

function lineOf(text, index) {
  return text.slice(0, index).split("\n").length;
}

/** Balanced parentheses starting at `start` (the '(' character). */
function parenBlock(text, start) {
  var depth = 0;
  var inString = false;
  var escaped = false;
  for (var i = start; i < text.length; i++) {
    var ch = text.charAt(i);
    if (inString) {
      if (escaped) escaped = false;
      else if (ch === "\\") escaped = true;
      else if (ch === '"') inString = false;
      continue;
    }
    if (ch === '"') inString = true;
    else if (ch === "(") depth++;
    else if (ch === ")") {
      depth--;
      if (depth === 0) return { body: text.slice(start + 1, i), end: i + 1 };
    }
  }
  return { body: text.slice(start + 1), end: text.length };
}

/** Index of the first comma at nesting depth zero, or -1. */
function topLevelComma(body) {
  var depth = 0;
  var inString = false;
  var escaped = false;
  for (var i = 0; i < body.length; i++) {
    var ch = body.charAt(i);
    if (inString) {
      if (escaped) escaped = false;
      else if (ch === "\\") escaped = true;
      else if (ch === '"') inString = false;
      continue;
    }
    if (ch === '"') inString = true;
    else if (ch === "(" || ch === "[" || ch === "{") depth++;
    else if (ch === ")" || ch === "]" || ch === "}") depth--;
    else if (ch === "," && depth === 0) return i;
  }
  return -1;
}

/** Resolve string literals, simple const names and const-only format! paths. */
function resolvePath(expr, consts) {
  var value = expr.trim().replace(/^&/, "");
  var literal = value.match(/^"([^"]*)"$/);
  if (literal) return literal[1];
  var name = value.match(/^(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Z][A-Z0-9_]*)$/);
  if (name && Object.prototype.hasOwnProperty.call(consts, name[1])) return consts[name[1]];
  var formatted = value.match(/^format!\("([^"]*)"\)$/);
  if (formatted) {
    return formatted[1].replace(/\{([A-Z][A-Z0-9_]*)\}/g, function (whole, key) {
      return Object.prototype.hasOwnProperty.call(consts, key) ? consts[key] : whole;
    });
  }
  return null;
}

function readConsts() {
  if (CONST_CACHE) return CONST_CACHE;
  var values = Object.create(null);
  rustFiles().forEach(function (rel) {
    var text = src(rel);
    var pattern = /\b(?:pub(?:\([^)]*\))?\s+)?const\s+([A-Z][A-Z0-9_]*)\s*:\s*&\s*'?static\s+str\s*=\s*([\s\S]*?);/g;
    var match;
    while ((match = pattern.exec(text)) !== null) {
      var expr = match[2].trim();
      var literal = expr.match(/^"([^"]*)"$/);
      if (literal) {
        values[match[1]] = literal[1];
        continue;
      }
      var concat = expr.match(/^concat!\(([\s\S]*)\)$/);
      if (!concat) continue;
      var parts = concat[1].split(",").map(function (part) {
        return part.trim();
      });
      if (parts.every(function (part) { return /^"[^"]*"$/.test(part); })) {
        values[match[1]] = parts.map(function (part) {
          return part.slice(1, -1);
        }).join("");
      }
    }
  });
  CONST_CACHE = values;
  return values;
}

/** Every .route(path, method(handler)) registration in shipped Rust. */
function routeRegistrations() {
  if (ROUTE_CACHE) return ROUTE_CACHE;
  consts = readConsts();
  var out = [];
  rustFiles().forEach(function (rel) {
    var text = src(rel);
    var position = 0;
    while (true) {
      var index = text.indexOf(".route(", position);
      if (index < 0) break;
      var paren = parenBlock(text, index + ".route".length);
      position = paren.end;
      var comma = topLevelComma(paren.body);
      if (comma < 0) continue;
      var routePath = resolvePath(paren.body.slice(0, comma), consts);
      if (routePath === null) continue;
      var tail = paren.body.slice(comma + 1);
      var macro = /\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/g;
      var found;
      while ((found = macro.exec(tail)) !== null) {
        var fn = found[1];
        var verb = fn === "any" ? "any" : fn.replace(/^axum_/, "");
        if (verb !== "any" && METHODS.indexOf(verb) < 0) continue;
        var inner = parenBlock(tail, found.index + found[0].length - 1).body;
        var handlerMatch = inner.match(/^\s*([A-Za-z_][A-Za-z0-9_:]+)/);
        var handler = handlerMatch ? handlerMatch[1] : "?";
        var verbs = verb === "any" ? ANY_VERBS : [verb.toUpperCase()];
        verbs.forEach(function (method) {
          out.push({
            method: method,
            path: routePath,
            handler: handler,
            source: rel,
            line: lineOf(text, index),
          });
        });
      }
    }
  });
  ROUTE_CACHE = out;
  return out;
}

/** Every ApiEndpoint literal in the LLM endpoint inventory. */
function parseInventory(files) {
  if (!files && INVENTORY_CACHE) return INVENTORY_CACHE;
  var out = [];
  (files || inventoryFiles()).forEach(function (rel) {
    var text = src(rel);
    var position = 0;
    while (true) {
      var index = text.indexOf("ApiEndpoint", position);
      if (index < 0) break;
      var brace = text.indexOf("{", index);
      if (brace < 0) break;
      var depth = 0;
      var inString = false;
      var escaped = false;
      var end = brace;
      for (var i = brace; i < text.length; i++) {
        var ch = text.charAt(i);
        if (inString) {
          if (escaped) escaped = false;
          else if (ch === "\\") escaped = true;
          else if (ch === '"') inString = false;
          continue;
        }
        if (ch === '"') inString = true;
        else if (ch === "{") depth++;
        else if (ch === "}") {
          depth--;
          if (depth === 0) {
            end = i + 1;
            break;
          }
        }
      }
      var block = text.slice(brace, end);
      var fields = {};
      var fieldPattern = /\b(method|path|summary)\s*:\s*"([^"]*)"/g;
      var field;
      while ((field = fieldPattern.exec(block)) !== null) fields[field[1]] = field[2];
      position = end;
      if (fields.method && fields.path) {
        out.push({
          method: fields.method,
          path: fields.path,
          summary: fields.summary || "",
          source: rel,
          line: lineOf(text, index),
        });
      }
    }
  });
  if (!files) INVENTORY_CACHE = out;
  return out;
}

/** Curated (prompt-discoverable) ApiEndpoint list, separate from the inventory. */
function curatedCatalogFiles() {
  return walk("botserver/src/core/bot/api_catalog/commands_list/endpoints", ".rs");
}

function canonical(routePath) {
  return routePath.replace(
    /:([A-Za-z_][A-Za-z0-9_]*)|\{([A-Za-z_][A-Za-z0-9_]*)\}|\*([A-Za-z_][A-Za-z0-9_]*)/g,
    function (whole, colon, brace, star) { return "{" + (colon || brace || star) + "}"; }
  );
}

function paramStyle(routePath) {
  var styles = [];
  var pattern = /:([A-Za-z_][A-Za-z0-9_]*)|\{([A-Za-z_][A-Za-z0-9_]*)\}|\*([A-Za-z_][A-Za-z0-9_]*)/g;
  var match;
  while ((match = pattern.exec(routePath)) !== null) {
    styles.push(match[1] ? ":" : (match[2] ? "{}" : "*"));
  }
  return styles.join(",");
}

function report() {
  log("");
  log("=".repeat(74));
  log("ROUTE CONTRACT OFFLINE STATIC AUDIT (#2546)");
  log("=".repeat(74));
  log("Read-only scan of shipped Rust routers, endpoint inventory and LLM catalog.");
  log("No router is constructed, no HTTP request is made, no runtime PASS is claimed.");
  log("-".repeat(74));
  OBSERVED.forEach(function (line) { log("[OBSERVE] " + line); });
  log("-".repeat(74));
  RESULTS.forEach(function (r) {
    log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
  });
  if (FINDINGS.length) {
    log("-".repeat(74));
    FINDINGS.forEach(function (f) { log("[FINDING] " + f.name + " :: " + f.detail); });
  }
  log("=".repeat(74));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(74));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  fs, path, REPO_DIR, BOTSERVER, CATALOG_MOD, FIND_API, CI_WORKFLOW,
  ROUTE_ROOTS, INVENTORY_ROOT, METHODS, ANY_VERBS,
  log, test, assert, equal, finding, observe, exists, readRepo, src, walk,
  rustFiles, inventoryFiles, curatedCatalogFiles, lineOf, parenBlock,
  topLevelComma, resolvePath, readConsts, routeRegistrations, parseInventory,
  canonical, paramStyle, finish,
};
