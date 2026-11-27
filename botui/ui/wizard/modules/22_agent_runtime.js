/* First-run wizard — Agent Mode optional card in the install step. */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var B = App.B, esc = App.esc, t = App.t;
  var state = App.state, draft = App.draft;

  function persistDraft() { return App.persistDraft(); }
  function say(text) { return App.say(text); }

  /* -- formatting -- */

  function fmtBytes(bytes) {
    if (bytes == null || bytes <= 0) { return ""; }
    var lang = document.documentElement.lang || "pt-BR";
    var isPt = lang.startsWith("pt");
    if (bytes >= 1073741824) {
      var gb = (bytes / 1073741824).toFixed(1);
      if (isPt) { gb = gb.replace(".", ","); }
      return gb + " GB";
    }
    var mb = (bytes / 1048576).toFixed(0);
    return mb + " MB";
  }

  function stepName(step) {
    var map = {
      wsl: t("agent.step-wsl"),
      rootfs_download: t("agent.step-rootfs"),
      distro_import: t("agent.step-import"),
      first_boot: t("agent.step-boot"),
      image_download: t("agent.step-image-dl"),
      image_import: t("agent.step-image-import"),
      finish: t("agent.step-finish")
    };
    return map[step] || step;
  }

  /* -- state -- */

  var agentState = {
    status: null,
    choice: "later",
    installing: false,
    installed: false,
    failed: null,
    progress: null,
    pollTimer: null,
    unlisten: null
  };

  function isBlocked() {
    var s = agentState.status;
    return s && (s.state === "unsupported" || s.state === "policy_blocked");
  }

  function isManual() {
    var s = agentState.status;
    return s && s.managed === false;
  }

  /* -- rendering -- */

  function renderAgentCard() {
    var el = document.getElementById("agentRuntimeCard");
    if (!el) { return; }
    var s = agentState.status;

    if (!s) {
      el.innerHTML = '<p class="note">' + esc(t("agent.card-desc")) + '</p>';
      return;
    }

    if (s.state === "ready") {
      agentState.installed = true;
      el.innerHTML = '<p class="note ok">' + esc(t("agent.installed")) + '</p>';
      return;
    }

    if (isManual()) {
      el.innerHTML = '<p class="note">' + esc(t("agent.managed")) + '</p>';
      return;
    }

    if (isBlocked()) {
      var reason = s.state === "unsupported"
        ? t("agent.unsupported") : t("agent.policy-blocked");
      el.innerHTML = '<p class="note err">' + esc(reason) + '</p>';
      return;
    }

    if (agentState.installing) {
      var prog = agentState.progress;
      var pct = prog ? Math.round(prog.percent || 0) : 0;
      var stepLabel = prog ? stepName(prog.step) : "";
      el.innerHTML =
        '<p class="note">' + esc(t("agent.installing")) + '</p>' +
        '<div class="install-bar" style="margin:8px 0"><div class="install-bar-fill" style="width:' + pct + '%"></div></div>' +
        (stepLabel ? '<p class="note" style="font-size:12px">' + esc(stepLabel) +
          ' &mdash; ' + pct + '%' +
          (prog && prog.total_bytes > 0 ? ' (' + fmtBytes(prog.done_bytes) + ' / ' + fmtBytes(prog.total_bytes) + ')' : '') +
          '</p>' : '');
      return;
    }

    if (agentState.installed) {
      el.innerHTML = '<p class="note ok">' + esc(t("agent.installed")) + '</p>';
      return;
    }

    if (agentState.failed) {
      el.innerHTML =
        '<p class="note err">' + esc(t("agent.install-failed", { reason: agentState.failed })) + '</p>' +
        '<div class="inline-row" style="margin-top:8px">' +
        '<button type="button" class="btn btn-secondary btn-sm" id="agentRetryBtn">' +
        esc(t("nav.retry")) + '</button></div>';
      var retry = document.getElementById("agentRetryBtn");
      if (retry) { retry.addEventListener("click", startAgentInstall); }
      return;
    }

    /* Default: show choice */
    var sizeInfo = '';
    if (s.download_bytes) {
      sizeInfo += '<span class="note" style="margin-right:12px">' +
        esc(t("agent.download-size", { size: fmtBytes(s.download_bytes) })) + '</span>';
    }
    if (s.disk_bytes) {
      sizeInfo += '<span class="note">' +
        esc(t("agent.disk-size", { size: fmtBytes(s.disk_bytes) })) + '</span>';
    }

    var checked = agentState.choice === "now" ? " checked" : "";
    var checkedLater = agentState.choice === "later" ? " checked" : "";

    el.innerHTML =
      '<p class="note" style="margin-bottom:8px">' + esc(t("agent.card-desc")) + '</p>' +
      (sizeInfo ? '<div style="margin-bottom:8px">' + sizeInfo + '</div>' : '') +
      (s.needs_admin ? '<p class="note" style="font-size:12px;margin-bottom:8px">' +
        esc(t("agent.admin-note")) + '</p>' : '') +
      '<div class="field" style="margin-bottom:0">' +
        '<label class="radio-label" style="margin-bottom:4px">' +
        '<input type="radio" name="agentChoice" value="later"' + checkedLater + ' /> ' +
        esc(t("agent.install-later")) + '</label>' +
        '<label class="radio-label">' +
        '<input type="radio" name="agentChoice" value="now"' + checked + ' /> ' +
        esc(t("agent.install-now")) + '</label>' +
      '</div>';

    el.querySelectorAll('input[name="agentChoice"]').forEach(function (radio) {
      radio.addEventListener("change", function () {
        agentState.choice = radio.value;
        draft.agent_runtime_install = radio.value === "now";
        persistDraft();
      });
    });
  }

  /* -- install lifecycle -- */

  function startAgentInstall() {
    if (agentState.installing || agentState.installed) { return; }
    agentState.installing = true;
    agentState.failed = null;
    renderAgentCard();

    B.onAgentRuntimeProgress(function (ev) {
      agentState.progress = ev;
      if (ev.state === "ready" || ev.step === "finish" && ev.percent >= 100) {
        agentState.installing = false;
        agentState.installed = true;
        stopAgentPoll();
      }
      renderAgentCard();
    });

    startAgentPoll();

    B.agentRuntimeInstall().then(function () {
      agentState.installing = false;
      agentState.installed = true;
      stopAgentPoll();
      renderAgentCard();
      say(t("agent.installed"));
    }).catch(function (e) {
      agentState.installing = false;
      agentState.failed = e && e.message ? e.message : String(e);
      stopAgentPoll();
      renderAgentCard();
    });
  }

  function startAgentPoll() {
    stopAgentPoll();
    agentState.pollTimer = setInterval(function () {
      var el = document.getElementById("agentRuntimeCard");
      if (!el || !document.body.contains(el)) { stopAgentPoll(); return; }
      B.agentRuntimeStatus().then(function (s) {
        if (!s) { return; }
        agentState.status = s;
        if (s.state === "ready") {
          agentState.installing = false;
          agentState.installed = true;
          stopAgentPoll();
        }
        renderAgentCard();
      }).catch(function () {});
    }, 1500);
  }

  function stopAgentPoll() {
    if (agentState.pollTimer) { clearInterval(agentState.pollTimer); agentState.pollTimer = null; }
  }

  /* -- public interface -- */

  App.bindAgentRuntime = function () {
    B.agentRuntimeStatus().then(function (s) {
      agentState.status = s;
      if (draft.agent_runtime_install) { agentState.choice = "now"; }
      renderAgentCard();
    }).catch(function () {
      renderAgentCard();
    });
  };

  App.startAgentInstallIfChosen = function () {
    if (agentState.choice === "now" && !agentState.installed && !agentState.installing && !isBlocked() && !isManual()) {
      startAgentInstall();
    }
  };

  App.agentRuntimeState = agentState;

})();
