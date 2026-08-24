"use strict";
/* Agent Executor (#1157): turns a natural-language goal into an ordered plan
   via POST /api/agent/plan and executes each step by opening the target app.
   Used by Spotlight and the Concierge app. */

const AgentExecutor = (() => {
  let executing = false;

  function init() {
    window.addEventListener("gb-agent-execute", (e) => {
      if (e.detail && e.detail.goal) execute(e.detail.goal);
    });
  }

  function execute(goal) {
    if (!goal || executing) return;
    executing = true;
    notify("Concierge", "Planning: “" + goal.substring(0, 80) + "”…", "info");
    fetch("/api/agent/plan", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ goal: goal }),
    })
      .then((r) => r.json())
      .then((plan) => {
        const steps = (plan && plan.steps) || [];
        if (!steps.length) {
          notify("Concierge", "No steps found for that goal.", "warning");
          executing = false;
          return;
        }
        notify("Concierge", "Executing " + steps.length + " step" + (steps.length > 1 ? "s" : "") + "…", "info");
        runSteps(steps, 0);
      })
      .catch(() => {
        notify("Concierge", "Planning failed — is the API reachable?", "error");
        executing = false;
      });
  }

  function runSteps(steps, i) {
    if (i >= steps.length) {
      notify("Concierge", "Goal complete.", "success");
      executing = false;
      return;
    }
    const step = steps[i];
    openApp(step.app, step.title || step.action || step.app, step.params || "");
    setTimeout(() => runSteps(steps, i + 1), 700);
  }

  function openApp(appId, title, params) {
    const app = (window.APPS_REGISTRY || []).find((a) => a.id === appId);
    if (!app) {
      notify("Concierge", "App “" + appId + "” is not installed.", "warning");
      return;
    }
    if (window.WindowManager) {
      window.WindowManager.open(app.id, title || app.title, "");
      const qs = params ? encodeURIComponent(params) + "&" : "";
      const sep = app.hxGet.indexOf("?") === -1 ? "?" : "&";
      fetch(app.hxGet + sep + qs + "_=" + Date.now())
        .then((resp) => resp.text())
        .then((html) => {
          const body = document.getElementById("window-body-" + app.id);
          if (body && window.WindowManager._injectBodyContent) {
            window.WindowManager._injectBodyContent(app.id, html);
          }
        })
        .catch(() => {});
    }
  }

  function notify(title, msg, kind) {
    if (window.GBToasts) {
      window.GBToasts.show(title, msg, kind);
    } else {
      console.log("[" + title + "] " + msg);
    }
  }

  return { init, execute, openApp };
})();

window.AgentExecutor = AgentExecutor;