/* Usage and cost transparency (GB6-25).
 * Tracks message counts, token estimates, and storage usage locally.
 * Shows a usage panel accessible from the sidebar. */
"use strict";

(function () {
  var STORAGE_KEY = "gb_usage_stats";

  function loadStats() {
    try { return JSON.parse(localStorage.getItem(STORAGE_KEY) || "{}"); } catch (e) { return {}; }
  }

  function saveStats(stats) {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(stats)); } catch (e) {}
  }

  function getToday() { return new Date().toISOString().split("T")[0]; }

  function trackMessage(role, contentLength, model, messageId) {
    var stats = loadStats();
    var today = getToday();
    if (!stats[today]) stats[today] = { messages_sent: 0, messages_received: 0, tokens_estimated: 0 };
    // GB6-25 — a streamed reply arrives in several frames; counting each frame
    // doubled the numbers. A message id is counted once, and the caller only
    // reports complete frames.
    if (messageId) {
      var seen = stats[today].seen || [];
      if (seen.indexOf(messageId) !== -1) return;
      seen.push(messageId);
      stats[today].seen = seen.slice(-500);
    }
    if (role === "user") {
      stats[today].messages_sent++;
    } else {
      stats[today].messages_received++;
    }
    var tokens = Math.ceil((contentLength || 0) / 4);
    stats[today].tokens_estimated += tokens;
    // GB6-25 — no cost is derived locally: a hardcoded price list produced
    // numbers that could not be reconciled with billing. The server reports
    // cost only when the deployment publishes real prices.
    saveStats(stats);
  }

  function getUsageSummary(days) {
    var stats = loadStats();
    var result = { messages_sent: 0, messages_received: 0, tokens_estimated: 0, days: 0 };
    var cutoff = new Date();
    cutoff.setDate(cutoff.getDate() - (days || 30));
    var cutoffStr = cutoff.toISOString().split("T")[0];
    Object.keys(stats).forEach(function (day) {
      if (day >= cutoffStr) {
        var d = stats[day];
        result.messages_sent += d.messages_sent || 0;
        result.messages_received += d.messages_received || 0;
        result.tokens_estimated += d.tokens_estimated || 0;
        result.days++;
      }
    });
    return result;
  }

  function formatCost(amount) {
    if (amount < 0.01) return "< $0.01";
    return "$" + amount.toFixed(2);
  }

  function formatTokens(count) {
    if (count < 1000) return count + "";
    if (count < 1000000) return (count / 1000).toFixed(1) + "K";
    return (count / 1000000).toFixed(2) + "M";
  }

  function statCard(label, value, sub) {
    return '<div style="background:#0f172a;border-radius:8px;padding:14px;text-align:center;">'
      + '<div style="color:#64748b;font-size:11px;text-transform:uppercase;letter-spacing:0.05em;margin-bottom:4px;">' + label + '</div>'
      + '<div style="color:#f8fafc;font-size:24px;font-weight:700;">' + value + '</div>'
      + (sub ? '<div style="color:#64748b;font-size:11px;margin-top:2px;">' + sub + '</div>' : '')
      + '</div>';
  }

  function cardMarkup(title, summary, note) {
    return '<div style="color:#94a3b8;font-size:12px;margin-bottom:16px;">' + title + '</div>'
      + '<div style="display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin-bottom:20px;">'
      + statCard("Messages", summary.messages, summary.messages_sub)
      + statCard("Tokens", summary.tokens, summary.tokens_sub)
      + statCard("Cost", summary.cost, summary.cost_sub)
      + '</div>'
      + (note ? '<p style="color:#475569;font-size:11px;margin:0;">' + note + '</p>' : '');
  }

  function localSummary(days) {
    var local = getUsageSummary(days);
    return {
      messages: local.messages_sent + local.messages_received,
      messages_sub: local.messages_sent + " sent",
      tokens: formatTokens(local.tokens_estimated),
      tokens_sub: "estimated locally",
      cost: "unavailable",
      cost_sub: "no price list",
    };
  }

  async function serverSummary(days) {
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    var response = await fetch("/api/user/usage/summary?days=" + days, {
      headers: token ? { Authorization: "Bearer " + token } : {},
    });
    if (!response.ok) throw new Error("HTTP " + response.status);
    var payload = await response.json();
    var totals = payload.totals || {};
    var priced = payload.pricing_available === true && totals.cost != null;
    return {
      messages: totals.messages || 0,
      messages_sub: "recorded",
      tokens: formatTokens(totals.tokens || 0),
      tokens_sub: "recorded",
      cost: priced ? formatCost(totals.cost) : "unavailable",
      cost_sub: priced ? "from price list" : "no price list",
    };
  }

  async function showUsagePanel() {
    var modal = document.getElementById("usage-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "usage-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:99999;backdrop-filter:blur(8px);";
    modal.innerHTML = '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:520px;padding:24px;color:#94a3b8;">Loading usage…</div>';
    modal.addEventListener("click", function (e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);

    var week;
    var month;
    var note;
    try {
      week = await serverSummary(7);
      month = await serverSummary(30);
      note = "Counts come from recorded message history. Cost is reported only when the deployment publishes a price list (GB_LLM_PRICING); otherwise it stays unavailable instead of being estimated.";
    } catch (error) {
      week = localSummary(7);
      month = localSummary(30);
      note = "The usage service is unreachable, so these numbers are local estimates (4 characters per token). Cost stays unavailable because no authoritative price list was read.";
    }
    var box = modal.firstElementChild;
    if (!box) return;
    box.innerHTML = '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:20px;"><h3 style="margin:0;color:#f8fafc;">Usage &amp; Cost</h3><button type="button" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">×</button></div>'
      + cardMarkup("Last 7 days", week, "")
      + cardMarkup("Last 30 days", month, note);
    box.querySelector("button").addEventListener("click", function () { modal.remove(); });
  }

  document.addEventListener("gb-ws-frame", function (e) {
    var data = e.detail;
    if (!data) return;
    var content = data.content || data.text || "";
    if (data.is_complete === false) return;
    if (data.message_type === 2 || data.role === "assistant" || data.role === "bot") {
      var messageId = data.message_id || data.id || (data.timestamp || "") + ":" + content.length;
      trackMessage("bot", content.length, data.model || "auto", "bot:" + messageId);
    }
  });

  window.addEventListener("gb-chat-message-sent", function (e) {
    var input = document.getElementById("messageInput");
    var model = "auto";
    var picker = document.getElementById("modelPicker");
    if (picker) model = picker.value || "auto";
    var text = input ? (input.value || "") : "";
    trackMessage("user", text.length, model, "user:" + Date.now() + ":" + text.length);
  });

  window.GBUsageStats = {
    track: trackMessage,
    getSummary: getUsageSummary,
    showPanel: showUsagePanel,
  };
})();
