"use strict";
/* Sheets ingest (#2494) — tab shell, toolbar wiring and public facade. Every
 * tab is local-only; boot never contacts the server or claims a real parse. */

(function () {
  var C = window.SheetIngestCore;
  var dialog = null;
  var opener = null;
  var activeTab = "preview";

  var TABS = [
    { id: "preview", key: "sheet-ingest-tab-preview", fallback: "Parsed grid", module: "SheetIngestPreview" },
    { id: "map", key: "sheet-ingest-tab-map", fallback: "Column mapping", module: "SheetIngestMap" },
    { id: "review", key: "sheet-ingest-tab-review", fallback: "Review and provenance", module: "SheetIngestReview" },
    { id: "batch", key: "sheet-ingest-tab-batch", fallback: "Batch", module: "SheetIngestBatch" },
    { id: "policy", key: "sheet-ingest-tab-policy", fallback: "Privacy and retention", module: "SheetIngestPolicy" },
  ];

  var ctx = {
    state: function () { return C.state(); },
    setStatus: function (message, isError) {
      if (dialog) dialog.setStatus(message, Boolean(isError));
    },
    rerender: function () { rerender(); },
    selectCell: function (fileId, row, col) {
      window.SheetIngestPreview.select(fileId, row, col);
      rerender();
    },
    clearSelection: function () { window.SheetIngestPreview.clear(); },
  };

  function ensureDialog() {
    if (dialog && dialog.root.isConnected !== false) return dialog;
    dialog = C.makeDialog();
    TABS.forEach(function (tab) {
      var button = C.element("button", "ss-ingest-tab", C.t(tab.key, tab.fallback));
      button.type = "button";
      button.id = "ss-ingest-tab-" + tab.id;
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
    var current = dialog.tabs.querySelector("#ss-ingest-tab-" + id);
    if (current && current.focus) current.focus({ preventScroll: true });
  }

  function open(trigger) {
    var built = ensureDialog();
    opener = trigger || document.getElementById("sheetIngestBtn") || null;
    C.mutate(function (target) {
      window.SheetIngestSim.ensureParse(target, target.activeFileId);
    });
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
    rerender();
  }

  function wireButton() {
    var button = document.getElementById("sheetIngestBtn");
    if (!button || button.dataset.ingestWired === "true") return;
    button.dataset.ingestWired = "true";
    button.addEventListener("click", function (event) {
      toggle(event.currentTarget || button);
    });
  }

  function boot() {
    wireButton();
    window.addEventListener("gb-sheet-locale", relocalize);
  }

  window.SheetIngest = {
    open: open,
    close: close,
    toggle: toggle,
    switchTab: switchTab,
    rerender: rerender,
    relocalize: relocalize,
    ctx: ctx,
    tabs: TABS,
    activeTab: function () { return activeTab; },
  };

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
})();
