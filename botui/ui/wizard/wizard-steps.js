/* Step rendering, validation and collection for the first-run wizard.
 * Every user-visible string goes through window.WizardI18n (see wizard-i18n.js);
 * the English dictionary is the fallback for any missing key. */
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

  function modelDetail(draft) {
    var m = draft.model;
    if (m.mode === "claude-code" || m.mode === "chatgpt-plan") {
      var name = m.mode === "claude-code" ? "Claude Code" : "ChatGPT plan";
      return '<div class="card"><p class="note ok">' + esc(t("model.subscription-note", { name: name })) + "</p></div>";
    }
    if (m.mode === "local") {
      return '<div class="card"><p class="note ok">' + esc(t("model.local-note")) + "</p></div>";
    }
    if (m.mode === "free") {
      var freeOptions = W.FREE_PROVIDERS.map(function (p) {
        return '<option value="' + p.id + '"' + (m.provider === p.id ? " selected" : "") + ">" + esc(p.name) + "</option>";
      }).join("");
      var freeEndpoint = m.endpoint || W.defaultModelEndpoint(m.provider);
      var freeModel = m.model || W.defaultModelName(m.provider);
      var free = W.freeProviderById(m.provider);
      var keyHint = free && free.needsKey
        ? t("model.key-hint-llm7")
        : t("model.key-hint-keyless");
      return '' +
        '<div class="card">' +
          '<div class="field-grid">' +
            '<div class="field"><label for="modelProvider">' + esc(t("model.free-provider")) + "</label>" +
              '<select id="modelProvider">' + freeOptions + "</select>" +
              '<span class="hint">' + esc(t("model.free-hint")) + "</span></div>" +
            '<div class="field"><label for="modelEndpoint">' + esc(t("model.endpoint")) + "</label>" +
              '<input type="text" id="modelEndpoint" value="' + esc(freeEndpoint) + '" placeholder="https://…" />' +
              '<span class="hint">' + esc(t("model.endpoint-hint")) + "</span></div>" +
            '<div class="field"><label for="modelName">' + esc(t("model.name")) + "</label>" +
              '<input type="text" id="modelName" value="' + esc(freeModel) + '" placeholder="' + esc(t("model.name-placeholder")) + '" /></div>' +
            '<div class="field"><label for="modelKey">' + esc(t("model.key-optional")) + "</label>" +
              '<input type="password" id="modelKey" value="' + esc(m.api_key) + '" autocomplete="off" placeholder="' + esc(t("model.key-placeholder")) + '" />' +
              '<span class="hint">' + esc(keyHint) + "</span></div>" +
            '<div class="inline-row">' +
              '<button type="button" class="btn btn-secondary btn-sm" id="testModelBtn">' + esc(t("model.test")) + "</button>" +
              '<span class="note" id="modelTestStatus" role="status">' + esc(t("model.untested")) + "</span>" +
            "</div>" +
          "</div>" +
        "</div>";
    }
    var providerOptions = ["openai", "anthropic", "nvidia", "custom"].map(function (id) {
      return '<option value="' + id + '"' + (m.provider === id ? " selected" : "") + ">" +
        esc(id === "custom" ? t("model.other") : id.charAt(0).toUpperCase() + id.slice(1)) +
        "</option>";
    }).join("");
    var endpoint = m.endpoint || (m.provider === "custom" ? "" : W.defaultModelEndpoint(m.provider));
    var modelName = m.model || (m.provider === "custom" ? "" : W.defaultModelName(m.provider));
    return '' +
      '<div class="card">' +
        '<div class="field-grid">' +
          '<div class="field"><label for="modelProvider">' + esc(t("model.provider")) + "</label>" +
            '<select id="modelProvider">' + providerOptions + "</select></div>" +
          '<div class="field"><label for="modelEndpoint">' + esc(t("model.endpoint")) + "</label>" +
            '<input type="text" id="modelEndpoint" value="' + esc(endpoint) + '" placeholder="https://…" />' +
            '<span class="hint">' + esc(t("model.endpoint-hint")) + "</span></div>" +
          '<div class="field"><label for="modelName">' + esc(t("model.name")) + "</label>" +
            '<input type="text" id="modelName" value="' + esc(modelName) + '" placeholder="' + esc(t("model.name-placeholder")) + '" /></div>' +
          '<div class="field"><label for="modelKey">' + esc(t("model.key")) + "</label>" +
            '<input type="password" id="modelKey" value="' + esc(m.api_key) + '" autocomplete="off" placeholder="' + esc(t("model.key-placeholder-own")) + '" />' +
            '<span class="hint">' + esc(t("model.key-hint-own")) + "</span></div>" +
          '<div class="inline-row">' +
            '<button type="button" class="btn btn-secondary btn-sm" id="testModelBtn">' + esc(t("model.test")) + "</button>" +
            '<span class="note" id="modelTestStatus" role="status">' + esc(t("model.untested")) + "</span>" +
          "</div>" +
        "</div>" +
      "</div>";
  }

  function renderWelcome(draft, ctx) {
    return '' +
      '<div class="brand-hero"><img src="/wizard/logo.svg" alt="General Bots" /></div>' +
      '<p class="lead">' + esc(t("welcome.lead")) + "</p>" +
      '<div class="card">' +
        '<p class="note">' + esc(t("welcome.note1")) + "</p>" +
        '<p class="note">' + esc(t("welcome.note2")) + "</p>" +
      "</div>" +
      '<div class="field-grid" style="margin-top:16px">' +
        '<div class="field"><label for="displayName">' + esc(t("welcome.name-label")) + "</label>" +
          '<input type="text" id="displayName" value="' + esc(draft.display_name) + '" placeholder="' + esc(t("welcome.name-placeholder")) + '" /></div>' +
        '<p class="note">' + esc(t("welcome.workspace")) + " <strong>" + esc(ctx.workspace_path) + "</strong></p>" +
      "</div>";
  }

  function renderModel(draft) {
    var choices = W.MODELS.map(function (option) {
      var selected = draft.model.mode === option.id;
      var logo = option.logo
        ? '<img class="model-logo" src="' + esc(option.logo) + '" alt="" aria-hidden="true" />'
        : '<span class="model-logo model-logo-text" aria-hidden="true">' + esc(option.id === "local" ? "⌂" : (option.id === "free" ? "☁" : "⚿")) + "</span>";
      var badge = option.badgeKey
        ? '<span class="model-badge">' + esc(t(option.badgeKey)) + "</span>"
        : "";
      return '<button type="button" class="model-card' + (selected ? " selected" : "") + '"' +
        ' data-model-mode="' + esc(option.id) + '" aria-pressed="' + (selected ? "true" : "false") + '">' +
        logo +
        '<span class="model-copy">' +
          '<span class="model-title">' + esc(t(option.titleKey)) + "</span>" +
          '<span class="model-desc">' + esc(t(option.descKey)) + "</span>" +
          badge +
        "</span></button>";
    }).join("");
    return '' +
      '<p class="lead">' + esc(t("model.lead")) + "</p>" +
      '<div class="model-grid">' + choices + "</div>" +
      '<div style="margin-top:14px" id="modelDetail">' + modelDetail(draft) + "</div>";
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
      [t("privacy.crashes"), draft.telemetry ? t("privacy.crash-on") : t("privacy.crash-off"), t("privacy.opt-in")]
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
      '<div class="checkbox-row">' +
        '<input type="checkbox" id="telemetryToggle"' + (draft.telemetry ? " checked" : "") + " />" +
        '<label for="telemetryToggle">' + esc(t("privacy.telemetry")) + "</label>" +
      "</div>" +
      '<div class="inline-row" style="margin-top:12px">' +
        '<button type="button" class="btn btn-secondary btn-sm" id="revealVaultBtn">' + esc(t("privacy.reveal-vault")) + "</button>" +
        '<span class="note" id="vaultNote">' + esc(t("privacy.vault-note")) + "</span>" +
      "</div>" +
      '<p class="note install-next">' + esc(t("privacy.next-note")) + "</p>";
  }

  /* The Borland-style install screen: one prominent progress bar, then a
     traffic light per component (yellow = working, green = ready, red = failed). */
  function renderInstall(draft, ctx) {
    var planned = (ctx.components || []).filter(function (component) {
      return draft.model.mode === "local" || component.id !== "llm";
    });
    return '' +
      '<div class="install-top">' +
        '<div class="install-head">' +
          '<span class="install-kicker">' + esc(t("install.title")) + "</span>" +
          '<span class="install-percent" id="installPercent">0%</span>' +
        "</div>" +
        '<div class="install-bar"><div class="install-bar-fill" id="installFill"></div></div>' +
        '<p class="install-status" id="installStatus">' + esc(t("install.preparing", { count: planned.length })) + "</p>" +
      "</div>" +
      '<div class="component-board" id="componentList"></div>' +
      '<div id="provisionError" class="note err" role="status" style="margin-top:10px"></div>' +
      '<div class="inline-row" style="margin-top:12px">' +
        '<button type="button" class="btn btn-primary btn-sm" id="startSetupBtn">' + esc(t("install.start")) + "</button>" +
        '<button type="button" class="btn btn-secondary btn-sm" id="cancelSetupBtn" disabled>' + esc(t("install.cancel")) + "</button>" +
        '<button type="button" class="btn btn-secondary btn-sm" id="openLogsBtn">' + esc(t("install.open-logs")) + "</button>" +
        '<button type="button" class="btn btn-danger btn-sm" id="resetStackBtn">' + esc(t("install.reset")) + "</button>" +
      "</div>" +
      '<details class="details"><summary>' + esc(t("install.output")) + '</summary><pre class="log" id="provisionLog">' + esc(t("install.no-output")) + "</pre></details>";
  }

  function renderFinishAccount(draft) {
    var subscription = draft.model.mode === "claude-code" || draft.model.mode === "chatgpt-plan";
    if (!subscription) { return ""; }
    var name = draft.model.mode === "claude-code" ? "Claude Code" : "ChatGPT plan";
    var logo = draft.model.mode === "claude-code" ? "/wizard/logo-claude.svg" : "/wizard/logo-chatgpt.svg";
    var codeRow = draft.model.mode === "claude-code"
      ? '<div class="field" id="finishAccountCodeRow" hidden>' +
          '<label for="finishAccountCode">' + esc(t("accounts.code")) + "</label>" +
          '<div class="inline-row">' +
            '<input type="text" id="finishAccountCode" autocomplete="off" placeholder="' + esc(t("accounts.code-placeholder")) + '" />' +
            '<button type="button" class="btn btn-primary btn-sm" id="finishAccountCodeBtn">' + esc(t("accounts.submit")) + "</button>" +
          "</div></div>"
      : "";
    return '' +
      '<div class="card" id="finishAccountCard" style="margin-top:16px">' +
        '<div class="card-head"><span class="card-title"><img class="model-logo" src="' + logo + '" alt="" aria-hidden="true" /> ' +
          esc(t("finish.connect-title", { name: name })) + "</span>" +
          '<span class="badge" id="finishAccountStatus">' + esc(t("accounts.checking")) + "</span></div>" +
        '<p class="note" style="margin:8px 0">' + esc(t("finish.connect-note", { name: name })) + "</p>" +
        '<div class="inline-row">' +
          '<button type="button" class="btn btn-primary btn-sm" id="finishAccountConnect">' + esc(t("finish.connect-button", { name: name })) + "</button>" +
          '<button type="button" class="btn btn-secondary btn-sm" id="finishAccountDisconnect" hidden>' + esc(t("accounts.disconnect")) + "</button>" +
        "</div>" +
        '<div class="note" id="finishAccountDetail" role="status" style="margin-top:8px"></div>' +
        codeRow +
      "</div>";
  }

  function renderFinish(draft, ctx) {
    var modelLabel = draft.model.mode === "local"
      ? t("finish.model-local")
      : (draft.model.mode === "own-key" ? t("finish.model-own") : t("finish.model-hosted")) +
        (W.endpointHost(draft.model.endpoint) || draft.model.provider);
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
            '<input type="text" id="adminEmail" value="' + esc(draft.admin.email) + '" autocomplete="off" /></div>' +
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
        '<div class="summary-row"><span>' + esc(t("finish.crash")) + "</span><span>" + esc(draft.telemetry ? t("finish.enabled") : t("finish.off")) + "</span></div>" +
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
      case 6: return renderInstall(draft, ctx);
      case 7: return renderFinish(draft, ctx);
      default: return "<p>" + esc(t("wizard.unknown-step")) + "</p>";
    }
  }

  function heading(step) {
    switch (step) {
      case 1: return t("heading.welcome");
      case 2: return t("heading.model");
      case 3: return t("heading.workspace");
      case 4: return t("heading.services");
      case 5: return t("heading.privacy");
      case 6: return t("heading.install");
      case 7: return t("heading.finish");
      default: return t("wizard.title");
    }
  }

  function validate(step, draft) {
    if (step === 2) {
      if (draft.model.mode === "local") { return { ok: true }; }
      if (draft.model.mode === "claude-code" || draft.model.mode === "chatgpt-plan") {
        return { ok: true };
      }
      if (!draft.model.endpoint) { return { ok: false, reason: t("model.endpoint") + " — " + t("workspace.choose") }; }
      if (!draft.model.model) { return { ok: false, reason: t("model.name") }; }
      if (draft.model.mode !== "free" && !draft.model.api_key) {
        return { ok: false, reason: t("model.key") };
      }
      if (draft.model.mode === "free") {
        var free = W.freeProviderById(draft.model.provider);
        if (free && free.needsKey && !draft.model.api_key) {
          return { ok: false, reason: t("model.key-hint-llm7") };
        }
      }
      if (!draft.model.tested && draft.model.mode !== "free") { return { ok: false, reason: t("model.test") }; }
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
    modelDetail: modelDetail,
    esc: esc
  };
})();
