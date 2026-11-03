/* Model tier picker: persistence, labels and honest disabled state (#1549/#1637). */
"use strict";

(function () {
  var MODEL_KEY = "gb-model-preference";

  function picker() {
    return document.getElementById("modelPicker");
  }

  function authHeaders() {
    try {
      var token =
        (window.getGBAccessToken && window.getGBAccessToken()) ||
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        "";
      return token ? { Authorization: "Bearer " + token } : {};
    } catch (e) {
      return {};
    }
  }

  function apply(select, tiers) {
    tiers.forEach(function (tier) {
      var option = select.querySelector('option[value="' + tier.id + '"]');
      if (!option) return;
      option.textContent = tier.label + (tier.model ? " · " + tier.model : "");
      option.title = tier.tradeoff || "";
      option.disabled = tier.id !== "auto" && tier.configured === false;
      if (option.disabled && option.selected) select.value = "auto";
    });
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

  function restorePreference() {
    var select = picker();
    if (!select) return;
    try {
      var saved = localStorage.getItem(MODEL_KEY);
      if (saved) select.value = saved;
    } catch (e) {
      /* storage may be disabled */
    }
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
    setTimeout(hydrate, 800);
    setTimeout(hydrate, 2500);
  }

  window.hydrateModelTiers = hydrate;
  window.addEventListener("gb:auth:login", hydrate);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }
})();
