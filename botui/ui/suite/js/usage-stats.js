/* Usage and cost transparency (GB6-25).
 * Tracks message counts, token estimates, and storage usage locally.
 * Shows a usage panel accessible from the sidebar. */
"use strict";

(function () {
  var STORAGE_KEY = "gb_usage_stats";
  var TOKEN_COST_PER_1K = { fast: 0.0004, balanced: 0.003, quality: 0.015, auto: 0.003 };

  function loadStats() {
    try { return JSON.parse(localStorage.getItem(STORAGE_KEY) || "{}"); } catch (e) { return {}; }
  }

  function saveStats(stats) {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(stats)); } catch (e) {}
  }

  function getToday() { return new Date().toISOString().split("T")[0]; }

  function trackMessage(role, contentLength, model) {
    var stats = loadStats();
    var today = getToday();
    if (!stats[today]) stats[today] = { messages_sent: 0, messages_received: 0, tokens_estimated: 0, cost_estimated: 0 };
    if (role === "user") {
      stats[today].messages_sent++;
    } else {
      stats[today].messages_received++;
    }
    var tokens = Math.ceil((contentLength || 0) / 4);
    stats[today].tokens_estimated += tokens;
    var costPer1k = TOKEN_COST_PER_1K[model || "auto"] || TOKEN_COST_PER_1K.auto;
    stats[today].cost_estimated += (tokens / 1000) * costPer1k;
    saveStats(stats);
  }

  function getUsageSummary(days) {
    var stats = loadStats();
    var result = { messages_sent: 0, messages_received: 0, tokens_estimated: 0, cost_estimated: 0, days: 0 };
    var cutoff = new Date();
    cutoff.setDate(cutoff.getDate() - (days || 30));
    var cutoffStr = cutoff.toISOString().split("T")[0];
    Object.keys(stats).forEach(function (day) {
      if (day >= cutoffStr) {
        var d = stats[day];
        result.messages_sent += d.messages_sent || 0;
        result.messages_received += d.messages_received || 0;
        result.tokens_estimated += d.tokens_estimated || 0;
        result.cost_estimated += d.cost_estimated || 0;
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

  function showUsagePanel() {
    var modal = document.getElementById("usage-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "usage-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:99999;backdrop-filter:blur(8px);";

    var week = getUsageSummary(7);
    var month = getUsageSummary(30);

    function statCard(label, value, sub) {
      return '<div style="background:#0f172a;border-radius:8px;padding:14px;text-align:center;">'
        + '<div style="color:#64748b;font-size:11px;text-transform:uppercase;letter-spacing:0.05em;margin-bottom:4px;">' + label + '</div>'
        + '<div style="color:#f8fafc;font-size:24px;font-weight:700;">' + value + '</div>'
        + (sub ? '<div style="color:#64748b;font-size:11px;margin-top:2px;">' + sub + '</div>' : '')
        + '</div>';
    }

    modal.innerHTML = '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:520px;padding:24px;">'
      + '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:20px;"><h3 style="margin:0;color:#f8fafc;">Usage &amp; Cost</h3><button onclick="document.getElementById(\'usage-modal\').remove()" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">×</button></div>'
      + '<div style="color:#94a3b8;font-size:12px;margin-bottom:16px;">Last 7 days</div>'
      + '<div style="display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin-bottom:20px;">'
      + statCard("Messages", week.messages_sent + week.messages_received, week.messages_sent + " sent")
      + statCard("Tokens", formatTokens(week.tokens_estimated), "estimated")
      + statCard("Cost", formatCost(week.cost_estimated), "estimated")
      + '</div>'
      + '<div style="color:#94a3b8;font-size:12px;margin-bottom:16px;">Last 30 days</div>'
      + '<div style="display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin-bottom:20px;">'
      + statCard("Messages", month.messages_sent + month.messages_received, month.messages_sent + " sent")
      + statCard("Tokens", formatTokens(month.tokens_estimated), "estimated")
      + statCard("Cost", formatCost(month.cost_estimated), "estimated")
      + '</div>'
      + '<p style="color:#475569;font-size:11px;margin:0;">Costs are estimated based on model pricing and may differ from actual billing. Token counts use a 4-character average.</p>'
      + '</div>';
    modal.addEventListener("click", function (e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);
  }

  document.addEventListener("gb-ws-frame", function (e) {
    var data = e.detail;
    if (!data) return;
    var content = data.content || data.text || "";
    if (data.message_type === 2 || data.role === "assistant" || data.role === "bot") {
      trackMessage("bot", content.length, data.model || "auto");
    }
  });

  window.addEventListener("gb-chat-message-sent", function (e) {
    var input = document.getElementById("messageInput");
    var model = "auto";
    var picker = document.getElementById("modelPicker");
    if (picker) model = picker.value || "auto";
    trackMessage("user", input ? (input.value || "").length : 0, model);
  });

  window.GBUsageStats = {
    track: trackMessage,
    getSummary: getUsageSummary,
    showPanel: showUsagePanel,
  };
})();
