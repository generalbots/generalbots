"use strict";
/* Sheet advanced module: 27_audit_panel — dialog shell for local-only formula
   auditing (#2491). The panel is created on demand and never mutates cells. */

(function () {
  function t(key, fallback, params) {
    if (window.SheetI18n && typeof window.SheetI18n.t === "function") {
      return window.SheetI18n.t(key, fallback, params);
    }
    if (typeof window.gbT === "function") return window.gbT(key, fallback, params);
    if (typeof window.tr === "function") return window.tr(key, fallback, params);
    return fallback;
  }

  var panel = null;
  var opener = null;
  var activeTab = "trace";
  var graph = null;
  var snapshot = null;
  var watch = [];
  var statusText = "";

  var TABS = [
    { id: "trace", key: "sheet-audit-tab-trace", fallback: "Trace" },
    { id: "evaluate", key: "sheet-audit-tab-evaluate", fallback: "Evaluate" },
    { id: "watch", key: "sheet-audit-tab-watch", fallback: "Watch" },
    { id: "errors", key: "sheet-audit-tab-errors", fallback: "Errors" },
    { id: "cycles", key: "sheet-audit-tab-cycles", fallback: "Circular refs" },
    { id: "text", key: "sheet-audit-tab-text", fallback: "Formula text" },
    { id: "map", key: "sheet-audit-tab-map", fallback: "Dependency map" },
  ];

  function activeWorkbook() {
    var D = window.SheetDefaults;
    var inst = D && D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    return (inst && inst.loadedSheet) || window.__LOADED_SHEET || null;
  }
  function activeIndex() {
    var D = window.SheetDefaults;
    var inst = D && D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    if (inst && typeof inst.worksheetIndex === "number") return inst.worksheetIndex;
    if (window.SheetCore && typeof window.SheetCore.wsIndex === "function") return window.SheetCore.wsIndex();
    return D ? D.WORKSHEET_INDEX : 0;
  }
  function activeHost() {
    var D = window.SheetDefaults;
    var inst = D && D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    if (inst && inst.host) return inst.host;
    return document.getElementById("sheet-content") || document.body;
  }

  function selection() {
    if (window.SheetAdvanced && typeof window.SheetAdvanced.getSelection === "function") {
      return window.SheetAdvanced.getSelection();
    }
    return null;
  }
  function selectionRef() {
    var sel = selection();
    if (!sel || !window.SheetAuditGraph) return "";
    var G = window.SheetAuditGraph;
    var sheets = graph ? graph.sheets : [];
    var idx = activeIndex();
    var ref = G.colName(sel.startCol) + (sel.startRow + 1);
    var ws = sheets[idx];
    var sheetPrefix = ws && ws.name ? ws.name + "!" : "";
    var full = G.refOf(idx, sel.startRow, sel.startCol, sheets);
    return full && sheetPrefix && full.indexOf(sheetPrefix) === 0 ? ref : (full || ref);
  }

  function setStatus(message) {
    statusText = message || t("sheet-audit-status-ready", "Ready.");
    if (!panel || !panel.isConnected) return;
    var node = panel.querySelector(".sheet-audit-status");
    if (node) node.textContent = statusText;
  }
  function status() {
    return statusText;
  }

  function refreshGraph() {
    if (!window.SheetAuditGraph) return null;
    snapshot = window.SheetAuditGraph.normalizeSnapshot(activeWorkbook(), activeIndex());
    graph = window.SheetAuditGraph.buildGraph(snapshot);
    return graph;
  }
  function getGraph() {
    if (!graph) refreshGraph();
    return graph;
  }
  function getSnapshot() {
    if (!snapshot) refreshGraph();
    return snapshot;
  }

  function element(tag, className, text) {
    var node = document.createElement(tag);
    if (className) node.className = className;
    if (text != null) node.textContent = text;
    return node;
  }

  function tabButton(def) {
    var button = element("button", "sheet-audit-tab", t(def.key, def.fallback));
    button.type = "button";
    button.id = "sheet-audit-tab-" + def.id;
    button.setAttribute("role", "tab");
    button.setAttribute("aria-controls", "sheet-audit-section-" + def.id);
    button.setAttribute("aria-selected", def.id === activeTab ? "true" : "false");
    button.tabIndex = def.id === activeTab ? 0 : -1;
    button.dataset.tab = def.id;
    button.addEventListener("click", function () { switchTab(def.id); });
    return button;
  }

  function trapFocus(event) {
    if (event.key === "Escape") {
      event.preventDefault();
      closePanel();
      return;
    }
    if (event.key !== "Tab" || !panel) return;
    var focusable = Array.prototype.slice.call(panel.querySelectorAll(
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
    )).filter(function (node) { return node.offsetParent !== null || node === document.activeElement; });
    if (!focusable.length) return;
    var first = focusable[0];
    var last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function ensurePanel() {
    if (panel && panel.isConnected) return panel;
    var host = activeHost();
    if (!host) return null;

    panel = element("section", "sheet-audit-dialog");
    panel.id = "sheet-audit-dialog";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");
    panel.setAttribute("aria-labelledby", "sheet-audit-title");
    panel.tabIndex = -1;
    panel.style.display = "none";

    var header = element("header", "sheet-audit-header");
    var title = element("h2", "sheet-audit-title", t("sheet-audit-title", "Formula auditing"));
    title.id = "sheet-audit-title";
    var badge = element("span", "sheet-audit-badge", t("sheet-audit-local-badge", "Local"));
    badge.title = t("sheet-audit-local-note",
      "Local-only analysis of in-memory cells; no server graph or evaluator parity is implied.");
    var close = element("button", "sheet-audit-close", "\u00d7");
    close.type = "button";
    close.setAttribute("aria-label", t("sheet-audit-close", "Close formula auditing"));
    close.addEventListener("click", closePanel);
    header.appendChild(title);
    header.appendChild(badge);
    header.appendChild(close);

    var note = element("p", "sheet-audit-local-note", t("sheet-audit-local-note",
      "Local-only analysis of in-memory cells; no server graph or evaluator parity is implied."));

    var tablist = element("div", "sheet-audit-tabs");
    tablist.setAttribute("role", "tablist");
    tablist.setAttribute("aria-label", t("sheet-audit-title", "Formula auditing"));
    TABS.forEach(function (def) { tablist.appendChild(tabButton(def)); });

    var statusNode = element("p", "sheet-audit-status", statusText);
    statusNode.setAttribute("role", "status");
    statusNode.setAttribute("aria-live", "polite");

    var body = element("div", "sheet-audit-body");
    body.id = "sheet-audit-body";
    TABS.forEach(function (def) {
      var section = element("section", "sheet-audit-section");
      section.id = "sheet-audit-section-" + def.id;
      section.setAttribute("role", "tabpanel");
      section.setAttribute("aria-labelledby", "sheet-audit-tab-" + def.id);
      section.hidden = def.id !== activeTab;
      body.appendChild(section);
    });

    panel.appendChild(header);
    panel.appendChild(note);
    panel.appendChild(tablist);
    panel.appendChild(statusNode);
    panel.appendChild(body);
    panel.addEventListener("keydown", trapFocus);
    host.appendChild(panel);

    if (window.SheetI18n && typeof window.SheetI18n.localize === "function") {
      window.SheetI18n.localize(panel);
    }
    return panel;
  }

  function switchTab(id) {
    activeTab = id;
    if (!panel || !panel.isConnected) return;
    Array.prototype.forEach.call(panel.querySelectorAll(".sheet-audit-tab"), function (button) {
      var selected = button.dataset.tab === id;
      button.setAttribute("aria-selected", selected ? "true" : "false");
      button.tabIndex = selected ? 0 : -1;
    });
    Array.prototype.forEach.call(panel.querySelectorAll(".sheet-audit-section"), function (section) {
      section.hidden = section.id !== "sheet-audit-section-" + id;
    });
    renderActive();
    var tab = panel.querySelector('.sheet-audit-tab[data-tab="' + id + '"]');
    if (tab) tab.focus();
  }
  function currentTab() {
    return activeTab;
  }
  function section(id) {
    var p = ensurePanel();
    return p ? p.querySelector("#sheet-audit-section-" + id) : null;
  }
  function renderActive() {
    var S = window.SheetAuditSections;
    if (!S || typeof S.render !== "function") return;
    refreshGraph();
    var working = t("sheet-audit-status-working", "Analysing local cells\u2026");
    setStatus(working);
    try {
      S.render(activeTab, {
        panel: panel,
        graph: graph,
        snapshot: snapshot,
        index: activeIndex(),
        selection: selection(),
        selectionRef: selectionRef(),
        watch: watch,
        t: t,
      });
      setStatus(statusText === working ? t("sheet-audit-status-ready", "Ready.") : statusText);
    } catch (error) {
      if (window.console && console.warn) console.warn("[sheet-audit] render failed", error);
      setStatus(t("sheet-audit-status-ready", "Ready."));
    }
  }
  function rerender() {
    if (panel && panel.isConnected && panel.style.display !== "none") renderActive();
  }

  function open(trigger) {
    var p = ensurePanel();
    if (!p) return null;
    opener = trigger || (document.getElementById("sheetAuditBtn") || null);
    p.style.display = "block";
    refreshGraph();
    renderActive();
    p.focus({ preventScroll: true });
    return p;
  }
  function closePanel() {
    if (!panel || !panel.isConnected) return;
    panel.style.display = "none";
    if (window.SheetAuditOverlay && typeof window.SheetAuditOverlay.clearArrows === "function") {
      window.SheetAuditOverlay.clearArrows();
    }
    if (opener && opener.isConnected && typeof opener.focus === "function") {
      opener.focus({ preventScroll: true });
    }
  }
  function toggle(trigger) {
    if (panel && panel.isConnected && panel.style.display !== "none") closePanel();
    else open(trigger);
  }

  window.SheetAuditT = t;
  window.SheetAuditUI = {
    t: t,
    open: open,
    close: closePanel,
    toggle: toggle,
    switchTab: switchTab,
    currentTab: currentTab,
    section: section,
    setStatus: setStatus,
    status: status,
    refreshGraph: refreshGraph,
    getGraph: getGraph,
    getSnapshot: getSnapshot,
    activeIndex: activeIndex,
    activeWorkbook: activeWorkbook,
    selectionRef: selectionRef,
    rerender: rerender,
    getWatch: function () { return watch; },
    setWatch: function (next) { watch = Array.isArray(next) ? next : []; },
    element: element,
  };
})();
