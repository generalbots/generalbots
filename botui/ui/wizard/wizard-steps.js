/* Step rendering, validation and collection for the first-run wizard. */
(function () {
  "use strict";
  var W = window.GBWizard;
  var I = window.WizardI18n;
  var t = function (key, params) { return I.t(key, params); };
  var esc = function (value) {
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  };

  function statusBadge(text, kind) {
    return '<span class="badge ' + (kind || "") + '">' + esc(text) + "</span>";
  }

  function renderWelcome(draft, ctx) {
    var accentHtml = window.GBWizardApp && window.GBWizardApp.accentPickerHtml ? window.GBWizardApp.accentPickerHtml(draft, esc, t) : "";
    return '' +
      '<p class="welcome-greeting">' + esc(t("welcome.greeting")) + "</p>" +
      '<p class="lead">' + esc(t("welcome.lead")) + "</p>" +
      '<div class="card">' +
        '<p class="note">' + esc(t("welcome.note1")) + "</p>" +
        '<p class="note">' + esc(t("welcome.note2")) + "</p>" +
      "</div>" +
      '<div class="field-grid" style="margin-top:16px">' +
        '<div class="field"><label for="displayName">' + esc(t("welcome.name-label")) + "</label>" +
          '<input type="text" id="displayName" value="' + esc(draft.display_name) + '" placeholder="' + esc(t("welcome.name-placeholder")) + '" /></div>' +
        accentHtml +
        '<p class="note">' + esc(t("welcome.workspace")) + " <strong>" + esc(ctx.workspace_path) + "</strong></p>" +
      "</div>";
  }

  function renderModel(draft) {
    return window.GBWizardEngine.renderEngine(draft);
  }

  function renderWorkspace(draft, ctx) {
    var path = draft.workspace.path || ctx.workspace_path;
    return '' +
      '<p class="lead">' + esc(t("workspace.lead")) + "</p>" +
      '<div class="field-grid">' +
        '<div class="field"><label for="workspacePath">' + esc(t("workspace.folder")) + "</label>" +
          '<div class="inline-row">' +
            '<input type="text" id="workspacePath" value="' + esc(path) + '" />' +
            '<button type="button" class="btn btn-secondary btn-sm" id="browseFolderBtn">' + esc(t("workspace.browse")) + "</button>" +
          "</div>" +
          '<span class="hint" id="workspaceStatus" role="status">' + esc(t("workspace.checking")) + "</span></div>" +
        '<div class="field"><label for="workspaceSize">' + esc(t("workspace.size")) + "</label>" +
          '<select id="workspaceSize">' +
            '<option value="standard"' + (draft.workspace.size === "standard" ? " selected" : "") + ">" + esc(t("workspace.size-standard")) + "</option>" +
            '<option value="compact"' + (draft.workspace.size === "compact" ? " selected" : "") + ">" + esc(t("workspace.size-compact")) + "</option>" +
          "</select>" +
          '<span class="hint" id="workspaceFree">' + esc(t("workspace.free-hint")) + "</span></div>" +
      "</div>";
  }

  function renderServiceCard(provider, draft, ctx) {
    var saved = draft.services[provider.id] || {};
    var state = saved.connected ? t("services.connected") : (saved.client_id ? t("services.attention") : t("services.not-connected"));
    var kind = saved.connected ? "ok" : (saved.client_id ? "warn" : "");
    var redirect = (ctx.redirect_uri || provider.redirectTemplate).replace("{provider}", provider.id);
    return '' +
      '<div class="card" data-service="' + provider.id + '">' +
        '<div class="card-head"><span class="card-title">' + esc(provider.name) + "</span>" +
          statusBadge(state, kind) + "</div>" +
        '<p class="note" style="margin:8px 0">' + esc(t(provider.stepsKey)) + "</p>" +
        '<p class="note" style="margin:0 0 8px">' + esc(t(provider.scopeNoteKey)) + "</p>" +
        '<div class="field" style="margin-bottom:10px">' +
          '<label>' + esc(t("services.redirect")) + "</label>" +
          '<div class="inline-row">' +
            '<input type="text" readonly value="' + esc(redirect) + '" />' +
            '<button type="button" class="btn btn-secondary btn-sm" data-copy="' + esc(redirect) + '">' + esc(t("services.copy")) + "</button>" +
          "</div>" +
        "</div>" +
        '<div class="inline-row" style="margin-bottom:10px">' +
          '<button type="button" class="btn btn-secondary btn-sm" data-console="' + esc(provider.console) + '">' + esc(t(provider.consoleLabelKey)) + "</button>" +
        "</div>" +
        '<div class="field-grid">' +
          '<div class="field"><label>' + esc(t(provider.fields.id)) + '</label>' +
            '<input type="text" data-service-id="' + provider.id + '" value="' + esc(saved.client_id || "") + '" autocomplete="off" /></div>' +
          '<div class="field"><label>' + esc(t(provider.fields.secret)) + '</label>' +
            '<input type="password" data-service-secret="' + provider.id + '" value="' + esc(saved.client_secret || "") + '" autocomplete="off" />' +
            '<span class="hint">' + esc(t("services.secret-hint")) + "</span></div>" +
        "</div>" +
        '<p class="note" data-service-note="' + provider.id + '"></p>' +
      "</div>";
  }

  function renderServices(draft, ctx) {
    return '' +
      '<p class="lead">' + esc(t("services.lead")) + "</p>" +
      W.PROVIDERS.map(function (p) { return renderServiceCard(p, draft, ctx); }).join("") +
      '<p class="note">' + esc(t("services.note")) + "</p>";
  }

  function dataMapRows(draft, ctx) {
    var model = draft.model;
    var prompts;
    if (model.mode === "local") {
      prompts = t("privacy.local-prompts");
    } else {
      var host = W.endpointHost(model.endpoint || W.defaultModelEndpoint(model.provider)) || t("nav.unknown");
      prompts = t("privacy.sent-to", { host: host }) + (W.isLoopback(model.endpoint) ? t("privacy.on-this-machine") : "");
    }
    var connected = W.PROVIDERS.filter(function (p) {
      var s = draft.services[p.id];
      return s && s.client_id;
    }).map(function (p) { return p.name; });
    return [
      [t("privacy.conversations"), t("privacy.conversations-dest", { path: draft.workspace.path || ctx.workspace_path }), t("privacy.always")],
      [t("privacy.keys"), t("privacy.keys-dest", { path: ctx.stack_path }), t("privacy.when-saved")],
      [t("privacy.prompts"), prompts, t("privacy.every-message")],
      [t("privacy.services"), connected.length ? connected.join(", ") + t("privacy.only-actions") : t("privacy.no-service"), t("privacy.per-request")],
      [t("privacy.crashes"), t("privacy.crash-off"), "—"]
    ];
  }

  function renderPrivacy(draft, ctx) {
    var rows = dataMapRows(draft, ctx).map(function (row) {
      return "<tr><td>" + esc(row[0]) + "</td><td>" + esc(row[1]) + "</td><td>" + esc(row[2]) + "</td></tr>";
    }).join("");
    return '' +
      '<p class="lead">' + esc(t("privacy.lead")) + "</p>" +
      '<table class="datamap"><thead><tr><th>' + esc(t("privacy.data")) + "</th><th>" + esc(t("privacy.destination")) + "</th><th>" + esc(t("privacy.trigger")) + "</th></tr></thead><tbody>" +
      rows + "</tbody></table>" +
      '<div class="inline-row" style="margin-top:12px">' +
        '<button type="button" class="btn btn-secondary btn-sm" id="revealVaultBtn">' + esc(t("privacy.reveal-vault")) + "</button>" +
        '<span class="note" id="vaultNote">' + esc(t("privacy.vault-note")) + "</span>" +
      "</div>" +
      '<p class="note install-next">' + esc(t("privacy.next-note")) + "</p>";
  }

  function renderNetwork(draft) {
    var p2p = draft.network && draft.network.mode === "p2p";
    return '' +
      '<p class="lead">' + esc(t("network.lead")) + "</p>" +
      '<div class="card">' +
        '<div class="field-grid">' +
          '<div class="field">' +
            '<label class="radio-label">' +
              '<input type="radio" name="networkMode" id="networkModeP2P" value="p2p"' + (p2p ? " checked" : "") + " />" +
              " " + esc(t("network.p2p.title")) +
            "</label>" +
            '<span class="hint">' + esc(t("network.p2p.desc")) + "</span>" +
          "</div>" +
          '<div class="field">' +
            '<label class="radio-label">' +
              '<input type="radio" name="networkMode" id="networkModeOffline" value="offline"' + (!p2p ? " checked" : "") + " />" +
              " " + esc(t("network.offline.title")) +
            "</label>" +
            '<span class="hint">' + esc(t("network.offline.desc")) + "</span>" +
          "</div>" +
        "</div>" +
      "</div>" +
      '<p class="note">' + esc(t("network.note")) + "</p>";
  }
  function renderInstall(draft, ctx) {
    var planned = (ctx.components || []).filter(function (c) {
      return draft.model.mode === "local" || c.id !== "llm";
    });
    return '' +
      '<p class="lead" style="margin-bottom:14px">' + esc(t("install.desc")) + "</p>" +
      '<details class="install-host-details" id="hostReadiness" aria-live="polite">' +
        '<summary class="install-host-summary" id="hostSummaryLine">' +
          '<span id="hostSummaryText">' + esc(t("host.checking")) + "</span>" +
          '<span class="badge" id="hostBadge" style="margin-left:8px">' + esc(t("host.checking")) + "</span>" +
        "</summary>" +
        '<div class="install-host-body">' +
          '<div id="hostRows" style="margin-top:8px"></div>' +
          '<p class="note" id="hostNote" role="status" style="margin:8px 0 0"></p>' +
          '<label class="inline-row" id="hostAgentToggleRow" hidden style="margin-top:10px;gap:8px;cursor:pointer">' +
            '<input type="checkbox" id="hostAgentToggle">' +
            '<span>' + esc(t("host.enable-agent")) + "</span>" +
          "</label>" +
          '<div class="inline-row" style="margin-top:10px">' +
            '<button type="button" class="btn btn-secondary btn-sm" id="hostRecheckBtn">' + esc(t("host.recheck")) + "</button>" +
            '<button type="button" class="btn btn-primary btn-sm" id="hostContinueBtn" hidden>' + esc(t("host.continue")) + "</button>" +
            '<button type="button" class="btn btn-secondary btn-sm" id="hostSkipAgentBtn" hidden>' + esc(t("host.skip-agentic")) + "</button>" +
          "</div>" +
        "</div>" +
      "</details>" +
      '<div class="install-columns" id="installColumns">' +
        '<div class="install-col-steps">' +
          '<div class="install-head">' +
            '<span class="install-kicker" id="installStepCount">' + esc(t("install.steps-of", { ready: 0, total: planned.length })) + "</span>" +
            '<span class="install-percent" id="installPercent">0%</span>' +
          "</div>" +
          '<div class="install-bar"><div class="install-bar-fill" id="installFill"></div></div>' +
          '<div class="component-board" id="componentList"></div>' +
        "</div>" +
        '<div class="install-col-output" id="installOutputCol">' +
          '<div class="log-header">' +
            '<span class="log-title">' + esc(t("install.live-output")) + "</span>" +
            '<span class="log-count" id="logLineCount">0 ' + esc(t("install.lines")) + "</span>" +
          "</div>" +
          '<pre class="log" id="provisionLog">' + esc(t("install.no-output")) + "</pre>" +
          '<div class="inline-row" style="margin-top:8px">' +
            '<button type="button" class="btn btn-secondary btn-sm" id="openLogsBtn">' + esc(t("install.open-logs")) + "</button>" +
            '<button type="button" class="btn btn-text btn-sm btn-danger-text" id="resetStackBtn">' + esc(t("install.reset")) + "</button>" +
          "</div>" +
        "</div>" +
      "</div>" +
      '<div id="provisionError" class="note err" role="status"></div>' +
      '<p class="install-status" id="installStatus" style="margin-top:4px">' + esc(t("install.preparing", { count: planned.length })) + "</p>" +
      '<div class="install-footer">' +
        '<div class="install-footer-left">' +
          '<button type="button" class="install-toggle-details" id="toggleInstallDetails">' +
            '<span id="toggleDetailsIcon">&#9654;</span> ' + esc(t("install.show-details")) +
          "</button>" +
        "</div>" +
        '<div class="install-footer-right">' +
          '<button type="button" class="btn btn-primary btn-sm" id="startSetupBtn">' + esc(t("install.start")) + "</button>" +
          '<button type="button" class="btn btn-secondary btn-sm" id="cancelSetupBtn" disabled>' + esc(t("install.cancel")) + "</button>" +
        "</div>" +
      "</div>";
  }

  function renderFinishAccount(draft) {
    /* Subscription agents are connected AFTER setup, in the app. The wizard
       only shows a reminder when the person chose "Agente por assinatura". */
    if (!draft.want_agents) { return ""; }
    return '' +
      '<div class="card" id="finishAgentReminder" style="margin-top:16px">' +
        '<div class="card-head"><span class="card-title">' +
          esc(t("finish.agent-reminder-title")) + "</span></div>" +
        '<p class="note" style="margin:8px 0">' + esc(t("finish.agent-reminder-text")) + "</p>" +
      "</div>";
  }

  function renderFinish(draft, ctx) {
    var modelLabel;
    if (draft.want_agents) {
      var fb = draft.model.mode === "local" ? t("finish.model-local") : t("finish.model-free-fb");
      modelLabel = t("finish.model-acp", { fallback: fb });
    } else if (draft.model.mode === "local") {
      modelLabel = t("finish.model-local");
    } else if (draft.model.mode === "free") {
      modelLabel = t("finish.model-free-fb");
    } else {
      modelLabel = (draft.model.mode === "own-key" ? t("finish.model-own") : t("finish.model-hosted")) +
        (W.endpointHost(draft.model.endpoint) || draft.model.provider);
    }
    var connected = W.PROVIDERS.filter(function (p) {
      var s = draft.services[p.id];
      return s && s.client_id;
    }).map(function (p) { return p.name; });
    var notConnected = W.PROVIDERS.filter(function (p) {
      var s = draft.services[p.id];
      return !s || !s.client_id;
    }).map(function (p) { return p.name; });
    return '' +
      '<p class="lead">' + esc(t("finish.lead")) + "</p>" +
      '<div class="card" id="adminCard">' +
        '<div class="card-head"><span class="card-title">' + esc(t("finish.admin")) + "</span>" +
          statusBadge(t("finish.admin-created"), "ok") + "</div>" +
        '<p class="note" style="margin:8px 0">' + esc(t("finish.admin-note")) + "</p>" +
        '<div class="field-grid">' +
          '<div class="field"><label for="adminEmail">' + esc(t("finish.email")) + "</label>" +
            '<input type="email" id="adminEmail" value="' + esc(draft.admin.email) + '" autocomplete="off" /></div>' +
          '<div class="field"><label for="adminPassword">' + esc(t("finish.password")) + "</label>" +
            '<div class="inline-row">' +
              '<input type="password" id="adminPassword" readonly autocomplete="new-password" value="' + esc(draft.admin.password) + '" />' +
              '<button type="button" class="btn btn-secondary btn-sm" id="toggleAdminPasswordBtn" aria-pressed="false">' + esc(t("finish.show-password")) + "</button>" +
              '<button type="button" class="btn btn-secondary btn-sm" id="copyAdminBtn">' + esc(t("finish.copy")) + "</button>" +
              '<button type="button" class="btn btn-secondary btn-sm" id="regenerateAdminBtn">' + esc(t("finish.regenerate")) + "</button>" +
            "</div>" +
            '<span class="hint">' + esc(t("finish.password-hint")) + "</span></div>" +
        "</div>" +
      "</div>" +
      renderFinishAccount(draft) +
      '<div class="summary" style="margin-top:16px">' +
        '<div class="summary-row"><span>' + esc(t("finish.workspace")) + "</span><span>" + esc(draft.workspace.path || ctx.workspace_path) + "</span></div>" +
        '<div class="summary-row"><span>' + esc(t("finish.model")) + "</span><span>" + esc(modelLabel) + "</span></div>" +
        '<div class="summary-row"><span>' + esc(t("finish.connected")) + "</span><span>" + esc(connected.length ? connected.join(", ") : t("finish.none")) + "</span></div>" +
        '<div class="summary-row"><span>' + esc(t("finish.not-connected")) + "</span><span>" + esc(notConnected.length ? notConnected.join(", ") : t("finish.none")) + "</span></div>" +
        '<div class="summary-row"><span>' + esc(t("finish.crash")) + "</span><span>" + esc(t("finish.off")) + "</span></div>" +
      "</div>" +
      '<div class="card" id="mediaCapabilitiesCard" style="margin-top:16px">' +
        '<div class="card-head"><span class="card-title">' + esc(t("finish.media-title")) + "</span>" +
          '<span class="badge" id="mediaCapabilitiesBadge">' + esc(t("finish.media-checking")) + "</span></div>" +
        '<div id="mediaCapabilitiesRows" style="margin-top:8px"></div>' +
        '<p class="note" id="mediaCapabilitiesNote" role="status" style="margin:8px 0 0"></p>' +
      "</div>" +
      '<p class="note" style="margin-top:16px">' + esc(t("finish.change-later")) + "</p>" +
      '<div id="finishStatus" class="note" role="status" style="margin-top:12px"></div>';
  }

  function render(step, draft, ctx) {
    switch (step) {
      case 1: return renderWelcome(draft, ctx);
      case 2: return renderModel(draft);
      case 3: return renderWorkspace(draft, ctx);
      case 4: return renderServices(draft, ctx);
      case 5: return renderPrivacy(draft, ctx);
      case 6: return renderNetwork(draft);
      case 7: return renderInstall(draft, ctx);
      case 8: return renderFinish(draft, ctx);
      default: return "<p>" + esc(t("wizard.unknown-step")) + "</p>";
    }
  }

  var HEADINGS = [null, "heading.welcome", "heading.model", "heading.workspace", "heading.services", "heading.privacy", "heading.network", "heading.install", "heading.finish"];
  function heading(step) {
    return t(HEADINGS[step] || "wizard.title");
  }

  function validate(step, draft) {
    if (step === 2) {
      if (draft.model.mode === "local") { return { ok: true }; }
      if (draft.model.mode === "free") {
        var free = W.freeProviderById(draft.model.provider);
        if (free && free.needsKey && !draft.model.api_key) {
          return { ok: false, reason: t("model.key-hint-llm7") };
        }
        return { ok: true };
      }
      if (draft.model.mode === "own-key") {
        if (!draft.model.endpoint) { return { ok: false, reason: t("model.endpoint") + " — " + t("workspace.choose") }; }
        if (!draft.model.model) { return { ok: false, reason: t("model.name") }; }
        if (!draft.model.api_key) { return { ok: false, reason: t("model.key") }; }
        if (!draft.model.tested) { return { ok: false, reason: t("model.test") }; }
        return { ok: true };
      }
      return { ok: true };
    }
    if (step === 3) {
      if (!draft.workspace.path) { return { ok: false, reason: t("workspace.choose") }; }
      if (draft.workspace.blocked) { return { ok: false, reason: draft.workspace.block_reason || t("workspace.not-writable") }; }
      return { ok: true };
    }
    return { ok: true };
  }

  window.GBWizardSteps = {
    render: render,
    heading: heading,
    validate: validate,
    dataMapRows: dataMapRows,
    engineDetail: function (draft) { return window.GBWizardEngine.engineDetail(draft); },
    esc: esc
  };
})();
