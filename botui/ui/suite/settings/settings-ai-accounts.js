"use strict";

// AI accounts (#1654/#1655/#1656): per-user Claude Code and ChatGPT plan
// subscriptions. The API only returns status metadata; credentials stay in
// the agent VM (Claude) or Vault (ChatGPT).
(function () {
  var AGENTS = ["reasoning", "agentic", "fast"];
  var pollTimer = null;

  function token() {
    if (typeof window.getGBAccessToken === "function") {
      var value = window.getGBAccessToken();
      if (value) return value;
    }
    return (
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      ""
    );
  }

  function api(path, options) {
    var headers = { "Content-Type": "application/json" };
    var value = token();
    if (value) headers.Authorization = "Bearer " + value;
    return fetch(path, Object.assign({ headers: headers }, options || {}));
  }

  function json(path, options) {
    return api(path, options).then(function (response) {
      return response.json().catch(function () {
        return {};
      }).then(function (body) {
        if (!response.ok) {
          throw new Error(body.error || body.detail || "HTTP " + response.status);
        }
        return body;
      });
    });
  }

  function escapeHtml(value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function label(status) {
    switch (status) {
      case "linked": return "Connected";
      case "connecting": return "Connecting…";
      case "expired": return "Expired";
      case "error": return "Connection failed";
      default: return "Not connected";
    }
  }

  function status(message) {
    var node = document.getElementById("ai-accounts-status");
    if (node) node.textContent = message || "";
  }

  function card(account) {
    var provider = account.provider;
    var linked = account.status === "linked";
    var agentOptions = AGENTS.map(function (agent) {
      return '<option value="' + agent + '">' + agent + "</option>";
    }).join("");
    return (
      '<div class="setting-row" data-ai-account="' + escapeHtml(provider) + '" style="display:block;padding:14px 0;border-bottom:1px solid var(--border,#262b36)">' +
        '<div style="display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap">' +
          '<div>' +
            '<strong>' + (provider === "claude-code" ? "Claude Code" : "ChatGPT plan") + "</strong>" +
            '<div style="font-size:12px;color:var(--text-muted,#8b93a7)">' +
              (provider === "claude-code"
                ? "Runs the Claude Code CLI inside your agent VM; the credential never leaves it."
                : "Uses your ChatGPT plan through the local OAuth loopback; the token lives in Vault.") +
            "</div>" +
          "</div>" +
          '<span class="badge" data-ai-status="' + escapeHtml(provider) + '">' + label(account.status) + "</span>" +
        "</div>" +
        (linked && account.account_email
          ? '<div style="font-size:12px;color:var(--text-muted,#8b93a7);margin-top:4px">' + escapeHtml(account.account_email) + (account.plan ? " · " + escapeHtml(account.plan) : "") + "</div>"
          : "") +
        '<div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin-top:10px">' +
          '<button type="button" class="btn-primary btn-sm" data-ai-connect="' + escapeHtml(provider) + '"' + (linked ? " hidden" : "") + ">Connect</button>" +
          '<button type="button" class="btn-secondary btn-sm" data-ai-disconnect="' + escapeHtml(provider) + '"' + (linked ? "" : " hidden") + ">Disconnect</button>" +
          (linked
            ? '<select data-ai-agent="' + escapeHtml(provider) + '" style="padding:8px 10px;border:1px solid var(--border,#334155);border-radius:6px;background:var(--bg,#14142a);color:var(--text,#eee)">' + agentOptions + "</select>" +
              '<button type="button" class="btn-secondary btn-sm" data-ai-use="' + escapeHtml(provider) + '">Use for Vibe</button>'
            : "") +
        "</div>" +
        (provider === "claude-code"
          ? '<div data-ai-code-row="claude-code" hidden style="margin-top:10px">' +
              '<input type="text" data-ai-code="claude-code" placeholder="Paste the authorization code from Claude" style="width:min(520px,100%);padding:8px 10px;border:1px solid var(--border,#334155);border-radius:6px;background:var(--bg,#14142a);color:var(--text,#eee)" />' +
              '<button type="button" class="btn-primary btn-sm" data-ai-submit="claude-code" style="margin-left:8px">Submit code</button>' +
            "</div>"
          : "") +
      "</div>"
    );
  }

  function render(accounts) {
    var root = document.getElementById("ai-accounts-list");
    if (!root) return;
    if (!accounts.length) {
      root.innerHTML = '<div class="empty-state"><p>No AI account providers are available in this build.</p></div>';
      return;
    }
    root.innerHTML = accounts.map(card).join("");
    bind(accounts);
    var connecting = accounts.some(function (account) { return account.status === "connecting"; });
    if (connecting && !pollTimer) {
      pollTimer = setInterval(load, 3000);
    } else if (!connecting && pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  function update(provider, nextStatus, message) {
    var badge = document.querySelector('[data-ai-status="' + provider + '"]');
    if (badge) badge.textContent = label(nextStatus);
    var connect = document.querySelector('[data-ai-connect="' + provider + '"]');
    var disconnect = document.querySelector('[data-ai-disconnect="' + provider + '"]');
    if (connect) connect.hidden = nextStatus === "linked";
    if (disconnect) disconnect.hidden = nextStatus !== "linked";
    var row = document.querySelector('[data-ai-code-row="' + provider + '"]');
    if (row) row.hidden = !(provider === "claude-code" && (nextStatus === "connecting" || nextStatus === "waiting"));
    if (message) status(message);
  }

  function botId() {
    return json("/api/apps/integrations/context")
      .then(function (context) { return context.bot_id || ""; })
      .catch(function () { return ""; });
  }

  function bind(accounts) {
    accounts.forEach(function (account) {
      var provider = account.provider;
      var connect = document.querySelector('[data-ai-connect="' + provider + '"]');
      if (connect) {
        connect.addEventListener("click", function () {
          connect.disabled = true;
          update(provider, "connecting", "Starting the sign-in…");
          json("/api/ai-accounts/" + encodeURIComponent(provider) + "/connect", {
            method: "POST",
            body: "{}",
          })
            .then(function (data) {
              if (data.url) window.open(data.url, "_blank", "noopener");
              update(provider, data.status || data.state || "connecting", data.message || "");
              if (!pollTimer) pollTimer = setInterval(load, 3000);
            })
            .catch(function (error) { update(provider, "error", error.message); })
            .then(function () { connect.disabled = false; });
        });
      }
      var disconnect = document.querySelector('[data-ai-disconnect="' + provider + '"]');
      if (disconnect) {
        disconnect.addEventListener("click", function () {
          json("/api/ai-accounts/" + encodeURIComponent(provider) + "/disconnect", {
            method: "POST",
            body: "{}",
          })
            .then(function () { load(); })
            .catch(function (error) { status(error.message); });
        });
      }
      var submit = document.querySelector('[data-ai-submit="' + provider + '"]');
      if (submit) {
        submit.addEventListener("click", function () {
          var input = document.querySelector('[data-ai-code="' + provider + '"]');
          var code = input ? input.value.trim() : "";
          if (!code) return;
          submit.disabled = true;
          json("/api/ai-accounts/claude-code/code", {
            method: "POST",
            body: JSON.stringify({ code: code }),
          })
            .then(function (data) {
              update(provider, data.status || data.state || "connecting", data.message || "");
              if (input) input.value = "";
              load();
            })
            .catch(function (error) { status(error.message); })
            .then(function () { submit.disabled = false; });
        });
      }
      var use = document.querySelector('[data-ai-use="' + provider + '"]');
      if (use) {
        use.addEventListener("click", function () {
          var select = document.querySelector('[data-ai-agent="' + provider + '"]');
          var agent = select ? select.value : "agentic";
          use.disabled = true;
          botId().then(function (id) {
            if (!id) throw new Error("No active bot; open a bot first.");
            return json("/api/ai-accounts/" + encodeURIComponent(provider) + "/use-for-vibe", {
              method: "POST",
              body: JSON.stringify({ bot_id: id, agent: agent }),
            });
          })
            .then(function (data) {
              status(data.provider + " assigned to the " + data.agent + " Vibe slot (" + data.model + ").");
            })
            .catch(function (error) { status(error.message); })
            .then(function () { use.disabled = false; });
        });
      }
    });
  }

  function load() {
    var root = document.getElementById("ai-accounts-list");
    if (!root) return;
    json("/api/ai-accounts")
      .then(function (data) {
        var accounts = data.accounts || [];
        render(accounts);
        var anyLinked = accounts.some(function (account) { return account.status === "linked"; });
        if (!anyLinked) status("No subscription account is connected yet.");
      })
      .catch(function (error) {
        root.innerHTML = '<div class="empty-state"><p>AI accounts unavailable: ' + escapeHtml(error.message) + "</p></div>";
      });
  }

  function openFromHash() {
    if (window.location.hash === "#ai-accounts") load();
  }

  document.addEventListener("DOMContentLoaded", function () {
    openFromHash();
    document.querySelectorAll('a[href="#ai-accounts"]').forEach(function (link) {
      link.addEventListener("click", function () { setTimeout(load, 0); });
    });
  });
  window.addEventListener("hashchange", openFromHash);
  window.loadAiAccounts = load;
})();
