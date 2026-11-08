/**
 * webtest_gb7 — behavioural tests that execute the shipped functions
 * Split from webtest_gb7.js for #1731; keeps the original test order.
 */
"use strict";

const {
  test,
  assert,
  readFile,
  loadScript,
  memoryStorage,
} = require("./00_harness");

// ═══════════════════════════════════════════════════════════════════
// BEHAVIOURAL TESTS (execute the shipped functions, not just source text)
// ═══════════════════════════════════════════════════════════════════

test("BEHAVE GB7-09: artifact detector classifies prose, markdown, HTML and code", function () {
  var window = {};
  var context = loadScript("chat/chat-artifacts.js", {
    window: window,
    document: {},
    console: console,
  });
  var artifacts = context.window.GBArtifacts;
  assert(artifacts, "GBArtifacts was not exported");
  assert(artifacts.detect("too short") === null, "short text must not become an artifact");
  var prose = "This paragraph explains the quarterly result in detail. ".repeat(60);
  assert(artifacts.detect(prose) && artifacts.detect(prose).kind === "markdown",
    "a long prose answer must be offered as a document");
  var code = "```js\n" + "var x = 1;\n".repeat(60) + "```";
  assert(artifacts.detect(code).kind === "code", "fenced code must stay a code artifact");
  assert(artifacts.detect("<html><body>hi</body></html>").kind === "html", "HTML page not detected");
  assert(artifacts.slug("Meu Relatório de Vendas") === "meu-relatorio-de-vendas",
    "slug must transliterate the title");
});

test("BEHAVE GB7-09: artifact HTML sanitizer strips scripts and handlers", function () {
  function node() {
    return {
      attrs: [],
      removed: false,
      remove: function () { this.removed = true; },
      get attributes() { return this.attrs; },
      removeAttribute: function (name) { this.attrs = this.attrs.filter(function (a) { return a.name !== name; }); },
    };
  }
  var nodes = [];
  var document = {
    createElement: function () {
      return {
        innerHTML: "",
        content: {
          querySelectorAll: function (selector) {
            if (selector === "*") return nodes;
            return nodes;
          },
        },
      };
    },
  };
  var context = loadScript("chat/chat-artifacts.js", { window: {}, document: document, console: console });
  var evil = node();
  evil.attrs = [{ name: "onclick", value: "alert(1)" }, { name: "href", value: "javascript:alert(2)" }];
  nodes.push(evil);
  var cleaned = context.window.GBArtifacts.safeHtml("<a onclick='alert(1)'>x</a>");
  assert(evil.attrs.length === 0, "event handler and javascript: URL must be removed");
  assert(typeof cleaned === "string", "sanitizer must return markup");
});

test("BEHAVE GB7-25: usage tracking counts a message once and totals it", function () {
  var storage = memoryStorage();
  var window = {};
  window.addEventListener = function () {};
  var context = loadScript("js/usage-stats.js", {
    window: window,
    document: { addEventListener: function () {}, getElementById: function () { return null; } },
    localStorage: storage,
    sessionStorage: memoryStorage(),
    console: console,
  });
  var usage = context.window.GBUsageStats;
  assert(usage, "GBUsageStats was not exported");
  usage.track("user", 120, "auto", "m-1");
  usage.track("user", 120, "auto", "m-1");
  usage.track("bot", 240, "auto", "m-2");
  var summary = usage.getSummary(1);
  assert(summary.messages_sent === 1, "duplicate frame must be counted once, got " + summary.messages_sent);
  assert(summary.messages_received === 1, "bot reply not counted");
  assert(summary.tokens_estimated === 90, "4 chars/token estimate: " + summary.tokens_estimated);
});

test("BEHAVE GB7-13: share expiry offers 7 days and refuses a past date", function () {
  var selectValue = "7";
  var context = loadScript("drive/modules/08_modals.js", {
    window: { location: { origin: "http://localhost:8080" } },
    document: { getElementById: function (id) { return id === "share-expiry" ? { value: selectValue } : null; } },
    localStorage: memoryStorage(),
    sessionStorage: memoryStorage(),
    console: console,
  });
  var iso = context.shareExpiryIso();
  var when = Date.parse(iso);
  var days = (when - Date.now()) / 86400000;
  assert(days > 6.9 && days < 7.1, "expiry must be ~7 days, got " + days);
  selectValue = "never";
  assert(context.shareExpiryIso() === null, "never must send no expiry");
  assert(context.isExpired("2000-01-01T00:00:00Z") === true, "past link must be expired");
  assert(context.isExpired(null) === false, "no expiry must not be expired");
});

test("BEHAVE #1638: Drive search asks for every bucket and file content", async function () {
  var requested = null;
  var context = loadScript("drive/modules/05_files.js", {
    URLSearchParams: URLSearchParams,
    document: { getElementById: function () { return { innerHTML: "" }; } },
    apiRequest: function (endpoint) {
      requested = endpoint;
      return Promise.resolve([
        { name: "report.pdf", path: "docs/report.pdf", is_dir: false, size: 10, bucket: "bot.gbai" },
      ]);
    },
    renderFiles: function () {},
    showNotification: function () {},
    escapeHtml: function (value) { return String(value); },
    humanizeError: function (value) { return String(value); },
    currentPath: "",
    currentBucket: "bot.gbai",
    currentScope: "user",
    getEffectiveBucket: function () { return "bot.gbai"; },
    console: console,
  });
  return context.searchFiles("report", true).then(function () {
    assert(requested, "search did not call the API");
    assert(requested.indexOf("all_buckets=true") !== -1, "cross-bucket flag missing: " + requested);
    assert(requested.indexOf("content=true") !== -1, "content flag missing: " + requested);
    assert(requested.indexOf("query=report") !== -1, "query missing: " + requested);
  });
});
