/* Host prerequisite panel (#1742).
 *
 * The desktop shell runs `PROD-DEPENDENCIES.sh --check` (Linux) or
 * `PROD-DEPENDENCIES.ps1 -Check` (Windows) before provisioning and renders the
 * result here. Only a failing desktop-shell verdict locks setup; agent and
 * media gaps are optional and expose "Continue anyway". */
(function () {
  "use strict";

  var B = window.GBWizardBridge;
  var I = window.WizardI18n;
  var esc = window.GBWizardSteps.esc;
  var t = function (key, params) { return I.t(key, params); };

  function badgeState(result) {
    if (!result) { return { text: t("host.unknown"), kind: "" }; }
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
    return result.status === "limited" && !state.host.acknowledged;
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
    var host = state.host;
    var result = host.result;
    var badgeInfo = badgeState(result);
    if (badge) {
      badge.textContent = host.checking ? t("host.checking") : badgeInfo.text;
      badge.className = "badge " + badgeInfo.kind;
    }
    if (rows) {
      if (!result) { rows.innerHTML = ""; }
      else {
        rows.innerHTML = [["host.desktop", "desktop", result.desktop_ready],
          ["host.agent", "agent", result.agent_ready],
          ["host.media", "media", result.media_ready]].map(function (row) {
          var missing = missingItems(result, row[1]);
          var text = row[2]
            ? t("host.ready")
            : (missing.length ? t("host.missing") + ": " + missing.join("; ") : t("host.missing"));
          return '<div class="summary-row"><span>' + esc(t(row[0])) + "</span>" +
            '<span class="note ' + (row[2] ? "ok" : "err") + '">' + esc(text) + "</span></div>";
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
        message = result.message || "";
        if (result.remediation && result.status !== "ready") {
          message += (message ? " " : "") + t("host.remediation") + ": " + result.remediation;
        }
        kind = result.status === "desktop-blocked" ? "err" : (result.status === "limited" ? "warn" : "ok");
      }
      note.textContent = message;
      note.className = "note " + kind;
    }
    if (recheck) { recheck.disabled = host.checking || state.provisioning.running; }
    if (proceed) {
      proceed.hidden = !(result && result.status === "limited" && result.desktop_ready &&
        !host.acknowledged && !state.provisioning.running && !state.provisioning.ready);
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
      if (autoStart && result && result.status !== "desktop-blocked" && result.status !== "limited") {
        onStart();
      }
      if (result && result.status === "desktop-blocked") { onSay(t("host.blocked-say")); }
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
    syncStartButton: syncStartButton
  };
})();
