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

  function renderTerminalOutput(text) {
    var el = document.createElement("pre");
    el.style.cssText = "background:#0f172a;color:#e2e8f0;padding:12px 16px;border-radius:8px;font-size:12px;line-height:1.5;max-height:300px;overflow:auto;margin-top:8px;white-space:pre-wrap;word-break:break-all;";
    el.textContent = text;
    return el;
  }

  function executeTerminalAction(action, cardEl) {
    var cmd = action.command;
    if (!cmd) return;

    if (!isSafe(cmd)) {
      cardEl.innerHTML = '<div class="gb-consent-outcome"><span>🚫</span> <span>Command blocked — destructive pattern detected</span></div>';
      sendTerminalResult(action.action_id, { status: "blocked", error: "Command matches a blocked destructive pattern" });
      return;
    }

    cardEl.innerHTML = '<div class="gb-consent-outcome"><span>⏳</span> <span>Running...</span></div>';

    var headers = { "Content-Type": "application/json" };
    var tok = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (tok) headers["Authorization"] = "Bearer " + tok;

    fetch("/api/terminal/exec", {
      method: "POST",
      headers: headers,
      body: JSON.stringify({
        command: cmd,
        cwd: action.cwd || "",
        timeout_ms: action.timeout_ms || 30000,
        session_id: (ChatState && ChatState.currentSessionId) || "",
      }),
    })
      .then(function (r) { return r.json(); })
      .then(function (result) {
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

    card.addEventListener("click", function (ev) {
      var btn = ev.target.closest(".gb-consent-btn");
      if (!btn) return;
      var decision = btn.getAttribute("data-decision");
      card.querySelectorAll(".gb-consent-btn").forEach(function (b) { b.disabled = true; });
      if (decision === "allow") {
        executeTerminalAction(action, card);
      } else {
        card.classList.add("resolved");
        card.innerHTML = '<div class="gb-consent-outcome"><span>🚫</span> <span>Denied by user</span></div>';
        sendTerminalResult(reqId, { status: "denied" });
      }
    });

    body.appendChild(card);
    pane.appendChild(wrap);
    if (typeof scrollToBottom === "function") scrollToBottom(true);
  });
})();
