"use strict";
/* #2812 — one-time, expiring SSO handoff helpers.
 *
 * A bearer token must never appear in a URL query string: query strings are
 * recorded by proxies, server logs and Referer headers and stay in history.
 * The token is handed over in the URL *fragment* (never transmitted to a
 * server) and paired with a short-lived, single-use state value kept in a
 * cookie shared by the login and cloud/suite subdomains. A replayed fragment
 * carries a state that was already consumed, so it is rejected client-side.
 *
 * Pure functions take injectable `_now`, `_doc`, `_cookie`, `_host` overrides
 * so the Node self-test runs without a browser.
 */
var GB_SSO_KEY = "gb_sso_state";
var GB_SSO_TTL_MS = 60000;
var GB_SSO_SKEW_MS = 5000;

function gbSsoNonce() {
  if (typeof crypto !== "undefined" && crypto.getRandomValues) {
    var bytes = new Uint8Array(16);
    crypto.getRandomValues(bytes);
    var hex = "";
    for (var i = 0; i < bytes.length; i++) hex += ("0" + bytes[i].toString(16)).slice(-2);
    return hex;
  }
  return String(Date.now()) + "-" + String(Math.random()).slice(2) + "-" + String(Math.random()).slice(2);
}

// Cookies ignore ports, so a host-only cookie on localhost spans :5000/:4000.
// In production the login and cloud hosts share an eTLD+1 (login. and cloud.
// pragmatismo.com.br), addressed with a leading-dot domain cookie.
function gbSsoDomain(host) {
  if (!host || host === "localhost" || host === "127.0.0.1" || host === "::1") return "";
  if (!/^[a-z0-9.-]+$/i.test(host)) return "";
  var labels = host.split(".");
  // login.generalbots.org -> .generalbots.org (never the public suffix).
  if (labels.length > 2 && labels.every(function (l) { return /^[a-z0-9-]+$/i.test(l); })) {
    return "." + labels.slice(1).join(".");
  }
  return "";
}

function gbSsoWrite(doc, host, secure, value, maxAge) {
  var cookie = GB_SSO_KEY + "=" + value + "; Path=/; SameSite=Lax; Max-Age=" + maxAge;
  var domain = gbSsoDomain(host);
  if (domain) cookie += "; Domain=" + domain;
  if (secure) cookie += "; Secure";
  try { doc.cookie = cookie; } catch (e) { /* storage disabled */ }
}

function gbSsoRead(doc) {
  try {
    var parts = doc.cookie.split(";");
    for (var i = 0; i < parts.length; i++) {
      var p = parts[i].trim();
      if (p.indexOf(GB_SSO_KEY + "=") === 0) return p.slice(GB_SSO_KEY.length + 1);
    }
  } catch (e) { /* storage disabled */ }
  return null;
}

function gbSsoCreate(opts) {
  var o = opts || {};
  var state = gbSsoNonce();
  var ts = o._now ? o._now() : Date.now();
  // Browser call sites pass only _host/_secure. Defaulting to the page's
  // document is what actually writes the one-time state cookie; without it
  // the receiver discards the token on arrival. Node self-tests inject a
  // fake _doc and keep working without a DOM.
  var doc = o._doc || (typeof document !== "undefined" ? document : null);
  if (doc) {
    var secure = o._secure === true;
    var host = o._host || (typeof location !== "undefined" ? location.hostname : "");
    gbSsoWrite(doc, host, secure, encodeURIComponent(state) + "." + ts,
      Math.ceil(GB_SSO_TTL_MS / 1000));
  }
  return { state: state, ts: ts };
}

// Pure validation — hostile inputs return false, never throw.
function gbSsoValid(state, ts, cookieValue, now) {
  if (typeof state !== "string" || !state || !/^\d{10,15}$/.test(String(ts))) return false;
  var t = Number(ts);
  if (!isFinite(t) || now - t > GB_SSO_TTL_MS || t - now > GB_SSO_SKEW_MS) return false;
  if (typeof cookieValue !== "string" || !cookieValue) return false;
  var parts;
  try { parts = decodeURIComponent(cookieValue).split("."); } catch (e) { return false; }
  return parts.length === 2 && parts[0] === state && parts[1] === String(ts);
}

// One use only: the cookie is cleared whether or not the value matched.
function gbSsoConsume(state, ts, opts) {
  var o = opts || {};
  var doc = o._doc;
  var cookieValue = o._cookie !== undefined ? o._cookie : (doc ? gbSsoRead(doc) : null);
  var now = o._now ? o._now() : Date.now();
  var ok = gbSsoValid(state, ts, cookieValue, now);
  if (doc && cookieValue !== null) gbSsoWrite(doc, o._host, o._secure === true, "", -1);
  return ok;
}

// The identity travels in the fragment; the query is always cleared.
function gbSsoDestination(dest, identity, handoff, origin) {
  var target = new URL(dest.href || dest, origin);
  target.search = "";
  target.hash = "";
  var params = new URLSearchParams();
  params.set("gb_token", identity.token || "");
  params.set("email", identity.email || "");
  params.set("name", identity.name || "");
  params.set("gb_state", handoff.state);
  params.set("gb_ts", String(handoff.ts));
  target.hash = params.toString();
  return target.href;
}

function gbSsoReadFragment(hash) {
  var out = {};
  try {
    new URLSearchParams(String(hash || "").replace(/^#/, "")).forEach(function (v, k) { out[k] = v; });
  } catch (e) { /* malformed fragment */ }
  return out;
}

// Returns a canonical http(s) origin or null for hostile schemes/credentials.
function gbSsoTrustedOrigin(value, base) {
  if (!value || typeof value !== "string") return null;
  try {
    var u = new URL(value, base);
    if (u.protocol !== "http:" && u.protocol !== "https:") return null;
    if (u.username || u.password) return null;
    return u.origin;
  } catch (e) { return null; }
}

function gbSsoFakeDoc(seed) {
  var value = seed || "";
  return {
    get cookie() { return value; },
    set cookie(v) {
      var m = /^([^=]+)=([^;]*)/.exec(String(v));
      if (m && m[1] === GB_SSO_KEY) value = m[2] ? m[1] + "=" + m[2] : "";
    }
  };
}

// Browser console: __gbSsoSelfTest(). Node: tests/cloud_auth_redirect.test.js.
function gbSsoSelfTest() {
  var now = 1700000000000;
  var results = [];
  function check(label, pass) { results.push({ input: label, pass: !!pass }); }

  var doc = gbSsoFakeDoc();
  var h = gbSsoCreate({ _now: function () { return now; }, _doc: doc, _host: "localhost" });
  check("first use accepted", gbSsoConsume(h.state, h.ts, { _doc: doc, _now: function () { return now + 10; } }));
  check("replay of consumed state rejected",
    !gbSsoConsume(h.state, h.ts, { _doc: doc, _now: function () { return now + 20; } }));

  check("expired state rejected",
    !gbSsoValid("s", String(now - 60001), "s." + (now - 60001), now));
  check("future state rejected", !gbSsoValid("s", String(now + 6000), "s." + (now + 6000), now));
  check("mismatched state rejected", !gbSsoValid("a", String(now), "b." + now, now));
  check("missing cookie rejected", !gbSsoValid("s", String(now), "", now));
  check("malformed cookie rejected", !gbSsoValid("s", String(now), "s." + now + ".x", now));
  for (var i = 0; i < 6; i++) {
    var bad = ["", "..", "a.b.c", "abc", "-1", "999999999999999999999"];
    check("hostile value " + JSON.stringify(bad[i]) + " rejected",
      !gbSsoValid("s", bad[i], "s." + bad[i], now));
  }

  var url = gbSsoDestination(
    "https://cloud.example/dashboard?token=INJECTED#old",
    { token: "SECRET", email: "e@x", name: "n" },
    { state: "s", ts: now },
    "https://login.example"
  );
  var parsed = new URL(url);
  check("query carries no token", parsed.search === "");
  check("fragment carries token", parsed.hash.indexOf("gb_token=SECRET") !== -1);
  check("fragment carries state", parsed.hash.indexOf("gb_state=s") !== -1);
  check("old fragment replaced", parsed.hash.indexOf("old") === -1);

  check("javascript: origin rejected", gbSsoTrustedOrigin("javascript:alert(1)", "https://login.example") === null);
  check("credentialed origin rejected", gbSsoTrustedOrigin("https://u:p@cloud.example", "https://login.example") === null);
  check("trusted cloud origin accepted", gbSsoTrustedOrigin("https://cloud.example", "https://login.example") === "https://cloud.example");

  var failed = results.filter(function (r) { return !r.pass; }).length;
  return { passed: results.length - failed, failed: failed, results: results };
}

var api = {
  KEY: GB_SSO_KEY,
  TTL_MS: GB_SSO_TTL_MS,
  create: gbSsoCreate,
  valid: gbSsoValid,
  consume: gbSsoConsume,
  destination: gbSsoDestination,
  readFragment: gbSsoReadFragment,
  trustedOrigin: gbSsoTrustedOrigin,
  cookieDomain: gbSsoDomain,
  selfTest: gbSsoSelfTest
};

if (typeof window !== "undefined") window.GB_SSO_HANDOFF = api;
if (typeof module !== "undefined" && module.exports) module.exports = api;
