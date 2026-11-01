/* Step rendering, validation and collection for the first-run wizard. */
(function () {
  "use strict";

  var W = window.GBWizard;
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
    if (m.mode === "local") {
      return '<div class="card"><p class="note ok">The bundled model runs on this machine. No prompt leaves it.</p></div>';
    }
    if (m.mode === "free") {
      var freeOptions = W.FREE_PROVIDERS.map(function (p) {
        return '<option value="' + p.id + '"' + (m.provider === p.id ? " selected" : "") + ">" + esc(p.name) + "</option>";
      }).join("");
      var freeEndpoint = m.endpoint || W.defaultModelEndpoint(m.provider);
      var freeModel = m.model || W.defaultModelName(m.provider);
      var free = W.freeProviderById(m.provider);
      var keyHint = free && free.needsKey
        ? "LLM7.io requires a free account key."
        : "Kilo Code and OVHcloud work without a key; leave this empty for them.";
      return '' +
        '<div class="card">' +
          '<div class="field-grid">' +
            '<div class="field"><label for="modelProvider">Free provider</label>' +
              '<select id="modelProvider">' + freeOptions + "</select>" +
              '<span class="hint">Third-party free tiers under their own terms; no SLA. Do not send confidential data.</span></div>' +
            '<div class="field"><label for="modelEndpoint">Endpoint</label>' +
              '<input type="text" id="modelEndpoint" value="' + esc(freeEndpoint) + '" placeholder="https://…" />' +
              '<span class="hint">Prompts go only to this address.</span></div>' +
            '<div class="field"><label for="modelName">Model</label>' +
              '<input type="text" id="modelName" value="' + esc(freeModel) + '" placeholder="model identifier" /></div>' +
            '<div class="field"><label for="modelKey">API key (only if the provider requires one)</label>' +
              '<input type="password" id="modelKey" value="' + esc(m.api_key) + '" autocomplete="off" placeholder="optional" />' +
              '<span class="hint">' + esc(keyHint) + "</span></div>" +
            '<div class="inline-row">' +
              '<button type="button" class="btn btn-secondary btn-sm" id="testModelBtn">Test connection</button>' +
              '<span class="note" id="modelTestStatus" role="status">Not tested yet.</span>' +
            "</div>" +
          "</div>" +
        "</div>";
    }
    var providerOptions = ["openai", "anthropic", "nvidia", "custom"].map(function (id) {
      return '<option value="' + id + '"' + (m.provider === id ? " selected" : "") + ">" +
        esc(id === "custom" ? "Other (custom endpoint)" : id.charAt(0).toUpperCase() + id.slice(1)) +
        "</option>";
    }).join("");
    var endpoint = m.endpoint || (m.provider === "custom" ? "" : W.defaultModelEndpoint(m.provider));
    var modelName = m.model || (m.provider === "custom" ? "" : W.defaultModelName(m.provider));
    return '' +
      '<div class="card">' +
        '<div class="field-grid">' +
          '<div class="field"><label for="modelProvider">Provider</label>' +
            '<select id="modelProvider">' + providerOptions + "</select></div>" +
          '<div class="field"><label for="modelEndpoint">Endpoint</label>' +
            '<input type="text" id="modelEndpoint" value="' + esc(endpoint) + '" placeholder="https://…" />' +
            '<span class="hint">Prompts go only to this address.</span></div>' +
          '<div class="field"><label for="modelName">Model</label>' +
            '<input type="text" id="modelName" value="' + esc(modelName) + '" placeholder="model identifier" /></div>' +
          '<div class="field"><label for="modelKey">API key</label>' +
            '<input type="password" id="modelKey" value="' + esc(m.api_key) + '" autocomplete="off" placeholder="stored in the local vault" />' +
            '<span class="hint">Never displayed again after setup and never written to a config file.</span></div>' +
          '<div class="inline-row">' +
            '<button type="button" class="btn btn-secondary btn-sm" id="testModelBtn">Test connection</button>' +
            '<span class="note" id="modelTestStatus" role="status">Not tested yet.</span>' +
          "</div>" +
        "</div>" +
      "</div>";
  }

  function renderWelcome(draft, ctx) {
    return '' +
      '<p class="lead">Runs on this machine; nothing leaves it unless you connect a service.</p>' +
      '<div class="card">' +
        '<p class="note">This setup creates a private workspace, starts the local services the assistant needs, and opens the assistant when everything answers.</p>' +
        '<p class="note">It takes about a minute. You can stop and continue later without losing what you entered.</p>' +
      "</div>" +
      '<div class="field-grid" style="margin-top:16px">' +
        '<div class="field"><label for="displayName">What should the assistant call you? (optional)</label>' +
          '<input type="text" id="displayName" value="' + esc(draft.display_name) + '" placeholder="Your name" /></div>' +
        '<p class="note">Workspace: <strong>' + esc(ctx.workspace_path) + "</strong></p>" +
      "</div>";
  }

  function renderModel(draft) {
    var choices = W.MODELS.map(function (option) {
      var selected = draft.model.mode === option.id;
      return '<label class="choice' + (selected ? " selected" : "") + '">' +
        '<input type="radio" name="modelMode" value="' + option.id + '"' + (selected ? " checked" : "") + " />" +
        '<span><span class="choice-title">' + esc(option.title) + '</span>' +
        '<span class="choice-desc">' + esc(option.desc) + "</span></span></label>";
    }).join("");
    return '' +
      '<p class="lead">Local model = nothing leaves the machine. Free hosted = prompts go to a third-party free tier. Your own key = prompts go to that provider only.</p>' +
      '<div class="choice-list">' + choices + "</div>" +
      '<div style="margin-top:14px" id="modelDetail">' + modelDetail(draft) + "</div>";
  }

  function renderWorkspace(draft, ctx) {
    var path = draft.workspace.path || ctx.workspace_path;
    return '' +
      '<p class="lead">Files and database live in this folder; you can move it later. Size is shown up front.</p>' +
      '<div class="field-grid">' +
        '<div class="field"><label for="workspacePath">Workspace folder</label>' +
          '<div class="inline-row">' +
            '<input type="text" id="workspacePath" value="' + esc(path) + '" />' +
            '<button type="button" class="btn btn-secondary btn-sm" id="browseFolderBtn">Browse…</button>' +
          "</div>" +
          '<span class="hint" id="workspaceStatus" role="status">Checking the folder…</span></div>' +
        '<div class="field"><label for="workspaceSize">Space for the local model</label>' +
          '<select id="workspaceSize">' +
            '<option value="standard"' + (draft.workspace.size === "standard" ? " selected" : "") + ">Standard — about 1.8 GB</option>" +
            '<option value="compact"' + (draft.workspace.size === "compact" ? " selected" : "") + ">Compact — about 900 MB</option>" +
          "</select>" +
          '<span class="hint" id="workspaceFree">Free space is checked when you choose a folder.</span></div>' +
      "</div>";
  }

  function renderServiceCard(provider, draft, ctx) {
    var saved = draft.services[provider.id] || {};
    var state = saved.connected ? "Connected" : (saved.client_id ? "Needs attention" : "Not connected");
    var kind = saved.connected ? "ok" : (saved.client_id ? "warn" : "");
    var redirect = (ctx.redirect_uri || provider.redirectTemplate).replace("{provider}", provider.id);
    return '' +
      '<div class="card" data-service="' + provider.id + '">' +
        '<div class="card-head"><span class="card-title">' + esc(provider.name) + "</span>" +
          statusBadge(state, kind) + "</div>" +
        '<p class="note" style="margin:8px 0">' + esc(provider.steps) + "</p>" +
        '<p class="note" style="margin:0 0 8px">' + esc(provider.scopeNote) + "</p>" +
        '<div class="field" style="margin-bottom:10px">' +
          '<label>Redirect URI</label>' +
          '<div class="inline-row">' +
            '<input type="text" readonly value="' + esc(redirect) + '" />' +
            '<button type="button" class="btn btn-secondary btn-sm" data-copy="' + esc(redirect) + '">Copy</button>' +
          "</div>" +
        "</div>" +
        '<div class="inline-row" style="margin-bottom:10px">' +
          '<button type="button" class="btn btn-secondary btn-sm" data-console="' + esc(provider.console) + '">' + esc(provider.consoleLabel) + "</button>" +
        "</div>" +
        '<div class="field-grid">' +
          '<div class="field"><label>' + esc(provider.fields.id) + '</label>' +
            '<input type="text" data-service-id="' + provider.id + '" value="' + esc(saved.client_id || "") + '" autocomplete="off" /></div>' +
          '<div class="field"><label>' + esc(provider.fields.secret) + '</label>' +
            '<input type="password" data-service-secret="' + provider.id + '" value="' + esc(saved.client_secret || "") + '" autocomplete="off" />' +
            '<span class="hint">Optional for desktop clients that use loopback redirects and PKCE.</span></div>' +
        "</div>" +
        '<p class="note" data-service-note="' + provider.id + '"></p>' +
      "</div>";
  }

  function renderServices(draft, ctx) {
    return '' +
      '<p class="lead">Each service is optional and can be added later in Settings → Integrations.</p>' +
      W.PROVIDERS.map(function (p) { return renderServiceCard(p, draft, ctx); }).join("") +
      '<p class="note">Skipping leaves no partial state: nothing is stored and nothing is authorized.</p>';
  }

  function dataMapRows(draft, ctx) {
    var model = draft.model;
    var prompts;
    if (model.mode === "local") {
      prompts = "Stays on this machine (bundled local model)";
    } else {
      var host = W.endpointHost(model.endpoint || W.defaultModelEndpoint(model.provider)) || "the provider you choose";
      prompts = "Sent to " + host + (W.isLoopback(model.endpoint) ? " (on this machine)" : "");
    }
    var connected = W.PROVIDERS.filter(function (p) {
      var s = draft.services[p.id];
      return s && s.client_id;
    }).map(function (p) { return p.name; });
    return [
      ["Conversations, generated files, database", "This machine — " + (draft.workspace.path || ctx.workspace_path), "Always"],
      ["API keys and OAuth tokens", "Local vault on this machine — " + ctx.stack_path, "When you save them"],
      ["Prompts and answers", prompts, "Every message"],
      ["Mail and files", connected.length ? connected.join(", ") + " — only for actions you ask for" : "No service connected", "Per request"],
      ["Crash reports", draft.telemetry ? "Crash signature, version and operating system — no prompt text, no file names" : "Nowhere (telemetry is off)", "Opt-in only"]
    ];
  }

  function renderPrivacy(draft, ctx) {
    var rows = dataMapRows(draft, ctx).map(function (row) {
      return "<tr><td>" + esc(row[0]) + "</td><td>" + esc(row[1]) + "</td><td>" + esc(row[2]) + "</td></tr>";
    }).join("");
    return '' +
      '<p class="lead">Conversations and files stay local; keys are sealed in the local vault; only your chosen model sees prompts.</p>' +
      '<table class="datamap"><thead><tr><th>Data</th><th>Destination</th><th>Trigger</th></tr></thead><tbody>' +
      rows + "</tbody></table>" +
      '<div class="checkbox-row">' +
        '<input type="checkbox" id="telemetryToggle"' + (draft.telemetry ? " checked" : "") + " />" +
        '<label for="telemetryToggle">Send anonymous crash reports. Off by default; enabling sends only the crash signature, version and operating system.</label>' +
      "</div>" +
      '<div class="inline-row" style="margin-top:12px">' +
        '<button type="button" class="btn btn-secondary btn-sm" id="revealVaultBtn">Reveal vault location</button>' +
        '<span class="note" id="vaultNote">The assistant never displays a stored secret again.</span>' +
      "</div>" +
      '<h2 style="font-size:15px;margin:22px 0 6px">Set up this machine</h2>' +
      '<p class="note" style="margin:0 0 10px" id="provisionIntro">Starting the local services. Each step reports its own state.</p>' +
      '<div class="components" id="componentList"></div>' +
      '<div id="provisionError" class="note err" role="status" style="margin-top:10px"></div>' +
      '<div class="inline-row" style="margin-top:12px">' +
        '<button type="button" class="btn btn-primary btn-sm" id="startSetupBtn">Start setup</button>' +
        '<button type="button" class="btn btn-secondary btn-sm" id="cancelSetupBtn" disabled>Cancel</button>' +
        '<button type="button" class="btn btn-secondary btn-sm" id="openLogsBtn">Open log folder</button>' +
        '<button type="button" class="btn btn-danger btn-sm" id="resetStackBtn">Reset stack</button>' +
      "</div>" +
      '<details class="details"><summary>Details</summary><pre class="log" id="provisionLog">No output yet.</pre></details>';
  }

  function renderFinish(draft, ctx) {
    var modelLabel = draft.model.mode === "local"
      ? "Bundled local model (nothing leaves this machine)"
      : (draft.model.mode === "own-key" ? "Your own key — " : "Hosted — ") +
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
      '<p class="lead">Review what this machine will do. Save the administrator account, then open the assistant.</p>' +
      '<div class="card" id="adminCard">' +
        '<div class="card-head"><span class="card-title">Administrator account</span>' +
          statusBadge("Created by setup", "ok") + "</div>" +
        '<p class="note" style="margin:8px 0">Use these to sign in to the management area. Save the password now — it is shown only on this screen.</p>' +
        '<div class="field-grid">' +
          '<div class="field"><label for="adminEmail">Email</label>' +
            '<input type="text" id="adminEmail" value="' + esc(draft.admin.email) + '" autocomplete="off" /></div>' +
          '<div class="field"><label for="adminPassword">Password</label>' +
            '<div class="inline-row">' +
              '<input type="text" id="adminPassword" readonly value="' + esc(draft.admin.password) + '" />' +
              '<button type="button" class="btn btn-secondary btn-sm" id="copyAdminBtn">Copy</button>' +
              '<button type="button" class="btn btn-secondary btn-sm" id="regenerateAdminBtn">Regenerate</button>' +
            "</div>" +
            '<span class="hint">Stored only on this machine, in the local vault configuration.</span></div>' +
        "</div>" +
      "</div>" +
      '<div class="summary" style="margin-top:16px">' +
        '<div class="summary-row"><span>Workspace</span><span>' + esc(draft.workspace.path || ctx.workspace_path) + "</span></div>" +
        '<div class="summary-row"><span>Assistant model</span><span>' + esc(modelLabel) + "</span></div>" +
        '<div class="summary-row"><span>Connected services</span><span>' + esc(connected.length ? connected.join(", ") : "None") + "</span></div>" +
        '<div class="summary-row"><span>Not connected</span><span>' + esc(notConnected.length ? notConnected.join(", ") : "None") + "</span></div>" +
        '<div class="summary-row"><span>Crash reports</span><span>' + (draft.telemetry ? "Enabled" : "Off") + "</span></div>" +
      "</div>" +
      '<p class="note" style="margin-top:16px">You can change every choice later in Settings. Nothing is sent anywhere until you ask the assistant to use a connected service.</p>' +
      '<div id="finishStatus" class="note" role="status" style="margin-top:12px"></div>';
  }

  function render(step, draft, ctx) {
    switch (step) {
      case 1: return renderWelcome(draft, ctx);
      case 2: return renderModel(draft);
      case 3: return renderWorkspace(draft, ctx);
      case 4: return renderServices(draft, ctx);
      case 5: return renderPrivacy(draft, ctx);
      case 6: return renderFinish(draft, ctx);
      default: return "<p>Unknown step.</p>";
    }
  }

  function heading(step) {
    switch (step) {
      case 1: return "Welcome";
      case 2: return "Choose the assistant model";
      case 3: return "Choose a workspace";
      case 4: return "Connect services";
      case 5: return "What leaves this machine";
      case 6: return "Ready to open the assistant";
      default: return "Setup";
    }
  }

  function validate(step, draft) {
    if (step === 2) {
      if (draft.model.mode === "local") { return { ok: true }; }
      if (!draft.model.endpoint) { return { ok: false, reason: "Enter the provider endpoint." }; }
      if (!draft.model.model) { return { ok: false, reason: "Enter the model identifier." }; }
      if (draft.model.mode !== "free" && !draft.model.api_key) {
        return { ok: false, reason: "Enter the key for this provider, or choose a free or local option." };
      }
      if (draft.model.mode === "free") {
        var free = W.freeProviderById(draft.model.provider);
        if (free && free.needsKey && !draft.model.api_key) {
          return { ok: false, reason: free.name + " requires a free account key." };
        }
      }
      if (!draft.model.tested) { return { ok: false, reason: "Test the connection before continuing." }; }
      return { ok: true };
    }
    if (step === 3) {
      if (!draft.workspace.path) { return { ok: false, reason: "Choose a workspace folder." }; }
      if (draft.workspace.blocked) { return { ok: false, reason: draft.workspace.block_reason || "The folder is not writable." }; }
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
