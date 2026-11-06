/**
 * webtest_video test cases TC08-TC15 (backup, privacy, settings, update, apps)
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
    id: "TC08", title: "Backup Key Prefix Guard",
    desc: "data-export.js only restores localStorage keys prefixed with gb_",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch data-export.js", "Check ALLOWED_PREFIX / gb_ guard"];
      await page.evaluate(overlayScript("TC08", "Backup Import Guard", "Only gb_ prefixed keys restored from backup", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/js/data-export.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const pass = src.includes("ALLOWED_PREFIX") || src.includes('"gb_"');
      await page.evaluate(completeStep(1, pass));
      await page.evaluate(showResult(pass, "prefix guard found: " + pass));
      await page.waitForTimeout(1200);
      return { pass, detail: "prefix guard: " + pass };
    },
  },
  {
    id: "TC09", title: "Drive Context Framing",
    desc: "drive_context includes untrusted:true and source:user_drive",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch chat-init.js", "Check untrusted: true", "Check source: user_drive"];
      await page.evaluate(overlayScript("TC09", "Drive Context", "Untrusted data must be flagged before sending to LLM", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/chat/chat-init.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const untrusted = src.includes("untrusted: true");
      await page.evaluate(completeStep(1, untrusted));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const source = src.includes('source: "user_drive"');
      await page.evaluate(completeStep(2, source));

      const pass = untrusted && source;
      await page.evaluate(showResult(pass, `untrusted:${untrusted} source:${source}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `untrusted:${untrusted} source:${source}` };
    },
  },
  {
    id: "TC10", title: "No Admin Self-Assignment",
    desc: "__BOT_IS_PUBLIC__ must assign roles:[\"user\"], never admin",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch drive/02_api.js", "Verify roles:[\"admin\"] absent", "Verify roles:[\"user\"] present"];
      await page.evaluate(overlayScript("TC10", "No Admin Self-Assign", "Public bot users get 'user' role, never 'admin'", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const src = await page.evaluate(async () => await (await fetch("/suite/drive/modules/02_api.js?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const noAdmin = !src.includes('roles: ["admin"]');
      await page.evaluate(completeStep(1, noAdmin));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const hasUser = src.includes('roles: ["user"]');
      await page.evaluate(completeStep(2, hasUser));

      const pass = noAdmin && hasUser;
      await page.evaluate(showResult(pass, `admin removed:${noAdmin} user set:${hasUser}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `admin removed:${noAdmin} user set:${hasUser}` };
    },
  },
  {
    id: "TC11", title: "Privacy Defaults",
    desc: "GBPrivacy starts with all telemetry/tracking disabled",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Load GBPrivacy settings", "Check telemetry OFF", "Check usage stats OFF", "Check crash reports OFF"];
      await page.evaluate(overlayScript("TC11", "Privacy Defaults", "All telemetry must be OFF by default — user opts in", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      const s = await page.evaluate(() => window.GBPrivacy ? GBPrivacy.getSettings() : null);
      await page.evaluate(completeStep(0, !!s));
      await page.waitForTimeout(W);

      const checks = [
        { ok: s && s.telemetry_enabled === false, lbl: "telemetry" },
        { ok: s && s.share_usage_stats === false, lbl: "usage_stats" },
        { ok: s && s.crash_reports === false, lbl: "crash_reports" },
      ];
      let allOk = !!s;
      for (let i = 0; i < checks.length; i++) {
        await page.evaluate(activateStep(i + 1));
        await page.waitForTimeout(300);
        allOk = allOk && checks[i].ok;
        await page.evaluate(completeStep(i + 1, checks[i].ok));
        await page.waitForTimeout(W);
      }
      const detail = s ? `telemetry:${s.telemetry_enabled} stats:${s.share_usage_stats} crash:${s.crash_reports}` : "GBPrivacy missing";
      await page.evaluate(showResult(allOk, detail));
      await page.waitForTimeout(1200);
      return { pass: allOk, detail };
    },
  },
  {
    id: "TC12", title: "Accessibility",
    desc: "GBAccessibility injects skip-nav link and ARIA landmarks",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Check GBAccessibility loaded", "Find #gb-skip-nav element", "Count aria-label attributes"];
      await page.evaluate(overlayScript("TC12", "Accessibility", "Skip navigation and ARIA landmarks must be injected", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      const loaded = await page.evaluate(() => typeof window.GBAccessibility !== "undefined");
      await page.evaluate(completeStep(0, loaded));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const skip = await page.evaluate(() => !!document.getElementById("gb-skip-nav"));
      await page.evaluate(completeStep(1, skip));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const ariaCount = await page.evaluate(() => document.querySelectorAll("[aria-label]").length);
      const ariaOk = ariaCount > 0;
      await page.evaluate(completeStep(2, ariaOk));

      const pass = loaded && skip && ariaOk;
      await page.evaluate(showResult(pass, `skip-nav:${skip} aria-labels:${ariaCount}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `skip-nav:${skip} aria-labels:${ariaCount}` };
    },
  },
  {
    id: "TC13", title: "Settings Preview Toggle",
    desc: "Settings HTML has preview toggle wired to GBPreviewMode",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Fetch settings/index.html", "Find #settings-preview-toggle", "Check JS wiring in settings-main.js"];
      await page.evaluate(overlayScript("TC13", "Settings Toggle", "Preview mode toggle must exist and be wired in settings", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      const html = await page.evaluate(async () => await (await fetch("/suite/settings/index.html?_=" + Date.now())).text());
      await page.evaluate(completeStep(0, true));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const toggle = html.includes("settings-preview-toggle") && html.includes("togglePreviewMode");
      await page.evaluate(completeStep(1, toggle));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const js = await page.evaluate(async () => await (await fetch("/suite/settings/settings-main.js?_=" + Date.now())).text());
      const wired = js.includes("togglePreviewMode") && js.includes("gb-preview-changed");
      await page.evaluate(completeStep(2, wired));

      const pass = toggle && wired;
      await page.evaluate(showResult(pass, `toggle:${toggle} wired:${wired}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `toggle:${toggle} wired:${wired}` };
    },
  },
  {
    id: "TC14", title: "Auto-Update Module",
    desc: "GBUpdate has check(), version, dismiss(), showReleaseNotes()",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Check GBUpdate exists", "Verify check() function", "Verify version string", "Verify dismiss + releaseNotes"];
      await page.evaluate(overlayScript("TC14", "Auto-Update", "GBUpdate module must expose update lifecycle methods", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      const exists = await page.evaluate(() => typeof window.GBUpdate !== "undefined");
      await page.evaluate(completeStep(0, exists));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const hasCheck = await page.evaluate(() => typeof GBUpdate.check === "function");
      await page.evaluate(completeStep(1, hasCheck));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const hasVersion = await page.evaluate(() => typeof GBUpdate.version === "string");
      await page.evaluate(completeStep(2, hasVersion));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(3));
      await page.waitForTimeout(300);
      const hasOthers = await page.evaluate(() => typeof GBUpdate.dismiss === "function" && typeof GBUpdate.showReleaseNotes === "function");
      await page.evaluate(completeStep(3, hasOthers));

      const pass = exists && hasCheck && hasVersion && hasOthers;
      const ver = exists ? await page.evaluate(() => GBUpdate.version) : "n/a";
      await page.evaluate(showResult(pass, `version:${ver} check:${hasCheck} dismiss:${hasOthers}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `version:${ver} check:${hasCheck} dismiss:${hasOthers}` };
    },
  },
  {
    id: "TC15", title: "Apps Registry",
    desc: "APPS_REGISTRY includes chat and drive apps",
    run: async (page) => {
      await loadHarness(page);
      const steps = ["Load APPS_REGISTRY", "Find 'chat' app", "Find 'drive' app"];
      await page.evaluate(overlayScript("TC15", "Apps Registry", "Registry must include core apps: chat and drive", steps));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(0));
      await page.waitForTimeout(300);
      const count = await page.evaluate(() => (window.APPS_REGISTRY || []).length);
      await page.evaluate(completeStep(0, count > 0));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(1));
      await page.waitForTimeout(300);
      const hasChat = await page.evaluate(() => (window.APPS_REGISTRY || []).some(a => a.id === "chat"));
      await page.evaluate(completeStep(1, hasChat));
      await page.waitForTimeout(W);

      await page.evaluate(activateStep(2));
      await page.waitForTimeout(300);
      const hasDrive = await page.evaluate(() => (window.APPS_REGISTRY || []).some(a => a.id === "drive"));
      await page.evaluate(completeStep(2, hasDrive));

      const pass = count > 0 && hasChat && hasDrive;
      await page.evaluate(showResult(pass, `total:${count} chat:${hasChat} drive:${hasDrive}`));
      await page.waitForTimeout(1200);
      return { pass, detail: `total:${count} chat:${hasChat} drive:${hasDrive}` };
    },
  },
];
