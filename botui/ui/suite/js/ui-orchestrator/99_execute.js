(function (window, document) {
  "use strict";
  var N = document.__gb_mod_ui_orchestrator;
  if (!N) return;
  var S = N.s;
  var reportStep = N.reportStep;
  var ensureOverlay = N.ensureOverlay;
  var resetStepCard = N.resetStepCard;
  var EXECUTORS = N.EXECUTORS;
  var openApp = N.openApp;
  var waitFor = N.waitFor;
  var normalize = N.normalize;
  var STEP_TIMEOUT_MS = N.STEP_TIMEOUT_MS;
  var getElementRect = N.getElementRect;
  var moveCursorTo = N.moveCursorTo;
  var highlightFlash = N.highlightFlash;
  var pulseClick = N.pulseClick;
  var lastPlanSignature = "";
  var lastPlanAt = 0;

  function executePlan(plan) {
    if (!plan || !plan.steps || !plan.steps.length) {
      reportStep("Empty UI plan", "warn");
      return Promise.resolve(false);
    }
    // The plan can reach both the stock socket handler and a parallel tab's
    // socket handler. Run one copy; a duplicate within three seconds is the
    // same automation, not a second request.
    var signature = JSON.stringify(plan);
    if (signature === lastPlanSignature && Date.now() - lastPlanAt < 3000) {
      return Promise.resolve(false);
    }
    lastPlanSignature = signature;
    lastPlanAt = Date.now();
    // #1691 — never drive an app the product profile does not expose. The
    // prompt already lists only enabled apps; this is the defense-in-depth
    // check for a stale or hallucinated plan.
    var registry = Array.isArray(window.APPS_REGISTRY) ? window.APPS_REGISTRY : [];
    if (registry.length) {
      var wanted = [];
      if (plan.app) wanted.push(plan.app);
      plan.steps.forEach(function (step) {
        if (step && step.op === "open" && step.app) wanted.push(step.app);
      });
      for (var i = 0; i < wanted.length; i++) {
        var available = registry.some(function (app) { return app.id === wanted[i]; });
        if (!available) {
          reportStep("App '" + wanted[i] + "' is not enabled in this product", "warn");
          return Promise.resolve(false);
        }
      }
    }
    if (S.ACTIVE) {
      reportStep("Another automation is already running", "warn");
      return Promise.resolve(false);
    }
    S.ACTIVE = { completed: 0, total: plan.steps.length, failed: 0 };
    ensureOverlay();
    resetStepCard();
    reportStep("Starting automation for app '" + (plan.app || "?") + "'", "info");

    var chain = Promise.resolve(true);
    plan.steps.forEach(function (step) {
      chain = chain.then(function (prevOk) {
        if (prevOk === false) {
          // A hard failure (e.g. the app window never opened) aborts the
          // remaining steps instead of letting them spray warnings and
          // finish with a misleading success report.
          return false;
        }
        var fn = EXECUTORS[step.op] || null;
        if (!fn) {
          reportStep("Unknown step op '" + step.op + "'", "warn");
          S.ACTIVE.failed++;
          return false;
        }
        return fn(step, plan).then(function (ok) {
          S.ACTIVE.completed++;
          if (!ok) S.ACTIVE.failed++;
          var evt = new CustomEvent("gb:ui-step", {
            detail: { text: "Step " + S.ACTIVE.completed + "/" + S.ACTIVE.total, status: "progress" },
          });
          window.dispatchEvent(evt);
          return ok;
        });
      });
    });

    return chain
      .then(function (lastOk) {
        var failed = S.ACTIVE.failed || (lastOk === false ? 1 : 0);
        // Steps that reported success may already have changed the app, so the
        // failure summary must not claim otherwise — the earlier wording
        // ("nothing was changed") was false whenever a later step failed.
        var applied = Math.max(0, (S.ACTIVE.completed || 0) - failed);
        S.ACTIVE = null;
        if (S.focusRing) S.focusRing.style.display = "none";
        if (failed > 0) {
          var tail = applied > 0
            ? "; " + applied + " earlier step(s) already ran, so the app may be partly changed"
            : " (nothing was changed)";
          reportStep("Automation FAILED — " + failed + " of " + plan.steps.length + " steps did not complete" + tail, "warn");
          return false;
        }
        reportStep("Automation complete: " + plan.steps.length + "/" + plan.steps.length + " steps", "done");
        return true;
      })
      .catch(function (e) {
        reportStep("Automation error: " + e.message, "warn");
        S.ACTIVE = null;
        return false;
      });
  }

  function focusEntity(result) {
    if (!result) return Promise.resolve(false);
    return openApp(result.app).then(function () {
      reportStep("Opening " + result.type + " '" + result.title + "'", "info");
      var query = result.title || "";
      return waitFor(function () {
        var rows = document.querySelectorAll("tr, [data-entity], [data-id]");
        for (var i = 0; i < rows.length; i++) {
          var txt = normalize(rows[i].textContent);
          if (query && txt.indexOf(normalize(query)) !== -1) return rows[i];
        }
        return null;
      }, STEP_TIMEOUT_MS).then(function (found) {
        if (!found) {
          reportStep("Could not locate record in window", "warn");
          return false;
        }
        var rect = getElementRect(found);
        found.scrollIntoView({ block: "center", behavior: "smooth" });
        setTimeout(function () {
          moveCursorTo(rect, function () {
            highlightFlash(found);
            pulseClick();
            try { found.click(); } catch (e) {}
            reportStep("Focused " + result.type + " '" + result.title + "'", "done");
            setTimeout(function () {
              if (S.focusRing) S.focusRing.style.display = "none";
            }, 600);
          });
        }, 400);
        return true;
      });
    });
  }

  window.GBUiOrchestrator = {
    executePlan: executePlan,
    focusEntity: focusEntity,
    isRunning: function () { return !!S.ACTIVE; },
  };

  N.executePlan = executePlan;
  N.focusEntity = focusEntity;
  delete document.__gb_mod_ui_orchestrator;
})(window, document);
