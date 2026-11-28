/* First-run wizard — install row names and states, and the failure notice. */
(function () {
  "use strict";

  var App = window.GBWizardApp;
  var B = App.B, esc = App.esc, t = App.t, state = App.state;
  var INSTALL_STEP = 7;

  /* The provisioner announces internal ids (vault, tables, llm-config,
     components, botserver…); the row shows the translated name. */
  function componentName(component) {
    var id = component && component.id ? component.id : "";
    var key = "install.comp." + id;
    var translated = t(key);
    if (translated !== key) { return translated; }
    return (component && component.label) || id;
  }

  /* What the row says, read from the provisioner's progress messages. It
     installs from the bundled files and never reports a download, so
     "downloading" shows only when a message really says so. */
  function rowState(entry) {
    if (!entry) { return t("install.row.pending"); }
    var message = String(entry.message || "");
    if (entry.state === "failed") { return t("install.row.failed"); }
    if (entry.state === "ready") {
      if (/skipped/i.test(message)) { return t("install.row.skipped"); }
      if (/first use/i.test(message)) { return t("install.row.first-use"); }
      return t("install.row.ready");
    }
    if (entry.state === "starting") {
      if (/download/i.test(message)) { return t("install.row.downloading"); }
      if (/bundled files|^installing /i.test(message)) { return t("install.row.bundled"); }
      if (/configuration/i.test(message)) { return t("install.row.configuring"); }
      return t("install.row.starting");
    }
    if (entry.state === "pending") { return t("install.row.preparing"); }
    return t("install.row.pending");
  }

  function setRowState(node, entry) {
    var source = node && node.querySelector(".install-step-source");
    if (source) { source.textContent = rowState(entry); }
  }

  function failedComponentId(components) {
    var id = "";
    (components || []).forEach(function (c) { if (c && c.state === "failed") { id = c.id; } });
    return id;
  }

  function clearProvisionError() {
    state.provisioning.failedId = "";
    var box = document.getElementById("provisionError");
    if (box) { box.innerHTML = ""; box.className = "note err"; }
  }

  /* A failure ends the run at whatever step it arrives: the install step
     comes back with the message, the step that stopped, the log folder and
     the actions to retry or open the logs. */
  function showProvisionError(message, componentId) {
    var reason = String(message || t("nav.unknown"));
    state.provisioning.running = false;
    state.provisioning.failed = reason;
    state.provisioning.failedId = componentId || state.provisioning.failedId ||
      failedComponentId(state.provisioning.components);
    if (state.step !== INSTALL_STEP) { App.goTo(INSTALL_STEP); }
    paintFailure();
    App.setDetailsOpen(true);
    App.setProvisionUi(false);
    App.updateInstallProgress();
    App.say(t("install.failed-title") + ". " + reason);
  }

  /* Paints the failure kept in state; the install step calls it again
     after a re-render (language change, coming back to the step). */
  function paintFailure() {
    var reason = state.provisioning.failed;
    var componentId = state.provisioning.failedId;
    var box = document.getElementById("provisionError");
    if (box && reason) {
      var ctx = state.context || {};
      var step = componentId ? componentName({ id: componentId }) : "";
      box.className = "note err install-failure";
      box.setAttribute("role", "alert");
      box.innerHTML =
        '<strong class="install-failure-title">' + esc(t("install.failed-title")) + "</strong>" +
        (step ? '<span class="install-failure-line">' + esc(t("install.failed-step", { name: step })) + "</span>" : "") +
        '<span class="install-failure-line install-failure-reason">' + esc(reason) + "</span>" +
        (ctx.log_path ? '<span class="install-failure-line install-failure-log">' +
          esc(t("install.failed-log", { path: ctx.log_path })) + "</span>" : "") +
        '<span class="inline-row install-failure-actions">' +
          '<button type="button" class="btn btn-primary btn-sm" id="retryComponentBtn">' + esc(t("install.retry")) + "</button>" +
          '<button type="button" class="btn btn-secondary btn-sm" id="failureLogsBtn">' + esc(t("install.open-logs")) + "</button>" +
        "</span>";
      var retry = document.getElementById("retryComponentBtn");
      if (retry) {
        retry.addEventListener("click", function () {
          clearProvisionError();
          App.startProvisioning();
        });
      }
      var logs = document.getElementById("failureLogsBtn");
      if (logs) { logs.addEventListener("click", function () { B.openFolder("logs"); }); }
    }
  }

  App.componentName = componentName;
  App.rowState = rowState;
  App.setRowState = setRowState;
  App.failedComponentId = failedComponentId;
  App.clearProvisionError = clearProvisionError;
  App.showProvisionError = showProvisionError;
  App.paintFailure = paintFailure;
})();
