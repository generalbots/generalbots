"use strict";
/* Sheets print, reporting and scheduled delivery (#2438) — tab shell, toolbar
 * wiring and public facade. Every tab is local-only; boot never contacts the
 * server and never claims a real PDF, schedule, delivery or workbook save. */

(function () {
  var C = window.SheetReportingCore;
  var dialog = null;
  var opener = null;
  var activeTab = "print";

  var TABS = [
    { id: "print", key: "sheet-rpt-tab-print", fallback: "Print & setup", module: "SheetReportingPrint" },
    { id: "preview", key: "sheet-rpt-tab-preview", fallback: "PDF parity preview", module: "SheetReportingPreview" },
    { id: "dash", key: "sheet-rpt-tab-dash", fallback: "Dashboards", module: "SheetReportingDashboard" },
    { id: "sched", key: "sheet-rpt-tab-sched", fallback: "Scheduled reports", module: "SheetReportingSchedule" },
  ];

  var ctx = {
    setStatus: function (message, isError) {
      if (dialog) dialog.setStatus(message, Boolean(isError));
    },
    rerender: function () { rerender(); },
    refreshPreview: function () { refreshPreview(); },
  };

  function ensureDialog() {
    if (dialog && dialog.root.isConnected !== false) return dialog;
    dialog = C.makeDialog("sheet-rpt-title", "Print and reports - local preview");
    TABS.forEach(function (tab) {
      var button = C.element("button", "ss-rpt-tab", C.t(tab.key, tab.fallback));
      button.type = "button";
      button.id = "ss-rpt-tab-" + tab.id;
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

  function refreshPreview() {
    if (!dialog || activeTab !== "preview") return;
    rerender();
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
    var current = dialog.tabs.querySelector("#ss-rpt-tab-" + id);
    if (current && current.focus) current.focus({ preventScroll: true });
  }

  function open(trigger) {
    var built = ensureDialog();
    opener = trigger || document.getElementById("sheetReportingBtn") || null;
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
      node.textContent = C.t(key, node.textContent);
    });
    var placeholders = dialog.root.querySelectorAll("[data-i18n-placeholder]");
    Array.prototype.forEach.call(placeholders, function (node) {
      var key = node.getAttribute("data-i18n-placeholder");
      if (key) node.placeholder = C.t(key, node.placeholder);
    });
    rerender();
  }

  function wireButton() {
    var button = document.getElementById("sheetReportingBtn");
    if (!button || button.dataset.rptWired === "true") return;
    button.dataset.rptWired = "true";
    button.addEventListener("click", function (event) { toggle(event.currentTarget || button); });
  }

  function boot() {
    wireButton();
    window.addEventListener("gb-sheet-locale", relocalize);
  }

  window.SheetReporting = {
    open: open,
    close: close,
    toggle: toggle,
    switchTab: switchTab,
    relocalize: relocalize,
    rerender: rerender,
    refreshPreview: refreshPreview,
    tabs: TABS,
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
})();
