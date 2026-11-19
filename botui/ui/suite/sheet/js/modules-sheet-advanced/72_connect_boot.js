"use strict";
/* Sheets data connectivity (#2492) - tab shell, toolbar wiring and public
 * facade. window.SheetConnect. Opening the dialog never contacts the server. */

(function () {
  var C = window.SheetConnectCore;
  var dialog = null;
  var opener = null;
  var activeTab = "connectors";

  var TABS = [
    { id: "connectors", key: "sheet-connect-tab-connectors", fallback: "Connectors", module: "SheetConnectConnectors" },
    { id: "query", key: "sheet-connect-tab-query", fallback: "Query steps", module: "SheetConnectQuery" },
    { id: "parameters", key: "sheet-connect-tab-parameters", fallback: "Parameters", module: "SheetConnectParams" },
    { id: "refresh", key: "sheet-connect-tab-refresh", fallback: "Refresh", module: "SheetConnectRefresh" },
    { id: "lineage", key: "sheet-connect-tab-lineage", fallback: "Lineage", module: "SheetConnectLineage" },
  ];

  var ctx = {
    state: function () { return C.state(); },
    setStatus: function (message, isError) {
      if (dialog) dialog.setStatus(message, Boolean(isError));
    },
    rerender: function () { rerender(); },
  };

  function ensureDialog() {
    if (dialog && dialog.root.isConnected !== false) return dialog;
    dialog = C.makeDialog();
    TABS.forEach(function (tab) {
      var button = C.element("button", "ss-connect-tab", C.t(tab.key, tab.fallback));
      button.type = "button";
      button.id = "ss-connect-tab-" + tab.id;
      button.dataset.tab = tab.id;
      button.setAttribute("role", "tab");
      button.setAttribute("aria-selected", tab.id === activeTab ? "true" : "false");
      button.tabIndex = tab.id === activeTab ? 0 : -1;
      button.setAttribute("data-i18n", tab.key);
      button.addEventListener("click", function () { switchTab(tab.id); });
      button.addEventListener("keydown", function (event) {
        if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") return;
        var index = TABS.map(function (item) { return item.id; }).indexOf(tab.id);
        var step = event.key === "ArrowRight" ? 1 : TABS.length - 1;
        event.preventDefault();
        switchTab(TABS[(index + step) % TABS.length].id);
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
    var current = dialog.tabs.querySelector("#ss-connect-tab-" + id);
    if (current && current.focus) current.focus({ preventScroll: true });
  }

  function open(trigger) {
    var built = ensureDialog();
    opener = trigger || document.getElementById("sheetConnectBtn") || null;
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
      if (key) node.textContent = C.t(key, node.textContent);
    });
    var labels = dialog.root.querySelectorAll("[data-i18n-aria-label]");
    Array.prototype.forEach.call(labels, function (node) {
      var key = node.getAttribute("data-i18n-aria-label");
      if (key) node.setAttribute("aria-label", C.t(key, node.getAttribute("aria-label")));
    });
    var placeholders = dialog.root.querySelectorAll("[data-i18n-placeholder]");
    Array.prototype.forEach.call(placeholders, function (node) {
      var key = node.getAttribute("data-i18n-placeholder");
      if (key) node.setAttribute("placeholder", C.t(key, node.getAttribute("placeholder")));
    });
    rerender();
  }

  function wireButton() {
    var button = document.getElementById("sheetConnectBtn");
    if (!button || button.dataset.connectWired === "true") return;
    button.dataset.connectWired = "true";
    button.addEventListener("click", function (event) {
      toggle(event.currentTarget || button);
    });
  }

  function boot() {
    wireButton();
    window.addEventListener("gb-sheet-locale", relocalize);
  }

  window.SheetConnect = {
    open: open, close: close, toggle: toggle, switchTab: switchTab,
    rerender: rerender, relocalize: relocalize, tabs: TABS,
    activeTab: function () { return activeTab; },
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
})();
