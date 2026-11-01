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
    assert(rail.length === 6, "six steps are present", String(rail.length));
    assert(rail[0].getAttribute("aria-current") === "step", "rail marks the current step");

    var body = text("#stepBody");
    assert(body.indexOf("Runs on this machine") !== -1, "step 1 states the local contract");

    var forbidden = ["Windows", "macOS", "Linux", "C:\\", "drive letter"];
    var allText = text("#wizardRoot");
    var leaked = forbidden.filter(function (word) { return allText.indexOf(word) !== -1; });
    assert(leaked.length === 0, "copy names no operating system", leaked.join(", "));

    assert(click("#nextBtn"), "Next is clickable on step 1");
    await waitFor(function () { return window.GBWizard.state.step === 2; }, 1500);
    assert(window.GBWizard.state.step === 2, "Next advances to the model step");

    // Own key without a key must block.
    var own = document.querySelector('input[name="modelMode"][value="own-key"]');
    if (own) { own.checked = true; fire('input[name="modelMode"][value="own-key"]', "change"); }
    await waitFor(function () { return !!document.getElementById("modelKey"); }, 1000);
    click("#nextBtn");
    assert(window.GBWizard.state.step === 2, "Next is blocked without a provider key");

    var local = document.querySelector('input[name="modelMode"][value="local"]');
    if (local) { local.checked = true; fire('input[name="modelMode"][value="local"]', "change"); }
    await waitFor(function () { return !document.getElementById("modelKey"); }, 1000);
    click("#nextBtn");
    await waitFor(function () { return window.GBWizard.state.step === 3; }, 1500);
    assert(window.GBWizard.state.step === 3, "local model passes validation");

    await waitFor(function () {
      var status = document.getElementById("workspaceStatus");
      return status && /writable/i.test(status.textContent);
    }, 3000);
    assert(/writable/i.test(text("#workspaceStatus")), "workspace writability is reported");

    click("#nextBtn");
    await waitFor(function () { return window.GBWizard.state.step === 4; }, 1500);
    assert(window.GBWizard.state.step === 4, "services step is reachable");
    assert(text("#stepBody").indexOf("Gmail") !== -1, "service cards are rendered");
    var redirectNode = document.querySelector("#stepBody input[readonly]");
    var redirect = redirectNode ? redirectNode.value : "";
    assert(redirect.indexOf("/api/integrations/oauth/gmail/callback") !== -1,
      "redirect URI is exact and derived from the instance", redirect);

    click("#nextBtn");
    await waitFor(function () { return window.GBWizard.state.step === 5; }, 1500);
    assert(window.GBWizard.state.step === 5, "privacy step is reachable");
    var map = text("#stepBody");
    assert(map.indexOf("Stays on this machine") !== -1,
      "data map reflects the bundled local model");
    assert(map.indexOf("Nothing") !== -1 || map.indexOf("Nowhere") !== -1,
      "data map states that crash reports go nowhere by default");

    click("#startSetupBtn");
    var provisioned = await waitFor(function () {
      return window.GBWizard.state.provisioning.ready === true;
    }, 12000);
    assert(provisioned, "provisioning reaches ready");
    var okDots = document.querySelectorAll("#componentList .dot.ok").length;
    assert(okDots >= 6, "every component reports ready", String(okDots));

    click("#nextBtn");
    await waitFor(function () { return window.GBWizard.state.step === 6; }, 1500);
    assert(window.GBWizard.state.step === 6, "finish step is reachable");
    assert(text("#stepBody").indexOf("Workspace") !== -1, "finish summarises the choices");
    assert(text("#stepBody").indexOf("Administrator account") !== -1,
      "finish shows the administrator account");
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
