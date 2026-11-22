/* Model tier picker: persistence, labels, account groups and honest disabled state (#1549/#1637). */
"use strict";

(function () {
  var MODEL_KEY = "gb-model-preference";
  var ACCOUNT_VALUE_PREFIX = "acct:";
  var AGENT_VALUE_PREFIX = "sub:";
  var AGENT_NAMES = {
    "claude-code": "Claude Code",
    "codex-cli": "Codex",
    "antigravity": "Antigravity",
    "chatgpt-plan": "ChatGPT"
  };

  function picker() {
    return document.getElementById("modelPicker");
  }

  function authHeaders() {
    if (window.GBShared && window.GBShared.auth) return window.GBShared.auth.headers();
    try {
      var token =
        (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
        (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
        sessionStorage.getItem("gb-access-token") ||
        "";
      return token ? { Authorization: "Bearer " + token } : {};
    } catch (e) {
      return {};
    }
  }

  function tr(key, fallback) {
    if (window.i18n && typeof window.i18n.t === "function") {
      var value = window.i18n.t(key);
      return value === key ? fallback : value;
    }
    return fallback;
  }

  function apply(select, tiers) {
    var models = {};
    tiers.forEach(function (tier) { if (tier.model) models[tier.model] = true; });
    // With a single configured model the tiers cannot trade cost against
    // quality; disable them and say so instead of offering a fake choice.
    var singleModel = Object.keys(models).length <= 1;
    select.dataset.singleModel = singleModel ? "1" : "0";
    select.title = singleModel
      ? "This bot uses one model; no separate cost/speed tier is configured."
      : "";
    tiers.forEach(function (tier) {
      var isAlt = tier.id !== "auto";
      var option = select.querySelector('option[value="' + tier.id + '"]');
      if (!option) return;
      // Never leak a raw model id into the picker: the test suite (TS-01)
      // requires plain-language cost/speed/fit labels. Prefer the localized
      // catalogue; the API label is the fallback for an unknown tier.
      var apiLabel = tier.label + (tier.tradeoff ? " · " + tier.tradeoff : "");
      option.textContent = tr("chat-model-" + tier.id, apiLabel);
      option.title = tier.inherits_default
        ? (tier.tradeoff || "") +
          " Uses the bot's default model until a tier-specific model is configured."
        : tier.tradeoff || "";
      option.disabled = isAlt && (tier.configured === false || singleModel);
      if (isAlt && singleModel) {
        option.title = "Same model as Auto — no separate tier model is configured.";
      }
      if (option.disabled && option.selected) select.value = "auto";
    });
  }

  function findOption(select, value) {
    for (var i = 0; i < select.options.length; i++) {
      if (select.options[i].value === value) return select.options[i];
    }
    return null;
  }

  function restoreSaved(select) {
    try {
      var saved = localStorage.getItem(MODEL_KEY);
      if (saved && findOption(select, saved)) select.value = saved;
    } catch (e) {
      /* storage may be disabled */
    }
  }

  function hydrate() {
    var select = picker();
    if (!select) return;
    var botName =
      window.__INITIAL_BOT_NAME__ ||
      (window.ChatState && window.ChatState.currentBotName) ||
      "default";
    var url = "/api/chat/models" + (botName ? "?bot=" + encodeURIComponent(botName) : "");
    fetch(url, { cache: "no-store", headers: authHeaders() })
      .then(function (response) { return response.ok ? response.json() : null; })
      .then(function (data) {
        if (data && Array.isArray(data.tiers)) apply(select, data.tiers);
      })
      .catch(function () {
        /* keep the static honest labels when the endpoint is unavailable */
      });
  }

  function shortName(model) {
    var value = String(model || "");
    if (value.indexOf(AGENT_VALUE_PREFIX) === 0) {
      var id = value.slice(AGENT_VALUE_PREFIX.length);
      return AGENT_NAMES[id] || id;
    }
    return value.replace(/\.gguf$/i, "");
  }

  function enabledModels(account) {
    var raw = account.enabled || account.models || account.model_ids || [];
    if (!Array.isArray(raw)) return [];
    return raw.map(function (item) {
      if (typeof item === "string") return item;
      if (!item || item.enabled !== true) return "";
      return item.model || item.id || item.name || "";
    }).filter(Boolean);
  }

  function selectAccount(select, value) {
    var option = findOption(select, value);
    if (!option) return;
    select.value = value;
    select.dispatchEvent(new Event("change", { bubbles: true }));
  }

  function clearAccounts(select) {
    select.querySelectorAll("optgroup.gb-model-account-group").forEach(function (group) {
      group.remove();
    });
    var parent = select.parentNode;
    if (!parent) return;
    parent.querySelectorAll(".gb-model-account-groups, .gb-model-accounts-toggle").forEach(function (node) {
      node.remove();
    });
  }

  function renderAccounts(select, accounts) {
    clearAccounts(select);
    if (!Array.isArray(accounts) || !accounts.length) {
      console.debug("[chat-model-picker] no provider accounts configured");
      return;
    }
    var panel = document.createElement("div");
    panel.className = "gb-model-account-groups";
    var toggle = document.createElement("button");
    toggle.type = "button";
    toggle.className = "gb-model-accounts-toggle";
    toggle.setAttribute("aria-expanded", "false");
    toggle.setAttribute("aria-label", tr("tour-chat-model-title", "Choose model"));
    toggle.title = tr("tour-chat-model-title", "Choose model");
    toggle.innerHTML = '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" aria-hidden="true"><polyline points="6 9 12 15 18 9"/></svg>';
    toggle.addEventListener("click", function () {
      var open = panel.classList.toggle("open");
      toggle.setAttribute("aria-expanded", String(open));
    });
    function choose(value) {
      selectAccount(select, value);
      panel.classList.remove("open");
      toggle.setAttribute("aria-expanded", "false");
    }
    var values = {};
    accounts.forEach(function (account) {
      var models = enabledModels(account);
      if (!models.length) return;
      var provider = account.provider || account.id || account.name || "";
      var label = account.name || account.provider || account.id || provider;
      if (!provider) return;
      var group = document.createElement("optgroup");
      group.className = "gb-model-account-group";
      group.label = label;
      var details = document.createElement("details");
      details.className = "gb-model-account-group";
      var summary = document.createElement("summary");
      summary.textContent = label;
      details.appendChild(summary);
      models.forEach(function (model) {
        var value = ACCOUNT_VALUE_PREFIX + provider + "/" + model;
        if (values[value]) return;
        values[value] = true;
        var option = document.createElement("option");
        option.value = value;
        option.textContent = shortName(model);
        group.appendChild(option);
        var button = document.createElement("button");
        button.type = "button";
        button.textContent = shortName(model);
        button.addEventListener("click", function () { choose(value); });
        details.appendChild(button);
      });
      if (group.options.length) {
        select.appendChild(group);
        panel.appendChild(details);
      }
    });
    if (!panel.childNodes.length) {
      console.debug("[chat-model-picker] no enabled provider models");
      return;
    }
    if (select.parentNode) {
      select.parentNode.insertBefore(toggle, select.nextSibling);
      select.parentNode.insertBefore(panel, toggle.nextSibling);
    }
    restoreSaved(select);
  }

  function hydrateAccounts() {
    var select = picker();
    if (!select) return;
    fetch("/api/llm/accounts", { cache: "no-store", headers: authHeaders() })
      .then(function (response) {
        if (response.status === 404) {
          console.debug("[chat-model-picker] provider accounts endpoint unavailable");
          return null;
        }
        return response.ok ? response.json() : null;
      })
      .then(function (data) {
        if (data && Array.isArray(data.accounts)) renderAccounts(select, data.accounts);
      })
      .catch(function (error) {
        console.debug("[chat-model-picker] provider accounts unavailable", error && error.message);
      });
  }

  /* Subscription agents connected in Settings > AI accounts: picking one
     answers the turn through its own agent VM (`sub:<provider>`), leaving
     the bot's default model untouched. Mirrors the assistant model menu. */
  function hydrateAgents() {
    var select = picker();
    if (!select) return;
    select.querySelectorAll("optgroup.gb-model-agent-group").forEach(function (node) {
      node.remove();
    });
    fetch("/api/ai-accounts", { cache: "no-store", headers: authHeaders() })
      .then(function (response) { return response.ok ? response.json() : null; })
      .then(function (data) {
        if (!data || !Array.isArray(data.accounts)) return;
        var agents = data.accounts.filter(function (account) {
          return account.status === "linked" && account.provider;
        });
        if (!agents.length) return;
        window.gbLinkedAgents = agents;
        var group = document.createElement("optgroup");
        group.className = "gb-model-agent-group";
        group.label = tr("chat-model-agents", "Agents");
        agents.forEach(function (account) {
          var option = document.createElement("option");
          option.value = AGENT_VALUE_PREFIX + account.provider;
          option.textContent = AGENT_NAMES[account.provider] || account.provider;
          group.appendChild(option);
        });
        select.appendChild(group);
        restoreSaved(select);
      })
      .catch(function (error) {
        console.debug("[chat-model-picker] linked agents unavailable", error && error.message);
      });
  }

  function restorePreference() {
    var select = picker();
    if (!select) return;
    restoreSaved(select);
    select.addEventListener("change", function () {
      try {
        localStorage.setItem(MODEL_KEY, select.value);
      } catch (e) {
        /* storage may be disabled */
      }
    });
  }

  function boot() {
    restorePreference();
    hydrate();
    hydrateAccounts();
    hydrateAgents();
    setTimeout(hydrate, 800);
    setTimeout(hydrate, 2500);
  }

  window.hydrateModelTiers = hydrate;
  window.hydrateModelAccounts = hydrateAccounts;
  window.hydrateModelAgents = hydrateAgents;
  window.addEventListener("gb:auth:login", function () {
    hydrate();
    hydrateAccounts();
    hydrateAgents();
  });
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
