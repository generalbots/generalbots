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

  // The desktop edition reads text in images with the system's own engine and
  // offers no barcode reading, so the generic media tools are not required.
  function mediaRequired() {
    var app = window.GBWizardApp;
    return !(app && app.state && app.state.context && app.state.context.desktop_local);
  }

  function mediaOk(result) {
    return !!result && (result.media_ready || !mediaRequired());
  }

  // A host whose only gaps are the skipped agent (Chat/Drive-only mode, #1961)
  // or media tools this edition does not use is ready: no "Continue anyway".
  function skippedAgentOnlyGap(result) {
    return !!(result && result.desktop_ready && mediaOk(result) &&
      (result.agent_ready || skipAgentic()));
  }

  function hostReady(result) {
    return !!result && (result.status === "ready" || skippedAgentOnlyGap(result));
  }

  /* One sentence for the summary row: running, ready, or the problem. */
  function summaryState(state) {
    var host = state.host;
    var result = host.result;
    if (host.checking) { return { text: t("host.checking"), kind: "run" }; }
    if (state.provisioning.ready || hostReady(result)) { return { text: t("host.summary-ready"), kind: "ok" }; }
    if (!host.checked) { return { text: t("host.checking"), kind: "run" }; }
    if (!result) { return { text: t("host.summary-no-check"), kind: "warn" }; }
    if (result.status === "desktop-blocked") {
      return { text: t("host.summary-blocked", { items: missingItems(result, "desktop").join(", ") || "—" }), kind: "err" };
    }
    if (windowsAgentBlocked(result)) { return { text: t("host.summary-agent"), kind: "err" }; }
    if (result.status === "limited") {
      var gaps = missingItems(result, "media").concat(missingItems(result, "agent"));
      return { text: t("host.summary-limited", { items: gaps.join(", ") || "—" }), kind: "warn" };
    }
    return { text: t("host.summary-no-check"), kind: "warn" };
  }

  function renderSummary(state) {
    var line = document.getElementById("hostSummaryLine");
    var text = document.getElementById("hostSummaryText");
    var icon = document.getElementById("hostSummaryIcon");
    var details = document.getElementById("hostReadiness");
    if (!line || !text) { return; }
    var info = summaryState(state);
    text.textContent = info.text;
    line.setAttribute("data-kind", info.kind);
    if (icon) {
      icon.innerHTML = info.kind === "run" ? '<span class="install-spinner"></span>' :
        '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"' +
        ' stroke-linecap="round" stroke-linejoin="round"><path d="' +
        (info.kind === "ok" ? "M20 6 9 17l-5-5" : "M12 8v5M12 16.5v.5M10.3 3.9 2.4 18a2 2 0 0 0 1.7 3h15.8a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z") +
        '"/></svg>';
    }
    if (details) {
      var limited = !!state.host.result && state.host.result.status === "limited";
      if (info.kind === "err" || (info.kind === "warn" && limited)) {
        details.setAttribute("open", "");
      } else if (info.kind === "ok") {
        details.removeAttribute("open");
      }
    }
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
      // Any remaining optional gap needs acknowledgement, unless it is only
      // the skipped agent or media tools this edition does not use.
      return !skippedAgentOnlyGap(result);
    }
    return false;
  }

  function syncStartButton(state) {
    var start = document.getElementById("startSetupBtn");
    if (!start) { return; }
    start.disabled = state.provisioning.running || state.provisioning.ready || locked(state);
  }

  function isWindowsHost(state) {
    var platform = String((state.context && state.context.platform) || "").toLowerCase();
    return platform.indexOf("win") === 0;
  }

  // #1961 — a desktop-local install is a Chat/Drive product; agent mode is an
  // explicit opt-in there, so the toggle (not a blocking WSL2 error) is the
  // control the user sees first.
  function showAgentToggle(state) {
    return !!(state.context && state.context.desktop_local) && isWindowsHost(state);
  }

  function render(state) {
    var rows = document.getElementById("hostRows");
    var note = document.getElementById("hostNote");
    var recheck = document.getElementById("hostRecheckBtn");
    var proceed = document.getElementById("hostContinueBtn");
    var skipAgent = document.getElementById("hostSkipAgentBtn");
    var agentToggleRow = document.getElementById("hostAgentToggleRow");
    var agentToggle = document.getElementById("hostAgentToggle");
    var host = state.host;
    var result = host.result;
    var agentBlocked = windowsAgentBlocked(result);
    var withToggle = showAgentToggle(state);
    renderSummary(state);
    if (rows) {
      if (!result) { rows.innerHTML = ""; }
      else {
        var checks = [["host.desktop", "desktop", result.desktop_ready],
          ["host.agent", "agent", skipAgentic() ? true : result.agent_ready]];
        if (mediaRequired()) { checks.push(["host.media", "media", result.media_ready]); }
        rows.innerHTML = checks.map(function (row) {
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
          kind = "";
        } else if (hostReady(result)) {
          message = "";
        } else {
          message = result.remediation ? t("host.remediation") + ": " + result.remediation : "";
          kind = result.status === "desktop-blocked" ? "err" : "warn";
        }
      }
      note.textContent = message;
      note.className = "note " + kind;
    }
    if (recheck) { recheck.disabled = host.checking || state.provisioning.running; }
    if (agentToggleRow) { agentToggleRow.hidden = !withToggle; }
    if (agentToggle) {
      agentToggle.checked = !skipAgentic();
      agentToggle.disabled = host.checking || state.provisioning.running || state.provisioning.ready;
    }
    if (skipAgent) {
      // With the opt-in toggle present, the standalone skip button would be a
      // second, contradictory control for the same draft flag.
      skipAgent.hidden = withToggle ||
        !(agentBlocked && !state.provisioning.running && !state.provisioning.ready);
    }
    if (proceed) {
      // Continue anyway remains for non-agent optional gaps (e.g. media). It
      // must not bypass Windows WSL2 agent requirements (#1937), and a
      // Chat/Drive-only install whose only gap is the skipped agent is already
      // startable (#1961).
      var showContinue = !!result && result.status === "limited" && result.desktop_ready &&
        !state.host.acknowledged && !state.provisioning.running && !state.provisioning.ready &&
        !agentBlocked && !skippedAgentOnlyGap(result);
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
      if (autoStart && result && result.status !== "desktop-blocked" && !windowsAgentBlocked(result) &&
          (result.status !== "limited" || skippedAgentOnlyGap(result))) {
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
