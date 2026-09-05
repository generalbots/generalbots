"use strict";

/**
 * Workspace tabs (#1168-fe) — model, rendering and persistence.
 * Tab shape: {id, kind: "chat"|"history"|"app", title, pinned, sessionId?, botId?, faviconGlyph?}
 * Local store: localStorage "gb.tabs.v1" as {tabs:[...], updated_at}.
 * Server mirror: GET/PUT /api/user/workspace/tabs (Bearer JWT), merged
 * last-writer-wins by the locally tracked updated_at field.
 * Feature flag OFF by default; activates on "+" click, history-in-new-tab,
 * ?tabs=1, or when a restored workspace holds more than the default tab.
 */

window.GBTabs = {
  STORAGE_KEY: "gb.tabs.v1",
  state: { enabled: false, tabs: [], activeTabId: null },
  unread: {},
  _persistTimer: null,
};

(function () {
  function token() {
    try {
      if (window.getGBAccessToken) {
        var t = window.getGBAccessToken();
        if (t) return t;
      }
      return (
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        localStorage.getItem("management_token") ||
        ""
      );
    } catch (e) {
      return "";
    }
  }

  function authHeaders(extra) {
    var h = extra || {};
    var t = token();
    if (t) h["Authorization"] = "Bearer " + t;
    return h;
  }

  GBTabs.token = token;
  GBTabs.authHeaders = authHeaders;

  GBTabs.newId = function () {
    return "tab-" + Date.now().toString(36) + "-" + Math.floor(Math.random() * 1e6).toString(36);
  };

  GBTabs.defaultTab = function () {
    var name =
      (window.ChatState && window.ChatState.currentBotName) ||
      window.__INITIAL_BOT_NAME__ || "Chat";
    return {
      id: "tab-default",
      kind: "chat",
      title: name.charAt(0).toUpperCase() + name.slice(1),
      pinned: true,
      faviconGlyph: "\u{1F4AC}",
    };
  };

  function readLocal() {
    try {
      var raw = localStorage.getItem(GBTabs.STORAGE_KEY);
      if (!raw) return null;
      var rec = JSON.parse(raw);
      if (!rec || !Array.isArray(rec.tabs) || !rec.tabs.length) return null;
      return rec;
    } catch (e) {
      return null;
    }
  }

  function writeLocal(tabs) {
    try {
      localStorage.setItem(
        GBTabs.STORAGE_KEY,
        JSON.stringify({ tabs: tabs, updated_at: new Date().toISOString() })
      );
    } catch (e) { /* storage unavailable */ }
  }

  GBTabs.readLocal = readLocal;

  GBTabs.isActive = function () {
    return GBTabs.state.enabled;
  };

  GBTabs.activeTab = function () {
    var tabs = GBTabs.state.tabs;
    for (var i = 0; i < tabs.length; i++) {
      if (tabs[i].id === GBTabs.state.activeTabId) return tabs[i];
    }
    return tabs.length ? tabs[0] : null;
  };

  GBTabs.activate = function () {
    if (GBTabs.state.enabled) return;
    GBTabs.state.enabled = true;
    if (!GBTabs.state.tabs.length) {
      GBTabs.state.tabs.push(GBTabs.defaultTab());
    }
    if (!GBTabs.activeTab()) {
      GBTabs.state.activeTabId = GBTabs.state.tabs[0].id;
    }
    GBTabs.renderStrip();
  };

  GBTabs.createTab = function (opts) {
    GBTabs.activate();
    var tab = {
      id: GBTabs.newId(),
      kind: opts.kind || "chat",
      title: opts.title || "New tab",
      pinned: !!opts.pinned,
    };
    if (opts.sessionId) tab.sessionId = opts.sessionId;
    if (opts.botId) tab.botId = opts.botId;
    if (opts.faviconGlyph) tab.faviconGlyph = opts.faviconGlyph;
    // Enforce the strip cap: drop the OLDEST unpinned tab when full.
    if (GBTabs.state.tabs.length >= 10) {
      var dropIdx = -1;
      for (var i = 0; i < GBTabs.state.tabs.length; i++) {
        if (!GBTabs.state.tabs[i].pinned) { dropIdx = i; break; }
      }
      if (dropIdx === -1) dropIdx = 1; // all pinned: drop the second (oldest non-default)
      var dropped = GBTabs.state.tabs.splice(dropIdx, 1)[0];
      delete GBTabs.unread[dropped.id];
    }
    GBTabs.state.tabs.push(tab);
    GBTabs.focusTab(tab.id);
    writeLocal(GBTabs.state.tabs);
    GBTabs.schedulePersist();
    return tab;
  };

  GBTabs.closeTab = function (id) {
    var tabs = GBTabs.state.tabs;
    if (tabs.length <= 1) return;
    var idx = -1;
    for (var i = 0; i < tabs.length; i++) {
      if (tabs[i].id === id) { idx = i; break; }
    }
    if (idx === -1) return;
    tabs.splice(idx, 1);
    delete GBTabs.unread[id];
    if (GBTabs.state.activeTabId === id) {
      var next = tabs[Math.min(idx, tabs.length - 1)];
      GBTabs.state.activeTabId = next.id;
      window.dispatchEvent(new CustomEvent("gb-tab-focused", { detail: { tab: next } }));
    }
    GBTabs.renderStrip();
    writeLocal(GBTabs.state.tabs);
    GBTabs.schedulePersist();
  };

  GBTabs.focusTab = function (id) {
    var tab = null;
    for (var i = 0; i < GBTabs.state.tabs.length; i++) {
      if (GBTabs.state.tabs[i].id === id) { tab = GBTabs.state.tabs[i]; break; }
    }
    if (!tab) return;
    GBTabs.state.activeTabId = id;
    delete GBTabs.unread[id];
    GBTabs.renderStrip();
    window.dispatchEvent(new CustomEvent("gb-tab-focused", { detail: { tab: tab } }));
  };

  GBTabs.renameTab = function (id, title) {
    var clean = String(title || "").trim();
    if (!clean) return;
    for (var i = 0; i < GBTabs.state.tabs.length; i++) {
      if (GBTabs.state.tabs[i].id === id) {
        GBTabs.state.tabs[i].title = clean.substring(0, 60);
        break;
      }
    }
    GBTabs.renderStrip();
    writeLocal(GBTabs.state.tabs);
    GBTabs.schedulePersist();
  };

  GBTabs.togglePin = function (id) {
    for (var i = 0; i < GBTabs.state.tabs.length; i++) {
      if (GBTabs.state.tabs[i].id === id) {
        GBTabs.state.tabs[i].pinned = !GBTabs.state.tabs[i].pinned;
        break;
      }
    }
    GBTabs.renderStrip();
    writeLocal(GBTabs.state.tabs);
    GBTabs.schedulePersist();
  };

  GBTabs.moveTab = function (fromIdx, toIdx) {
    var tabs = GBTabs.state.tabs;
    if (fromIdx === toIdx) return;
    if (fromIdx < 0 || fromIdx >= tabs.length) return;
    if (toIdx < 0 || toIdx >= tabs.length) return;
    var moved = tabs.splice(fromIdx, 1)[0];
    tabs.splice(toIdx, 0, moved);
    writeLocal(GBTabs.state.tabs);
    GBTabs.schedulePersist();
  };

  GBTabs.markUnread = function (id) {
    if (!id || id === GBTabs.state.activeTabId) return;
    GBTabs.unread[id] = true;
    GBTabs.renderStrip();
  };

  GBTabs.renderStrip = function () {
    var wrapper = document.getElementById("chatContentWrapper");
    if (!wrapper) return;
    var strip = document.getElementById("gbTabStrip");
    if (!GBTabs.state.enabled) {
      if (strip) strip.remove();
      return;
    }
    if (!strip) {
      strip = document.createElement("div");
      strip.id = "gbTabStrip";
      strip.className = "gb-tab-strip";
      wrapper.insertBefore(strip, wrapper.firstChild);
    }
    var html = "";
    GBTabs.state.tabs.forEach(function (tab, idx) {
      var glyph = tab.faviconGlyph ||
        (tab.title ? tab.title.charAt(0).toUpperCase() : "\u{1F4AC}");
      var cls = "gb-tab" + (tab.id === GBTabs.state.activeTabId ? " active" : "");
      if (tab.pinned) cls += " pinned";
      if (GBTabs.unread[tab.id]) cls += " unread";
      html += '<div class="' + cls + '" data-tab-id="' + tab.id + '" data-tab-idx="' + idx + '">' +
        '<span class="gb-tab-glyph">' + escapeHtml(glyph) + "</span>" +
        '<span class="gb-tab-title" title="' + escapeHtml(tab.title) + '">' +
        escapeHtml(tab.title) + "</span>" +
        '<span class="gb-tab-dot"></span>' +
        '<span class="gb-tab-pin">\u{1F4CC}</span>' +
        '<button type="button" class="gb-tab-close" title="Close tab">\u00D7</button>' +
        "</div>";
    });
    html += '<button type="button" class="gb-tab-new" id="gbTabNew" title="New tab">+</button>';
    strip.innerHTML = html;
  };

  GBTabs.schedulePersist = function () {
    if (GBTabs._persistTimer) clearTimeout(GBTabs._persistTimer);
    GBTabs._persistTimer = setTimeout(function () {
      GBTabs._persistTimer = null;
      persistToServer(GBTabs.state.tabs);
    }, 800);
  };

  function persistToServer(tabs) {
    if (!token()) return;
    fetch("/api/user/workspace/tabs", {
      method: "PUT",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify({ tabs: tabs }),
    }).catch(function () { /* server mirror is best-effort */ });
  }

  function recordUpdatedAt(rec) {
    return (rec && (rec.updated_at || rec.updatedAt)) || "";
  }

  /**
   * Restore on load. Merges the local store with the server copy,
   * last-writer-wins by updated_at; the winner is written back both ways
   * (heal) when they differ.
   */
  var MAX_TABS = 10;

  /** Keep the strip usable: the newest MAX_TABS tabs, default tab first.
   *  A leaked/accumulated store (hundreds of "Chat 1" rows) made the strip
   *  unusable and slowed every render. */
  function capTabs(tabs) {
    if (!Array.isArray(tabs) || tabs.length <= MAX_TABS) return tabs || [];
    var pinnedFirst = tabs.slice().sort(function (a, b) {
      return (b.pinned ? 1 : 0) - (a.pinned ? 1 : 0);
    });
    var kept = pinnedFirst.slice(0, MAX_TABS);
    // Preserve original relative order of the survivors.
    return tabs.filter(function (t) { return kept.indexOf(t) !== -1; });
  }

  GBTabs.restore = function () {
    var local = readLocal();
    if (local && Array.isArray(local.tabs)) {
      GBTabs.state.tabs = capTabs(local.tabs);
      if (GBTabs.state.tabs.length !== local.tabs.length) writeLocal(GBTabs.state.tabs);
    }
    if (!token()) {
      maybeActivateFrom(local ? recordUpdatedAt(local) : "");
      return;
    }
    fetch("/api/user/workspace/tabs", { headers: authHeaders() })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (server) {
        var serverTabs = server && (server.tabs || server.items);
        if (!Array.isArray(serverTabs) || !serverTabs.length) {
          writeLocal(GBTabs.state.tabs);
          maybeActivateFrom(local ? recordUpdatedAt(local) : new Date().toISOString());
          return;
        }
        var localStamp = local ? recordUpdatedAt(local) : "";
        var serverStamp = recordUpdatedAt(server);
        if (localStamp && serverStamp && localStamp >= serverStamp) {
          GBTabs.state.tabs = capTabs(local.tabs);
          persistToServer(GBTabs.state.tabs);
        } else {
          GBTabs.state.tabs = capTabs(serverTabs);
          writeLocal(GBTabs.state.tabs);
        }
        maybeActivateFrom(serverStamp || localStamp);
      })
      .catch(function () {
        maybeActivateFrom(local ? recordUpdatedAt(local) : "");
      });
  };

  function maybeActivateFrom(stamp) {
    var qs = new URLSearchParams(window.location.search);
    if (qs.get("tabs") === "1") {
      GBTabs.activate();
      return;
    }
    if (GBTabs.state.tabs.length > 1) {
      GBTabs.activate();
      if (stamp) writeLocal(GBTabs.state.tabs);
    }
  }

  /**
   * WS multiplexing hooks (#1168). Intercepts every socket assigned to
   * ChatState.ws without editing chat-websocket.js:
   * - outgoing frames gain tabId (+ session_id of the focused tab) only when
   *   tabs mode is enabled AND the active tab carries a sessionId — a single
   *   default tab keeps the legacy payload untouched;
   * - incoming frames are re-dispatched as document event "gb-ws-frame" and
   *   frames tagged with a foreign tabId raise that tab's unread dot.
   */
  function attach(ws) {
    if (!ws || ws.__gbTabsHooked) return;
    ws.__gbTabsHooked = true;
    var origSend = ws.send.bind(ws);
    ws.send = function (data) {
      try {
        if (GBTabs.state.enabled && typeof data === "string") {
          var obj = JSON.parse(data);
          var tab = GBTabs.activeTab();
          if (obj && typeof obj === "object" && !obj.tabId && tab && tab.sessionId) {
            obj.session_id = tab.sessionId;
            obj.tabId = tab.id;
            data = JSON.stringify(obj);
          }
        }
      } catch (e) { /* non-JSON frames pass through untouched */ }
      return origSend(data);
    };
    ws.addEventListener("message", function (event) {
      var data;
      try { data = JSON.parse(event.data); } catch (e) { return; }
      try {
        document.dispatchEvent(new CustomEvent("gb-ws-frame", { detail: data }));
      } catch (e) { /* listener errors must not break the socket */ }
      if (data && data.tabId && GBTabs.state.enabled &&
          data.tabId !== GBTabs.state.activeTabId) {
        GBTabs.markUnread(data.tabId);
      }
    });
  }

  (function installSocketHooks() {
    if (!window.ChatState) return;
    var current = window.ChatState.ws;
    try {
      Object.defineProperty(window.ChatState, "ws", {
        configurable: true,
        get: function () { return current; },
        set: function (v) { current = v; attach(v); },
      });
    } catch (e) {
      current = window.ChatState.ws;
    }
  })();

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", GBTabs.restore);
  } else {
    GBTabs.restore();
  }
})();
