"use strict";
/* #2812 — one-time SSO handoff: tokens in fragments, state replay protection.
 * run: node tests/cloud_auth_handoff.test.js
 */
const SSO = require("../ui/login/js/sso-handoff.js");
const auth = require("../ui/login/js/cloud-auth.js");

let failed = 0;
function check(condition, label) {
  if (condition) console.log("PASS " + label);
  else { failed++; console.error("FAIL " + label); }
}

const now = 1700000000000;
const clock = function () { return now; };

// One-time semantics: a consumed state cannot be replayed.
const doc = { _value: "", get cookie() { return this._value; }, set cookie(v) {
  const m = /^([^=]+)=([^;]*)/.exec(String(v));
  if (m && m[1] === SSO.KEY) this._value = m[2] ? m[1] + "=" + m[2] : "";
} };
const h = SSO.create({ _now: clock, _doc: doc, _host: "localhost" });
check(SSO.consume(h.state, h.ts, { _doc: doc, _now: clock }) === true, "first state use accepted");
check(SSO.consume(h.state, h.ts, { _doc: doc, _now: clock }) === false, "replayed state rejected (one-time)");

// Expiry and clock skew.
check(SSO.valid("s", String(now - 60001), "s." + (now - 60001), now) === false, "expired state rejected");
check(SSO.valid("s", String(now + 6000), "s." + (now + 6000), now) === false, "future state rejected");

// Hostile inputs never throw or validate.
const hostile = ["", "..", "a.b.c", "abc", "-1", "0", "99999999999999999999", "1e3", " 123 ", null, undefined, {}, []];
for (const v of hostile) {
  check(SSO.valid("s", v, "s." + v, now) === false, "hostile ts " + JSON.stringify(v) + " rejected");
}
check(SSO.valid("", String(now), "s." + now, now) === false, "empty state rejected");
check(SSO.valid("s", String(now), "%ZZ", now) === false, "malformed cookie rejected");
check(SSO.valid("s", String(now), "s." + now + ".extra", now) === false, "extra cookie parts rejected");

// The token never lands in a query string, including when injected via redirect.
const dest = auth.buildAuthDestination(
  new URL("https://cloud.example/dashboard?token=INJECTED#old"),
  { token: "SECRET", email: "e@x", name: "n" },
  { state: "s", ts: now },
  "https://login.example"
);
const parsed = new URL(dest);
check(parsed.search === "", "query string cleared (no bearer token sent to server)");
check(parsed.hash.indexOf("gb_token=SECRET") !== -1, "token present in fragment only");
check(parsed.hash.indexOf("gb_state=s") !== -1, "state present in fragment");
check(parsed.hash.indexOf("INJECTED") === -1 && parsed.hash.indexOf("old") === -1, "injected params discarded");

// Browser call sites omit _doc: gbSsoCreate must default to the page document
// so the receiver can consume the one-time state and keep the token.
const browserDoc = { _value: "", get cookie() { return this._value; }, set cookie(v) {
  const m = /^([^=]+)=([^;]*)/.exec(String(v));
  if (m && m[1] === SSO.KEY) this._value = m[2] ? m[1] + "=" + m[2] : "";
} };
global.document = browserDoc;
global.location = { hostname: "login.pragmatismo.com.br", protocol: "https:" };
const bh = SSO.create({ _host: "login.pragmatismo.com.br", _secure: true });
check(browserDoc.cookie.indexOf(SSO.KEY + "=") === 0, "browser handoff writes state cookie without an explicit _doc");
check(SSO.consume(bh.state, bh.ts, { _doc: browserDoc }) === true, "browser-issued state is consumable");
delete global.document;
delete global.location;

// Trusted-origin parser refuses hostile schemes/credentials.
check(SSO.trustedOrigin("javascript:alert(1)", "https://login.example") === null, "javascript: origin rejected");
check(SSO.trustedOrigin("https://u:p@cloud.example", "https://login.example") === null, "credentialed origin rejected");
check(SSO.trustedOrigin("https://cloud.example", "https://login.example") === "https://cloud.example", "cloud origin accepted");

// Shared-cookie domain must be the registrable domain, never a public suffix.
check(SSO.cookieDomain("localhost") === "", "localhost uses a host-only cookie");
check(SSO.cookieDomain("127.0.0.1") === "", "loopback IP uses a host-only cookie");
check(SSO.cookieDomain("login.pragmatismo.com.br") === ".pragmatismo.com.br", "login host shares registrable domain");
check(SSO.cookieDomain("cloud.pragmatismo.com.br") === ".pragmatismo.com.br", "cloud host shares registrable domain");
check(SSO.cookieDomain("") === "" && SSO.cookieDomain("..") === "", "hostile host yields no domain");

const self = SSO.selfTest();
check(self.failed === 0, "in-browser self-test (" + self.passed + " passed, " + self.failed + " failed)");
for (const r of self.results) if (!r.pass) console.error("  " + r.input);

console.log("cloud_auth_handoff: " + (failed === 0 ? "ALL PASS" : failed + " FAILED"));
process.exit(failed === 0 ? 0 : 1);
