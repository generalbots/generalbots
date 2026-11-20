/**
 * Offline harness for #2482 docs rich-table helper verification.
 * Run with: node botui/ui/suite/tests/webtest-docs-table.js
 *
 * Mirrors webtest-docs-find/00_harness.js (ordered modules, counters,
 * assertions, vm loader, report) and adds a deliberately small fake DOM:
 * elements are plain objects with childNodes/parentNode/attributes, and
 * outerHTML assignment is parsed back into a fresh fake tree. The REAL
 * shipped docs/modules/33*_tables* modules are loaded into a vm sandbox;
 * no production code is copied or modified here.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var TESTS_DIR = path.resolve(__dirname, "..");
var PASS = 0;
var FAIL = 0;
var RESULTS = [];

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
    RESULTS.push({ name: name, status: "FAIL", error: e.message });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function fileExists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

function loadScript(relPath, sandbox) {
  var context = vm.isContext(sandbox) ? sandbox : vm.createContext(sandbox);
  vm.runInContext(readFile(relPath), context, { filename: relPath });
  return context;
}

/* ---- minimal DOM doubles -------------------------------------------- */

function attributes(tagText) {
  var out = [];
  var re = /([^\s=/>]+)="([^"]*)"/g;
  var match = re.exec(tagText);
  while (match) {
    out.push([match[1], match[2]]);
    match = re.exec(tagText);
  }
  return out;
}

function element(tagName, initialAttrs) {
  var el = {
    nodeType: 1,
    tagName: String(tagName).toUpperCase(),
    childNodes: [],
    parentNode: null,
    dataset: {},
    _attrs: (initialAttrs || []).map(function (pair) { return [pair[0], String(pair[1])]; }),
    _html: "",
  };

  Object.defineProperty(el, "attributes", {
    get: function () {
      return el._attrs.map(function (pair) { return { name: pair[0], value: pair[1] }; });
    },
  });
  Object.defineProperty(el, "textContent", {
    get: function () {
      if (!el.childNodes.length) return el._html;
      return el.childNodes.map(function (child) {
        return child.nodeType === 1 ? child.textContent : child.data;
      }).join("");
    },
  });

  el.getAttribute = function (name) {
    for (var i = 0; i < el._attrs.length; i++) {
      if (el._attrs[i][0] === name) return el._attrs[i][1];
    }
    return null;
  };
  el.setAttribute = function (name, value) {
    for (var i = 0; i < el._attrs.length; i++) {
      if (el._attrs[i][0] === name) {
        el._attrs[i][1] = String(value);
        return;
      }
    }
    el._attrs.push([name, String(value)]);
  };
  el.removeAttribute = function (name) {
    el._attrs = el._attrs.filter(function (pair) { return pair[0] !== name; });
  };
  el.appendChild = function (child) {
    child.parentNode = el;
    el.childNodes.push(child);
    return child;
  };
  el.replaceChild = function (oldChild, fresh) {
    var i = el.childNodes.indexOf(oldChild);
    if (i < 0) return null;
    el.childNodes[i] = fresh;
    fresh.parentNode = el;
    oldChild.parentNode = null;
    return oldChild;
  };
  el.contains = function (node) {
    var cursor = node;
    while (cursor) {
      if (cursor === el) return true;
      cursor = cursor.parentNode;
    }
    return false;
  };
  Object.defineProperty(el, "innerHTML", {
    get: function () {
      if (!el.childNodes.length) return el._html;
      return el.childNodes.map(function (child) {
        return child.nodeType === 1 ? child.outerHTML : child.data;
      }).join("");
    },
    set: function (html) {
      el.childNodes = [];
      el._html = String(html);
    },
  });
  Object.defineProperty(el, "outerHTML", {
    get: function () {
      var attrs = el._attrs.map(function (pair) {
        return " " + pair[0] + '="' + pair[1] + '"';
      }).join("");
      return "<" + el.tagName.toLowerCase() + attrs + ">" + el.innerHTML +
        "</" + el.tagName.toLowerCase() + ">";
    },
    set: function (html) {
      el._onWrite(String(html));
    },
  });
  el._onWrite = function () {};

  el.querySelectorAll = function (selector) {
    var wanted = selector.split(",").map(function (s) { return s.trim().toLowerCase(); });
    var found = [];
    (function walk(node) {
      node.childNodes.forEach(function (child) {
        if (child.nodeType !== 1) return;
        if (wanted.indexOf(child.tagName.toLowerCase()) >= 0) found.push(child);
        walk(child);
      });
    })(el);
    return found;
  };
  el.querySelector = function (selector) {
    var match = /^\[data-docs-tables-id="([^"]+)"\]$/.exec(selector);
    if (!match) return null;
    var found = null;
    (function walk(node) {
      node.childNodes.forEach(function (child) {
        if (found || child.nodeType !== 1) return;
        if (child.getAttribute("data-docs-tables-id") === match[1]) found = child;
        else walk(child);
      });
    })(el);
    return found;
  };
  return el;
}

/** Parse a serialized <table> back into a fresh fake tree (outerHTML double). */
function parseTableHtml(html) {
  var tableMatch = /^<table([^>]*)>([\s\S]*)<\/table>$/.exec(String(html));
  if (!tableMatch) return null;
  var table = element("table", attributes(tableMatch[1]));
  var sectionRe = /<(thead|tbody|tfoot)([^>]*)>([\s\S]*?)<\/\1>/g;
  var sectionMatch = sectionRe.exec(tableMatch[2]);
  while (sectionMatch) {
    var section = element(sectionMatch[1], attributes(sectionMatch[2]));
    var rowRe = /<tr([^>]*)>([\s\S]*?)<\/tr>/g;
    var rowMatch = rowRe.exec(sectionMatch[3]);
    while (rowMatch) {
      var row = element("tr", attributes(rowMatch[1]));
      var cellRe = /<(td|th)([^>]*)>([\s\S]*?)<\/\1>/g;
      var cellMatch = cellRe.exec(rowMatch[2]);
      while (cellMatch) {
        var cell = element(cellMatch[1], attributes(cellMatch[2]));
        cell.innerHTML = cellMatch[3];
        row.appendChild(cell);
        cellMatch = cellRe.exec(rowMatch[2]);
      }
      section.appendChild(row);
      rowMatch = rowRe.exec(sectionMatch[3]);
    }
    table.appendChild(section);
    sectionMatch = sectionRe.exec(tableMatch[2]);
  }
  return table;
}

function makeRange(intersects) {
  return {
    _intersects: (intersects || []).slice(),
    _contents: null,
    _node: null,
    commonAncestorContainer: null,
    intersectsNode: function (node) { return this._intersects.indexOf(node) >= 0; },
    selectNode: function (node) {
      this._node = node;
      this.commonAncestorContainer = node.parentNode || node;
    },
    selectNodeContents: function (node) {
      this._contents = node;
      this._intersects = [node];
      this.commonAncestorContainer = node;
    },
    collapse: function () { this._collapsed = true; },
  };
}

function makeSelection(ranges) {
  var list = (ranges || []).slice();
  return {
    anchorNode: null,
    focusNode: null,
    _ranges: list,
    get rangeCount() { return list.length; },
    getRangeAt: function (i) { return list[i]; },
    removeAllRanges: function () { list.length = 0; },
    addRange: function (range) { list.push(range); },
    setRanges: function (next) {
      list.length = 0;
      (next || []).forEach(function (range) { list.push(range); });
    },
  };
}

/**
 * Load both real table modules into one vm context with stubbed browser
 * globals. opts.execCommand === false forces the outerHTML fallback path.
 */
function tableEnvironment(opts) {
  var options = opts || {};
  var events = { saves: 0, writes: [], formats: [], refreshes: [] };
  var article = element("article", [["contenteditable", "true"]]);
  article.dataset.docId = "doc-1";
  var selection = makeSelection(options.ranges);
  var documentStub = {
    createElement: function (tag) { return element(tag); },
    createRange: function () { return makeRange(); },
    querySelector: function (selector) {
      return selector === "article[contenteditable]" ? article : null;
    },
  };
  if (options.execCommand !== false) documentStub.execCommand = function () { return true; };
  var windowStub = {
    getSelection: function () { return selection; },
    DocsTablesUI: {
      refresh: function (table) { events.refreshes.push(table); },
    },
  };
  var sandbox = {
    window: windowStub,
    document: documentStub,
    console: console,
    scheduleSave: function () { events.saves++; },
    updatePageCount: function () {},
    gbFormat: function (command, html) {
      events.formats.push({ command: command, html: html });
      var last = selection._ranges[selection._ranges.length - 1];
      var target = last && last._node;
      var parsed = parseTableHtml(html);
      if (!target || !target.parentNode || !parsed) return false;
      target.parentNode.replaceChild(target, parsed);
      return true;
    },
  };
  loadScript("docs/modules/33_tables_model.js", sandbox);
  loadScript("docs/modules/33b_tables_commit.js", sandbox);
  return {
    events: events,
    article: article,
    selection: selection,
    window: windowStub,
    NS: windowStub.DocsTables,
    parse: parseTableHtml,
    attach: function (table) {
      article.appendChild(table);
      table._onWrite = function (html) {
        events.writes.push(html);
        var parsed = parseTableHtml(html);
        if (parsed) article.replaceChild(table, parsed);
      };
      return table;
    },
  };
}

function report() {
  log("\n" + "=".repeat(60));
  log("DOCS RICH TABLE OFFLINE RESULTS (#2482)");
  log("=".repeat(60));
  RESULTS.forEach(function (r) {
    log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
  });
  log("=".repeat(60));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(60));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  fs,
  path,
  SUITE_DIR,
  TESTS_DIR,
  test,
  assert,
  readFile,
  fileExists,
  loadScript,
  element,
  parseTableHtml,
  makeRange,
  makeSelection,
  tableEnvironment,
  log,
  finish,
};
