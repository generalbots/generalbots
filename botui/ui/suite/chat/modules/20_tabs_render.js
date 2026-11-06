"use strict";

/**
 * Workspace tabs (#1168-fe) — tab operations, strip overflow, persistence
 * and restore. Split from 20_tabs.js for #1731: same functions and globals,
 * loaded right after the core file. Shared helpers stay on window.GBTabs.
 */
(function () {
  // Closing a chat tab must also dispose its runtime record: the parallel
  // conversation owns a WebSocket and a detached pane, and leaving them alive
  // kept receiving frames (and unread badges) for a tab that no longer exists.
  // App/history tabs have no MC record, so this is a no-op for them.
  function disposeChatRecord(id) {
    var MC = window.GBMultiChat;
    var rec = MC && MC.tabs ? MC.tabs[id] : null;
    if (!rec) return;
    if (rec.ws) {
      try { rec.ws.close(); } catch (e) { /* already closed */ }
    }
    try { delete MC.tabs[id]; } catch (e) { /* frozen */ }
  }

  GBTabs.closeTab = function (id) {
    var tabs = GBTabs.state.tabs;
    if (tabs.length <= 1) return;
    var idx = -1;
    for (var i = 0; i < tabs.length; i++) {
      if (tabs[i].id === id) { idx = i; break; }
    }
    if (idx === -1) return;
    tabs.splice(idx, 1);
    disposeChatRecord(id);
    delete GBTabs.unread[id];
    if (GBTabs.state.activeTabId === id) {
      var next = tabs[Math.min(idx, tabs.length - 1)];
      GBTabs.state.activeTabId = next.id;
      window.dispatchEvent(new CustomEvent("gb-tab-focused", { detail: { tab: next } }));
    }
    GBTabs.renderStrip();
    GBTabs.writeLocal(GBTabs.state.tabs);
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
    GBTabs.writeLocal(GBTabs.state.tabs);
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
    GBTabs.writeLocal(GBTabs.state.tabs);
    GBTabs.schedulePersist();
  };

  GBTabs.moveTab = function (fromIdx, toIdx) {
    var tabs = GBTabs.state.tabs;
    if (fromIdx === toIdx) return;
    if (fromIdx < 0 || fromIdx >= tabs.length) return;
    if (toIdx < 0 || toIdx >= tabs.length) return;
    var moved = tabs.splice(fromIdx, 1)[0];
    tabs.splice(toIdx, 0, moved);
    GBTabs.writeLocal(GBTabs.state.tabs);
    GBTabs.schedulePersist();
  };

  GBTabs.markUnread = function (id) {
    if (!id || id === GBTabs.state.activeTabId) return;
    GBTabs.unread[id] = true;
    GBTabs.renderStrip();
  };


  GBTabs.showOverflowMenu = function (button) {
    var existing = document.getElementById("gbTabOverflowMenu");
    if (existing) { existing.remove(); return; }
    var visible = GBTabs.visibleTabs();
    var hidden = GBTabs.state.tabs.filter(function (tab) {
      return visible.indexOf(tab) === -1;
    });
    if (!hidden.length) return;
    var menu = document.createElement("div");
    menu.id = "gbTabOverflowMenu";
    menu.className = "gb-tab-overflow-menu";
    menu.setAttribute("role", "menu");
    hidden.forEach(function (tab) {
      var item = document.createElement("button");
      item.type = "button";
      item.className = "gb-tab-overflow-item" +
        (tab.id === GBTabs.state.activeTabId ? " active" : "") +
        (GBTabs.unread[tab.id] ? " unread" : "");
      item.setAttribute("role", "menuitem");
      var glyph = tab.faviconGlyph ||
        (tab.title ? tab.title.charAt(0).toUpperCase() : "\u{1F4AC}");
      item.innerHTML = '<span class="gb-tab-overflow-glyph">' + escapeHtml(glyph) + "</span>" +
        '<span class="gb-tab-overflow-title">' + escapeHtml(tab.title) + "</span>";
      item.addEventListener("click", function () {
        menu.remove();
        GBTabs.focusTab(tab.id);
      });
      menu.appendChild(item);
    });
    document.body.appendChild(menu);
    var rect = button.getBoundingClientRect();
    var width = menu.offsetWidth || 240;
    menu.style.left = Math.max(8, Math.min(rect.left, window.innerWidth - width - 8)) + "px";
    menu.style.top = Math.min(rect.bottom + 6, window.innerHeight - menu.offsetHeight - 8) + "px";
    setTimeout(function () {
      document.addEventListener("click", function close(event) {
        if (!menu.contains(event.target)) {
          menu.remove();
          document.removeEventListener("click", close);
        }
      });
    }, 0);
  };

  GBTabs.schedulePersist = function () {
    if (GBTabs._persistTimer) clearTimeout(GBTabs._persistTimer);
    GBTabs._persistTimer = setTimeout(function () {
      GBTabs._persistTimer = null;
      persistToServer(GBTabs.state.tabs);
    }, 800);
  };

  function persistToServer(tabs) {
    if (!GBTabs.token()) return;
    fetch("/api/user/workspace/tabs", {
      method: "PUT",
      headers: GBTabs.authHeaders({ "Content-Type": "application/json" }),
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
    var local = GBTabs.readLocal();
    if (local && Array.isArray(local.tabs)) {
      GBTabs.state.tabs = capTabs(local.tabs);
      if (GBTabs.state.tabs.length !== local.tabs.length) GBTabs.writeLocal(GBTabs.state.tabs);
    }
    if (!GBTabs.token()) {
      maybeActivateFrom(local ? recordUpdatedAt(local) : "");
      return;
    }
    fetch("/api/user/workspace/tabs", { headers: GBTabs.authHeaders() })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (server) {
        var serverTabs = server && (server.tabs || server.items);
        if (!Array.isArray(serverTabs) || !serverTabs.length) {
          GBTabs.writeLocal(GBTabs.state.tabs);
          maybeActivateFrom(local ? recordUpdatedAt(local) : new Date().toISOString());
          return;
        }
        var localStamp = local ? recordUpdatedAt(local) : "";
        var serverStamp = recordUpdatedAt(server);
        // Preserve tabs created while the server merge was in flight. The
        // "+" tab used to vanish from the strip (active id without a record)
        // when restore() replaced state with the older server list.
        var fresh = GBTabs.state.tabs.filter(function (tab) {
          return tab.fresh && Date.now() - tab.fresh < 300000;
        });
        if (localStamp && serverStamp && localStamp >= serverStamp) {
          GBTabs.state.tabs = capTabs(local.tabs);
          persistToServer(GBTabs.state.tabs);
        } else {
          GBTabs.state.tabs = capTabs(serverTabs);
        }
        fresh.forEach(function (tab) {
          if (!GBTabs.state.tabs.some(function (known) { return known.id === tab.id; })) {
            GBTabs.state.tabs.push(tab);
          }
        });
        if (GBTabs.state.activeTabId &&
            !GBTabs.state.tabs.some(function (tab) { return tab.id === GBTabs.state.activeTabId; })) {
          GBTabs.state.activeTabId = GBTabs.state.tabs[0] ? GBTabs.state.tabs[0].id : null;
        }
        GBTabs.renderStrip();
        GBTabs.writeLocal(GBTabs.state.tabs);
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
    // The flag-off default was a dead end: the "+" affordance that activates
    // tabs only rendered AFTER activation, so single-tab users could never
    // discover the feature. Render the strip (default tab + "+") always —
    // one slim bar above the chat; closing tabs down to one keeps it usable.
    GBTabs.activate();
    if (stamp) GBTabs.writeLocal(GBTabs.state.tabs);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", GBTabs.restore);
  } else {
    GBTabs.restore();
  }
  // The chat window body is injected AFTER DOMContentLoaded (lazy fetch in
  // openDeepLink), so #chatContentWrapper does not exist when restore() first
  // runs and renderStrip() early-returns. Retry once when the chat window
  // actually opens so the tab strip mounts in single-chat mode too.
  document.addEventListener("gb-window-changed", function (e) {
    var d = (e && e.detail) || {};
    if (d.id !== "chat" || d.action !== "open") return;
    if (!document.getElementById("chatContentWrapper")) return;
    if (GBTabs.state.enabled) {
      GBTabs.renderStrip();
    } else if (!window.__gbTabsChatRetried) {
      window.__gbTabsChatRetried = true;
      GBTabs.restore();
    }
  });
})();
