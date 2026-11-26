"use strict";
/* Docs language tools — accessible dialog shell (#2484).
 * role=dialog + aria-modal, tablist/tabpanel, focus trap, Esc, focus restore
 * and a polite live status region. */

(function () {
  var NS = window.DocsLang;
  if (!NS) return;

  var TABS = [
    ["spelling", "docs-lang-tab-spelling", "Spelling"],
    ["style", "docs-lang-tab-style", "Grammar and style"],
    ["thesaurus", "docs-lang-tab-thesaurus", "Thesaurus"],
    ["glossary", "docs-lang-tab-glossary", "Glossary"],
    ["language", "docs-lang-tab-language", "Language and privacy"],
    ["report", "docs-lang-tab-report", "Report"]
  ];

  function host() {
    return document.getElementById("docs-modal-container");
  }
  NS.dialogHost = host;

  function tabButtons() {
    return TABS.map(function (tab) {
      var active = NS.state.activeTab === tab[0];
      return '<button type="button" class="docs-lang-tab" role="tab" data-panel="' + tab[0] + '"' +
        ' aria-selected="' + (active ? "true" : "false") + '"' +
        (active ? ' tabindex="0"' : ' tabindex="-1"') + '>' +
        NS.escapeHtml(NS.tr(tab[1], tab[2])) + "</button>";
    }).join("");
  }

  function panels() {
    return TABS.map(function (tab) {
      var active = NS.state.activeTab === tab[0];
      return '<div class="docs-lang-panel" role="tabpanel" data-panel="' + tab[0] + '"' +
        (active ? "" : " hidden") + "></div>";
    }).join("");
  }

  function trapFocus(event) {
    var dialog = document.querySelector(".docs-lang-dialog");
    if (!dialog) return;
    var focusables = Array.prototype.slice.call(dialog.querySelectorAll(
      'button,input,select,textarea,[href],[tabindex]:not([tabindex="-1"])'
    )).filter(function (node) { return !node.disabled && node.offsetParent !== null; });
    if (!focusables.length) return;
    var first = focusables[0];
    var last = focusables[focusables.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function keyHandler(event) {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    } else if (event.key === "Tab") {
      trapFocus(event);
    }
  }

  function switchTab(name) {
    if (!name) return;
    NS.state.activeTab = name;
    var root = document.querySelector(".docs-lang-dialog");
    if (!root) return;
    Array.prototype.slice.call(root.querySelectorAll('[role="tab"]')).forEach(function (button) {
      var active = button.getAttribute("data-panel") === name;
      button.setAttribute("aria-selected", active ? "true" : "false");
      button.setAttribute("tabindex", active ? "0" : "-1");
    });
    Array.prototype.slice.call(root.querySelectorAll('[role="tabpanel"]')).forEach(function (panel) {
      panel.hidden = panel.getAttribute("data-panel") !== name;
    });
    NS.rerender();
  }
  NS.switchTab = switchTab;

  function bind(root) {
    var closeBtn = root.querySelector("#docs-lang-close");
    if (closeBtn) closeBtn.addEventListener("click", close);
    var undoBtn = root.querySelector("#docs-lang-undo");
    if (undoBtn) undoBtn.addEventListener("click", function () {
      NS.undo();
      NS.refresh();
      NS.rerender();
    });
    Array.prototype.slice.call(root.querySelectorAll('[role="tab"]')).forEach(function (button) {
      button.addEventListener("click", function () { switchTab(button.getAttribute("data-panel")); });
      button.addEventListener("keydown", function (event) {
        if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") return;
        event.preventDefault();
        var index = TABS.map(function (t) { return t[0]; }).indexOf(NS.state.activeTab);
        var next = event.key === "ArrowRight" ? index + 1 : index - 1;
        if (next < 0) next = TABS.length - 1;
        if (next >= TABS.length) next = 0;
        switchTab(TABS[next][0]);
        var target = root.querySelector('[role="tab"][data-panel="' + TABS[next][0] + '"]');
        if (target) target.focus();
      });
    });
  }

  function close() {
    var node = host();
    if (node) node.innerHTML = "";
    if (NS.state.keyHandler) {
      document.removeEventListener("keydown", NS.state.keyHandler, true);
      NS.state.keyHandler = null;
    }
    var last = NS.state.lastFocus;
    if (last && typeof last.focus === "function") last.focus();
    NS.state.lastFocus = null;
  }
  NS.close = close;

  function open() {
    var container = host();
    if (!container) return;
    NS.loadState();
    NS.state.lastFocus = document.activeElement;
    NS.refresh();
    NS.refreshReport();
    container.innerHTML = [
      '<div class="docs-modal docs-lang-backdrop" style="position:fixed;inset:0;background:color-mix(in srgb, var(--palette-0f172a) 85%, transparent);display:flex;align-items:center;justify-content:center;z-index:var(--z-modal-backdrop);">',
      '<div class="docs-lang-dialog" role="dialog" aria-modal="true" aria-labelledby="docs-lang-heading" style="background:var(--palette-1e293b);border:1px solid #334155;border-radius:12px;width:760px;max-width:94vw;max-height:86vh;padding:18px;color:var(--palette-f8fafc);display:flex;flex-direction:column;gap:12px;">',
      '<div class="docs-lang-head" style="display:flex;align-items:center;justify-content:space-between;gap:12px;">',
      '<h3 id="docs-lang-heading" style="margin:0;font-size:16px;">' + NS.escapeHtml(NS.tr("docs-lang-title", "Language and writing tools")) + "</h3>",
      '<div style="display:flex;gap:8px;">',
      '<button type="button" id="docs-lang-undo" class="docs-lang-btn">' + NS.escapeHtml(NS.tr("docs-lang-undo", "Undo last change")) + "</button>",
      '<button type="button" id="docs-lang-close" class="docs-lang-btn" aria-label="' + NS.escapeHtml(NS.tr("docs-lang-close", "Close")) + '">&times;</button>',
      "</div></div>",
      '<div class="docs-lang-tabs" role="tablist" aria-label="' + NS.escapeHtml(NS.tr("docs-lang-title", "Language and writing tools")) + '">' + tabButtons() + "</div>",
      '<div class="docs-lang-panels">' + panels() + "</div>",
      '<div id="docs-lang-status" role="status" aria-live="polite" class="docs-lang-status"></div>',
      "</div></div>"
    ].join("");
    bind(container);
    container.addEventListener("click", function (event) {
      if (event.target && event.target.classList && event.target.classList.contains("docs-lang-backdrop")) close();
    });
    NS.state.keyHandler = keyHandler;
    document.addEventListener("keydown", keyHandler, true);
    var first = container.querySelector('[role="tab"][aria-selected="true"]');
    if (first) first.focus();
    NS.rerender();
  }
  NS.open = open;

  function rerender() {
    var root = document.querySelector(".docs-lang-dialog");
    if (!root) return;
    var active = NS.state.activeTab;
    var pane = root.querySelector('[role="tabpanel"][data-panel="' + active + '"]');
    if (!pane) return;
    var renderers = {
      spelling: NS.renderSpelling,
      style: NS.renderStyle,
      thesaurus: NS.renderThesaurus,
      glossary: NS.renderGlossary,
      language: NS.renderLanguage,
      report: NS.renderReport
    };
    var render = renderers[active];
    if (typeof render === "function") pane.innerHTML = render();
  }
  NS.rerender = rerender;
})();
