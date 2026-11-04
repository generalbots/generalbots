"use strict";

// GB Sidebar Enhancements (#1161) — coordinator + shared helpers.
//
// Renders extra sections into the v17 sidebar rail around the elements
// sidebar.js produces. Concrete renderers live in sidebar-sections.js
// (pins/files/actions/user card) and sidebar-history.js
// (grouped + searchable conversations). This file owns the shared state,
// helpers, and boot sequence.

(function () {
  if (window.GBSidebarEnhance) return;

  var PINS_KEY = "gb_pinned_apps";
  // #1303 — default pinned set: Chat, Explorer (Vibe removed from pins).
  var DEFAULT_PINS = ["chat", "drive"];

  var state = {
    pinned: readPins(),
  };

  // ── Shared helpers (exposed as window.GBSidebarBase) ─────────

  function tr(key, fallback) {
    if (window.i18n && typeof window.i18n.t === "function") {
      var value = window.i18n.t(key);
      return value === key ? fallback : value;
    }
    return fallback;
  }

  function readPins() {
    // An empty or missing store must fall back to the default pins —
    // returning "[]" here leaves the left-bar pin strip empty on first run.
    try {
      var raw = localStorage.getItem(PINS_KEY);
      if (raw) {
        var parsed = JSON.parse(raw);
        if (Array.isArray(parsed) && parsed.length) {
          return parsed.filter(Boolean).slice(0, 16);
        }
      }
    } catch (e) {}
    return DEFAULT_PINS.slice();
  }

  function savePins() {
    try {
      localStorage.setItem(PINS_KEY, JSON.stringify(state.pinned));
    } catch (e) {}
  }

  function findApp(id) {
    var reg = window.APPS_REGISTRY || [];
    for (var i = 0; i < reg.length; i++) {
      if (reg[i].id === id) return reg[i];
    }
    return null;
  }

  function byId(id) {
    return document.getElementById(id);
  }

  function iconOf(app) {
    if (!app) return "";
    return (
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
      (app.icon || "") +
      "</svg>"
    );
  }

  function authToken() {
    var token = "";
    if (window.getGBAccessToken) token = window.getGBAccessToken() || "";
    if (!token) {
      try {
        token =
          localStorage.getItem("gb-access-token") ||
          sessionStorage.getItem("gb-access-token") ||
          localStorage.getItem("management_token") ||
          "";
      } catch (e) {}
    }
    return token;
  }

  function authHeaders() {
    var token = authToken();
    return token ? { Authorization: "Bearer " + token } : {};
  }

  function section(title, bodyId) {
    var host = document.createElement("div");
    host.className = "gb-side-sec";
    host.innerHTML =
      '<div class="chat-sidebar-section-label gb-side-sec-title">' +
      title +
      "</div>" +
      '<div class="gb-side-sec-body" id="' +
      bodyId +
      '"></div>';
    return host;
  }

  window.GBSidebarBase = {
    state: state,
    PINS_KEY: PINS_KEY,
    DEFAULT_PINS: DEFAULT_PINS,
    readPins: readPins,
    savePins: savePins,
    findApp: findApp,
    byId: byId,
    getById: byId,
    iconOf: iconOf,
    tr: tr,
    authToken: authToken,
    authHeaders: authHeaders,
    section: section,
  };

  // ── Boot ─────────────────────────────────────────────────────

  function insertSections(content) {
    var nav = byId("sidebarAppsNav");
    var sections = [
      section(tr("sidebar-pinned", "Pinned"), "gbSidePins"),
      section(tr("sidebar-quick-files", "Quick Files"), "gbQuickFiles"),
      section(tr("sidebar-quick-actions", "Quick actions"), "gbQuickActions"),
    ];
    if (nav && nav.parentNode) {
      sections.forEach(function (sec) {
        nav.parentNode.insertBefore(sec, nav.nextSibling);
      });
    } else {
      sections.forEach(function (sec) {
        content.appendChild(sec);
      });
    }
  }

  function refreshSectionLabels() {
    var labels = {
      gbSidePins: ["sidebar-pinned", "Pinned"],
      gbQuickFiles: ["sidebar-quick-files", "Quick Files"],
      gbQuickActions: ["sidebar-quick-actions", "Quick actions"],
    };
    Object.keys(labels).forEach(function (id) {
      var body = byId(id);
      if (!body || !body.parentElement) return;
      var label = body.parentElement.querySelector(".gb-side-sec-title");
      if (label) label.textContent = tr(labels[id][0], labels[id][1]);
    });
  }

  function boot() {
    var content = document.querySelector(".chat-sidebar-content");
    if (!content) return;
    if (!byId("gbSidePins")) insertSections(content);
    refreshSectionLabels();

    if (window.GBSidebarSections) {
      window.GBSidebarSections.renderPins();
      window.GBSidebarSections.bindPinReorder();
      window.GBSidebarSections.renderActions();
      window.GBSidebarSections.renderQuickFiles();
      window.GBSidebarSections.enhanceUserCard();
    }

    if (window.GBSidebarHistory) {
      window.GBSidebarHistory.install();
      document.addEventListener("gb-bot-changed", function () {
        window.GBSidebarHistory.resetForBot();
      });
    }

    // Translations may arrive after this boot; refresh the labels and the
    // generated chips when the catalogue becomes available or the user
    // switches language.
    window.addEventListener("localeChanged", function () {
      refreshSectionLabels();
      if (window.GBSidebarSections) {
        window.GBSidebarSections.renderPins();
        window.GBSidebarSections.renderActions();
        window.GBSidebarSections.enhanceUserCard();
      }
    });
    window.addEventListener("i18n:localeChanged", function () {
      refreshSectionLabels();
      if (window.GBSidebarSections) {
        window.GBSidebarSections.renderPins();
        window.GBSidebarSections.renderActions();
        window.GBSidebarSections.enhanceUserCard();
      }
    });

    // #1596/#1614 — the product config resolves after boot, so re-render the
    // app-derived surfaces once the real app set is known.
    document.addEventListener("gb-product-config", function () {
      if (window.GBSidebarSections) {
        window.GBSidebarSections.renderPins();
        window.GBSidebarSections.renderActions();
      }
    });

    // Keep pins fresh when the pinned launcher gains an app.
    window.addEventListener("gb-launcher-pin-request", function (event) {
      var detail = event && event.detail;
      if (detail && detail.kind === "app" && detail.appId) {
        if (state.pinned.indexOf(detail.appId) === -1) {
          state.pinned.push(detail.appId);
          savePins();
          if (window.GBSidebarSections) {
            window.GBSidebarSections.renderPins();
          }
        }
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }

  window.GBSidebarEnhance = {
    refreshPins: function () {
      if (window.GBSidebarSections) window.GBSidebarSections.renderPins();
    },
  };
})();