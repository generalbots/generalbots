"use strict";
/* #2809 — open-redirect guard for the login/signup token handoff.
 * run: node tests/cloud_auth_redirect.test.js
 */
const { validateRedirect, runRedirectSelfTest } = require("../ui/login/js/cloud-auth.js");

let failed = 0;
function check(condition, label) {
  if (condition) console.log("PASS " + label);
  else { failed++; console.error("FAIL " + label); }
}

const origin = "https://login.example";
const cloud = "https://cloud.example";

// Foreign origins and protocol-relative hosts must never be navigated to.
check(validateRedirect("https://evil.example/dashboard?token=STOLEN", origin, cloud) === null,
  "absolute https://evil.example rejected");
check(validateRedirect("//evil.example/dashboard", origin, cloud) === null,
  "protocol-relative //evil.example rejected");
check(validateRedirect("https://login.example@evil.example/x", origin, cloud) === null,
  "credential-confusion url rejected");
check(validateRedirect("javascript:alert(1)", origin, cloud) === null,
  "javascript: scheme rejected");

// Legitimate in-app destinations still work.
const local = validateRedirect("/drive", origin, cloud);
check(!!local && local.origin === origin && local.pathname === "/drive", "relative /drive allowed");
const cloudDest = validateRedirect(cloud + "/dashboard", origin, cloud);
check(!!cloudDest && cloudDest.origin === cloud, "allowlisted cloud origin allowed");
check(validateRedirect(origin + "/login", origin, cloud) === null, "auth target /login rejected");

const self = runRedirectSelfTest();
check(self.failed === 0, "browser self-test cases (" + self.passed + " passed, " + self.failed + " failed)");
for (const r of self.results) {
  if (!r.pass) console.error("  " + r.input + " -> " + r.actual + " (expected " + r.expected + ")");
}

console.log("cloud_auth_redirect: " + (failed === 0 ? "ALL PASS" : failed + " FAILED"));
process.exit(failed === 0 ? 0 : 1);
