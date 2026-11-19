"use strict";
/* Sheets enterprise controls (#2437) — tab shell, toolbar wiring and public
 * facade. All modules are local-only previews; boot never contacts a server. */

(function () {
  var C = window.SheetControls;
  var dialog = null;
  var opener = null;
  var activeTab = "protection";

  var TABS = [
    { id: "protection", key: "sheet-ctrl-tab-protection", fallback: "Protection", module: "SheetControlsProtection" },
    { id: "inputs", key: "sheet-ctrl-tab-inputs", fallback: "Input ranges", module: "SheetControlsInputs" },
    { id: "acl", key: "sheet-ctrl-tab-acl", fallback: "Range access", module: "SheetControlsAcl" },
    { id: "classification", key: "sheet-ctrl-tab-classification", fallback: "Classification", module: "SheetControlsClassification" },
    { id: "links", key: "sheet-ctrl-tab-links", fallback: "External links", module: "SheetControlsLinks" },
    { id: "audit", key: "sheet-ctrl-tab-audit", fallback: "Audit trail", module: "SheetControlsAudit" },
    { id: "snapshot", key: "sheet-ctrl-tab-snapshot", fallback: "Snapshot", module: "SheetControlsSnapshot" },
  ];

  var ctx = {
    setStatus: function (message, isError) {
      if (dialog) dialog.setStatus(message, Boolean(isError));
    },
    rerender: function () { rerender(); },
  };

  function ensureDialog() {
    if (dialog && dialog.root.isConnected !== false) return dialog;
    dialog = C.makeDialog("sheet-ctrl-title", "Enterprise controls - local preview");
    TABS.forEach(function (tab) {
      var button = C.element("button", "ss-ctrl-tab", C.t(tab.key, tab.fallback));
      button.type = "button";
      button.id = "ss-ctrl-tab-" + tab.id;
      button.dataset.tab = tab.id;
      button.setAttribute("role", "tab");
      button.setAttribute("aria-selected", tab.id === activeTab ? "true" : "false");
      button.tabIndex = tab.id === activeTab ? 0 : -1;
      button.setAttribute("data-i18n", tab.key);
      button.addEventListener("click", function () { switchTab(tab.id); });
      button.addEventListener("keydown", function (event) {
        if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") return;
        var index = TABS.map(function (item) { return item.id; }).indexOf(tab.id);
        var next = (index + (event.key === "ArrowRight" ? 1 : TABS.length - 1)) % TABS.length;
        event.preventDefault();
        switchTab(TABS[next].id);
      });
      dialog.tabs.appendChild(button);
    });
    return dialog;
  }

  function rerender() {
    if (!dialog) return;
    dialog.body.textContent = "";
    var tab = TABS.filter(function (item) { return item.id === activeTab; })[0] || TABS[0];
    var module = window[tab.module];
    if (module && typeof module.render === "function") module.render(dialog.body, ctx);
  }

  function switchTab(id) {
    activeTab = id;
    if (!dialog) return;
    Array.prototype.forEach.call(dialog.tabs.children, function (button) {
      var selected = button.dataset.tab === id;
      button.setAttribute("aria-selected", selected ? "true" : "false");
      button.tabIndex = selected ? 0 : -1;
    });
    rerender();
    var current = dialog.tabs.querySelector("#ss-ctrl-tab-" + id);
    if (current && current.focus) current.focus();
  }

  function open(trigger) {
    var built = ensureDialog();
    opener = trigger || document.getElementById("sheetEnterpriseControlsBtn") || null;
    built.root.style.display = "block";
    switchTab(activeTab);
    if (built.root.focus) built.root.focus({ preventScroll: true });
    return built.root;
  }

  function close() {
    if (dialog) dialog.close();
    if (opener && opener.isConnected !== false && opener.focus) opener.focus({ preventScroll: true });
  }

  function toggle(trigger) {
    if (dialog && dialog.root.isConnected !== false && dialog.root.style.display !== "none") close();
    else open(trigger);
  }

  function relocalize() {
    if (!dialog) return;
    var nodes = dialog.root.querySelectorAll("[data-i18n]");
    Array.prototype.forEach.call(nodes, function (node) {
      var key = node.getAttribute("data-i18n");
      if (!key) return;
      var value = C.t(key, node.textContent);
      node.textContent = value;
    });
    rerender();
  }

  function wireButton() {
    var button = document.getElementById("sheetEnterpriseControlsBtn");
    if (!button || button.dataset.ctrlWired === "true") return;
    button.dataset.ctrlWired = "true";
    button.addEventListener("click", function (event) { toggle(event.currentTarget || button); });
  }

  function boot() {
    wireButton();
    if (window.SheetControlsProtection) window.SheetControlsProtection.applyGuards();
    if (window.SheetControlsClassification) window.SheetControlsClassification.paintBanner();
    if (window.SheetControlsInputs) window.SheetControlsInputs.highlight();
    if (window.SheetControlsLinks) window.SheetControlsLinks.refreshState();
    window.addEventListener("gb-sheet-locale", relocalize);
  }

  window.SheetEnterpriseControls = {
    open: open,
    close: close,
    toggle: toggle,
    switchTab: switchTab,
    relocalize: relocalize,
    rerender: rerender,
    tabs: TABS,
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
})();
