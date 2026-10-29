/* Sandbox — unified approval queue for bot actions (GB6-21).
 * All sensitive bot actions (file writes, terminal commands, browser
 * navigation, external API calls) are routed through this sandbox.
 * The user sees a consolidated approval panel and can approve/deny
 * in bulk or individually. */
"use strict";

(function () {
  var pendingActions = [];
  var actionLog = [];
  var MAX_LOG = 200;

  var ACTION_CATEGORIES = {
    file_write: { label: "Write file", risk: "low", icon: "📝" },
    file_delete: { label: "Delete file", risk: "high", icon: "🗑️" },
    terminal: { label: "Terminal command", risk: "high", icon: "💻" },
    browser: { label: "Browser action", risk: "medium", icon: "🌐" },
    api_call: { label: "External API call", risk: "medium", icon: "🔗" },
    data_export: { label: "Export data", risk: "medium", icon: "📤" },
  };

  function addPending(action) {
    action._id = action.action_id || ("sandbox-" + Date.now() + "-" + Math.random().toString(36).substr(2, 4));
    action._timestamp = Date.now();
    pendingActions.push(action);
    updateBadge();
    return action._id;
  }

  function removePending(id) {
    pendingActions = pendingActions.filter(function (a) { return a._id !== id; });
    updateBadge();
  }

  function logAction(action, decision) {
    actionLog.unshift({
      id: action._id,
      category: action.category,
      detail: action.detail || action.command || action.url || "",
      decision: decision,
      timestamp: Date.now(),
    });
    if (actionLog.length > MAX_LOG) actionLog = actionLog.slice(0, MAX_LOG);
    try { localStorage.setItem("gb_sandbox_log", JSON.stringify(actionLog.slice(0, 50))); } catch (e) {}
  }

  function loadLog() {
    try { actionLog = JSON.parse(localStorage.getItem("gb_sandbox_log") || "[]"); } catch (e) { actionLog = []; }
  }

  function updateBadge() {
    var badge = document.getElementById("gb-sandbox-badge");
    if (!badge) return;
    if (pendingActions.length > 0) {
      badge.textContent = pendingActions.length;
      badge.style.display = "inline-flex";
    } else {
      badge.style.display = "none";
    }
  }

  function showSandboxPanel() {
    var panel = document.getElementById("gb-sandbox-panel");
    if (panel) panel.remove();
    panel = document.createElement("div");
    panel.id = "gb-sandbox-panel";
    panel.style.cssText = "position:fixed;top:0;right:0;width:380px;height:100vh;background:#1e293b;border-left:1px solid #334155;z-index:95000;display:flex;flex-direction:column;box-shadow:-4px 0 24px rgba(0,0,0,0.3);";

    var header = '<div style="display:flex;justify-content:space-between;align-items:center;padding:16px;border-bottom:1px solid #334155;"><h3 style="margin:0;color:#f8fafc;font-size:14px;">Sandbox — Pending Approvals</h3><button onclick="document.getElementById(\'gb-sandbox-panel\').remove()" style="background:none;border:none;color:#94a3b8;font-size:20px;cursor:pointer;">×</button></div>';

    var body = '<div style="flex:1;overflow-y:auto;padding:12px;">';
    if (pendingActions.length === 0) {
      body += '<p style="color:#64748b;text-align:center;padding:24px;">No pending actions.</p>';
    } else {
      body += '<div style="margin-bottom:12px;display:flex;gap:8px;">'
        + '<button onclick="GBSandbox.approveAll()" style="flex:1;background:#3b82f6;border:none;border-radius:6px;color:#fff;padding:8px;cursor:pointer;font-size:12px;">Approve All (' + pendingActions.length + ')</button>'
        + '<button onclick="GBSandbox.denyAll()" style="flex:1;background:#dc2626;border:none;border-radius:6px;color:#fff;padding:8px;cursor:pointer;font-size:12px;">Deny All</button>'
        + '</div>';
      pendingActions.forEach(function (a) {
        var cat = ACTION_CATEGORIES[a.category] || { label: a.category, icon: "⚙️", risk: "medium" };
        var riskColor = a.risk === "high" || cat.risk === "high" ? "#ef4444" : a.risk === "low" || cat.risk === "low" ? "#22c55e" : "#f59e0b";
        body += '<div style="border:1px solid #334155;border-radius:8px;padding:12px;margin-bottom:8px;">'
          + '<div style="display:flex;align-items:center;gap:8px;margin-bottom:6px;"><span>' + cat.icon + '</span><span style="color:#f8fafc;font-size:13px;font-weight:500;">' + escapeHtml(cat.label) + '</span><span style="margin-left:auto;font-size:10px;padding:2px 6px;border-radius:4px;background:' + riskColor + '20;color:' + riskColor + ';">' + (cat.risk || "medium") + '</span></div>'
          + '<div style="color:#94a3b8;font-size:12px;font-family:monospace;margin-bottom:8px;word-break:break-all;">' + escapeHtml((a.detail || a.command || a.url || "").substring(0, 200)) + '</div>'
          + '<div style="display:flex;gap:6px;">'
          + '<button onclick="GBSandbox.approve(\'' + a._id + '\')" style="background:#3b82f6;border:none;border-radius:4px;color:#fff;padding:4px 12px;cursor:pointer;font-size:11px;">Approve</button>'
          + '<button onclick="GBSandbox.deny(\'' + a._id + '\')" style="background:#334155;border:none;border-radius:4px;color:#f8fafc;padding:4px 12px;cursor:pointer;font-size:11px;">Deny</button>'
          + '</div></div>';
      });
    }
    body += '</div>';

    var logSection = '<div style="border-top:1px solid #334155;padding:12px;max-height:200px;overflow-y:auto;">'
      + '<div style="color:#64748b;font-size:11px;text-transform:uppercase;letter-spacing:0.05em;margin-bottom:8px;">Recent actions</div>';
    actionLog.slice(0, 20).forEach(function (entry) {
      var icon = entry.decision === "approved" ? "✅" : entry.decision === "denied" ? "🚫" : "⏳";
      logSection += '<div style="color:#94a3b8;font-size:11px;padding:3px 0;">' + icon + ' ' + escapeHtml(entry.category + ": " + (entry.detail || "").substring(0, 60)) + '</div>';
    });
    logSection += '</div>';

    panel.innerHTML = header + body + logSection;
    document.body.appendChild(panel);
  }

  function approve(id) {
    var action = pendingActions.find(function (a) { return a._id === id; });
    if (!action) return;
    logAction(action, "approved");
    removePending(id);
    if (action._resolve) action._resolve("approved");
    showSandboxPanel();
  }

  function deny(id) {
    var action = pendingActions.find(function (a) { return a._id === id; });
    if (!action) return;
    logAction(action, "denied");
    removePending(id);
    if (action._resolve) action._resolve("denied");
    showSandboxPanel();
  }

  function approveAll() {
    var copy = pendingActions.slice();
    copy.forEach(function (a) { logAction(a, "approved"); if (a._resolve) a._resolve("approved"); });
    pendingActions = [];
    updateBadge();
    showSandboxPanel();
  }

  function denyAll() {
    var copy = pendingActions.slice();
    copy.forEach(function (a) { logAction(a, "denied"); if (a._resolve) a._resolve("denied"); });
    pendingActions = [];
    updateBadge();
    showSandboxPanel();
  }

  loadLog();

  window.GBSandbox = {
    addPending: addPending,
    removePending: removePending,
    approve: approve,
    deny: deny,
    approveAll: approveAll,
    denyAll: denyAll,
    showPanel: showSandboxPanel,
    getPending: function () { return pendingActions.slice(); },
    getLog: function () { return actionLog.slice(); },
  };
})();
