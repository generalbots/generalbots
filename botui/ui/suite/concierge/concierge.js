"use strict";
/* Concierge app (#1157): goal input that plans via /api/agent/plan and
   executes steps through the shared AgentExecutor. */

(function () {
  if (window.GBConcierge) return;

  const HISTORY_KEY = "gb-concierge-history";

  function readHistory() {
    try {
      return JSON.parse(localStorage.getItem(HISTORY_KEY) || "[]");
    } catch (e) {
      return [];
    }
  }

  function pushHistory(goal) {
    const list = readHistory();
    list.unshift({ goal: goal, ts: Date.now() });
    writeHistory(list.slice(0, 12));
  }

  function writeHistory(list) {
    try {
      localStorage.setItem(HISTORY_KEY, JSON.stringify(list));
    } catch (e) {}
  }

  function run(goal) {
    goal = (goal || "").trim();
    if (!goal) return;
    pushHistory(goal);
    const stepsBox = document.getElementById("conciergeSteps");
    if (stepsBox) {
      stepsBox.innerHTML = '<div class="concierge-empty">Planning…</div>';
    }
    fetch("/api/agent/plan", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ goal: goal }),
    })
      .then(function (r) { return r.json(); })
      .then(function (plan) {
        const steps = (plan && plan.steps) || [];
        if (stepsBox) {
          if (!steps.length) {
            stepsBox.innerHTML = '<div class="concierge-empty">No steps found.</div>';
          } else {
            stepsBox.innerHTML = steps
              .map(function (s, i) {
                return '<div class="concierge-step"><span class="step-num">' + (i + 1) + '</span><span>' + escapeHtml(s.action || s.title || s.app) + '</span><span class="step-app">' + escapeHtml(s.app || "") + "</span></div>";
              })
              .join("");
          }
        }
        renderHistory();
        if (window.AgentExecutor && steps.length) {
          // Let the shared executor open each app in sequence.
          steps.forEach(function (s, i) {
            setTimeout(function () {
              window.AgentExecutor.openApp(s.app, s.title || s.action || s.app, s.params || "");
            }, i * 700);
          });
        }
      })
      .catch(function () {
        if (stepsBox) stepsBox.innerHTML = '<div class="concierge-empty">Planning failed — check the API.</div>';
      });
  }

  function renderHistory() {
    const box = document.getElementById("conciergeHistory");
    if (!box) return;
    const list = readHistory();
    if (!list.length) return;
    box.innerHTML =
      '<div class="concierge-history-title">Recent goals</div>' +
      list
        .map(function (h) {
          return '<div class="concierge-step" data-goal="' + escapeHtml(h.goal) + '"><span class="step-num">↻</span><span>' + escapeHtml(h.goal) + "</span></div>";
        })
        .join("");
    Array.from(box.querySelectorAll("[data-goal]")).forEach(function (el) {
      el.addEventListener("click", function () {
        const input = document.getElementById("conciergeGoal");
        if (input) input.value = el.dataset.goal;
        run(el.dataset.goal);
      });
    });
  }

  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  document.addEventListener("DOMContentLoaded", function () {
    const input = document.getElementById("conciergeGoal");
    const btn = document.getElementById("conciergeRun");
    if (input) {
      input.addEventListener("keydown", function (e) {
        if (e.key === "Enter") run(input.value);
      });
    }
    if (btn) btn.addEventListener("click", function () { if (input) run(input.value); });
    renderHistory();
  });

  window.GBConcierge = { run: run };
})();