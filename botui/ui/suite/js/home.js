(function () {
  "use strict";

  var ICON_SVG = {
    doc: '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>',
    sheet:
      '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="3" y1="9" x2="21" y2="9"/><line x1="9" y1="3" x2="9" y2="21"/></svg>',
    slides:
      '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>',
    paper:
      '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>',
  };

  var KEYBOARD_SHORTCUTS = {
    1: "#chat",
    2: "#drive",
    3: "#tasks",
    4: "#mail",
    5: "#calendar",
    6: "#meet",
  };

  function getIconForType(type) {
    return ICON_SVG[type] || ICON_SVG.doc;
  }

  function escapeHtml(text) {
    var div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }

  function renderRecentDocuments(items) {
    if (!items || items.length === 0) {
      return;
    }

    var container = document.getElementById("recent-documents");
    if (!container) {
      return;
    }

    var html = items
      .slice(0, 4)
      .map(function (item) {
        var safeUrl = escapeHtml(item.url || "");
        var safeType = escapeHtml(item.type || "doc");
        var safeName = escapeHtml(item.name || "");
        var safeMeta = escapeHtml(item.meta || "");

        return (
          '<div class="recent-card" data-url="' +
          safeUrl +
          '">' +
          '<div class="recent-icon ' +
          safeType +
          '">' +
          getIconForType(item.type) +
          "</div>" +
          '<div class="recent-info">' +
          '<span class="recent-name">' +
          safeName +
          "</span>" +
          '<span class="recent-meta">' +
          safeMeta +
          "</span>" +
          "</div>" +
          "</div>"
        );
      })
      .join("");

    container.innerHTML = html;

    container.querySelectorAll(".recent-card").forEach(function (card) {
      card.addEventListener("click", function () {
        var url = this.getAttribute("data-url");
        if (url) {
          window.location.href = url;
        }
      });
    });
  }

  function loadRecentDocuments() {
    fetch("/api/activity/recent")
      .then(function (response) {
        if (!response.ok) {
          throw new Error("Failed to fetch recent documents");
        }
        return response.json();
      })
      .then(function (items) {
        renderRecentDocuments(items);
      })
      .catch(function () {
        console.log("Using placeholder recent documents");
      });
  }

  function setupHomeSearch() {
    var homeSearch = document.getElementById("home-search");
    if (homeSearch) {
      homeSearch.addEventListener("focus", function () {
        var omnibox = document.getElementById("omniboxInput");
        if (omnibox) {
          omnibox.focus();
        }
      });
    }
  }

  function setupKeyboardShortcuts() {
    document.addEventListener("keydown", function (e) {
      if (e.altKey && !e.ctrlKey && !e.shiftKey) {
        var target = KEYBOARD_SHORTCUTS[e.key];
        if (target) {
          e.preventDefault();
          var link = document.querySelector('a[href="' + target + '"]');
          if (link) {
            link.click();
          }
        }
      }
    });
  }

  function initHome() {
    loadRecentDocuments();
    setupHomeSearch();
  }

  function isHomeVisible() {
    return document.querySelector(".home-container") !== null;
  }

  setupKeyboardShortcuts();

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      if (isHomeVisible()) {
        initHome();
      }
    });
  } else {
    if (isHomeVisible()) {
      initHome();
    }
  }

  document.body.addEventListener("htmx:afterSwap", function (evt) {
    if (evt.detail.target && evt.detail.target.id === "main-content") {
      if (isHomeVisible()) {
        initHome();
      }
    }
  });
})();
