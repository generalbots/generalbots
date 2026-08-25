"use strict";
/* Agent Control (#1176): desktop-wide permission layer for agent actions.
   Every app an agent wants to drive is gated by the user's consent, stored
   per-app in localStorage. Consent states:
     "allow"  — the agent may act in this app without asking
     "deny"   — the agent may never act in this app
     "ask"    — prompt the user every time (default)
   The settings panel (AgentControl.showPanel) renders the full matrix. */

const AgentControl = (() => {
  const STORAGE_KEY = "gb.agent.consent";
  const DEFAULT_ACTIONS = ["read", "write", "execute", "open"];

  function loadConsent() {
    try {
      return JSON.parse(localStorage.getItem(STORAGE_KEY)) || {};
    } catch (_e) {
      return {};
    }
  }

  function saveConsent(consent) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(consent));
    } catch (_e) {
      /* storage full or blocked — non-fatal */
    }
  }

  function permission(appId, action) {
    const consent = loadConsent();
    const app = consent[appId] || {};
    return app[action] || app["*"] || "ask";
  }

  function setPermission(appId, action, value) {
    const consent = loadConsent();
    consent[appId] = consent[appId] || {};
    consent[appId][action] = value;
    saveConsent(consent);
  }

  function knownApps() {
    const registry = window.APPS_REGISTRY || [];
    if (registry.length) {
      return registry.map((a) => ({ id: a.id, title: a.title || a.id }));
    }
    return [{ id: "default", title: "All apps" }];
  }

  /* Ask the user once for a specific app+action. resolve(true) allows and
     remembers; resolve(false) denies for this call only. Rendered as a
     floating tool window (VB6-style), never a modal or native dialog. */
  function ask(appId, action, reason, resolve) {
    const app = (window.APPS_REGISTRY || []).find((a) => a.id === appId);
    const title = (app && app.title) || appId;
    const html =
      '<div class="gb-ac-ask">' +
      '<p>The desktop agent wants to <strong>' + escapeHtml(action) + "</strong> in “" + escapeHtml(title) + "”" +
      (reason ? " (" + escapeHtml(reason) + ")" : "") + ".</p>" +
      '<label class="gb-ac-ask-always"><input type="checkbox" id="gbAcAlways" /> Always allow in this app</label>' +
      '<div class="gb-ac-ask-actions">' +
      '<button id="gbAcDeny" class="gb-ac-btn danger">Deny</button>' +
      '<button id="gbAcAllow" class="gb-ac-btn">Allow</button>' +
      "</div></div>";
    const cleanup = function () {
      if (window.WindowManager && window.WindowManager.close) {
        window.WindowManager.close("agent-ask");
      } else {
        const host = document.getElementById("gbAgentAskHost");
        if (host) host.remove();
      }
    };
    const wireAsk = function () {
      const allowBtn = document.getElementById("gbAcAllow");
      const denyBtn = document.getElementById("gbAcDeny");
      const alwaysBox = document.getElementById("gbAcAlways");
      if (allowBtn) {
        allowBtn.addEventListener("click", function () {
          if (alwaysBox && alwaysBox.checked) setPermission(appId, action, "allow");
          cleanup();
          resolve(true);
        });
      }
      if (denyBtn) {
        denyBtn.addEventListener("click", function () {
          cleanup();
          resolve(false);
        });
      }
    };
    if (window.WindowManager && window.WindowManager.openToolWindowBody) {
      const body = window.WindowManager.openToolWindowBody("agent-ask", "Agent permission needed", { htmlContent: html });
      if (body) {
        wireAsk();
        return;
      }
    }
    // Isolated fallback: small floating panel, still not a native modal.
    const host = document.createElement("div");
    host.id = "gbAgentAskHost";
    host.innerHTML = html;
    host.style.cssText =
      "position:fixed;top:25%;left:35%;z-index:9999;background:var(--gb-surface,#fff);" +
      "border:1px solid var(--gb-border,#ccc);border-radius:8px;box-shadow:0 8px 28px rgba(0,0,0,.25);" +
      "padding:1rem;min-width:320px;font-family:system-ui,sans-serif";
    document.body.appendChild(host);
    wireAsk();
  }

  /* Gate an agent action: runs fn only when the user's consent allows it.
     When consent is "ask", prompts first in a floating tool window. */
  function gate(appId, action, reason, fn) {
    const state = permission(appId, action);
    if (state === "deny") {
      notifyAgent("Blocked", "Agent action “" + action + "” in “" + appId + "” is denied.", "error");
      return;
    }
    if (state === "allow") {
      fn();
      return;
    }
    ask(appId, action, reason, function (allowed) {
      if (allowed) fn();
      else notifyAgent("Blocked", "Agent action “" + action + "” in “" + appId + "” was declined.", "warning");
    });
  }

  function notifyAgent(title, message, kind) {
    if (window.GBToasts && window.GBToasts.show) {
      window.GBToasts.show(title, message, kind);
    } else if (window.console) {
      window.console.log("[" + title + "] " + message);
    }
  }

  /* Renders the per-app consent matrix in a floating settings window. */
  function showPanel() {
    const apps = knownApps();
    const consent = loadConsent();
    let rows = "";
    for (const app of apps) {
      const per = consent[app.id] || {};
      const cells = DEFAULT_ACTIONS.map(function (action) {
        const value = per[action] || per["*"] || "ask";
        return (
          "<td><select data-app=\"" + escapeAttr(app.id) + "\" data-action=\"" + action + "\">" +
          '<option value="allow"' + (value === "allow" ? " selected" : "") + '>Allow</option>' +
          '<option value="ask"' + (value === "ask" ? " selected" : "") + '>Ask</option>' +
          '<option value="deny"' + (value === "deny" ? " selected" : "") + '>Deny</option>' +
          "</select></td>"
        );
      }).join("");
      rows +=
        "<tr><td><strong>" + escapeHtml(app.title) + "</strong><br><code>" + escapeHtml(app.id) + "</code></td>" +
        cells +
        "</tr>";
    }
    const html =
      '<div class="gb-agent-control">' +
      "<h3>🤖 Agent permissions</h3>" +
      '<p class="gb-ac-hint">The desktop agent (Concierge/Spotlight) must ask before acting in each app. Set a default per action below.</p>' +
      '<table class="gb-ac-table"><thead><tr><th>App</th>' +
      DEFAULT_ACTIONS.map(function (a) { return "<th>" + a + "</th>"; }).join("") +
      "</tr></thead><tbody>" + rows + "</tbody></table>" +
      '<button id="gbAcClose" class="gb-ac-btn">Done</button>' +
      "</div>";

    if (window.WindowManager && window.WindowManager.openToolWindowBody) {
      window.WindowManager.openToolWindowBody("agent-control", "Agent Control", { htmlContent: html });
      wirePanelEvents();
    } else {
      const host = document.createElement("div");
      host.id = "gbAgentControlHost";
      host.innerHTML = html;
      document.body.appendChild(host);
      wirePanelEvents();
    }
  }

  function wirePanelEvents() {
    const panel = document.getElementById("gbAgentControlHost");
    document.querySelectorAll(".gb-agent-control select[data-app]").forEach(function (sel) {
      sel.addEventListener("change", function () {
        setPermission(sel.dataset.app, sel.dataset.action, sel.value);
      });
    });
    const closeBtn = document.getElementById("gbAcClose");
    if (closeBtn) {
      closeBtn.addEventListener("click", function () {
        if (window.WindowManager && window.WindowManager.close) {
          window.WindowManager.close("agent-control");
        } else if (panel) {
          panel.remove();
        }
      });
    }
  }

  function escapeHtml(s) {
    return String(s == null ? "" : s).replace(/[&<>"']/g, function (c) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c];
    });
  }
  function escapeAttr(s) {
    return escapeHtml(s);
  }

  function init() {
    window.AgentControl = {
      permission: permission,
      setPermission: setPermission,
      gate: gate,
      showPanel: showPanel,
      DEFAULT_ACTIONS: DEFAULT_ACTIONS,
    };
    // Global shortcut for app HTMLs: window.AgentControl.gate('drive','write',…)
    window.addEventListener("gb-agent-control-panel", function () {
      showPanel();
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
  return {
    init: init,
    gate: gate,
    permission: permission,
    setPermission: setPermission,
    showPanel: showPanel,
  };
})();
