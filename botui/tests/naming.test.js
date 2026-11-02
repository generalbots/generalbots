"use strict";
/* Naming guard (#1624): split files must be named for their responsibility,
 * never for their position. Fails if a `partN.rs` or `*.pN.js` reappears.
 *
 * run: node tests/naming.test.js
 */
const fs = require("fs");
const path = require("path");

const ROOT = path.join(__dirname, "..", "..");
let passed = 0;
let failed = 0;
function check(condition, label) {
  if (condition) passed++;
  else { failed++; console.error("FAIL " + label); }
}

function walk(dir, out) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "target" || entry.name === ".git" || entry.name === "node_modules") continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else out.push(full);
  }
}

const files = [];
walk(ROOT, files);
const rsParts = files.filter((f) => /(^|\/)part\d+\.rs$/.test(f));
const jsParts = files.filter((f) => /\.p\d+\.js$/.test(f));

check(rsParts.length === 0, "no partN.rs slices remain" + (rsParts.length ? ": " + rsParts.slice(0, 5).join(", ") : ""));
check(jsParts.length === 0, "no *.pN.js slices remain" + (jsParts.length ? ": " + jsParts.slice(0, 5).join(", ") : ""));

// The renamed slices must still be reachable: every <script src> resolves.
const ui = path.join(ROOT, "botui", "ui");
const missing = [];
for (const file of files.filter((f) => f.startsWith(ui) && f.endsWith(".html"))) {
  const text = fs.readFileSync(file, "utf8");
  for (const m of text.matchAll(/(?:src|href)="(\/[^"]+\.(?:js|css))(?:\?[^"]*)?"/g)) {
    const rel = m[1].replace(/^\//, "");
    if (!fs.existsSync(path.join(ui, rel))) missing.push(file + " -> " + m[1]);
  }
}
// Login pages serve /js/cloud-auth.js through the cloud proxy, not from disk.
const unexpected = missing.filter((m) => !/cloud-auth\.js$/.test(m));
check(unexpected.length === 0, "every local script/style reference resolves" + (unexpected.length ? ": " + unexpected.slice(0, 5).join("; ") : ""));

console.log(`naming: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
