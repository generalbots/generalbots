/**
 * #2743 — offline verification of the versioned chat sanitizer policy.
 *
 * The browser runs GBSanitize.selfTest() (real DOM, parse -> serialize ->
 * re-parse); this module runs the SAME corpus's policy expectations in Node's
 * vm so the bypass corpus is executable in CI without a DOM. The checksum
 * gate fails if chat-sanitize.js drifts from its recorded manifest.
 */
"use strict";

const crypto = require("crypto");
const {
  test,
  assert,
  readFile,
  loadScript,
} = require("./00_harness");

const SOURCE = "chat/chat-sanitize.js";
const MANIFEST = "chat/sanitize-manifest.json";
const sourceText = readFile(SOURCE);
const manifest = JSON.parse(readFile(MANIFEST));
const sandbox = loadScript(SOURCE, { window: {} });
const sanitizer = sandbox.GBSanitize;

test("SANITIZE #2743: version and checksum manifest match the shipped sanitizer", function () {
  assert(sanitizer && sanitizer.VERSION, "GBSanitize did not load");
  assert(manifest.version === sanitizer.VERSION,
    "manifest version " + manifest.version + " != code " + sanitizer.VERSION);
  assert(manifest.policy === "allowlist", "policy must be an allowlist");
  const actual = crypto.createHash("sha256").update(sourceText).digest("hex");
  assert(actual === manifest.sha256,
    "sanitizer checksum drift: " + actual + " != " + manifest.sha256);
});

test("SANITIZE #2743: every XSS/mXSS corpus payload is rejected by the policy", function () {
  assert(sanitizer.XSS_CORPUS && sanitizer.XSS_CORPUS.length >= 12,
    "bypass corpus missing or too small");
  assert(manifest.corpus.count === sanitizer.XSS_CORPUS.length,
    "manifest corpus count does not match code");
  const categories = new Set(sanitizer.XSS_CORPUS.map(function (p) { return p.category; }));
  manifest.corpus.categories.forEach(function (category) {
    assert(categories.has(category), "corpus category missing: " + category);
  });
  ["namespace", "nested-foreign", "attribute-mutation", "event-handler", "uri"].forEach(function (required) {
    assert(categories.has(required), "required regression family missing: " + required);
  });

  sanitizer.XSS_CORPUS.forEach(function (payload) {
    (payload.nodes || []).forEach(function (tag) {
      assert(sanitizer.tagAllowed(tag, sanitizer.HTML_NS) === false,
        payload.id + ": tag allowed: " + tag);
    });
    (payload.attrs || []).forEach(function (attr) {
      assert(sanitizer.attrAllowed(attr) === false,
        payload.id + ": attribute allowed: " + attr);
    });
    (payload.urls || []).forEach(function (url) {
      assert(sanitizer.uriIsAllowed(url) === false,
        payload.id + ": URL allowed: " + url);
    });
    (payload.styles || []).forEach(function (style) {
      assert(sanitizer.styleIsAllowed(style) === false,
        payload.id + ": style allowed: " + style);
    });
  });
});

test("SANITIZE #2743: dangerous schemes are blocked and feature schemes survive", function () {
  ["javascript:alert(1)", "vbscript:x", "livescript:x", "JaVa\tScRiPt:x",
    "data:text/html,<script>", "data:image/svg+xml,<svg>", "blob:https://x/y",
    "about:blank", "view-source:https://x"].forEach(function (url) {
    assert(sanitizer.uriIsAllowed(url) === false, "scheme not blocked: " + url);
  });
  ["data:image/png;base64,AAA", "data:image/jpeg;base64,AAA", "https://x/y",
    "/relative/path", "#anchor", "app://drive?a=b", "vmfile:foo/bar.png"].forEach(function (url) {
    assert(sanitizer.uriIsAllowed(url) === true, "legitimate URL blocked: " + url);
  });
});

test("SANITIZE #2743: namespace confusion is rejected, inert text is preserved", function () {
  [["p", sanitizer.HTML_NS, true], ["a", sanitizer.HTML_NS, true],
    ["script", sanitizer.HTML_NS, false], ["svg", sanitizer.HTML_NS, false],
    ["mglyph", sanitizer.HTML_NS, false]].forEach(function (row) {
    assert(sanitizer.tagAllowed(row[0], row[1]) === row[2],
      "tag decision wrong: " + row[0]);
  });
  assert(sanitizer.tagAllowed("p", sanitizer.MATH_NS) === false,
    "an HTML tag must not survive in the MathML namespace");
  assert(sanitizer.tagAllowed("mrow", sanitizer.MATH_NS) === true,
    "legitimate KaTeX MathML must survive");
  assert(sanitizer.tagAllowed("img", "http://www.w3.org/2000/svg") === false,
    "any SVG-namespace element must be removed");
});

test("SANITIZE #2743: parse/serialize fixed point and fail-closed are present", function () {
  assert(/for \(var pass = 0; pass < MAX_PASSES; pass\+\+\)/.test(sourceText),
    "idempotent re-sanitization loop missing");
  assert(sourceText.indexOf("if (output === current) return output;") !== -1,
    "fixed-point early return missing");
  assert(sourceText.indexOf("return escapeText(raw);") !== -1,
    "non-convergent input must fail closed to escaped text");
  assert(sourceText.indexOf("selfTest: selfTest") !== -1,
    "runtime DOM self-test missing");
  assert(sourceText.indexOf("http") === -1 || !/<script\s+src=["']https?:/i.test(sourceText),
    "no CDN script links allowed");
});

test("SANITIZE #2743: every innerHTML sink uses the shared sanitizer", function () {
  const sinks = {
    "chat/messages/50_message_b.js": "parsed = GBSanitize.html(parsed)",
    "chat/messages/70_stream.js": "parsed = GBSanitize.html(parsed)",
    "chat/modules/multichat/02_dom.js": "GBSanitize.html(parsed)",
    "chat/chat-artifacts.js": "GBSanitize.html(html)",
    "chat/projector-inline-state.js": "GBSanitize.html(parsed)",
  };
  Object.keys(sinks).forEach(function (file) {
    assert(readFile(file).indexOf(sinks[file]) !== -1,
      "unsanitized sink: " + file);
  });
});

test("SIZE #2743: touched sanitizer files stay under the 450-line limit", function () {
  [SOURCE, "tests/webtest-gb7/55_sanitize_xss.js"].forEach(function (file) {
    const lines = readFile(file).split("\n").length;
    assert(lines <= 450, file + " has " + lines + " lines (max 450)");
  });
});
