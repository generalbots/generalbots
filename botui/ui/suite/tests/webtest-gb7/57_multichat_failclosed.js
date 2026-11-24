/**
 * #2707 — runtime proof that both chat UIs fail closed when GBSanitize is
 * missing. Unlike the source-string checks, this module EXECUTES the render
 * paths in a Node vm with a minimal DOM shim and real XSS payloads, asserting
 * that no payload is ever assigned as raw HTML to innerHTML.
 *
 * Run with: node botui/ui/suite/tests/webtest_gb7.js
 */
"use strict";

const { test, assert, loadScript } = require("./00_harness");

// XSS payloads that must never become executable markup in a chat bubble.
const PAYLOADS = [
  '<img src=x onerror="window.__XSS__=1">',
  '<script>window.__XSS__=1</script>',
  '<svg><foreignObject><img onerror=window.__XSS__=1></foreignObject></svg>',
  '<a href="javascript:window.__XSS__=1">click</a>',
  '<iframe src="javascript:window.__XSS__=1"></iframe>',
  '<body onload="window.__XSS__=1">',
];

function escapeText(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

// Small DOM stand-in: enough for appendToContainer + escapeHtml. textContent
// mirrors the browser by producing escaped HTML, so raw tags are detectable.
function FakeEl(tag) {
  this.tagName = String(tag || "div").toUpperCase();
  this.children = [];
  this.className = "";
  this.id = "";
  this.scrollTop = 0;
  this.scrollHeight = 0;
  this.isConnected = true;
  this.style = { cssText: "" };
  this._text = "";
  this._html = "";
}
FakeEl.prototype.appendChild = function (child) {
  this.children.push(child);
  return child;
};
FakeEl.prototype.querySelectorAll = function () {
  return [];
};
FakeEl.prototype.querySelector = function () {
  return null;
};
Object.defineProperty(FakeEl.prototype, "textContent", {
  get: function () {
    return this._text + this.children.map(function (c) { return c.textContent; }).join("");
  },
  set: function (value) {
    this._text = String(value);
    this.children = [];
    this._html = escapeText(this._text);
  },
});
Object.defineProperty(FakeEl.prototype, "innerHTML", {
  get: function () { return this._html; },
  set: function (value) {
    this._html = String(value);
    this._text = "";
    this.children = [];
  },
});

function renderMultichat(payload, withSanitizer) {
  var win = {
    GBMultiChat: {},
    GBSanitize: withSanitizer
      ? { html: function (html) { return "<b>" + html + "</b>"; } }
      : undefined,
  };
  var sandbox = {
    window: win,
    document: { createElement: function (tag) { return new FakeEl(tag); } },
    marked: { parse: function (text) { return text; } },
    setTimeout: function () {},
  };
  loadScript("chat/modules/multichat/02_dom.js", sandbox);
  var container = new FakeEl("main");
  container.__msgs = new FakeEl("main");
  var div = win.GBMultiChat.appendToContainer(container, "bot", payload, "m1");
  return div.children[0];
}

function renderMinimal(payload, withSanitizer) {
  var win = {
    GBSanitize: withSanitizer
      ? { html: function (html) { return "<b>" + html + "</b>"; } }
      : undefined,
  };
  var sandbox = {
    window: win,
    document: { createElement: function (tag) { return new FakeEl(tag); } },
    console: console,
  };
  loadScript("../minimal/js/app/02_messages.js", sandbox);
  return sandbox.safeHtml(payload);
}

test("ISSUE #2707: multichat renders XSS payloads as inert text when GBSanitize is absent", function () {
  PAYLOADS.forEach(function (payload) {
    var inner = renderMultichat(payload, false);
    assert(inner.textContent === payload,
      "payload not rendered as plain text: " + payload);
    assert(inner.innerHTML.indexOf("<img") === -1 &&
           inner.innerHTML.indexOf("<script") === -1 &&
           inner.innerHTML.indexOf("<svg") === -1 &&
           inner.innerHTML.indexOf("<iframe") === -1 &&
           inner.innerHTML.indexOf("<body") === -1,
      "raw HTML reached innerHTML: " + inner.innerHTML);
  });
});

test("ISSUE #2707: multichat still uses the sanitizer when it is present", function () {
  var inner = renderMultichat(PAYLOADS[0], true);
  assert(inner.innerHTML.indexOf("<b>") === 0,
    "sanitizer branch did not receive the parsed payload");
});

test("ISSUE #2707: minimal client escapes XSS payloads when GBSanitize is absent", function () {
  PAYLOADS.forEach(function (payload) {
    var rendered = renderMinimal(payload, false);
    assert(rendered.indexOf("<img") === -1 &&
           rendered.indexOf("<script") === -1 &&
           rendered.indexOf("<svg") === -1 &&
           rendered.indexOf("<iframe") === -1 &&
           rendered.indexOf("<body") === -1,
      "minimal client emitted raw HTML: " + rendered);
    assert(rendered.indexOf("&lt;") !== -1,
      "minimal client did not escape the payload: " + rendered);
  });
});

test("ISSUE #2707: minimal client uses the sanitizer when it is present", function () {
  assert(renderMinimal(PAYLOADS[0], true).indexOf("<b>") === 0,
    "minimal sanitizer branch not used");
});
