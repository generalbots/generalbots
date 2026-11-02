/* First-run wizard state: steps, draft shape and provider guidance. */
(function () {
  "use strict";

  var STEPS = [
    { id: "welcome", key: "step.welcome" },
    { id: "model", key: "step.model" },
    { id: "workspace", key: "step.workspace" },
    { id: "services", key: "step.services" },
    { id: "privacy", key: "step.privacy" },
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

  var MODELS = [
    {
      id: "local",
      titleKey: "model.local.title",
      descKey: "model.local.desc"
    },
    {
      id: "free",
      titleKey: "model.free.title",
      descKey: "model.free.desc"
    },
    {
      id: "own-key",
      titleKey: "model.ownkey.title",
      descKey: "model.ownkey.desc"
    },
    {
      id: "hosted",
      titleKey: "model.hosted.title",
      descKey: "model.hosted.desc"
    }
  ];

  // The free tier ships a single keyless provider. Kilo's gateway is the only
  // free endpoint verified to follow the native vm_exec JSON protocol and to
  // return well-formed tool calls (OVH answers in prose, LLM7 requires a key).
  var FREE_PROVIDERS = [
    {
      id: "kilo",
      name: "Kilo Code",
      endpoint: "https://api.kilo.ai/api/gateway/chat/completions",
      model: "dots-studio/dots-3-note-preview:free",
      needsKey: false
    }
  ];

  function freeProviderById(id) {
    for (var i = 0; i < FREE_PROVIDERS.length; i += 1) {
      if (FREE_PROVIDERS[i].id === id) { return FREE_PROVIDERS[i]; }
    }
    return null;
  }

  var state = {
    step: 1,
    totalSteps: STEPS.length,
    context: null,
    resumed: false,
    draft: {
      display_name: "",
      admin: { email: "admin@localhost", password: "", user_id: "" },
      // Default to the keyless free tier (Kilo). It needs no download, answers
      // immediately, and keeps the first install small; the bundled local model
      // stays available as an explicit choice.
      model: { mode: "free", provider: "kilo", endpoint: "https://api.kilo.ai/api/gateway/chat/completions", model: "dots-studio/dots-3-note-preview:free", api_key: "", tested: false },
      workspace: { path: "", size: "standard" },
      services: {},
      telemetry: false
    },
    provisioning: {
      running: false,
      ready: false,
      failed: null,
      startedAt: 0,
      components: []
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
