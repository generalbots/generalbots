/* First-run wizard state: steps, draft shape and provider guidance. */
(function () {
  "use strict";

  var STEPS = [
    { id: "welcome", key: "step.welcome" },
    { id: "model", key: "step.model" },
    { id: "workspace", key: "step.workspace" },
    { id: "services", key: "step.services" },
    { id: "privacy", key: "step.privacy" },
    { id: "network", key: "step.network" },
    { id: "install", key: "step.install" },
    { id: "finish", key: "step.finish" }
  ];

  var PROVIDERS = [
    {
      id: "gmail",
      name: "Gmail",
      console: "https://console.cloud.google.com/apis/credentials",
      consoleLabelKey: "services.gmail.console",
      redirectTemplate: "http://127.0.0.1:8080/api/integrations/oauth/gmail/callback",
      stepsKey: "services.gmail.steps",
      scopeNoteKey: "services.gmail.scope",
      fields: { id: "services.client-id", secret: "services.client-secret" }
    },
    {
      id: "onedrive",
      name: "OneDrive",
      console: "https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade",
      consoleLabelKey: "services.onedrive.console",
      redirectTemplate: "http://127.0.0.1:8080/api/integrations/oauth/onedrive/callback",
      stepsKey: "services.onedrive.steps",
      scopeNoteKey: "services.onedrive.scope",
      fields: { id: "services.client-id", secret: "services.client-secret" }
    },
    {
      id: "dropbox",
      name: "Dropbox",
      console: "https://www.dropbox.com/developers/apps",
      consoleLabelKey: "services.dropbox.console",
      redirectTemplate: "http://127.0.0.1:8080/api/integrations/oauth/dropbox/callback",
      stepsKey: "services.dropbox.steps",
      scopeNoteKey: "services.dropbox.scope",
      fields: { id: "services.client-id", secret: "services.client-secret" }
    }
  ];

  /* Four engines, same order and words as the app's Settings > Modelo page
     (10c_settings_garage.js INFO map). Subscription agents (Claude Code, Codex,
     Antigravity) are connected AFTER setup, inside the app. */
  var MODELS = [
    { id: "local",   num: "01", titleKey: "model.local.title",  descKey: "model.local.desc",  consequenceKey: "model.local.consequence",  tone: "ok",  privacy: 5, icon: "M3 4h18v12H3zM8 20h8M12 16v4" },
    { id: "own-key", num: "02", titleKey: "model.ownkey.title", descKey: "model.ownkey.desc", consequenceKey: "model.ownkey.consequence", tone: "mid", privacy: 3, icon: "M3 4h18v6H3zM3 14h18v6H3zM7 7h.01M7 17h.01" },
    { id: "free",    num: "03", titleKey: "model.free.title",   descKey: "model.free.desc",   consequenceKey: "model.free.consequence",   tone: "out", privacy: 1, icon: "M13 2 3 14h9l-1 8 10-12h-9z" },
    { id: "acp",     num: "04", titleKey: "model.acp.title",    descKey: "model.acp.desc",    consequenceKey: "model.acp.consequence",    tone: "out", privacy: 2, icon: "M4 4h16v16H4zM8 9l3 3-3 3M13 15h3" }
  ];

  // The free tier ships a single keyless provider. Kilo's gateway is the only
  // free endpoint verified to follow the native vm_exec JSON protocol and to
  // return well-formed tool calls (OVH answers in prose, LLM7 requires a key).
  // The model id must match FREE_MODEL in botserver/crates/botsettings.
  var FREE_MODEL_ID = "nvidia/nemotron-3-ultra-550b-a55b:free";
  var FREE_PROVIDERS = [
    {
      id: "kilo",
      name: "Kilo Code",
      endpoint: "https://api.kilo.ai/api/gateway/chat/completions",
      model: FREE_MODEL_ID,
      needsKey: false
    }
  ];

  function freeProviderById(id) {
    for (var i = 0; i < FREE_PROVIDERS.length; i += 1) {
      if (FREE_PROVIDERS[i].id === id) { return FREE_PROVIDERS[i]; }
    }
    return null;
  }

  /* Setup always starts on the free engine: the other three need the running
     app (machine scan, provider lists, key tests, agent sign-in) and are set
     in Settings > Modelo afterwards. */
  function freeModel() {
    var free = FREE_PROVIDERS[0];
    return { mode: "free", provider: free.id, endpoint: free.endpoint, model: free.model, api_key: "", tested: false };
  }

  /* A service is on when its switch is on; drafts saved before the switch
     existed count as on when they already carry a client ID. */
  function serviceEnabled(saved) {
    if (!saved) { return false; }
    return typeof saved.enabled === "boolean" ? saved.enabled : !!saved.client_id;
  }

  /* What the provisioning and finish requests carry: only switched-on
     services that have a client ID, in the ServiceClient shape. */
  function sentServices(services) {
    var out = {};
    Object.keys(services || {}).forEach(function (id) {
      var s = services[id];
      if (serviceEnabled(s) && s.client_id) {
        out[id] = { client_id: s.client_id, client_secret: s.client_secret || "" };
      }
    });
    return out;
  }

  var state = {
    step: 1,
    totalSteps: STEPS.length,
    context: null,
    resumed: false,
    draft: {
      display_name: "",
      admin: { email: "admin@localhost", password: "", user_id: "" },
      model: freeModel(),
      workspace: { path: "", size: "standard" },
      services: {},
      network: { mode: "p2p" },
      appearance: { accent: "" },
      // Windows agent mode requires WSL2; Chat/Drive-only users can skip all
      // agent-related setup. Persisted so re-runs remember the choice (#1937).
      skip_all_agentic_work: false
    },
    provisioning: {
      running: false,
      ready: false,
      failed: null,
      startedAt: 0,
      components: []
    },
    host: {
      checking: false,
      checked: false,
      acknowledged: false,
      result: null,
      error: ""
    }
  };

  function clone(value) {
    return JSON.parse(JSON.stringify(value));
  }

  function providerById(id) {
    for (var i = 0; i < PROVIDERS.length; i += 1) {
      if (PROVIDERS[i].id === id) { return PROVIDERS[i]; }
    }
    return null;
  }

  function endpointHost(endpoint) {
    if (!endpoint) { return ""; }
    try { return new URL(endpoint).host; } catch (e) { return endpoint; }
  }

  function isLoopback(endpoint) {
    if (!endpoint) { return false; }
    try {
      var host = new URL(endpoint).hostname;
      return host === "localhost" || host === "127.0.0.1" || host === "::1";
    } catch (e) {
      return endpoint.indexOf("localhost") !== -1 || endpoint.indexOf("127.0.0.1") !== -1;
    }
  }

  function formatBytes(bytes) {
    if (!bytes && bytes !== 0) { return (window.WizardI18n ? window.WizardI18n.t("nav.unknown") : "unknown"); }
    var units = ["B", "KB", "MB", "GB", "TB"];
    var value = bytes;
    var unit = 0;
    while (value >= 1024 && unit < units.length - 1) { value /= 1024; unit += 1; }
    return value.toFixed(value >= 10 || unit === 0 ? 0 : 1) + " " + units[unit];
  }

  function defaultModelEndpoint(provider) {
    for (var i = 0; i < FREE_PROVIDERS.length; i += 1) {
      if (FREE_PROVIDERS[i].id === provider) { return FREE_PROVIDERS[i].endpoint; }
    }
    switch (provider) {
      case "claude-code": return "";
      case "chatgpt-plan": return "";
      case "openai": return "https://api.openai.com/v1/chat/completions";
      case "anthropic": return "https://api.anthropic.com/v1/messages";
      case "nvidia": return "https://integrate.api.nvidia.com/v1/chat/completions";
      default: return "";
    }
  }

  function defaultModelName(provider) {
    for (var i = 0; i < FREE_PROVIDERS.length; i += 1) {
      if (FREE_PROVIDERS[i].id === provider) { return FREE_PROVIDERS[i].model; }
    }
    switch (provider) {
      case "claude-code": return "claude-sonnet-4-5";
      case "chatgpt-plan": return "gpt-5-codex";
      case "openai": return "gpt-5.6-sol";
      case "anthropic": return "claude-sonnet-5-5";
      case "nvidia": return "meta/llama-4-maverick-17b-128e-instruct";
      default: return "";
    }
  }

  window.GBWizard = {
    STEPS: STEPS,
    PROVIDERS: PROVIDERS,
    MODELS: MODELS,
    FREE_PROVIDERS: FREE_PROVIDERS,
    freeProviderById: freeProviderById,
    freeModel: freeModel,
    serviceEnabled: serviceEnabled,
    sentServices: sentServices,
    state: state,
    clone: clone,
    providerById: providerById,
    endpointHost: endpointHost,
    isLoopback: isLoopback,
    formatBytes: formatBytes,
    defaultModelEndpoint: defaultModelEndpoint,
    defaultModelName: defaultModelName
  };
})();
