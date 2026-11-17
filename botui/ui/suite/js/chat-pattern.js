"use strict";
/**
 * Chat background pattern — shared by the chat app and Settings → Appearance.
 *
 * The choice lives in the single `gb-chat-pattern` key so both surfaces agree.
 * Nothing is drawn here: the module only publishes the choice as a data
 * attribute on <html>; chat-assistant-look.css owns the actual patterns, which
 * keeps the CSP free of inline styles.
 */

(function () {
  var KEY = "gb-chat-pattern";
  var IDS = ["none", "image", "dots", "microdots", "grid", "blueprint", "weave", "mosaic"];

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function label(id) {
    var names = {
      none: ["chatbg-none", "None"],
      image: ["chatbg-image", "Paper"],
      dots: ["chatbg-dots", "Dots"],
      microdots: ["chatbg-microdots", "Micro dots"],
      grid: ["chatbg-grid", "Grid"],
      blueprint: ["chatbg-blueprint", "Blueprint"],
      weave: ["chatbg-weave", "Weave"],
      mosaic: ["chatbg-mosaic", "Mosaic"]
    };
    var entry = names[id] || names.image;
    return t(entry[0], entry[1]);
  }

  function get() {
    try {
      var stored = localStorage.getItem(KEY);
      if (stored && IDS.indexOf(stored) !== -1) return stored;
    } catch (error) {
      /* private mode */
    }
    return "image";
  }

  function apply(id) {
    var chosen = IDS.indexOf(id) === -1 ? "image" : id;
    var root = document.documentElement;
    if (!root) return chosen;
    root.setAttribute("data-gb-chat-pattern", chosen);
    return chosen;
  }

  function set(id) {
    var chosen = apply(id);
    try {
      localStorage.setItem(KEY, chosen);
    } catch (error) {
      /* the choice still applies to this page */
    }
    window.dispatchEvent(new CustomEvent("gb-chat-pattern-changed", { detail: { id: chosen } }));
    return chosen;
  }

  function boot() {
    apply(get());
  }

  window.GBChatPattern = {
    IDS: IDS,
    KEY: KEY,
    get: get,
    set: set,
    apply: apply,
    label: label
  };

  window.addEventListener("storage", function (event) {
    if (event.key === KEY) apply(get());
  });

  boot();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  }
})();
