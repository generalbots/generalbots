/**
 * webtest_video runner
 * Split from webtest_video.js for #1731.
 */
"use strict";

const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");
const { VIDEOS_DIR, GALLERY_PATH, startServer } = require("./00_common");
const gallery = require("./30_gallery");

async function main(TESTS) {
  fs.mkdirSync(VIDEOS_DIR, { recursive: true });
  try { for (const f of fs.readdirSync(VIDEOS_DIR)) { try { fs.unlinkSync(path.join(VIDEOS_DIR, f)); } catch(e){} } } catch(e) {}

  const server = await startServer();
  const browser = await chromium.launch({ headless: true });
  const results = [];

  gbLog.debug("\n" + "=".repeat(60));
  gbLog.debug("GB7+BBSIA VIDEO EVIDENCE — " + TESTS.length + " tests");
  gbLog.debug("=".repeat(60) + "\n");

  for (const tc of TESTS) {
    const ctx = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      recordVideo: { dir: VIDEOS_DIR, size: { width: 1280, height: 720 } },
    });
    const page = await ctx.newPage();
    page.on("pageerror", () => {});
    const t0 = Date.now();
    let res;
    try { res = await tc.run(page); } catch (e) { res = { pass: false, detail: "ERROR: " + e.message }; }
    const dur = Date.now() - t0;

    await page.close();
    await ctx.close();

    await new Promise(r => setTimeout(r, 500));
    const unnamed = fs.readdirSync(VIDEOS_DIR).filter(f => f.endsWith(".webm") && !results.some(r => f === r.id + ".webm") && f !== tc.id + ".webm");
    if (unnamed.length) {
      unnamed.sort((a,b) => fs.statSync(path.join(VIDEOS_DIR,b)).mtimeMs - fs.statSync(path.join(VIDEOS_DIR,a)).mtimeMs);
      try { fs.renameSync(path.join(VIDEOS_DIR, unnamed[0]), path.join(VIDEOS_DIR, tc.id + ".webm")); } catch(e){}
    }

    gbLog.debug(`[${res.pass?"PASS":"FAIL"}] ${tc.id}: ${tc.title} (${(dur/1000).toFixed(1)}s)`);
    if (!res.pass) gbLog.debug(`       ${res.detail}`);
    results.push({ id: tc.id, title: tc.title, desc: tc.desc, pass: res.pass, detail: res.detail, dur });
  }

  await browser.close();
  server.close();

  fs.writeFileSync(GALLERY_PATH, gallery(results));
  const p = results.filter(r=>r.pass).length, f = results.length - p;
  gbLog.debug("\n" + "=".repeat(60));
  gbLog.debug(`TOTAL: ${results.length} | PASS: ${p} | FAIL: ${f}`);
  gbLog.debug("=".repeat(60));
  gbLog.debug("\nGallery: " + GALLERY_PATH);

  require("child_process").exec(`start "" "${GALLERY_PATH}"`);
  process.exit(f > 0 ? 1 : 0);
}

module.exports = main;
