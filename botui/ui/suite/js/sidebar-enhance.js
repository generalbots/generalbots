"use strict";

// GB Sidebar Enhancements (#1161) — coordinator + shared helpers.
//
// Renders extra sections into the v17 sidebar rail around the elements
// sidebar.js produces. Concrete renderers live in sidebar-sections.js
// (pins/workspaces/files/actions/user card) and sidebar-history.js
// (grouped + searchable conversations). This file owns the shared state,
// helpers, and boot sequence.

(function () {
  if (window.GBSidebarEnhance) return;

  var PINS_KEY = "gb_pinned_apps";
  var DEFAULT_PINS = ["chat", "browser", "terminal", "drive"];

  var state = {
    pinned: readPins(),
    bots: [],
  };

  // ── Shared helpers (exposed as window.GBSidebarBase) ─────────

  function readPins() {
    try {
      var parsed = JSON.parse(localStorage.getItem(PINS_KEY) || "[]");
      if (Array.isArray(parsed)) return parsed.filter(Boolean).slice(0, 16);
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

  function activeBotName() {
    if (window.GBResolveActiveBot) return window.GBResolveActiveBot();
    return (
      window.__SELECTED_BOT_NAME__ ||
      window.__INITIAL_BOT_NAME__ ||
      "default"
    );
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
    authToken: authToken,
    authHeaders: authHeaders,
    activeBotName: activeBotName,
    section: section,
  };

  // ── Workspace data ───────────────────────────────────────────

  function fetchBots() {
    return fetch("/api/cloud/bots", { headers: authHeaders() })
      .then(function (r) {
        return r.ok ? r.json() : Promise.reject(r.status);
      })
      .then(function (data) {
        return (data.bots || []).map(function (b) {
          return { name: b.name, label: b.description || b.name };
        });
      })
      .catch(function () {
        return fetch("/api/bots/list", { headers: authHeaders() })
          .then(function (r) {
            return r.ok ? r.json() : [];
          })
          .then(function (list) {
            return (Array.isArray(list) ? list : []).map(function (b) {
              return { name: b.name, label: b.name };
            });
          })
          .catch(function () {
            return [];
          });
      });
  }

  // ── Boot ─────────────────────────────────────────────────────

  function insertSections(content) {
    var nav = byId("sidebarAppsNav");
    var sections = [
      section("Pinned", "gbSidePins"),
      section("Workspaces", "gbSideWorkspaces"),
      section("Quick Files", "gbQuickFiles"),
      section("Quick actions", "gbQuickActions"),
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

  function boot() {
    var content = document.querySelector(".chat-sidebar-content");
    if (!content) return;
    if (!byId("gbSidePins")) insertSections(content);

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

    if (!state.bots.length) {
      fetchBots().then(function (bots) {
        state.bots = bots;
        if (window.GBSidebarSections) {
          window.GBSidebarSections.renderWorkspaces();
        }
      });
    } else if (window.GBSidebarSections) {
      window.GBSidebarSections.renderWorkspaces();
    }

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
    refreshWorkspaces: function () {
      if (window.GBSidebarSections) window.GBSidebarSections.renderWorkspaces();
    },
  };
})();