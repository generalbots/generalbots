/* chat/modules/74_subagents.js — per-agent progress for parallel fan-out
 * (#1694). The server publishes `gb.subagent` frames with id/title/status and
 * an excerpt; this module keeps one card in the conversation and updates the
 * matching row in place, so finished parts stay reviewable. */
"use strict";

(function () {
  var rows = Object.create(null);
  var order = [];

  function esc(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function label(status) {
    switch (status) {
      case "running": return "working";
      case "succeeded": return "done";
      case "failed": return "failed";
      default: return "queued";
    }
  }

  function progress() {
    var out = { total: order.length, pending: 0, running: 0, succeeded: 0, failed: 0 };
    order.forEach(function (id) {
      var status = rows[id] && rows[id].status;
      if (status === "running") out.running += 1;
      else if (status === "succeeded") out.succeeded += 1;
      else if (status === "failed") out.failed += 1;
      else out.pending += 1;
    });
    out.done = out.succeeded + out.failed;
    out.percent = out.total ? Math.round((out.done * 100) / out.total) : 100;
    return out;
  }

  function host() {
    var card = document.getElementById("gb-subagents-card");
    if (card && document.body.contains(card)) return card;
    var messages = document.getElementById("messages");
    if (!messages) return null;
    card = document.createElement("div");
    card.id = "gb-subagents-card";
    card.className = "gb-subagents";
    card.setAttribute("role", "status");
    messages.appendChild(card);
    return card;
  }

  function render() {
    var card = host();
    if (!card) return;
    var p = progress();
    var overall =
      p.running > 0 ? "running" :
      p.pending > 0 ? "pending" :
      p.failed > 0 ? "failed" : "succeeded";
    var rowsHtml = order
      .map(function (id) {
        var row = rows[id] || {};
        var excerpt = row.result_excerpt ? String(row.result_excerpt).slice(0, 160) : "";
        return (
          '<div class="gb-subagent-row" data-agent-id="' + esc(id) + '">' +
          '<span class="gb-subagent-dot is-' + esc(row.status || "pending") + '" aria-hidden="true"></span>' +
          '<span class="gb-subagent-name">' + esc(row.title || id) + "</span>" +
          '<span class="gb-subagent-state">' + esc(label(row.status)) + "</span>" +
          (excerpt ? '<span class="gb-subagent-excerpt">' + esc(excerpt) + "</span>" : "") +
          "</div>"
        );
      })
      .join("");
    card.innerHTML =
      '<div class="gb-subagents-head"><span>Sub-agents</span>' +
      '<span class="gb-subagents-count">' + p.done + "/" + p.total + " done · " + p.percent + "%" +
      (overall === "failed" ? " · some failed" : "") + "</span></div>" +
      rowsHtml;
  }

  function apply(payload) {
    if (!payload || !payload.id) return null;
    if (!rows[payload.id]) order.push(payload.id);
    rows[payload.id] = {
      title: payload.title || (rows[payload.id] && rows[payload.id].title) || payload.id,
      status: payload.status || "pending",
      result_excerpt: payload.result_excerpt || "",
    };
    render();
    return progress();
  }

  function reset() {
    rows = Object.create(null);
    order = [];
    var card = document.getElementById("gb-subagents-card");
    if (card) card.remove();
  }

  window.GBSubagents = { apply: apply, reset: reset, progress: progress };
})();
