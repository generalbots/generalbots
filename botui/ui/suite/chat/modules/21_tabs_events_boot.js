"use strict";

/**
 * Workspace tabs (#1168-fe) — boot: delete handling, document listeners,
 * keyboard cycling. Split from 21_tabs_events.js for #1731; helpers come
 * from GBTabs.events, populated by the previous file.
 */
(function () {
  var E = window.GBTabs.events;
  var tabIdFromEvent = E.tabIdFromEvent;
  var closeTabGuarded = E.closeTabGuarded;
  var closeMenus = E.closeMenus;
  var onPointerDown = E.onPointerDown;
  var onClick = E.onClick;
  var onDblClick = E.onDblClick;
  var openPicker = E.openPicker;
  var openTabMenu = E.openTabMenu;
  var bindHistoryContextMenu = E.bindHistoryContextMenu;

  // ── Boot ──

  // #1753 — deleting the open conversation re-mints its session by calling
  // initChat(). Duplicate delete events (hover button + context menu, or a
  // re-injected module binding this listener twice) used to run two /api/auth
  // rounds and leave two sockets fighting. Hold a per-session guard until the
  // new session arrives (or a timeout) so a deletion reinitializes once.
  // #2285 — the guard must not be page-global: a delete reinit for one session
  // must not make another session's needed reinit permanently dead. A request
  // that arrives while the same session is in flight is coalesced into one
  // queued re-run on release instead of being dropped.
  var gbChatReinitMap = Object.create(null);
  function reinitializeAfterDelete(sid) {
    if (typeof window.initChat !== "function") return;
    var key = String(sid || "__current__");
    var state = gbChatReinitMap[key];
    if (state && state.busy) {
      state.queued = true;
      return;
    }
    if (!state) {
      state = gbChatReinitMap[key] = { busy: false, queued: false, timer: null };
    }
    state.busy = true;
    state.queued = false;
    var release = function () {
      if (!state.busy) return;
      state.busy = false;
      if (state.timer) {
        clearTimeout(state.timer);
        state.timer = null;
      }
      if (state.queued) {
        state.queued = false;
        setTimeout(function () { reinitializeAfterDelete(sid); }, 0);
      }
    };
    window.addEventListener("gb-chat-session-changed", release, { once: true });
    state.timer = setTimeout(release, 15000);
    try {
      window.initChat();
    } catch (err) {
      release();
    }
  }

  function init() {
    // BBSIA: deleting a conversation from the rail closes any tab bound to
    // that session so the UI cannot keep talking to a deleted history row.
    window.addEventListener("gb-conversation-deleted", function (e) {
      var sid = e.detail && e.detail.sessionId;
      if (!sid) return;
      // The server row is gone, but the browser kept replaying it: the
      // per-session transcript and the pinned session pointer must go too,
      // otherwise boot restores a deleted conversation.
      try {
        localStorage.removeItem("gb_chat_history_" + sid);
        Object.keys(localStorage).forEach(function (key) {
          if (key.indexOf("gb.chat.session.") === 0 && localStorage.getItem(key) === String(sid)) {
            localStorage.removeItem(key);
          }
        });
      } catch (err) { /* storage unavailable */ }
      if (!window.GBTabs || !GBTabs.state || !Array.isArray(GBTabs.state.tabs)) return;
      GBTabs.state.tabs.slice().forEach(function (t) {
        if (String(t.sessionId || "") !== String(sid)) return;
        // closeTab refuses to remove the last tab; clear its binding instead
        // so the deleted session is not kept as the active conversation.
        if (GBTabs.state.tabs.length <= 1) {
          // The last tab cannot close; unbind it fully instead so neither the
          // strip nor the persisted store keeps pointing at the deleted row.
          t.sessionId = null;
          t.title = "Chat";
          t.kind = "chat";
          t.fresh = Date.now();
          try { delete GBTabs.unread[t.id]; } catch (err) { /* no badge */ }
          var rec = window.GBMultiChat && GBMultiChat.tabs && GBMultiChat.tabs[t.id];
          if (rec) {
            rec.sessionId = null;
            rec.title = "Chat";
            rec.scrollback = [];
            rec.needsHistoryReplay = false;
            var recMsgs = rec.pane && rec.pane.querySelector(".chat-pane-messages");
            if (recMsgs) recMsgs.innerHTML = "";
          }
          try { GBTabs.renderStrip(); } catch (err) { /* not rendered */ }
          if (typeof GBTabs.schedulePersist === "function") GBTabs.schedulePersist();
        } else {
          // closeTab also disposes the parallel tab's socket/record (#1753).
          try { GBTabs.closeTab(t.id); } catch (err) { /* already gone */ }
        }
      });
      if (window.ChatState && String(ChatState.currentSessionId || "") === String(sid)) {
        var box = document.getElementById("messages");
        if (box) box.innerHTML = "";
        ChatState.currentSessionId = "";
        try {
          window.dispatchEvent(new CustomEvent("gb-chat-session-deleted", { detail: { sessionId: sid } }));
        } catch (err) { /* CustomEvent unavailable */ }
        // Mint a fresh session and reconnect; staying on the deleted id made
        // the next message fail server-side. The guard keeps a duplicate
        // event from running a second initChat/socket pair.
        reinitializeAfterDelete(sid);
      }
    });

    // Event DELEGATION on document: the strip is created dynamically by
    // GBTabs.renderStrip(), and this module's init can run before that (or
    // before every WM re-injection). Binding to a strip element that does
    // not exist yet (or to a node that is later replaced) silently killed
    // every tab click — clicking a tab did nothing. document-level capture
    // survives all strip re-creations.
    document.addEventListener("pointerdown", function (e) {
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      onPointerDown(e);
    }, true);
    document.addEventListener("click", function (e) {
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      onClick(e);
    });
    document.addEventListener("dblclick", function (e) {
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      onDblClick(e);
    });
    document.addEventListener("contextmenu", function (e) {
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      // Right-click on "+": open the history picker (recent conversations).
      if (e.target.closest("#gbTabNew")) {
        e.preventDefault();
        openPicker(e);
        return;
      }
      var id = tabIdFromEvent(e);
      if (!id || e.target.closest(".gb-tab-close")) return;
      e.preventDefault();
      openTabMenu(e.clientX, e.clientY, id);
    });
    // #1283 — middle-click (auxclick button 1) closes the tab.
    document.addEventListener("auxclick", function (e) {
      if (e.button !== 1) return;
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      var id = tabIdFromEvent(e);
      if (id) { e.preventDefault(); closeTabGuarded(id); }
    });
    document.addEventListener("click", function (e) {
      // The SAME click that opens the picker on "+" also reaches this
      // closer: the strip delegation runs first and appends the menu, then
      // this listener fires with the button still as event.target, closing
      // the picker in the same tick it opened. Skip opener-origin clicks.
      if (e.target.closest && e.target.closest("#gbTabNew")) return;
      if (!e.target.closest(".gb-context-menu")) closeMenus();
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape") closeMenus();
    });
    bindHistoryContextMenu();
    bindKeyboardCycle();
    if (window.GBMultiChat) GBMultiChat.start();
  }

  // #1283 — Ctrl+Tab / Ctrl+Shift+Tab cycle tabs; Ctrl+W closes the active
  // tab (guarded). Plain browser-tab muscle memory, zero new chrome.
  function bindKeyboardCycle() {
    document.addEventListener("keydown", function (e) {
      if (!GBTabs.state.enabled || GBTabs.state.tabs.length < 2) return;
      if (e.ctrlKey && !e.altKey && !e.metaKey && e.key === "Tab") {
        e.preventDefault();
        var tabs = GBTabs.state.tabs;
        var idx = tabs.findIndex(function (t) { return t.id === GBTabs.state.activeTabId; });
        var next = e.shiftKey
          ? tabs[(idx - 1 + tabs.length) % tabs.length]
          : tabs[(idx + 1) % tabs.length];
        if (window.GBMultiChat && GBMultiChat.tabs[next.id]) GBMultiChat.switchTo(next.id);
        else GBTabs.focusTab(next.id);
      } else if (e.ctrlKey && !e.shiftKey && !e.altKey && !e.metaKey && (e.key === "w" || e.key === "W")) {
        var active = GBTabs.activeTab();
        if (active && !active.pinned && GBTabs.state.tabs.length > 1) {
          e.preventDefault();
          closeTabGuarded(active.id);
        }
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
