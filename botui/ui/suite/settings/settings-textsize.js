"use strict";

// Settings → Appearance → Text size (#2568). Same choice the assistant shell
// offers: it reads/writes the shared `gb-text-size` key and the same
// --id-text-* variables the canonical manager (suite/identity/textsize.js)
// applies, so a size picked in either shell carries over. The manager itself
// is not loaded here: /suite/identity/* is not served to the settings page
// (403), so this module owns the small amount of logic the control needs.
(function () {
  var KEY = "gb-text-size";
  var PX = { small: 14, default: 16, large: 18, xlarge: 20 };
  var IDS = ["small", "default", "large", "xlarge"];

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  function label(id) {
    if (id === "small") return t("ts-small", "Small");
    if (id === "large") return t("ts-large", "Large");
    if (id === "xlarge") return t("ts-xlarge", "Extra large");
    return t("ts-default", "Default");
  }

  function stored() {
    try {
      var value = localStorage.getItem(KEY);
      if (value && PX[value] !== undefined) return value;
    } catch (e) {
      /* private mode */
    }
    return "default";
  }

  // Must match applySize() in suite/identity/textsize.js.
  function applySize(name) {
    if (!PX[name]) name = "default";
    var px = PX[name];
    var root = document.documentElement.style;
    root.setProperty("--id-scale", String(px / 14));
    root.setProperty("--id-text-base", px + "px");
    root.setProperty("--id-text-md", px + 1 + "px");
    root.setProperty("--id-text-lg", px + 2 + "px");
    root.setProperty("--id-text-xl", px + 3 + "px");
    root.setProperty("--id-text-sm", Math.max(11, px - 2) + "px");
    root.setProperty("--id-text-xs", Math.max(10, px - 4) + "px");
  }

  function set(name) {
    if (!PX[name]) name = "default";
    try {
      localStorage.setItem(KEY, name);
    } catch (e) {
      /* private mode: the choice still applies to this page */
    }
    applySize(name);
  }

  function render() {
    var row = document.getElementById("settings-textsize-row");
    if (!row) return;
    var current = stored();
    var html = "";
    for (var i = 0; i < IDS.length; i++) {
      var id = IDS[i];
      var active = id === current;
      html += '<button type="button" role="radio" class="ts-btn' + (active ? " ts-btn-active" : "") +
        '" data-size="' + id + '" aria-checked="' + (active ? "true" : "false") + '">' +
        label(id) + "</button>";
    }
    row.innerHTML = html;
    var buttons = row.querySelectorAll(".ts-btn");
    for (var j = 0; j < buttons.length; j++) {
      buttons[j].addEventListener("click", function (event) {
        set(event.currentTarget.getAttribute("data-size"));
        render();
      });
    }
  }

  applySize(stored());

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", render);
  } else {
    render();
  }

  // The Appearance panel arrives through HTMX (01 → 02 → 03), after first paint.
  document.body.addEventListener("htmx:afterSwap", render);
  window.loadTextSizePanel = render;
})();
