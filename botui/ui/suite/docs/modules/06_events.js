"use strict";
/* docs events — sidebar tabs, HTMX afterSwap, auth */

document.addEventListener("click", function (e) {
  var tab = e.target.closest("[data-sidebar-tab]");
  if (tab) {
    var which = tab.dataset.sidebarTab;
    $$(".sidebar-tab").forEach(function (b) {
      b.classList.toggle("active", b === tab);
      b.style.background = b === tab ? "#1e293b" : TITLE_BG;
      b.style.color = b === tab ? TITLE_COLOR : "#94a3b8";
    });
    $$(".sidebar-content").forEach(function (c) {
      c.style.display = c.dataset.sidebarContent === which ? "flex" : "none";
    });
    try { sessionStorage.setItem(SIDEBAR_TAB_KEY, which); } catch (_) {}
  }
});

function initSidebar() {
  var saved = null;
  try { saved = sessionStorage.getItem(SIDEBAR_TAB_KEY); } catch (_) {}
  if (saved) {
    var btn = document.querySelector('[data-sidebar-tab="' + saved + '"]');
    if (btn) btn.click();
  }
}

function initAuth() {
  if (window.GBAuthGuard) GBAuthGuard.injectLoginButton(document.getElementById("gb-auth-button"));
}

document.addEventListener("htmx:afterSwap", function (e) {
  if (e.target.id === "docs-content") {
    attachEditorHandlers(e.target);
  }
});
