/* First-run wizard — subscription account linking on the Finish step (#1655). */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var B = App.B, t = App.t;
  var state = App.state, draft = App.draft;
  var accountTimers = {};

  /* Forwarders resolve the functions defined in sibling modules at call time. */

  function ensureAdmin() { return App.ensureAdmin(); }

  /* ---------------- subscription account on Finish (#1655) ---------------- */

  function accountLabel(status) {
    switch (status) {
      case "linked": return t("accounts.linked");
      case "connecting": return t("accounts.connecting");
      case "expired": return t("accounts.expired");
      case "error": return t("accounts.error");
      default: return t("accounts.unlinked");
    }
  }

  function finishAccountNodes() {
    return {
      badge: document.getElementById("finishAccountStatus"),
      detail: document.getElementById("finishAccountDetail"),
      connect: document.getElementById("finishAccountConnect"),
      disconnect: document.getElementById("finishAccountDisconnect"),
      codeRow: document.getElementById("finishAccountCodeRow"),
      code: document.getElementById("finishAccountCode"),
      codeBtn: document.getElementById("finishAccountCodeBtn")
    };
  }

  function renderFinishAccount(status, message) {
    var nodes = finishAccountNodes();
    if (nodes.badge) { nodes.badge.textContent = accountLabel(status); }
    if (nodes.detail) { nodes.detail.textContent = message || ""; }
    if (nodes.connect) { nodes.connect.hidden = status === "linked"; }
    if (nodes.disconnect) { nodes.disconnect.hidden = status !== "linked"; }
    if (nodes.codeRow) {
      nodes.codeRow.hidden = !(draft.model.mode === "claude-code" &&
        (status === "connecting" || status === "waiting"));
    }
  }

  function stopAccountTimer() {
    if (accountTimers.finish) { clearTimeout(accountTimers.finish); }
    delete accountTimers.finish;
  }

  function finishUseChat(provider) {
    return B.aiUseChat(provider, draft.admin)
      .then(function () {
        renderFinishAccount("linked", t("finish.account-selected"));
      })
      .catch(function (error) {
        renderFinishAccount("linked", String(error && error.message ? error.message : error));
      });
  }

  function pollFinishAccount(provider) {
    stopAccountTimer();
    var deadline = Date.now() + 300000;
    function tick() {
      if (state.step !== 8) { return; }
      B.aiStatus(provider, draft.admin).then(function (data) {
        var account = (data && data.account) || data || {};
        var status = account.status || "unlinked";
        renderFinishAccount(status, account.message || "");
        if (status === "linked") { finishUseChat(provider); return; }
        if (status === "error" || status === "expired") { return; }
        if (Date.now() < deadline) { accountTimers.finish = setTimeout(tick, 3000); }
      }).catch(function () {
        if (Date.now() < deadline) { accountTimers.finish = setTimeout(tick, 5000); }
      });
    }
    accountTimers.finish = setTimeout(tick, 2000);
  }

  function bindFinishAccount() {
    var provider = draft.model.mode;
    if (provider !== "claude-code" && provider !== "chatgpt-plan") { return; }
    ensureAdmin();
    var nodes = finishAccountNodes();
    B.saveAdmin(draft.admin).catch(function () {}).then(function () {
      return B.aiStatus(provider, draft.admin);
    }).then(function (data) {
      var account = (data && data.account) || data || {};
      var status = account.status || "unlinked";
      renderFinishAccount(status, account.message || "");
      if (status === "linked") { finishUseChat(provider); }
      if (status === "connecting") { pollFinishAccount(provider); }
    }).catch(function () {});

    if (nodes.connect) {
      nodes.connect.addEventListener("click", function () {
        nodes.connect.disabled = true;
        renderFinishAccount("connecting", t("accounts.starting"));
        B.aiConnect(provider, draft.admin).then(function (data) {
          if (data && data.url) { B.openUrl(data.url); }
          var status = (data && (data.status || data.state)) || "connecting";
          renderFinishAccount(status, data && data.message);
          if (status !== "linked") { pollFinishAccount(provider); }
        }).catch(function (error) {
          renderFinishAccount("error", String(error && error.message ? error.message : error));
        }).then(function () { nodes.connect.disabled = false; });
      });
    }
    if (nodes.disconnect) {
      nodes.disconnect.addEventListener("click", function () {
        B.aiDisconnect(provider, draft.admin).then(function () {
          renderFinishAccount("unlinked", "");
        }).catch(function (error) {
          renderFinishAccount("error", String(error && error.message ? error.message : error));
        });
      });
    }
    if (nodes.codeBtn) {
      nodes.codeBtn.addEventListener("click", function () {
        var code = nodes.code ? nodes.code.value.trim() : "";
        if (!code) { return; }
        nodes.codeBtn.disabled = true;
        B.aiCode(code, draft.admin).then(function (data) {
          var status = (data && (data.status || data.state)) || "error";
          renderFinishAccount(status, data && data.message);
          if (status === "linked") { if (nodes.code) nodes.code.value = ""; finishUseChat(provider); }
        }).catch(function (error) {
          renderFinishAccount("error", String(error && error.message ? error.message : error));
        }).then(function () { nodes.codeBtn.disabled = false; });
      });
    }
  }

  App.accountLabel = accountLabel;
  App.finishAccountNodes = finishAccountNodes;
  App.renderFinishAccount = renderFinishAccount;
  App.stopAccountTimer = stopAccountTimer;
  App.finishUseChat = finishUseChat;
  App.pollFinishAccount = pollFinishAccount;
  App.bindFinishAccount = bindFinishAccount;
})();
