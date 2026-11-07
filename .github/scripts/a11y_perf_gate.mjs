#!/usr/bin/env node
/*
 * Accessibility and performance gate for the suite shell (#1680, #1791).
 *
 * Runs Lighthouse against a page and fails on a serious accessibility
 * regression. The performance floor is deliberately a smoke check (startup
 * dominates and the page talks to a local botserver); the real budget is
 * measured on a live run.
 *
 * Usage:
 *   node .github/scripts/a11y_perf_gate.mjs <url> [--require-desktop]
 *
 * `--require-desktop` also fails when the audited page redirected away from
 * the requested shell URL. The botserver-served job sets it so a redirect to
 * /suite/login.html can never be mistaken for a passing desktop page.
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

const args = process.argv.slice(2);
const requireDesktop = args.includes("--require-desktop");
const url = args.find((arg) => !arg.startsWith("--")) ||
  "http://127.0.0.1:3000/suite/desktop.html";
// Unique per process: concurrent gate runs must not read each other's report.
const out = `/tmp/lighthouse-gate-${process.pid}.json`;

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
if (process.env.A11Y_KEEP_REPORT === "1") {
  console.log(`lighthouse report kept at ${out}`);
} else {
  rmSync(out, { force: true });
}

const accessibility = Math.round((report.categories.accessibility?.score ?? 0) * 100);
const performance = Math.round((report.categories.performance?.score ?? 0) * 100);
const serious = Object.values(report.audits).filter(
  (audit) =>
    audit.score !== null &&
    audit.score < 1 &&
    audit.scoreDisplayMode === "binary" &&
    audit.id !== "bf-cache",
);

// Fallback for the #1791 regression: Lighthouse's landmark-one-main audit can
// be satisfied by the wrong tree, so also assert from the rendered DOM that at
// least one main landmark exists. Chrome's --dump-dom runs the boot scripts.
function countMainLandmarks(pageUrl) {
  const chrome = process.env.CHROME_PATH;
  if (!chrome) return -1;
  let dom = "";
  try {
    dom = execFileSync(
      chrome,
      [
        "--headless",
        "--no-sandbox",
        "--disable-gpu",
        "--virtual-time-budget=5000",
        "--dump-dom",
        pageUrl,
      ],
      { encoding: "utf8", timeout: 60_000, stdio: ["ignore", "pipe", "ignore"] },
    );
  } catch {
    return -1;
  }
  const tagMains = (dom.match(/<main\b/gi) || []).length;
  const roleMains = (dom.match(/<[^>]+\brole=["']main["'][^>]*>/gi) || [])
    .filter((tag) => !/^<main\b/i.test(tag)).length;
  return tagMains + roleMains;
}

const requestedPath = new URL(url).pathname.replace(/\/$/, "");
const finalPath = new URL(report.finalUrl).pathname.replace(/\/$/, "");
const mainLandmarks = countMainLandmarks(report.finalUrl);

console.log(
  `accessibility=${accessibility} performance=${performance} main-landmarks=${mainLandmarks} final=${report.finalUrl}`,
);
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
if (requireDesktop && finalPath !== requestedPath) {
  console.error(
    `refusing to certify ${requestedPath}: the audit redirected to ${report.finalUrl}`,
  );
  failed = true;
}
if (mainLandmarks === 0) {
  console.error("no <main> or [role=main] landmark in the rendered page");
  failed = true;
} else if (mainLandmarks < 0) {
  console.warn("could not render the page to verify a main landmark");
}
process.exit(failed ? 1 : 0);
