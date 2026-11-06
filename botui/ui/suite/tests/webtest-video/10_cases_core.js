/**
 * webtest_video test cases TC01-TC07 (desktop globals, windows, security)
 * Split from webtest_video.js for #1731; keeps the original test order.
 */
"use strict";

const {
  overlayScript,
  activateStep,
  completeStep,
  showResult,
  W,
  loadHarness,
} = require("./00_common");

module.exports = [
  {
    id: "TC01", title: "Desktop Globals Load",
    desc: "Verify WindowManager, GBPreviewMode, GBUpdate, GBPrivacy, GBAccessibility are available after bootstrap",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Check WindowManager", "Check GBPreviewMode", "Check GBUpdate", "Check GBPrivacy & GBAccessibility", "Verify APPS_REGISTRY"];
      await page.evaluate(overlayScript("TC01", "Desktop Globals Load", "All core modules must be present after harness loads", steps));
      await page.waitForTimeout(W);

      const checks = [
        { expr: "typeof window.WindowManager !== 'undefined'", label: "WindowManager" },
        { expr: "typeof window.GBPreviewMode !== 'undefined'", label: "GBPreviewMode" },
        { expr: "typeof window.GBUpdate !== 'undefined'", label: "GBUpdate" },
        { expr: "typeof window.GBPrivacy !== 'undefined' && typeof window.GBAccessibility !== 'undefined'", label: "GBPrivacy+A11y" },
        { expr: "Array.isArray(window.APPS_REGISTRY) && window.APPS_REGISTRY.length > 0", label: "APPS_REGISTRY" },
      ];
      let allOk = true;
      for (let i = 0; i < checks.length; i++) {
        await page.evaluate(activateStep(i));
        await page.waitForTimeout(W);
        const ok = await page.evaluate(checks[i].expr);
        allOk = allOk && ok;
        await page.evaluate(completeStep(i, ok));
        await page.waitForTimeout(300);
      }
      const detail = await page.evaluate(() => {
        return "WM:" + (typeof WindowManager !== "undefined") + " PM:" + (typeof GBPreviewMode !== "undefined")
          + " Up:" + (typeof GBUpdate !== "undefined") + " Pr:" + (typeof GBPrivacy !== "undefined")
          + " A11y:" + (typeof GBAccessibility !== "undefined") + " Reg:" + (window.APPS_REGISTRY||[]).length;
      });
      await page.evaluate(showResult(allOk, detail));
      await page.waitForTimeout(1200);
      return { pass: allOk, detail };
    },
  },
  {
    id: "TC02", title: "Window Manager Open & Close",
    desc: "Open a window programmatically, verify it appears, then close it",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Open test window", "Verify window exists", "Close window", "Verify window removed"];
      await page.evaluate(overlayScript("TC02", "Window Manager", "Open/Close lifecycle test", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      await page.evaluate(() => {
        WindowManager.open("tc02", "Evidence Window",
          "<div style='padding:40px;color:#f8fafc;text-align:center'><h2 style='font-size:28px'>Window Opened!</h2><p style='color:#94a3b8;margin-top:12px'>Timestamp: " + new Date().toLocaleTimeString() + "</p></div>");
      });
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(600);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const existed = await page.evaluate(() => WindowManager.openWindows.some(w => w.id === "tc02"));
      await page.evaluate(completeStep(1, existed));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      await page.evaluate(() => WindowManager.close("tc02"));
      await page.evaluate(completeStep(2, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(3));
      await page.waitForTimeout(300);
      const gone = await page.evaluate(() => !WindowManager.openWindows.some(w => w.id === "tc02"));
      await page.evaluate(completeStep(3, gone));
      await page.waitForTimeout(300);

      const pass = existed && gone;
      await page.evaluate(showResult(pass, `Opened:${existed} Closed:${gone}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `Opened:${existed} Closed:${gone}` };
    },
  },
  {
    id: "TC03", title: "Preview Mode Event",
    desc: "GBPreviewMode.set() fires gb-preview-changed CustomEvent",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Register event listener", "Set preview ON", "Set preview OFF", "Verify 2 events fired"];
      await page.evaluate(overlayScript("TC03", "Preview Mode Event", "CustomEvent gb-preview-changed must fire on toggle", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.evaluate(() => { window._prevEvts = []; window.addEventListener("gb-preview-changed", e => window._prevEvts.push(e.detail)); });
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      await page.evaluate(() => GBPreviewMode.set(true));
      const isOn = await page.evaluate(() => GBPreviewMode.isOn());
      await page.evaluate(completeStep(1, isOn));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      await page.evaluate(() => GBPreviewMode.set(false));
      const isOff = await page.evaluate(() => !GBPreviewMode.isOn());
      await page.evaluate(completeStep(2, isOff));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(3));
      await page.waitForTimeout(300);
      const evtCount = await page.evaluate(() => window._prevEvts.length);
      const pass = evtCount === 2 && isOn && isOff;
      await page.evaluate(completeStep(3, pass));
      const detail = `Events:${evtCount} on:${isOn} off:${isOff}`;
      await page.evaluate(showResult(pass, detail));
      await page.waitForTimeout(1200);
      return { pass, detail };
    },
  },
  {
    id: "TC04", title: "Deep-Link Blocking",
    desc: "Preview apps blocked from opening when GBPreviewMode is OFF",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Turn preview OFF", "Try opening 'designer' deep-link", "Verify window did NOT open"];
      await page.evaluate(overlayScript("TC04", "Deep-Link Block", "Preview app deep-links must be blocked when preview is OFF", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.evaluate(() => GBPreviewMode.set(false));
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const before = await page.evaluate(() => WindowManager.openWindows.length);
      await page.evaluate(() => { try { WindowManager.openDeepLink("designer", {}); } catch(e){} });
      await page.evaluate(completeStep(1, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const after = await page.evaluate(() => WindowManager.openWindows.length);
      const pass = before === after;
      await page.evaluate(completeStep(2, pass));
      const detail = `Windows ${before} -> ${after} (blocked=${pass})`;
      await page.evaluate(showResult(pass, detail));
      await page.waitForTimeout(1200);
      return { pass, detail };
    },
  },
  {
    id: "TC05", title: "XSS Sanitization",
    desc: "sanitizeHtml strips script tags, onerror, javascript: URLs, onmouseover",
    run: async (page) => {
      await loadHarness(page);
      const vectors = [
        { input: '<script>alert(1)</scr' + 'ipt><b>ok</b>', bad: "script", name: "Script tag" },
        { input: '<img onerror="x" src=x>', bad: "onerror", name: "onerror handler" },
        { input: '<a href="javascript:void">x</a>', bad: "javascript:", name: "javascript: URL" },
        { input: '<div onmouseover="x">y</div>', bad: "onmouseover", name: "onmouseover" },
      ];
      const steps = vectors.map(v => `Block: ${v.name}`);
      await page.evaluate(overlayScript("TC05", "XSS Sanitization", "sanitizeHtml must neutralize all attack vectors", steps));
      await page.waitForTimeout(W);

      const sanitizeFn = await page.evaluate(async () => {
        var src = await (await fetch("/suite/drive/modules/08_modals.js?_=" + Date.now())).text();
        var fn = new Function(src + "; return typeof sanitizeHtml === 'function' ? sanitizeHtml : null;");
        window._testSanitize = fn();
        return !!window._testSanitize;
      });

      let allOk = true;
      for (let i = 0; i < vectors.length; i++) {
        await page.evaluate(activateStep(i));
        await page.waitForTimeout(300);
        const ok = await page.evaluate((v) => {
          var out = window._testSanitize(v.input);
          return out.indexOf(v.bad) === -1;
        }, vectors[i]);
        allOk = allOk && ok;
        await page.evaluate(completeStep(i, ok));
        await page.waitForTimeout(W);
      }
      const detail = allOk ? "All 4 XSS vectors neutralized" : "Some vectors survived";
      await page.evaluate(showResult(allOk, detail));
      await page.waitForTimeout(1200);
      return { pass: allOk, detail };
    },
  },
  {
    id: "TC06", title: "iframe Sandbox",
    desc: "Computer use iframes must NOT have allow-same-origin",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch 60_computer_use.js source", "Check allow-same-origin absent", "Check allow-scripts allow-forms present"];
      await page.evaluate(overlayScript("TC06", "iframe Sandbox", "allow-same-origin must be removed from sandbox attribute", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/chat/modules/60_computer_use.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const noSameOrigin = !src.includes("allow-same-origin");
      await page.evaluate(completeStep(1, noSameOrigin));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const hasScriptsForms = src.includes("allow-scripts allow-forms");
      await page.evaluate(completeStep(2, hasScriptsForms));
      await page.waitForTimeout(300);

      const pass = noSameOrigin && hasScriptsForms;
      await page.evaluate(showResult(pass, `same-origin absent:${noSameOrigin} scripts+forms:${hasScriptsForms}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `same-origin absent:${noSameOrigin} scripts+forms:${hasScriptsForms}` };
    },
  },
  {
    id: "TC07", title: "Citation URL Validation",
    desc: "Citations enforce https?:// scheme, noopener, noreferrer",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch 30_citations.js", "Check https?:// scheme filter", "Check noreferrer + noopener"];
      await page.evaluate(overlayScript("TC07", "Citation URLs", "Only https?:// URLs allowed, with noreferrer noopener", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/chat/modules/30_citations.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const scheme = /https\?:/.test(src);
      await page.evaluate(completeStep(1, scheme));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const refs = src.includes("noreferrer") && src.includes("noopener");
      await page.evaluate(completeStep(2, refs));

      const pass = scheme && refs;
      await page.evaluate(showResult(pass, `scheme:${scheme} noreferrer+noopener:${refs}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `scheme:${scheme} noreferrer+noopener:${refs}` };
    },
  },
];
