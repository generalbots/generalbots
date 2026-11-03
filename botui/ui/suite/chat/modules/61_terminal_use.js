/* Computer Use — terminal actions (GB6-20).
 * Listens for bot messages with action_type:"terminal" and renders a
 * consent card before relaying the command to the backend. The backend
 * executes the command in a sandboxed shell and returns stdout/stderr. */
"use strict";

(function () {
  var BLOCKED_PATTERNS = [
    /rm\s+(-rf?|--force)\s+\//i,
    /mkfs/i,
    /dd\s+if=/i,
    /shutdown/i,
    /reboot/i,
    /format\s+[a-z]:/i,
  ];

  function isSafe(cmd) {
    for (var i = 0; i < BLOCKED_PATTERNS.length; i++) {
      if (BLOCKED_PATTERNS[i].test(cmd)) return false;
    }
    return true;
  }

  function postJson(path, body) {
    var headers = { "Content-Type": "application/json" };
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    if (token) headers.Authorization = "Bearer " + token;
    return fetch(path, { method: "POST", headers: headers, body: JSON.stringify(body || {}) })
      .then(function (res) {
        if (!res.ok) {
          return res.json().catch(function () { return {}; }).then(function (payload) {
            throw new Error(payload.error || ("HTTP " + res.status));
          });
        }
        return res.json();
      });
  }

  function renderTerminalOutput(text) {
    var el = document.createElement("pre");
    el.style.cssText = "background:#0f172a;color:#e2e8f0;padding:12px 16px;border-radius:8px;font-size:12px;line-height:1.5;max-height:300px;overflow:auto;margin-top:8px;white-space:pre-wrap;word-break:break-all;";
    el.textContent = text;
    return el;
  }

  async function executeTerminalAction(action, cardEl) {
    var cmd = action.command;
    if (!cmd) return;

    if (!isSafe(cmd)) {
      cardEl.innerHTML = '<div class="gb-consent-outcome"><span>🚫</span> <span>Command blocked — destructive pattern detected</span></div>';
      sendTerminalResult(action.action_id, { status: "blocked", error: "Command matches a blocked destructive pattern" });
      return;
    }

    cardEl.innerHTML = '<div class="gb-consent-outcome"><span>⏳</span> <span>Recording approval...</span></div>';

    // #1645 — the approval is a server-side consent record bound to the action
    // id; `approved=true` in the body alone is refused.
    try {
      await postJson("/api/computer/actions/" + encodeURIComponent(action.action_id) + "/approve", {
        action_id: action.action_id,
      });
    } catch (err) {
      cardEl.innerHTML = '<div class="gb-consent-outcome"><span>❌</span> <span>' +
        escapeHtml(err.message) + '</span></div>';
      sendTerminalResult(action.action_id, { status: "error", error: err.message });
      return;
    }

    cardEl.innerHTML = '<div class="gb-consent-outcome"><span>⏳</span> <span>Running...</span></div>';

    postJson("/api/terminal/exec", {
      command: cmd,
      cwd: action.cwd || "",
      timeout_ms: action.timeout_ms || 30000,
      approved: true,
      action_id: action.action_id,
      session_id: (ChatState && ChatState.currentSessionId) || "",
    })
      .then(function (result) {
        if (result.status === "failed" || result.status === "error") {
          cardEl.innerHTML = '<div class="gb-consent-outcome"><span>❌</span> <span>' +
            escapeHtml(result.error || result.stderr || "Command failed") + '</span></div>';
          sendTerminalResult(action.action_id, { status: "error", error: result.error || result.stderr || "failed" });
          return;
        }
        var output = (result.stdout || "") + (result.stderr ? "\n[stderr]\n" + result.stderr : "");
        var exitCode = result.exit_code != null ? result.exit_code : -1;
        var statusIcon = exitCode === 0 ? "✅" : "⚠️";
        cardEl.innerHTML = "";
        var outcomeDiv = document.createElement("div");
        outcomeDiv.className = "gb-consent-outcome";
        outcomeDiv.innerHTML = '<span>' + statusIcon + '</span> <span>Exit code: ' + exitCode + '</span>';
        cardEl.appendChild(outcomeDiv);
        if (output.trim()) cardEl.appendChild(renderTerminalOutput(output.substring(0, 5000)));
        sendTerminalResult(action.action_id, { status: "ok", exit_code: exitCode, output: output.substring(0, 5000) });
      })
      .catch(function (err) {
        cardEl.innerHTML = '<div class="gb-consent-outcome"><span>❌</span> <span>Failed: ' + escapeHtml(err.message) + '</span></div>';
        sendTerminalResult(action.action_id, { status: "error", error: err.message });
      });
  }

  function sendTerminalResult(actionId, result) {
    if (actionId && /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(actionId)) {
      var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
      fetch("/api/computer/actions/" + encodeURIComponent(actionId) + "/result", {
        method: "POST",
        headers: token
          ? { "Content-Type": "application/json", Authorization: "Bearer " + token }
          : { "Content-Type": "application/json" },
        body: JSON.stringify(result),
      }).catch(function () { /* the chat frame below is the fallback */ });
    }
    if (ChatState && ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
      ChatState.ws.send(JSON.stringify({
        bot_id: ChatState.currentBotId,
        user_id: ChatState.currentUserId,
        session_id: ChatState.currentSessionId,
        message_type: "terminal_result",
        action_id: actionId,
        result: result,
        timestamp: new Date().toISOString(),
      }));
    }
  }

  document.addEventListener("gb-ws-frame", function (e) {
    var data = e.detail;
    if (!data) return;
    if (data.action_type !== "terminal" && data.message_type !== "terminal_action") return;
    var action = data.terminal_action || data;
    if (!action.command) return;

    var pane = document.getElementById("messages");
    if (!pane) return;

    var wrap = document.createElement("div");
    wrap.className = "message bot";
    wrap.innerHTML = '<div class="message-content bot-message"></div>';
    var body = wrap.querySelector(".message-content");

    var card = document.createElement("div");
    card.className = "gb-consent-card";
    var reqId = action.action_id || ("term-" + Date.now());
    card.setAttribute("data-request-id", reqId);

    var cmdDisplay = action.command.length > 120 ? action.command.substring(0, 117) + "..." : action.command;
    card.innerHTML =
      '<div class="gb-consent-head"><span class="gb-consent-glyph">💻</span><div class="gb-consent-headings"><div class="gb-consent-title">Terminal Command</div><div class="gb-consent-app">shell</div></div></div>'
      + '<div class="gb-consent-detail" style="font-family:monospace;font-size:13px;background:#0f172a;padding:10px 14px;border-radius:6px;margin-bottom:10px;">' + escapeHtml(action.command) + '</div>'
      + (action.reason ? '<div style="color:#94a3b8;font-size:12px;margin-bottom:10px;">' + escapeHtml(action.reason) + '</div>' : '')
      + '<div class="gb-consent-actions"><button type="button" class="gb-consent-btn primary" data-decision="allow">Run</button><button type="button" class="gb-consent-btn danger" data-decision="deny">Deny</button></div>';

    if (window.GBSandbox) {
      GBSandbox.addPending({
        action_id: reqId,
        category: "terminal",
        detail: action.command,
        risk: "high",
      });
    }

    card.addEventListener("click", function (ev) {
      var btn = ev.target.closest(".gb-consent-btn");
      if (!btn) return;
      var decision = btn.getAttribute("data-decision");
      card.querySelectorAll(".gb-consent-btn").forEach(function (b) { b.disabled = true; });
      if (decision === "allow") {
        executeTerminalAction(action, card);
        if (window.GBSandbox) GBSandbox.removePending(reqId);
      } else {
        card.classList.add("resolved");
        card.innerHTML = '<div class="gb-consent-outcome"><span>🚫</span> <span>Denied by user</span></div>';
        sendTerminalResult(reqId, { status: "denied" });
        if (window.GBSandbox) GBSandbox.removePending(reqId);
      }
    });

    body.appendChild(card);
    pane.appendChild(wrap);
    if (typeof scrollToBottom === "function") scrollToBottom(true);
  });

  // GB6-20 — the server no longer needs to push over the WebSocket: the action
  // queue is polled and replayed as the same frames the pipeline emits.
  function pollPendingTerminal() {
    var sid = window.ChatState && ChatState.currentSessionId;
    if (!sid) return;
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    fetch("/api/computer/actions/pending?kind=terminal&session_id=" + encodeURIComponent(sid), {
      headers: token ? { Authorization: "Bearer " + token } : {},
    })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        (data && data.actions ? data.actions : []).forEach(function (entry) {
          document.dispatchEvent(new CustomEvent("gb-ws-frame", {
            detail: {
              message_type: "terminal_action",
              action_type: "terminal",
              terminal_action: Object.assign({ action_id: entry.action_id }, entry.action || {}),
            },
          }));
        });
      })
      .catch(function () { /* offline: nothing to collect */ });
  }
  setInterval(pollPendingTerminal, 3000);

  // #1645 — the durable trail is visible without a console: a taskbar button
  // opens the last 50 commands with decision, exit code and timestamp.
  function ensureAuditButton() {
    if (document.getElementById("gbTerminalAuditBtn")) return;
    var host = document.querySelector(".taskbar-tray");
    if (!host) return;
    var button = document.createElement("button");
    button.id = "gbTerminalAuditBtn";
    button.className = "tray-btn";
    button.title = "Terminal audit trail";
    button.textContent = "⌨";
    button.addEventListener("click", openTerminalAudit);
    host.appendChild(button);
  }

  async function openTerminalAudit() {
    var existing = document.getElementById("gb-terminal-audit");
    if (existing) { existing.remove(); return; }
    var overlay = document.createElement("div");
    overlay.id = "gb-terminal-audit";
    overlay.style.cssText = "position:fixed;inset:0;background:rgba(15,23,42,0.8);z-index:9998;display:flex;align-items:center;justify-content:center;";
    var panel = document.createElement("div");
    panel.style.cssText = "background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:760px;max-height:80vh;overflow:auto;padding:20px;color:#e2e8f0;font:13px system-ui;";
    panel.innerHTML = '<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">' +
      '<strong>Terminal audit trail</strong>' +
      '<button id="gbTerminalAuditClose" style="background:#334155;border:none;border-radius:6px;color:#f8fafc;padding:6px 12px;cursor:pointer;">Close</button></div>' +
      '<p style="color:#94a3b8;margin:0 0 12px;">Last 50 commands · retained 90 days. Denied requests are listed too.</p>' +
      '<div id="gbTerminalAuditBody">Loading…</div>';
    overlay.appendChild(panel);
    overlay.addEventListener("click", function (e) { if (e.target === overlay) overlay.remove(); });
    document.body.appendChild(overlay);
    panel.querySelector("#gbTerminalAuditClose").addEventListener("click", function () { overlay.remove(); });

    try {
      var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
      var response = await fetch("/api/terminal/audit?limit=50", {
        headers: token ? { Authorization: "Bearer " + token } : {},
      });
      if (!response.ok) throw new Error("HTTP " + response.status);
      var payload = await response.json();
      var entries = payload.entries || [];
      var body = panel.querySelector("#gbTerminalAuditBody");
      if (!entries.length) {
        body.innerHTML = '<p style="color:#94a3b8;">No terminal command has been requested yet.</p>';
        return;
      }
      body.innerHTML = '<table style="width:100%;border-collapse:collapse;">' +
        '<thead><tr style="text-align:left;color:#94a3b8;"><th style="padding:6px;">When</th><th>Command</th><th>Decision</th><th>Exit</th></tr></thead><tbody>' +
        entries.map(function (entry) {
          return '<tr style="border-top:1px solid #334155;">' +
            '<td style="padding:6px;color:#94a3b8;">' + escapeHtml(entry.created_at || "") + '</td>' +
            '<td style="padding:6px;"><code>' + escapeHtml(entry.command || "") + '</code></td>' +
            '<td style="padding:6px;">' + escapeHtml(entry.decision || "") + '</td>' +
            '<td style="padding:6px;">' + (entry.exit_code == null ? "—" : escapeHtml(String(entry.exit_code))) + '</td>' +
            '</tr>';
        }).join("") + "</tbody></table>";
    } catch (err) {
      panel.querySelector("#gbTerminalAuditBody").textContent = "Audit unavailable: " + err.message;
    }
  }

  document.addEventListener("DOMContentLoaded", ensureAuditButton);
  if (document.readyState !== "loading") ensureAuditButton();
  window.GBTerminalAudit = { open: openTerminalAudit };
})();
