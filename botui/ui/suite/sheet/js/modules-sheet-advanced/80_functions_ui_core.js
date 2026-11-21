"use strict";
/* Issue #2434 - shared dialog shell for the function wizard, calculation
 * settings and self-test. Focus-trapped, Esc closes, focus is restored. */
(function () {
  var TABS = [
    { id: "functions", key: "sheet-fn-tab-functions", fallback: "Functions", module: "SheetFunctionsWizard" },
    { id: "settings", key: "sheet-fn-tab-settings", fallback: "Calculation settings", module: "SheetFunctionsSettingsUI" },
    { id: "tests", key: "sheet-fn-tab-tests", fallback: "Self-test", module: "SheetFunctionsSelfTest" },
  ];

  var dialog = null;
  var opener = null;
  var activeTab = "functions";

  function t(key, fallback) {
    if (window.gbT) return window.gbT(key, fallback);
    if (window.SheetI18n && window.SheetI18n.t) return window.SheetI18n.t(key, fallback);
    return fallback;
  }

  function element(tag, className, value) {
    var node = document.createElement(tag);
    if (className) node.className = className;
    if (value !== undefined && value !== null) node.textContent = String(value);
    return node;
  }

  function focusable(root) {
    if (!root || typeof root.querySelectorAll !== "function") return [];
    return Array.prototype.slice.call(root.querySelectorAll(
      'button:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ));
  }

  function setStatus(message, isError) {
    if (!dialog) return;
    dialog.status.textContent = message;
    dialog.status.classList.toggle("ss-fn-status-error", Boolean(isError));
  }

  function makeDialog() {
    var restore = document.activeElement;
    var root = element("div", "ss-fn-dialog");
    root.setAttribute("role", "dialog");
    root.setAttribute("aria-modal", "true");
    root.tabIndex = -1;
    var heading = element("h2", "", t("sheet-fn-title", "Functions, calculation settings and self-test - local preview"));
    heading.setAttribute("data-i18n", "sheet-fn-title");
    heading.id = "ss-fn-title";
    root.setAttribute("aria-labelledby", heading.id);
    var note = element("p", "ss-fn-local-note", t("sheet-fn-local-note",
      "Local only - this browser preview does not change the Rust engine or save settings to the workbook."));
    note.setAttribute("data-i18n", "sheet-fn-local-note");
    var header = element("div", "ss-fn-header");
    var close = element("button", "ss-fn-close", "\u00d7");
    close.type = "button";
    close.setAttribute("aria-label", t("sheet-fn-close", "Close"));
    close.setAttribute("data-i18n-aria-label", "sheet-fn-close");
    header.appendChild(heading);
    header.appendChild(close);
    var tabs = element("div", "ss-fn-tabs");
    tabs.setAttribute("role", "tablist");
    var status = element("p", "ss-fn-status", t("sheet-fn-status-ready", "Ready."));
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.setAttribute("data-i18n", "sheet-fn-status-ready");
    var body = element("div", "ss-fn-body");
    root.appendChild(note);
    root.appendChild(header);
    root.appendChild(tabs);
    root.appendChild(status);
    root.appendChild(body);

    function onKey(event) {
      if (event.key === "Escape") {
        event.preventDefault();
        api.close();
        return;
      }
      if (event.key !== "Tab") return;
      var items = focusable(root);
      if (!items.length) return;
      var first = items[0];
      var last = items[items.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }

    function onDown(event) {
      if (root.contains && !root.contains(event.target)) api.close();
    }

    var api = {
      root: root, tabs: tabs, body: body, status: status,
      close: function () {
        if (root.isConnected === false) return;
        document.removeEventListener("keydown", onKey, true);
        document.removeEventListener("pointerdown", onDown, true);
        if (root.remove) root.remove();
        dialog = null;
        if (restore && restore.isConnected !== false && typeof restore.focus === "function") {
          restore.focus({ preventScroll: true });
        }
      },
      setStatus: setStatus,
    };
    TABS.forEach(function (tab) {
      var button = element("button", "ss-fn-tab", t(tab.key, tab.fallback));
      button.type = "button";
      button.id = "ss-fn-tab-" + tab.id;
      button.dataset.tab = tab.id;
      button.setAttribute("role", "tab");
      button.setAttribute("data-i18n", tab.key);
      button.addEventListener("click", function () { switchTab(tab.id); });
      button.addEventListener("keydown", function (event) {
        if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") return;
        var index = TABS.map(function (item) { return item.id; }).indexOf(tab.id);
        var step = event.key === "ArrowRight" ? 1 : TABS.length - 1;
        event.preventDefault();
        switchTab(TABS[(index + step) % TABS.length].id);
      });
      tabs.appendChild(button);
    });
    close.addEventListener("click", api.close);
    document.addEventListener("keydown", onKey, true);
    document.addEventListener("pointerdown", onDown, true);
    var host = document.getElementById("sheet-modal-container") || document.body;
    if (host && host.appendChild) host.appendChild(root);
    return api;
  }

  function rerender() {
    if (!dialog) return;
    dialog.body.textContent = "";
    var tab = TABS.filter(function (item) { return item.id === activeTab; })[0] || TABS[0];
    var module = window[tab.module];
    if (module && typeof module.render === "function") {
      module.render(dialog.body, {
        setStatus: setStatus,
        rerender: rerender,
        activeTab: function () { return activeTab; },
      });
    }
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
    var current = dialog.tabs.querySelector("#ss-fn-tab-" + id);
    if (current && current.focus) current.focus({ preventScroll: true });
  }

  function open(trigger, tab) {
    if (!dialog) dialog = makeDialog();
    opener = trigger || document.getElementById("sheetFunctionsWizardBtn") || null;
    dialog.root.style.display = "block";
    switchTab(tab || "functions");
    dialog.root.focus({ preventScroll: true });
    return dialog.root;
  }

  function close() {
    if (dialog) dialog.close();
    if (opener && opener.isConnected !== false && opener.focus) opener.focus({ preventScroll: true });
  }

  function toggle(trigger, tab) {
    if (dialog && dialog.root.style.display !== "none") close();
    else open(trigger, tab);
  }

  function relocalize() {
    if (!dialog) return;
    Array.prototype.forEach.call(dialog.root.querySelectorAll("[data-i18n]"), function (node) {
      var key = node.getAttribute("data-i18n");
      if (key) node.textContent = t(key, node.textContent);
    });
    Array.prototype.forEach.call(dialog.root.querySelectorAll("[data-i18n-aria-label]"), function (node) {
      var key = node.getAttribute("data-i18n-aria-label");
      if (key) node.setAttribute("aria-label", t(key, node.getAttribute("aria-label")));
    });
    rerender();
  }

  window.SheetFunctionsUI = {
    TABS: TABS, t: t, element: element, setStatus: setStatus,
    open: open, close: close, toggle: toggle, switchTab: switchTab,
    rerender: rerender, relocalize: relocalize,
    activeTab: function () { return activeTab; },
  };
})();
