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

  function load(kind) {
    if (state.busy) return;
    state.busy = true;
    state.kind = kind;
    var box = list();
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
