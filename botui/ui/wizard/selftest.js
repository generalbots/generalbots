/* Headless acceptance checks for the first-run wizard (#1586-#1591). */
(function () {
  "use strict";

  var results = document.getElementById("results");
  var passed = 0;
  var failed = 0;
  var lines = [];

  function log(ok, name, detail) {
    if (ok) { passed += 1; } else { failed += 1; }
    lines.push((ok ? "PASS " : "FAIL ") + name + (detail ? " — " + detail : ""));
    results.textContent = lines.join("\n");
    window.__SELFTEST_RESULT__ = { passed: passed, failed: failed, lines: lines };
  }

  function assert(ok, name, detail) { log(!!ok, name, detail); }

  function text(selector) {
    var node = document.querySelector(selector);
    return node ? node.textContent : "";
  }

  function waitFor(predicate, timeoutMs) {
    return new Promise(function (resolve) {
      var started = Date.now();
      (function poll() {
        if (predicate()) { resolve(true); return; }
        if (Date.now() - started > timeoutMs) { resolve(false); return; }
        setTimeout(poll, 60);
      })();
    });
  }

  function click(selector) {
    var node = document.querySelector(selector);
    if (node) { node.click(); }
    return !!node;
  }

  function fire(selector, type) {
    var node = document.querySelector(selector);
    if (!node) { return false; }
    var event = document.createEvent("HTMLEvents");
    event.initEvent(type, true, true);
    node.dispatchEvent(event);
    return true;
  }

  async function run() {
    var ready = await waitFor(function () {
      return document.getElementById("wizardRoot").getAttribute("data-state") === "ready";
    }, 4000);
    assert(ready, "wizard renders with the stack stopped and no network");

    var rail = document.querySelectorAll("#railSteps .rail-step");
    assert(rail.length === 8, "eight steps are present", String(rail.length));
    assert(rail[0].getAttribute("aria-current") === "step", "rail marks the current step");

    var body = text("#stepBody");
    assert(body.indexOf("runs locally") !== -1, "step 1 states the local contract");
    assert(!document.getElementById("accentCustom") && !document.querySelector('#stepBody input[type="color"]'),
      "accent picker offers presets only (no custom colour field)");

    var forbidden = ["Windows", "macOS", "Linux", "C:\\", "drive letter"];
    var allText = text("#wizardRoot");
    var leaked = forbidden.filter(function (word) { return allText.indexOf(word) !== -1; });
    assert(leaked.length === 0, "copy names no operating system", leaked.join(", "));

    assert(click("#nextBtn"), "Next is clickable on step 1");
    await waitFor(function () { return window.GBWizard.state.step === 2; }, 1500);
    assert(window.GBWizard.state.step === 2, "Next advances to the model step");

    var engines = document.querySelectorAll('[data-engine]');
    assert(engines.length === 4, "four engine cards in step 2", String(engines.length));
    var engineIds = Array.prototype.map.call(engines, function (e) { return e.getAttribute("data-engine"); });
    assert(engineIds.join(",") === "local,own-key,free,acp", "engine order is local, own-key, free, acp", engineIds.join(","));
    var controls = document.querySelectorAll("#stepBody button, #stepBody input, #stepBody select, #stepBody [tabindex]");
    assert(controls.length === 0, "step 2 is an overview with no controls", String(controls.length));
    var start = document.querySelectorAll(".wiz-engine.start");
    assert(start.length === 1 && start[0].getAttribute("data-engine") === "free" &&
      /AT START/.test(start[0].textContent), "the free card is marked as the engine at start");
    assert(/Settings › Model/.test(text("#stepBody")), "step 2 says where the other engines are set later");
    assert(/asks before the first message/.test(text("#wizEngineDetail")), "free details say the app asks before the first send");
    var m = window.GBWizard.state.draft.model;
    var kilo = window.GBWizard.freeProviderById("kilo");
    assert(m.mode === "free" && m.provider === "kilo" && m.endpoint === kilo.endpoint && m.model === kilo.model &&
      m.api_key === "", "the draft provisions the free engine", JSON.stringify(m));
    click('[data-engine="local"]');
    assert(window.GBWizard.state.draft.model.mode === "free", "clicking a card changes nothing");

    click("#nextBtn");
    await waitFor(function () { return window.GBWizard.state.step === 3; }, 1500);
    assert(window.GBWizard.state.step === 3, "Next advances from the overview without fields");

    await waitFor(function () {
      var status = document.getElementById("workspaceStatus");
      return status && /writable/i.test(status.textContent);
    }, 3000);
    assert(/writable/i.test(text("#workspaceStatus")), "workspace writability is reported");

    click("#nextBtn");
    await waitFor(function () { return window.GBWizard.state.step === 4; }, 1500);
    assert(window.GBWizard.state.step === 4, "services step is reachable");
    assert(text("#stepBody").indexOf("Gmail") !== -1, "service cards are rendered");
    var toggles = document.querySelectorAll("[data-service-toggle]");
    var anyOn = Array.prototype.some.call(toggles, function (tg) { return tg.checked; });
    assert(toggles.length === 3 && !anyOn, "each service has a switch, off by default", String(toggles.length));
    assert(document.getElementById("serviceBody-gmail").hidden, "a service that is off shows no fields");
    var redirectNode = document.querySelector("#stepBody input[readonly]");
    var redirect = redirectNode ? redirectNode.value : "";
    assert(redirect.indexOf("/api/integrations/oauth/gmail/callback") !== -1,
      "redirect URI is exact and derived from the instance", redirect);
    click("#serviceToggle-gmail");
    assert(!document.getElementById("serviceBody-gmail").hidden &&
      window.GBWizard.state.draft.services.gmail.enabled === true, "switching Gmail on opens its fields and saves it");
    var gmailId = document.querySelector('[data-service-id="gmail"]');
    gmailId.value = "client-123";
    fire('[data-service-id="gmail"]', "input");
    click("#serviceToggle-onedrive");
    click("#serviceToggle-dropbox");
    var dbxId = document.querySelector('[data-service-id="dropbox"]');
    dbxId.value = "dbx-1";
    fire('[data-service-id="dropbox"]', "input");
    click("#serviceToggle-dropbox");
    assert(document.querySelector('[data-service="dropbox"]').classList.contains("off"), "switching off mutes and collapses the card");
    var sent = window.GBWizard.sentServices(window.GBWizard.state.draft.services);
    assert(JSON.stringify(sent) === '{"gmail":{"client_id":"client-123","client_secret":""}}',
      "only switched-on services with a client ID are sent", JSON.stringify(sent));

    click("#nextBtn");
    await waitFor(function () { return window.GBWizard.state.step === 5; }, 1500);
    assert(window.GBWizard.state.step === 5, "privacy step is reachable");
    var map = text("#stepBody");
    assert(map.indexOf("api.kilo.ai") !== -1, "data map names where the free model sends text");
    assert(map.indexOf("Gmail") !== -1 && map.indexOf("Dropbox") === -1, "data map lists only switched-on services");
    assert(map.indexOf("Nothing") !== -1 || map.indexOf("Nowhere") !== -1,
      "data map states that crash reports go nowhere by default");

    click("#nextBtn");
    await waitFor(function () { return window.GBWizard.state.step === 6; }, 1500);
    assert(window.GBWizard.state.step === 6, "network mode step is reachable");
    assert(!!document.getElementById("networkModeP2P"), "P2P radio button is on screen");
    assert(!!document.getElementById("networkModeOffline"), "Offline radio button is on screen");
    var p2pRadio = document.getElementById("networkModeP2P");
    assert(!!p2pRadio && p2pRadio.checked, "P2P is selected by default");

    var bridge = window.GBWizardBridge;
    var realProvision = bridge.provision;
    var sentRequest = null;
    bridge.provision = function (req) { sentRequest = JSON.parse(JSON.stringify(req)); return realProvision(req); };
    click("#nextBtn");
    // `next()` renders the installer synchronously, before the first timer.
    assert(window.GBWizard.state.step === 7, "install step is reachable");
    assert(!!document.getElementById("installPercent"), "installer progress is on screen");
    assert(!!document.getElementById("hostReadiness"), "host readiness is checked before services start");
    var hostChecked = await waitFor(function () {
      return window.GBWizard.state.host.checked === true;
    }, 4000);
    assert(hostChecked, "host readiness reports a result");
    var summary = text("#hostSummaryLine");
    assert(summary.indexOf("This machine is ready") !== -1 &&
      document.getElementById("hostSummaryLine").getAttribute("data-kind") === "ok",
      "host summary reads ready with the success state", summary);
    assert(!/not checked/i.test(text("#hostReadiness")), "host panel never says not checked when ready");
    var lit = await waitFor(function () {
      var dots = document.querySelectorAll("#componentList .dot.ok");
      var items = document.querySelectorAll("#componentList .install-step-item.done");
      return (dots.length + items.length) >= 6;
    }, 12000);
    var dotCount = document.querySelectorAll("#componentList .dot.ok").length;
    var itemCount = document.querySelectorAll("#componentList .install-step-item.done").length;
    assert(lit, "components light up green while the installer runs",
      String(dotCount + itemCount));
    var provisioned = await waitFor(function () {
      return window.GBWizard.state.provisioning.ready === true;
    }, 12000);
    assert(provisioned, "provisioning reaches ready");
    bridge.provision = realProvision;
    var req = sentRequest || {};
    assert(req.model && req.model.mode === "free" && req.model.provider === "kilo" &&
      req.model.endpoint === kilo.endpoint && req.model.model === kilo.model,
      "provisioning request carries the free engine", JSON.stringify(req.model));
    assert(JSON.stringify(req.services) === '{"gmail":{"client_id":"client-123","client_secret":""}}',
      "provisioning request carries only switched-on services", JSON.stringify(req.services));
    await waitFor(function () { return window.GBWizard.state.step === 8; }, 1500);
    assert(window.GBWizard.state.step === 8, "finish step is reachable");
    assert(text("#stepBody").indexOf("Workspace") !== -1, "finish summarises the choices");
    assert(text("#stepBody").indexOf("Administrator account") !== -1,
      "finish shows the administrator account");
    assert(!document.getElementById("mediaCapabilitiesCard"), "finish shows no media capabilities block in this edition");
    var adminPassword = document.querySelector("#adminPassword");
    assert(!!adminPassword && adminPassword.value.length >= 12,
      "an administrator password is generated for login");

    log(failed === 0, "TOTAL", passed + " passed, " + failed + " failed");
    document.title = failed === 0 ? "SELFTEST PASS" : "SELFTEST FAIL";
  }

  run().catch(function (e) {
    log(false, "unhandled error", String(e && e.message ? e.message : e));
    document.title = "SELFTEST FAIL";
  });
})();
