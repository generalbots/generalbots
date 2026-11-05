"use strict";

/* #1702 — the actions ledger: terminal and browser audit rows in one place.
   Both trails are durable server tables (terminal_audit / browser_audit), so
   this view is read-only: time, action, decision, outcome and screenshot. */

(function (ns) {
  var state = { kind: "terminal", busy: false };

  function token() {
    return (
      localStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      sessionStorage.getItem("gb-access-token") ||
      ""
    );
  }

  function esc(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function list() {
    return document.getElementById("ledger-list");
  }

  function empty(message) {
    return '<div class="ledger-empty">' + esc(message) + "</div>";
  }

  function renderTerminal(entries) {
    if (!entries.length) return empty("No terminal actions recorded yet.");
    var rows = entries
      .map(function (entry) {
        return (
          "<tr><td>" + esc((entry.created_at || "").replace("T", " ").slice(0, 19)) + "</td>" +
          "<td><code>" + esc(entry.command) + "</code></td>" +
          "<td>" + esc(entry.decision) + "</td>" +
          "<td>" + (entry.exit_code == null ? "—" : esc(entry.exit_code)) + "</td>" +
          "<td>" + esc((entry.output_excerpt || "").slice(0, 160)) + "</td></tr>"
        );
      })
      .join("");
    return (
      '<table class="ledger-table"><thead><tr><th>Time</th><th>Command</th>' +
      "<th>Decision</th><th>Exit</th><th>Output</th></tr></thead><tbody>" +
      rows + "</tbody></table>"
    );
  }

  function renderBrowser(entries) {
    if (!entries.length) return empty("No browser actions recorded yet.");
    var rows = entries
      .map(function (entry) {
        return (
          "<tr><td>" + esc((entry.created_at || "").replace("T", " ").slice(0, 19)) + "</td>" +
          "<td>" + esc(entry.action) + "</td>" +
          "<td>" + esc((entry.target || "").slice(0, 90)) + "</td>" +
          "<td>" + esc(entry.decision) + "</td>" +
          "<td>" + esc((entry.result || "").slice(0, 160)) + "</td>" +
          "<td>" + (entry.screenshot_path ? esc(entry.screenshot_path) : "—") + "</td></tr>"
        );
      })
      .join("");
    return (
      '<table class="ledger-table"><thead><tr><th>Time</th><th>Action</th>' +
      "<th>Target</th><th>Decision</th><th>Result</th><th>Screenshot</th></tr></thead><tbody>" +
      rows + "</tbody></table>"
    );
  }

  function budgetGroup() {
    return localStorage.getItem("gb-budget-group") || "default";
  }

  function budgetForm() {
    return (
      '<div class="ledger-budget-toolbar">' +
      '<label>Group <input id="ledger-budget-group" type="text" value="' + esc(budgetGroup()) + '"></label>' +
      '<label>Allowance (milli-USD) <input id="ledger-budget-amount" type="number" min="0" step="1" placeholder="10000"></label>' +
      '<button type="button" class="btn-secondary btn-sm" id="ledger-budget-save">Save budget</button>' +
      '<button type="button" class="btn-secondary btn-sm" id="ledger-budget-load">Load report</button>' +
      "</div>" +
      '<div id="ledger-budget-report">' + empty("Enter a group and load its report.") + "</div>"
    );
  }

  function renderBudgetReport(payload) {
    var budget = (payload && payload.budget) || {};
    var report = (payload && payload.report) || {};
    var members = report.members || [];
    var summary =
      '<p class="ledger-budget-summary">Spent <strong>' + esc(budget.spent_milli || 0) +
      "</strong> of <strong>" + esc(budget.budget_milli || 0) +
      "</strong> · remaining <strong>" + esc(budget.remaining_milli || 0) +
      "</strong> · " + esc(budget.percent_used || 0) + "%" +
      (budget.over_budget ? ' · <strong class="is-over">over budget</strong>' : "") + "</p>";
    if (!members.length) return summary + empty("No spend attributed yet.");
    var rows = members
      .map(function (member) {
        return (
          "<tr><td><code>" + esc(member.user_id) + "</code></td>" +
          "<td>" + esc(member.amount_milli) + "</td>" +
          "<td>" + esc(member.entries) + "</td></tr>"
        );
      })
      .join("");
    return (
      summary +
      '<table class="ledger-table"><thead><tr><th>Member</th><th>Spent (milli)</th>' +
      "<th>Entries</th></tr></thead><tbody>" + rows + "</tbody></table>"
    );
  }

  function loadBudget(group) {
    var box = document.getElementById("ledger-budget-report");
    if (!box) return;
    box.innerHTML = empty("Loading…");
    fetch("/api/groups/" + encodeURIComponent(group) + "/usage?limit=200", {
      headers: token() ? { Authorization: "Bearer " + token() } : {},
    })
      .then(function (response) {
        if (response.status === 404) throw new Error("no budget configured for this group");
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function (payload) {
        box.innerHTML = renderBudgetReport(payload);
      })
      .catch(function (error) {
        box.innerHTML = empty("Could not load the report: " + error.message);
      });
  }

  function saveBudget(group, amount) {
    fetch("/api/groups/" + encodeURIComponent(group) + "/budget", {
      method: "PUT",
      headers: Object.assign(
        { "Content-Type": "application/json" },
        token() ? { Authorization: "Bearer " + token() } : {}
      ),
      body: JSON.stringify({ budget_milli: amount }),
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function () {
        loadBudget(group);
      })
      .catch(function (error) {
        var box = document.getElementById("ledger-budget-report");
        if (box) box.innerHTML = empty("Could not save the budget: " + error.message);
      });
  }

  function bindBudget() {
    var groupInput = document.getElementById("ledger-budget-group");
    var amountInput = document.getElementById("ledger-budget-amount");
    var save = document.getElementById("ledger-budget-save");
    var reload = document.getElementById("ledger-budget-load");
    if (save) {
      save.addEventListener("click", function () {
        var group = (groupInput.value || "default").trim() || "default";
        localStorage.setItem("gb-budget-group", group);
        saveBudget(group, Number(amountInput.value || 0));
      });
    }
    if (reload) {
      reload.addEventListener("click", function () {
        var group = (groupInput.value || "default").trim() || "default";
        localStorage.setItem("gb-budget-group", group);
        loadBudget(group);
      });
    }
  }

  function loadTtft(hours) {
    var box = document.getElementById("ledger-ttft-report");
    if (!box) return;
    box.innerHTML = empty("Loading…");
    fetch("/api/runs/ttft?hours=" + encodeURIComponent(hours), {
      headers: token() ? { Authorization: "Bearer " + token() } : {},
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function (payload) {
        var summary = (payload && payload.summary) || {};
        var buckets = (payload && payload.trend) || [];
        var head =
          '<p class="ledger-budget-summary">p50 <strong>' + esc(summary.p50_ms || 0) +
          " ms</strong> · p95 <strong>" + esc(summary.p95_ms || 0) +
          " ms</strong> · avg <strong>" + esc(summary.avg_ms || 0) +
          " ms</strong> · n=" + esc(summary.count || 0) +
          " · min " + esc(summary.min_ms || 0) + " ms · max " + esc(summary.max_ms || 0) + " ms</p>";
        if (!buckets.length) {
          box.innerHTML = head + empty("No first-token samples in this window yet.");
          return;
        }
        var rows = buckets
          .map(function (bucket) {
            return (
              "<tr><td>" + esc(bucket.bucket) + "</td><td>" + esc(bucket.count) +
              "</td><td>" + esc(bucket.p50_ms) + "</td><td>" + esc(bucket.p95_ms) + "</td></tr>"
            );
          })
          .join("");
        box.innerHTML =
          head +
          '<table class="ledger-table"><thead><tr><th>Hour</th><th>Samples</th>' +
          "<th>p50 (ms)</th><th>p95 (ms)</th></tr></thead><tbody>" + rows + "</tbody></table>";
      })
      .catch(function (error) {
        box.innerHTML = empty("Could not load TTFT metrics: " + error.message);
      });
  }

  function bindTtft() {
    var hours = document.getElementById("ledger-ttft-hours");
    var reload = document.getElementById("ledger-ttft-load");
    if (reload) {
      reload.addEventListener("click", function () {
        loadTtft(Number(hours.value || 24));
      });
    }
  }

  function load(kind) {
    if (state.busy) return;
    state.busy = true;
    state.kind = kind;
    var box = list();
    if (kind === "ttft") {
      if (box) {
        box.innerHTML =
          '<div class="ledger-budget-toolbar">' +
          '<label>Window (hours) <input id="ledger-ttft-hours" type="number" min="1" max="720" value="24"></label>' +
          '<button type="button" class="btn-secondary btn-sm" id="ledger-ttft-load">Load</button>' +
          "</div>" +
          '<div id="ledger-ttft-report">' + empty("Load the window to see first-token latency.") + "</div>";
        bindTtft();
        loadTtft(24);
      }
      state.busy = false;
      return;
    }
    if (kind === "budget") {
      if (box) {
        box.innerHTML = budgetForm();
        bindBudget();
        loadBudget(budgetGroup());
      }
      state.busy = false;
      return;
    }
    if (box) box.innerHTML = empty("Loading…");
    var url =
      (kind === "browser" ? "/api/browser/audit" : "/api/terminal/audit") + "?limit=50";
    fetch(url, {
      headers: token() ? { Authorization: "Bearer " + token() } : {},
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.json();
      })
      .then(function (payload) {
        var entries = (payload && payload.entries) || [];
        if (!box) return;
        box.innerHTML =
          kind === "browser" ? renderBrowser(entries) : renderTerminal(entries);
      })
      .catch(function (error) {
        if (box) box.innerHTML = empty("Could not load the ledger: " + error.message);
      })
      .finally(function () {
        state.busy = false;
      });
  }

  function bind() {
    var section = document.getElementById("ledger-section");
    if (!section || section.dataset.bound === "1") return;
    section.dataset.bound = "1";
    section.querySelectorAll(".ledger-tab").forEach(function (tab) {
      tab.addEventListener("click", function () {
        section.querySelectorAll(".ledger-tab").forEach(function (other) {
          other.classList.toggle("active", other === tab);
        });
        load(tab.getAttribute("data-kind"));
      });
    });
    var refresh = section.querySelector("#ledger-refresh");
    if (refresh) refresh.addEventListener("click", function () { load(state.kind); });
    load(state.kind);
  }

  document.addEventListener("DOMContentLoaded", bind);
  if (document.readyState !== "loading") bind();
  ns.load = load;
})((window.GBLedger = window.GBLedger || {}));
