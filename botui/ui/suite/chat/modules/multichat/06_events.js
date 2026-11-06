"use strict";

// Vibe multi-chat (#1282/#1283/#1285-fe) — stock render guard and tab-strip event wiring.
// Split out of the former 22_multichat module for #1731 (450-line limit). Mechanical split:
// same functions, globals and DOM/network behavior. Load order matters — the
// module namespace lives on window.GBMultiChat, created by 01_state.js.

(function () {
  var MC = window.GBMultiChat;

  // Stock-render guard: while a PARALLEL tab is mounted, stock-pipeline
  // frames for the default conversation (boot greeting, late async replies)
  // still call addMessage() → getElementById('messages') — which resolves to
  // the MOUNTED pane, dropping them into the wrong conversation. Re-point
  // the canonical id to the parked stock surface for the duration of the
  // call so stock frames always land in the stock conversation.
  function installStockRenderGuard() {
    if (window.addMessage && !window.addMessage.__mcGuarded) {
      var origAdd = window.addMessage;
      var guarded = function (sender, content, msgId, reasoning) {
        var stockMain = document.querySelector("main[data-mc-stock]");
        var parallelActive = MC.activeId && MC.tabs[MC.activeId] &&
          !MC.tabs[MC.activeId].usesStockDom;
        if (!parallelActive || !stockMain) return origAdd.apply(this, arguments);
        var paneMsgs = document.getElementById("messages");
        if (paneMsgs) paneMsgs.removeAttribute("id");
        stockMain.id = "messages";
        try {
          return origAdd.apply(this, arguments);
        } finally {
          stockMain.removeAttribute("id");
          if (paneMsgs) paneMsgs.id = "messages";
        }
      };
      guarded.__mcGuarded = true;
      window.addMessage = guarded;
    }
  }

  /* ── Hook: "+" button creates a PARALLEL conversation tab ── */

  // Tabs created elsewhere (history click, tab picker, boot restore) only
  // exist in GBTabs. Materialise an MC record, mount its pane, connect its
  // own socket and replay the conversation — without this, "open from
  // history" focused an empty tab and the chat behind it never changed.
  window.addEventListener("gb-tab-focused", function (e) {
    var tab = e && e.detail && e.detail.tab;
    if (!tab || !window.GBTabs) return;
    if (tab.id === "tab-default") {
      // Clicking Default while a parallel pane is still mounted must restore
      // the stock surface even if MC.activeId was already tab-default.
      var def = MC.tabs["tab-default"];
      if (def) MC.mountActive(def);
      if (MC.activeId !== tab.id) MC.switchTo(tab.id);
      return;
    }
    var t = MC.tabs[tab.id];
    if (!t) {
      t = MC.ensureTab(tab.id, tab.title, tab.sessionId || null);
      t.kind = tab.kind || "chat";
      MC.mountActive(t);
      MC.connectTab(t);
      if (tab.sessionId) MC.replayHistory(t);
      MC.installSendRouter();
      return;
    }
    if (MC.activeId !== tab.id) MC.switchTo(tab.id);
    else MC.mountActive(t);
  });

  // A bot switch (ChatSwitchBot) re-authenticates and mints a new session in
  // place. The default tab must adopt it immediately, otherwise the ws.send
  // hook keeps stamping the previous conversation's id on every frame.
  window.addEventListener("gb-chat-session-changed", function (e) {
    var sid = e && e.detail && e.detail.session_id;
    if (!sid || !window.GBTabs) return;
    var def = MC.tabs["tab-default"];
    if (def) def.sessionId = sid;
    GBTabs.state.tabs.forEach(function (st) {
      if (st.id === "tab-default") st.sessionId = sid;
    });
    // The auth callback sets the session AFTER the chat module boots, so the
    // boot-time history check found no session and the conversation stayed
    // blank. Replay it here, once per session.
    var stock = MC.ensureStockSurface();
    if (stock && !stock.childElementCount && typeof loadSessionHistory === "function" &&
        stock.dataset.loadedSession !== sid) {
      stock.dataset.loadedSession = sid;
      loadSessionHistory(sid);
    }
    if (typeof GBTabs.schedulePersist === "function") GBTabs.schedulePersist();
  });

  // Shared by the "+" button and the sidebar's New Conversation action: a
  // fresh tab owns its own socket and session, so the new conversation does
  // not write into the previous one.
  MC.newChatTab = function () {
    if (MC.blocked) return false;
    if (!window.GBTabs) return false;
    var tab = GBTabs.createTab({
      kind: "chat",
      title: "Chat " + ++MC.seq,
      faviconGlyph: "\u{1F4AC}",
    });
    var t = MC.ensureTab(tab.id, tab.title, null);
    MC.mountActive(t);
    MC.connectTab(t);
    MC.installSendRouter();
    var input = MC.q("messageInput");
    if (input) input.focus();
    return true;
  };

  // ANY click on "+" starts a fresh parallel chat (capture phase, so the
  // stock click handler never sees it). Right-click on "+" still opens the
  // history picker (21_tabs_events contextmenu) — the strip is for
  // conversations, never for apps.
  document.addEventListener("click", function (e) {
    if (!e.target.closest || !e.target.closest("#gbTabNew")) return;
    if (MC.blocked) return; // boot pending — ignore creates until surfaces are ready
    e.stopPropagation();
    e.preventDefault();
    MC.newChatTab();
  }, true);

  /// Drops restored tabs that cannot represent a conversation: no session,
  /// a session already open in another tab, or a duplicate of the fresh one.
  /// Persisted carts used to accumulate dead "Chat 1" tiles on every reload.
  function pruneTabs() {
    if (!window.GBTabs || !GBTabs.state) return;
    var seen = {};
    if (window.ChatState && ChatState.currentSessionId) {
      seen[ChatState.currentSessionId] = true;
    }
    var kept = [];
    GBTabs.state.tabs.forEach(function (st) {
      if (st.id === "tab-default") {
        kept.unshift(st);
        return;
      }
      // A tab created seconds ago earns a grace period: its socket mints the
      // session asynchronously, and the strip render triggered right after
      // createTab used to wipe the tile before that session ever existed.
      if (!st.sessionId && st.fresh && Date.now() - st.fresh < 120000) {
        kept.push(st);
        return;
      }
      if (!st.sessionId || seen[st.sessionId]) return;
      seen[st.sessionId] = true;
      kept.push(st);
    });
    while (kept.length > 10) kept.splice(kept.length - 1, 1);
    if (kept.length !== GBTabs.state.tabs.length) {
      var dropped = GBTabs.state.tabs.filter(function (st) {
        return !kept.some(function (k) { return k.id === st.id; });
      });
      GBTabs.state.tabs = kept;
      dropped.forEach(function (st) {
        // The tile is gone: drop its pane record too, or it stays mounted and
        // keeps receiving frames for a session the strip no longer shows.
        if (MC.tabs[st.id]) {
          if (MC.tabs[st.id].pane && MC.tabs[st.id].pane.parentNode) {
            MC.tabs[st.id].pane.parentNode.removeChild(MC.tabs[st.id].pane);
          }
          delete MC.tabs[st.id];
        }
      });
      // A pruned tab may have been the active one; focus a surviving tab
      // rather than leaving activeTabId dangling (the strip showed no active
      // tile and the stock pane kept the wrong conversation).
      if (!kept.some(function (k) { return k.id === GBTabs.state.activeTabId; })) {
        var fallback = kept[0] || null;
        if (fallback) {
          GBTabs.state.activeTabId = fallback.id;
          setTimeout(function () {
            window.dispatchEvent(new CustomEvent("gb-tab-focused", { detail: { tab: fallback } }));
          }, 0);
        }
      }
      if (typeof GBTabs.schedulePersist === "function") GBTabs.schedulePersist();
    }
  }

  /// Restored tabs are labelled "Chat 1" from the persisted store. Rename
  /// them from the conversation history so the strip shows what each tab is.
  var titlesSynced = false;
  function syncTabTitles() {
    if (titlesSynced) return;
    if (!window.GBTabs || !GBTabs.state) return;
    var pending = GBTabs.state.tabs.filter(function (st) {
      return st.sessionId && st.id !== "tab-default";
    });
    if (!pending.length) return;
    titlesSynced = true;
    fetch("/api/chat/history/sessions?limit=50", {
      headers: { Authorization: "Bearer " + MC.token() },
    })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        if (!data || !Array.isArray(data.sessions)) return;
        var byId = {};
        data.sessions.forEach(function (s) { byId[s.session_id] = s.title; });
        var changed = false;
        // A restored tab whose session has no conversation (never used, or
        // encrypted under a dead key) is a dead "Chat N" tile: drop it.
        GBTabs.state.tabs = GBTabs.state.tabs.filter(function (st) {
          if (st.id === "tab-default") return true;
          if (byId[st.sessionId]) {
            if (st.title !== byId[st.sessionId]) {
              st.title = byId[st.sessionId];
              changed = true;
            }
            return true;
          }
          if (/^(Chat \d+|Conversation)$/.test(st.title)) {
            changed = true;
            return false;
          }
          return true;
        });
        if (changed) GBTabs.renderStrip();
      })
      .catch(function () { /* titles are cosmetic */ });
  }

  // The server mirror merges asynchronously and re-renders the strip; prune
  // first so a late merge cannot resurrect the junk tabs.
  (function hookStripRender() {
    if (!window.GBTabs || !GBTabs.renderStrip || GBTabs.renderStrip.__mcPruned) return;
    var orig = GBTabs.renderStrip;
    var wrapped = function () {
      pruneTabs();
      return orig.apply(this, arguments);
    };
    wrapped.__mcPruned = true;
    GBTabs.renderStrip = wrapped;
  })();
  /* ── Published for the sibling multichat modules ── */

  MC.installStockRenderGuard = installStockRenderGuard;
  MC.pruneTabs = pruneTabs;
  MC.syncTabTitles = syncTabTitles;
})();
