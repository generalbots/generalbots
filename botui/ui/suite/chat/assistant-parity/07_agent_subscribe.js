/* Assistant parity — subscription-agent vocabulary and locked Agent switch.
 *
 * Chat already hydrates `sub:<provider>` entries from /api/ai-accounts in
 * chat-model-picker.js, groups them in 02_model_dock.js and owns the Agent
 * switch in 46_chat_controls.js / chat-agent-mode.js. This module adds only the
 * missing pieces: assistant's engine-name vocabulary, the per-agent enabled
 * models (`sub:<provider>/<model>`) and the rule that a subscription model
 * holds the Agent switch on. It never opens a second socket or replaces the
 * native model picker.
 */
"use strict";
(function () {
  var P = (window.GBChatAssistantParity = window.GBChatAssistantParity || {});
  var mounted = false;
  var restore = null;
  var hydrated = Object.create(null);
  var retry = 0;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function headers(extra) {
    var h = extra || {};
    try {
      if (window.GBShared && window.GBShared.auth && typeof window.GBShared.auth.headers === "function") {
        var all = window.GBShared.auth.headers();
        Object.keys(all).forEach(function (key) { h[key] = all[key]; });
      }
    } catch (error) { /* fall through */ }
    try {
      var token = (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
        (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
        sessionStorage.getItem("gb-access-token") || "";
      if (token) h.Authorization = "Bearer " + token;
    } catch (error) { /* anonymous chat */ }
    return h;
  }
  /* Each tier is a function so the catalogue key stays literal at the gbT
   * call site (the i18n gate scans those call sites). */
  var NAMES = {
    local: [
      function () { return t("chat-vm-engine-local-short", "Local"); },
      function () { return t("chat-vm-engine-local-group", "Local models"); },
      function () { return t("chat-vm-engine-local-detail", "On this computer"); }
    ],
    free: [
      function () { return t("chat-vm-engine-free-short", "Free"); },
      function () { return t("chat-vm-engine-free-group", "Free model"); },
      function () { return t("chat-vm-engine-free-detail", "Free model"); }
    ],
    own: [
      function () { return t("chat-vm-engine-own-short", "Own key"); },
      function () { return t("chat-vm-engine-own-group", "Own keys"); },
      function () { return t("chat-vm-engine-own-detail", "Own server or key"); }
    ],
    acp: [
      function () { return t("chat-vm-engine-sub-short", "Subscription"); },
      function () { return t("chat-vm-engine-sub-group", "Subscriptions"); },
      function () { return t("chat-vm-engine-sub-detail", "Subscription agent"); }
    ]
  };
  NAMES.own_key = NAMES.own;
  var LIMIT_KIND = { subscription: "acp", api_key: "own", local: "local", free: "free" };
  function engineName(type, tier) {
    var entry = NAMES[type];
    if (!entry) return String(type || "");
    var fn = entry[["short", "group", "detail"].indexOf(tier || "short")] || entry[0];
    return fn();
  }
  function engineByLimitKind(kind, tier) {
    var mapped = LIMIT_KIND[kind];
    return mapped ? engineName(mapped, tier) : String(kind || "");
  }

  function select() {
    return document.getElementById("modelPicker");
  }
  function selected() {
    var picker = select();
    return picker ? String(picker.value || "auto") : "auto";
  }
  function isSubscription(value) {
    return /^sub:/.test(String(value || ""));
  }
  function providerOf(value) {
    return String(value || "").replace(/^sub:/, "").split("/")[0];
  }
  function agentLabel(provider) {
    var picker = select();
    if (picker) {
      for (var i = 0; i < picker.options.length; i++) {
        if (picker.options[i].value === "sub:" + provider ||
            picker.options[i].value.indexOf("sub:" + provider + "/") === 0) {
          return String(picker.options[i].textContent || provider).trim();
        }
      }
    }
    var linked = window.gbLinkedAgents || [];
    for (var j = 0; j < linked.length; j++) {
      if (linked[j] && linked[j].provider === provider) return linked[j].name || provider;
    }
    return provider;
  }
  function status() {
    var el = document.getElementById("gbParityAgentStatus");
    if (el) return el;
    el = document.createElement("div");
    el.id = "gbParityAgentStatus";
    el.className = "gb-parity-vm-status";
    el.setAttribute("role", "status");
    el.setAttribute("aria-live", "polite");
    var form = document.getElementById("chatForm");
    if (form && form.parentNode) form.parentNode.insertBefore(el, form);
    else document.body.appendChild(el);
    return el;
  }
  function say(text) {
    var el = status();
    if (el.textContent !== String(text || "")) el.textContent = String(text || "");
  }
  function storageKey() {
    var bot = (window.ChatState && ChatState.currentBotId) || window.__BOT_NAME__ || "default";
    var sid = (window.ChatState && ChatState.currentSessionId) || "global";
    return "gb-parity-agent-pref:" + bot + ":" + sid;
  }
  function readRestore() {
    try {
      var raw = localStorage.getItem(storageKey());
      return raw ? JSON.parse(raw) : null;
    } catch (error) { return null; }
  }
  function writeRestore(value) {
    try {
      if (value) localStorage.setItem(storageKey(), JSON.stringify(value));
      else localStorage.removeItem(storageKey());
    } catch (error) { /* private window */ }
  }
  function controls() {
    return window.GBChatControls || null;
  }
  function checkbox() {
    var panel = document.getElementById("chatControlsPanel");
    return panel ? panel.querySelector("#cc-agent") : null;
  }
  function lockSwitch(provider) {
    var gate = controls();
    if (gate && typeof gate.getState === "function") {
      var before = gate.getState();
      if (!restore) {
        var priorMode = window.AgentMode && typeof window.AgentMode.enabledForActiveTab === "function" &&
          window.AgentMode.enabledForActiveTab() ? "agent" : "chat";
        writeRestore({ agent: !!before.agent, mode: priorMode });
        restore = readRestore();
      }
      gate.setState({ agent: true });
    }
    var agentMode = window.AgentMode;
    if (agentMode && typeof agentMode.isActive === "function" && !agentMode.isActive() &&
        typeof agentMode.setMode === "function") {
      agentMode.setMode("agent");
    }
    var box = checkbox();
    if (box) {
      box.disabled = true;
      box.setAttribute("aria-disabled", "true");
      box.title = t("chat-vm-agent-locked-tip",
        "The selected agent runs in the Virtual Machine; choose another model to turn it off");
    }
    say(agentLabel(provider) + " " + t("chat-vm-agent-locked-note",
      "runs in the Virtual Machine, so the Agent switch stays on while it is selected. Choose another model to turn it off."));
  }
  function unlockSwitch() {
    var box = checkbox();
    if (box) {
      box.disabled = false;
      box.removeAttribute("aria-disabled");
      box.title = "";
    }
    var saved = restore || readRestore();
    if (saved) {
      var gate = controls();
      if (gate && typeof gate.setState === "function" && typeof saved.agent === "boolean") {
        gate.setState({ agent: saved.agent });
      }
      var agentMode = window.AgentMode;
      if (agentMode && typeof agentMode.setMode === "function" && saved.mode) {
        agentMode.setMode(saved.mode);
      }
      writeRestore(null);
      restore = null;
    }
    say("");
  }
  function sync() {
    var value = selected();
    if (isSubscription(value)) lockSwitch(providerOf(value));
    else unlockSwitch();
  }
  function agentGroup() {
    var picker = select();
    if (!picker) return null;
    return picker.querySelector("optgroup.gb-model-agent-group") ||
      (function () {
        for (var i = 0; i < picker.querySelectorAll("optgroup").length; i++) {
          var group = picker.querySelectorAll("optgroup")[i];
          if (/^sub:/.test(group.querySelector("option") ? group.querySelector("option").value : "")) {
            group.className += " gb-model-agent-group";
            return group;
          }
        }
        return null;
      })();
  }
  function addModels(group, provider, models) {
    var picker = select();
    if (!group || !picker) return;
    var seen = {};
    for (var i = 0; i < group.options.length; i++) seen[group.options[i].value] = true;
    (Array.isArray(models) ? models : []).forEach(function (model) {
      var id = typeof model === "string" ? model :
        (model && (model.model || model.id || model.name));
      if (!id) return;
      var value = "sub:" + provider + "/" + id;
      if (seen[value]) return;
      seen[value] = true;
      var option = document.createElement("option");
      option.value = value;
      option.textContent = agentLabel(provider) + " \u00B7 " + id;
      group.appendChild(option);
    });
  }
  function hydrateModels() {
    var group = agentGroup();
    if (!group) {
      if (retry++ < 6) window.setTimeout(hydrateModels, 400);
      return;
    }
    group.label = engineName("acp", "group");
    var linked = window.gbLinkedAgents;
    if (!Array.isArray(linked) || !linked.length) {
      fetch("/api/ai-accounts", { cache: "no-store", headers: headers() })
        .then(function (response) { return response.ok ? response.json() : null; })
        .then(function (data) {
          if (!data || !Array.isArray(data.accounts)) return;
          linked = data.accounts.filter(function (a) { return a && a.status === "linked" && a.provider; });
          window.gbLinkedAgents = linked;
          linked.forEach(fetchModels);
        })
        .catch(function () { /* linked agents are optional */ });
      return;
    }
    linked.forEach(fetchModels);
  }
  function fetchModels(account) {
    if (!account || !account.provider || hydrated[account.provider]) return;
    hydrated[account.provider] = true;
    fetch("/api/ai-accounts/" + encodeURIComponent(account.provider) + "/models", {
      cache: "no-store", headers: headers()
    }).then(function (response) { return response.ok ? response.json() : null; })
      .then(function (data) {
        var list = data && Array.isArray(data.models) ? data.models : [];
        var enabled = list.filter(function (model) { return !model || model.enabled !== false; })
          .map(function (model) { return typeof model === "string" ? model : (model.model || model.id || ""); })
          .filter(Boolean);
        addModels(agentGroup(), account.provider, enabled);
      })
      .catch(function () { /* model list is optional; the plain sub:<id> stays */ });
  }
  /* 10i_accounts.js equivalent: chat's chat-model-picker already reads
   * /api/llm/accounts and installs `acct:` options. Derive the normalized
   * account list from those options instead of issuing a second request. */
  function accountsFromPicker() {
    var picker = select();
    var result = [];
    if (!picker) return result;
    var groups = picker.querySelectorAll("optgroup.gb-model-account-group");
    Array.prototype.forEach.call(groups, function (group) {
      var models = [];
      Array.prototype.forEach.call(group.options, function (option) {
        if (option.value.indexOf("acct:") === 0) {
          models.push({ model: option.textContent, value: option.value });
        }
      });
      if (models.length) {
        result.push({ id: group.label, name: group.label, models: models });
      }
    });
    return result;
  }

  P.engineName = engineName;
  P.engineNameByLimitKind = engineByLimitKind;
  P.agentSubscribe = {
    mount: function () {
      if (mounted) return P;
      mounted = true;
      var picker = select();
      if (!picker) return P;
      picker.addEventListener("change", sync);
      /* 46_chat_controls.js creates the panel on gb-chat-ready: re-apply the
       * lock once the real checkbox exists. */
      window.addEventListener("gb-chat-ready", sync);
      window.addEventListener("gb:auth:login", function () {
        hydrated = Object.create(null);
        hydrateModels();
        sync();
      });
      var panel = document.getElementById("chatControlsPanel");
      if (panel) {
        panel.addEventListener("change", function (event) {
          if (!isSubscription(selected()) || !event.target || event.target.id !== "cc-agent") return;
          event.preventDefault();
          event.stopPropagation();
          event.target.checked = true;
          say(engineName("acp", "detail") + " \u00B7 " +
            t("chat-vm-agent-locked-note",
              "runs in the Virtual Machine, so the Agent switch stays on while it is selected. Choose another model to turn it off."));
        }, true);
      }
      if (typeof window.hydrateModelAccounts === "function") window.hydrateModelAccounts();
      if (typeof window.hydrateModelAgents === "function") window.hydrateModelAgents();
      hydrateModels();
      sync();
      return P;
    },
    sync: sync,
    accounts: accountsFromPicker,
    accountForPreference: function (pref) {
      if (!pref) return null;
      return accountsFromPicker().filter(function (account) {
        return account.models.some(function (model) { return model.value === pref; });
      })[0] || null;
    },
    isSubscription: isSubscription,
    providerOf: providerOf,
    agentLabel: agentLabel
  };
})();
