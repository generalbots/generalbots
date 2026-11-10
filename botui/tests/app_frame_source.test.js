"use strict";
/* Generated-app frame source (#2028 / #2107) — zero-dependency unit test.
 * run: node tests/app_frame_source.test.js
 *
 * The desktop webview's page origin is a custom scheme (`tauri://localhost`),
 * whose hostname reads `localhost`. A hostname-only loopback test called that
 * page loopback, so the inline document was refused and the card kept the
 * unreachable `http://127.0.0.1:8081` URL ("App could not be reached").
 */
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULE = path.join(__dirname, "..", "ui", "suite", "chat", "modules", "73_app_frame.js");
const SANDBOX_URL = "http://127.0.0.1:8081/sandbox/ui/9f0a?t=tok";
const SERVER_URL = "http://127.0.0.1:8080/sandbox/ui/9f0a?t=tok";

let passed = 0;
let failed = 0;

function check(condition, label) {
  if (condition) passed++;
  else { failed++; console.error("FAIL " + label); }
}

function loadAppFrame(pageHref, serverOrigin) {
  const sandbox = {
    console: console,
    Date: Date,
    Math: Math,
    JSON: JSON,
    Promise: Promise,
    URL: URL,
    Blob: Blob,
    Uint8Array: Uint8Array,
    setTimeout: function () {},
    clearTimeout: function () {},
    atob: function (value) { return Buffer.from(value, "base64").toString("binary"); },
    fetch: function () { return Promise.resolve({ ok: true }); },
    document: { createElement: function () { return null; }, getElementById: function () { return null; } },
    addEventListener: function () {},
    location: { href: pageHref, protocol: pageHref.split(":")[0] + ":", hostname: "" },
  };
  sandbox.window = sandbox;
  if (serverOrigin) sandbox.__GB_SERVER_ORIGIN__ = serverOrigin;
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(MODULE, "utf8"), sandbox, { filename: "73_app_frame.js" });
  return sandbox.AppFrame;
}

const payload = { url: SANDBOX_URL, title: "Weather" };
const payloadWithInline = { url: SANDBOX_URL, title: "Weather", html_base64: "PGgxPmhpPC9oMT4=" };

/* ── page classification table ─────────────────────────────────────────── */

const pages = [
  ["tauri://localhost/suite/desktop.html", false, true],
  ["asset://localhost/suite/desktop.html", false, true],
  ["http://tauri.localhost/suite/desktop.html", false, true],
  ["http://localhost:3000/chat", true, false],
  ["http://127.0.0.1:5858/suite/desktop.html", true, false],
  ["https://beiner.generalbots.org/drive", false, false],
];

pages.forEach(function (row) {
  const pageHref = row[0];
  const frame = loadAppFrame(pageHref);
  check(frame.loopbackHost(pageHref) === row[1],
    "loopbackHost(" + pageHref + ") = " + row[1]);
  check(frame.desktopAssetOrigin(pageHref) === row[2],
    "desktopAssetOrigin(" + pageHref + ") = " + row[2]);
});

/* ── desktop asset origins: inline doc first, then the sidecar origin ───── */

["tauri://localhost/suite/desktop.html", "asset://localhost/suite/desktop.html",
  "http://tauri.localhost/suite/desktop.html"].forEach(function (pageHref) {
  const frame = loadAppFrame(pageHref);
  check(frame.prefersInline(payloadWithInline, pageHref) === true,
    "inline document wins on " + pageHref);
  check(frame.frameSource(payloadWithInline, pageHref) === null,
    "no network source when the inline document exists on " + pageHref);
  check(frame.frameSource(payload, pageHref) === SERVER_URL,
    "loopback 8081 URL is rewritten to the sidecar origin on " + pageHref);
});

/* ── real loopback pages keep the URL untouched ─────────────────────────── */

["http://localhost:3000/chat", "http://127.0.0.1:5858/suite/desktop.html"].forEach(function (pageHref) {
  const frame = loadAppFrame(pageHref);
  check(frame.prefersInline(payload, pageHref) === false, "loopback page keeps the URL on " + pageHref);
  check(frame.frameSource(payload, pageHref) === SANDBOX_URL, "loopback URL unchanged on " + pageHref);
});

/* ── a remote page cannot reach loopback: use the inline document ───────── */

const remote = loadAppFrame("https://beiner.generalbots.org/drive");
check(remote.prefersInline(payload, "https://beiner.generalbots.org/drive") === true,
  "remote page prefers the inline document");
check(remote.frameSource(payload, "https://beiner.generalbots.org/drive") === null,
  "remote page never loads a loopback URL");
check(remote.frameSource({ url: "https://cdn.example/app.html" }, "https://beiner.generalbots.org/drive")
  === "https://cdn.example/app.html", "non-loopback URL is loaded as-is");

/* ── configured shell origin wins over the 127.0.0.1:8080 default ───────── */

const configured = loadAppFrame("tauri://localhost/suite/desktop.html", "http://127.0.0.1:5858/");
check(configured.frameSource(payload, "tauri://localhost/suite/desktop.html")
  === "http://127.0.0.1:5858/sandbox/ui/9f0a?t=tok",
  "__GB_SERVER_ORIGIN__ drives the rewrite");

console.log("\nApp frame source: " + passed + " passed, " + failed + " failed");
process.exit(failed ? 1 : 0);
