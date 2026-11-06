#!/usr/bin/env node
/*
 * Accessibility and performance gate for the suite shell (#1680).
 *
 * Runs Lighthouse against the statically served desktop page and fails on a
 * serious accessibility regression. The performance floor is deliberately a
 * smoke check (the page talks to a botserver that is not running in CI); the
 * real budget is measured on a live run.
 *
 * Usage: node .github/scripts/a11y_perf_gate.mjs http://127.0.0.1:3000/suite/desktop.html
 */
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, rmSync } from "node:fs";

function findChrome() {
  const fixed = [
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/opt/google/chrome/chrome",
  ];
  for (const path of fixed) if (existsSync(path)) return path;
  const playwright = `${process.env.HOME}/.cache/ms-playwright`;
  if (existsSync(playwright)) {
    for (const entry of readdirSync(playwright)) {
      for (const rel of ["chrome-linux64/chrome", "chrome-linux/chrome"]) {
        const path = `${playwright}/${entry}/${rel}`;
        if (existsSync(path)) return path;
      }
    }
  }
  return null;
}

if (!process.env.CHROME_PATH) {
  const chrome = findChrome();
  if (chrome) process.env.CHROME_PATH = chrome;
}

const url = process.argv[2] || "http://127.0.0.1:3000/suite/desktop.html";
const out = "/tmp/lighthouse-gate.json";

execFileSync(
  "npx",
  [
    "-y",
    "lighthouse@13",
    url,
    "--output=json",
    `--output-path=${out}`,
    "--only-categories=accessibility,performance",
    "--quiet",
    "--chrome-flags=--headless --no-sandbox --disable-gpu",
  ],
  { stdio: "inherit" },
);

const report = JSON.parse(readFileSync(out, "utf8"));
rmSync(out, { force: true });

const accessibility = Math.round((report.categories.accessibility?.score ?? 0) * 100);
const performance = Math.round((report.categories.performance?.score ?? 0) * 100);
const serious = Object.values(report.audits).filter(
  (audit) =>
    audit.score !== null &&
    audit.score < 1 &&
    audit.scoreDisplayMode === "binary" &&
    audit.id !== "bf-cache",
);

console.log(`accessibility=${accessibility} performance=${performance}`);
for (const audit of serious) console.log(` - ${audit.id}: ${audit.title}`);

let failed = false;
if (accessibility < 95) {
  console.error(`accessibility ${accessibility} is below the 95 floor`);
  failed = true;
}
if (performance < 20) {
  console.error(`performance ${performance} is below the 20 smoke floor`);
  failed = true;
}
if (serious.length > 0) {
  console.error(`${serious.length} binary accessibility audit(s) failed`);
  failed = true;
}
process.exit(failed ? 1 : 0);
