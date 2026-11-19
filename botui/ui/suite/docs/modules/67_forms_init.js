"use strict";
/* Forms/protection boot wiring (#2427): hydration and click delegation only. */
(function (window) {
  var NS = window.DocsForms;
  if (!NS || !NS.dialogs || !NS.protection) return;

  function byId(id) { return document.getElementById ? document.getElementById(id) : null; }

  function hydrate() {
    NS.loadState();
    NS.ensureStatus();
    NS.protection.applyBanner();
    var toggle = byId("docsFormsProtectBtn");
    if (toggle) toggle.setAttribute("aria-pressed", NS.protection.enabled() ? "true" : "false");
  }

  function boot() {
    hydrate();
    document.addEventListener("click", function (event) {
      var trigger = event.target && event.target.closest
        ? event.target.closest("[data-gb-forms-open]") : null;
      if (!trigger) return;
      event.preventDefault();
      event.stopPropagation();
      var action = trigger.getAttribute("data-gb-forms-open");
      if (action === "templates") NS.dialogs.openTemplates();
      else if (action === "controls") NS.dialogs.openControls();
      else if (action === "merge") NS.dialogs.openMerge();
      else if (action === "sign") NS.dialogs.openSign();
      else if (action === "protect") {
        NS.dialogs.openProtect();
        var toggle = byId("docsFormsProtectBtn");
        if (toggle) toggle.setAttribute("aria-pressed", NS.protection.enabled() ? "true" : "false");
      }
    });
    NS.protection.install();
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot);
  else boot();
})(window);
