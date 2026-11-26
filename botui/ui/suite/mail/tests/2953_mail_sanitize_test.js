/**
 * #2953 — offline proof that the Mail app cannot load an external asset.
 *
 * Run with: node botui/ui/suite/mail/tests/2953_mail_sanitize_test.js
 *
 * The hostile HTML corpus lives in modules/00_mail_sanitize.js and is executed
 * by `GBMailSanitize.selfTest()` (no DOM required). This runner also scans the
 * shipped Mail chrome for external stylesheet/font/pixel URLs and verifies the
 * sanitizer is loaded before the module that inserts signature HTML.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var MAIL_DIR = path.resolve(__dirname, "..");
var SANITIZER = path.join(MAIL_DIR, "modules/00_mail_sanitize.js");
var failures = [];

function check(condition, message) {
  if (condition) {
    console.log("PASS: " + message);
  } else {
    failures.push(message);
    console.log("FAIL: " + message);
  }
}

var source = fs.readFileSync(SANITIZER, "utf8");
var sandbox = { window: {}, console: console };
vm.runInNewContext(source, sandbox, { filename: "00_mail_sanitize.js" });
var api = sandbox.window.GBMailSanitize;

check(!!api, "GBMailSanitize is exposed on window");
if (!api) process.exit(1);

var result = api.selfTest();
check(result.pass,
  "hostile HTML self-test: " + result.total + " cases, " + result.failures.length + " failures");
result.failures.forEach(function (line) { console.log("  " + line); });
check(api.HOSTILE_CORPUS.length >= 8, "self-test covers remote img/link/@font-face/pixel cases");
var fragment = '<div class="mail-content-view"><button onclick="reply()">Reply</button>' +
  '<div class="mail-body"><p>Hi</p><img src="https://tracking.inflection.io/px.gif"></div></div>';
var cleanedFragment = api.cleanFragment(fragment);
check(cleanedFragment.indexOf("tracking.inflection.io") === -1
  && cleanedFragment.indexOf('onclick="reply()"') !== -1,
  "pre-swap fragment loses the pixel but keeps mail chrome handlers");
var attrValue = api.escapeJsAttr("O'Brien <img src=https://cdn.openai.com/x.png>");
check(attrValue.indexOf("\\'") !== -1 && attrValue.indexOf("&lt;img") !== -1
  && attrValue.indexOf("<img") === -1,
  "inline-handler values are HTML/JS escaped without breaking apostrophes");
check(source.indexOf("root.GBSanitize") !== -1 && source.indexOf("shared.html(scrubbed)") !== -1,
  "shared GBSanitize policy is reused when available");
check(source.indexOf("escapeText(scrubbed)") !== -1, "fails closed when GBSanitize is absent");

var chromeFiles = ["mail.html"]
  .concat(fs.readdirSync(path.join(MAIL_DIR, "partials")).map(function (name) {
    return "partials/" + name;
  }))
  .concat(["mail.css"])
  .concat(fs.readdirSync(path.join(MAIL_DIR, "mail")).map(function (name) {
    return "mail/" + name;
  }));

var external = /(?:https?:)?\/\/[^\s"')>]+/gi;
chromeFiles.forEach(function (relative) {
  var text = fs.readFileSync(path.join(MAIL_DIR, relative), "utf8");
  var matches = text.match(external) || [];
  check(matches.length === 0,
    relative + " has no external URL" + (matches.length ? " -> " + matches.join(", ") : ""));
});

var mailHtml = fs.readFileSync(path.join(MAIL_DIR, "mail.html"), "utf8");
var contentHtml = fs.readFileSync(path.join(MAIL_DIR, "partials/03_content.html"), "utf8");
[["mail.html", mailHtml], ["partials/03_content.html", contentHtml]].forEach(function (entry) {
  var name = entry[0];
  var text = entry[1];
  check(text.indexOf("chat/chat-sanitize.js") !== -1, name + " loads the shared sanitizer");
  check(text.indexOf("modules/00_mail_sanitize.js") !== -1, name + " loads the Mail sanitizer");
  check(text.indexOf("modules/00_mail_sanitize.js") < text.indexOf("modules/02_panels_compose.js"),
    name + " loads the Mail sanitizer before HTML insertion code");
});
var compose = fs.readFileSync(path.join(MAIL_DIR, "modules/02_panels_compose.js"), "utf8");
check(compose.indexOf("GBMailSanitize.html(sig.content_html)") !== -1,
  "signature HTML is sanitized before insertion");
var extras = fs.readFileSync(path.join(MAIL_DIR, "modules/05_init_extras.js"), "utf8");
check(extras.indexOf("GBMailSanitize.escapeText") !== -1
  && extras.indexOf("GBMailSanitize.escapeJsAttr") !== -1,
  "API-produced suggestion text is escaped before insertion");

console.log("\n" + (failures.length ? "FAILURES: " + failures.length : "ALL #2953 CHECKS PASSED"));
process.exit(failures.length ? 1 : 0);
