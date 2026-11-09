"use strict";
/* #1593 — per-conversation web-search switch in the chat composer.
 *
 * The button carries the full state: aria-pressed, a visible label and a
 * tooltip. The choice is stored per bot and user so a reload keeps it, and the
 * chat payload sends `web_search` with every message; when it is false the
 * server skips the whole search pipeline. Default is on. */
(function () {
  var STORAGE_PREFIX = "gb-web-search:v2:";
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
      // Off by default: no stored choice means the user never turned it on.
      // The early `true` returns made a fresh install start with web on, so
      // every message went through the search pipeline.
      if (raw === null) return false;
      return raw === "true";
    } catch (e) {
      return false;
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
      ? tr("chat-web-on", "Web search: on — answers can use live sources. Click to turn off.")
      : tr("chat-web-off", "Web search: off — answers use no live sources. Click to turn on.");
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

  // #1897 — a flip must be visible even when the module runs inside the
  // desktop shell, where the composer has no room for a wide label. Confirm
  // the new state through the shared toast stack as well.
  function confirmState() {
    if (window.GBToasts && typeof GBToasts.show === "function") {
      GBToasts.show(
        tr("chat-web-title", "Web search"),
        enabled
          ? tr("chat-web-on", "Web search: on — answers can use live sources.")
          : tr("chat-web-off", "Web search: off — answers use no live sources."),
        "info"
      );
    }
  }

  function setEnabled(next, quiet) {
    enabled = next === true;
    persist(enabled);
    paint();
    if (quiet !== true) confirmState();
    window.dispatchEvent(new CustomEvent("gb-web-search-changed", {
      detail: { enabled: enabled },
    }));
  }

  function toggle() {
    setEnabled(!enabled);
  }

  // #1897 — the composer is injected by the chat partial and re-rendered on
  // app switches, so element-level binding left later buttons dead (the same
  // silent no-op as #1892). Delegate once at document level.
  function onDocumentClick(event) {
    var target = event.target;
    if (!target || typeof target.closest !== "function") return;
    if (!target.closest("#webSearchBtn")) return;
    event.preventDefault();
    toggle();
  }

  function bind() {
    if (!document.documentElement.dataset.gbWebSearchBound) {
      document.documentElement.dataset.gbWebSearchBound = "1";
      document.addEventListener("click", onDocumentClick, false);
    }
    paint();
  }

  window.addEventListener("gb-chat-ready", bind);

  enabled = read();
  window.ChatWebSearch = {
    isEnabled: function () { return enabled; },
    setEnabled: function (next) { setEnabled(next, true); },
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
