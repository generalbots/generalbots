"use strict";
/* #1593 — per-conversation web-search switch in the chat composer.
 *
 * The button carries the full state: aria-pressed, a visible label and a
 * tooltip. The choice is stored per bot and user so a reload keeps it, and the
 * chat payload sends `web_search` with every message; when it is false the
 * server skips the whole search pipeline. Default is on. */
(function () {
  var STORAGE_PREFIX = "gb-web-search:";
  // Off by default: the desktop assistant answers from the model and the
  // knowledge base unless the user explicitly asks for live sources.
  var enabled = false;

  function botName() {
    if (window.__INITIAL_BOT_NAME__) return String(window.__INITIAL_BOT_NAME__);
    if (window.ChatState && window.ChatState.currentBotName) return String(window.ChatState.currentBotName);
    return "bot";
  }

  function userId() {
    if (window.ChatState && window.ChatState.currentUserId) return String(window.ChatState.currentUserId);
    return "anon";
  }

  function storageKey() {
    return STORAGE_PREFIX + botName() + ":" + userId();
  }

  function read() {
    try {
      var raw = window.localStorage.getItem(storageKey());
      if (raw === null) return true;
      return raw === "true";
    } catch (e) {
      return true;
    }
  }

  function persist(value) {
    try {
      window.localStorage.setItem(storageKey(), value ? "true" : "false");
    } catch (e) {
      /* storage can be unavailable in a private webview; the button still works */
    }
  }

  function tr(key, fallback) {
    if (window.i18n && typeof window.i18n.t === "function") {
      var value = window.i18n.t(key);
      return value === key ? fallback : value;
    }
    return fallback;
  }

  function copy() {
    return enabled
      ? tr("chat-web-on-title", "Web search: on — answers can use live sources. Click to turn off.")
      : tr("chat-web-off-title", "Web search: off — answers use no live sources. Click to turn on.");
  }

  function paint() {
    var button = document.getElementById("webSearchBtn");
    if (!button) return;
    button.setAttribute("aria-pressed", enabled ? "true" : "false");
    button.classList.toggle("is-off", !enabled);
    button.title = copy();
    button.setAttribute("aria-label", copy());
    var label = button.querySelector(".web-search-label");
    if (label) label.textContent = enabled ? tr("chat-web-on-short", "Web on") : tr("chat-web-off-short", "Web off");
  }

  function setEnabled(next) {
    enabled = next === true;
    persist(enabled);
    paint();
    window.dispatchEvent(new CustomEvent("gb-web-search-changed", {
      detail: { enabled: enabled },
    }));
  }

  function toggle() {
    setEnabled(!enabled);
  }

  function bind() {
    var button = document.getElementById("webSearchBtn");
    if (button && !button.dataset.bound) {
      button.dataset.bound = "1";
      button.addEventListener("click", function (event) {
        event.preventDefault();
        toggle();
      });
    }
    paint();
  }

  enabled = read();
  window.ChatWebSearch = {
    isEnabled: function () { return enabled; },
    setEnabled: setEnabled,
    toggle: toggle,
    storageKey: storageKey,
    refresh: function () { enabled = read(); paint(); },
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bind);
  } else {
    bind();
  }
})();
