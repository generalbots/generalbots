/* Host prerequisite panel (#1742, #1937).
 *
 * The desktop shell runs `PROD-DEPENDENCIES.sh --check` (Linux) or
 * `PROD-DEPENDENCIES.ps1 -Check` (Windows) before provisioning and renders the
 * result here. Desktop-shell failures always lock setup. On Windows, WSL2 is
 * required for agentic work and blocks agent-related setup unless the user
 * explicitly chooses "Skip ALL Agentic Work" (Chat/Drive-only). Media gaps
 * remain optional and expose "Continue anyway". */
(function () {
  "use strict";

  var B = window.GBWizardBridge;
  var I = window.WizardI18n;
  var esc = window.GBWizardSteps.esc;
  var t = function (key, params) { return I.t(key, params); };

  function draft() {
    return (window.GBWizardApp && window.GBWizardApp.state && window.GBWizardApp.state.draft) || {};
  }

  function skipAgentic() {
    return draft().skip_all_agentic_work === true;
  }

  function windowsAgentBlocked(result) {
    return !!(result && result.platform === "windows" && !result.agent_ready && result.desktop_ready && !skipAgentic());
  }

  function badgeState(result) {
    if (!result) { return { text: t("host.unknown"), kind: "" }; }
    if (windowsAgentBlocked(result)) { return { text: t("host.status-agent-blocked"), kind: "err" }; }
    if (result.status === "ready" || (skipAgentic() && result.desktop_ready && result.media_ready)) {
      return { text: t("host.status-ready"), kind: "ok" };
    }
    if (result.status === "ready") { return { text: t("host.status-ready"), kind: "ok" }; }
    if (result.status === "limited") { return { text: t("host.status-limited"), kind: "warn" }; }
    if (result.status === "desktop-blocked") { return { text: t("host.status-blocked"), kind: "err" }; }
    return { text: t("host.status-unavailable"), kind: "" };
  }

  function missingItems(result, feature) {
    if (!result || !result.items) { return []; }
    return result.items.filter(function (item) {
      return item.feature === feature && item.status === "missing";
    }).map(function (item) { return item.name; });
  }

  function locked(state) {
    if (state.host.checking) { return true; }
    var result = state.host.result;
    if (!state.host.checked || !result) { return false; }
    if (result.status === "desktop-blocked") { return true; }
    if (windowsAgentBlocked(result)) { return true; }
    // Media-only / other optional gaps: allow explicit continue.
    if (result.status === "limited" && !state.host.acknowledged) {
      // When agent is the only Windows blocker it is handled above; any
      // remaining limited state (e.g. media) still needs acknowledgement.
      return true;
    }
    return false;
  }

  function syncStartButton(state) {
    var start = document.getElementById("startSetupBtn");
    if (!start) { return; }
    start.disabled = state.provisioning.running || state.provisioning.ready || locked(state);
  }

  function render(state) {
    var badge = document.getElementById("hostBadge");
    var rows = document.getElementById("hostRows");
    var note = document.getElementById("hostNote");
    var recheck = document.getElementById("hostRecheckBtn");
    var proceed = document.getElementById("hostContinueBtn");
    var skipAgent = document.getElementById("hostSkipAgentBtn");
    var host = state.host;
    var result = host.result;
    var badgeInfo = badgeState(result);
    var agentBlocked = windowsAgentBlocked(result);
    if (badge) {
      badge.textContent = host.checking ? t("host.checking") : badgeInfo.text;
      badge.className = "badge " + badgeInfo.kind;
    }
    if (rows) {
      if (!result) { rows.innerHTML = ""; }
      else {
        rows.innerHTML = [["host.desktop", "desktop", result.desktop_ready],
          ["host.agent", "agent", skipAgentic() ? true : result.agent_ready],
          ["host.media", "media", result.media_ready]].map(function (row) {
          var missing = missingItems(result, row[1]);
          var ready = row[2];
          var text = ready
            ? (row[1] === "agent" && skipAgentic() && !result.agent_ready ? t("host.agent-skipped") : t("host.ready"))
            : (missing.length ? t("host.missing") + ": " + missing.join("; ") : t("host.missing"));
          return '<div class="summary-row"><span>' + esc(t(row[0])) + "</span>" +
            '<span class="note ' + (ready ? "ok" : "err") + '">' + esc(text) + "</span></div>";
        }).join("");
      }
    }
    if (note) {
      var message = "";
      var kind = "";
      if (host.checking) {
        message = t("host.checking");
      } else if (host.error) {
        message = t("host.check-failed", { reason: host.error });
        kind = "err";
      } else if (result) {
        if (agentBlocked) {
          message = t("host.agent-wsl-required");
          if (result.remediation) { message += " " + t("host.remediation") + ": " + result.remediation; }
          kind = "err";
        } else if (skipAgentic() && !result.agent_ready) {
          message = t("host.agent-skip-note");
          if (result.status === "limited" && !state.host.acknowledged && !result.media_ready) {
            message += " " + (result.message || "");
          }
          kind = "warn";
        } else {
          message = result.message || "";
          if (result.remediation && result.status !== "ready") {
            message += (message ? " " : "") + t("host.remediation") + ": " + result.remediation;
          }
          kind = result.status === "desktop-blocked" ? "err" : (result.status === "limited" ? "warn" : "ok");
        }
      }
      note.textContent = message;
      note.className = "note " + kind;
    }
    if (recheck) { recheck.disabled = host.checking || state.provisioning.running; }
    if (skipAgent) {
      skipAgent.hidden = !(agentBlocked && !state.provisioning.running && !state.provisioning.ready);
    }
    if (proceed) {
      // Continue anyway remains for non-agent optional gaps (e.g. media). It
      // must not bypass Windows WSL2 agent requirements (#1937).
      var showContinue = result && result.status === "limited" && result.desktop_ready &&
        !state.host.acknowledged && !state.provisioning.running && !state.provisioning.ready &&
        !agentBlocked;
      proceed.hidden = !showContinue;
    }
    syncStartButton(state);
  }

  function check(state, autoStart, onStart, onSay) {
    if (state.host.checking || state.provisioning.running) { return; }
    state.host.checking = true;
    state.host.checked = false;
    state.host.acknowledged = false;
    state.host.result = null;
    state.host.error = "";
    render(state);
    B.hostCheck().then(function (result) {
      state.host.checking = false;
      state.host.checked = true;
      state.host.result = result || null;
      render(state);
      if (autoStart && result && result.status !== "desktop-blocked" && !windowsAgentBlocked(result) && result.status !== "limited") {
        onStart();
      }
      if (result && result.status === "desktop-blocked") { onSay(t("host.blocked-say")); }
      if (windowsAgentBlocked(result)) { onSay(t("host.agent-wsl-say")); }
    }).catch(function (e) {
      state.host.checking = false;
      state.host.checked = true;
      state.host.result = null;
      state.host.error = String(e && e.message ? e.message : e);
      render(state);
      // A broken check must never trap the first run: continue without it.
      if (autoStart) { onStart(); }
    });
  }

  window.GBWizardHost = {
    render: render,
    check: check,
    locked: locked,
    syncStartButton: syncStartButton,
    windowsAgentBlocked: windowsAgentBlocked
  };
})();
