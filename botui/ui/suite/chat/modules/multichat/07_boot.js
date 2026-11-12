"use strict";

// Vibe multi-chat (#1282/#1283/#1285-fe) — stock surface restore, boot and send router.
// Split out of the former 22_multichat module for #1731 (450-line limit). Mechanical split:
// same functions, globals and DOM/network behavior. Load order matters — the
// module namespace lives on window.GBMultiChat, created by 01_state.js.

(function () {
  var MC = window.GBMultiChat;

  /* ── Public: called by 21_tabs_events after its own boot ── */

  var _started = false;
  var _bootId = null;
  var BOOT_AT = Date.now();

  // Reloads start from the canonical Default conversation. Restoring old
  // parallel tabs reconnected stale sockets/sessions and produced mixed
  // output, blank panes and ghost tabs. Tabs created after this boot are kept.
  // Guarantee the stock conversation has a message area. A previous pane
  // switch can remove every <main>, leaving the composer glued under the tabs.
  function ensureStockSurface() {
    var wrap = MC.wrapper();
    if (!wrap) return null;
    var footer = wrap.querySelector("footer");
    var stock = wrap.querySelector("main#messages");
    if (!stock) {
      var mains = wrap.querySelectorAll("main");
      for (var i = 0; i < mains.length; i++) {
        if (!mains[i].classList.contains("chat-pane-messages")) {
          stock = mains[i];
          break;
        }
      }
    }
    if (!stock) {
      stock = document.createElement("main");
      if (footer) wrap.insertBefore(stock, footer);
      else wrap.appendChild(stock);
    }
    stock.id = "messages";
    stock.style.display = "";
    var all = wrap.querySelectorAll("main");
    for (var j = 0; j < all.length; j++) {
      if (all[j] !== stock && !all[j].classList.contains("chat-pane-messages")) {
        all[j].remove();
      }
    }
    return stock;
  }

  function pruneRestoredTabs() {
    if (!window.GBTabs || !GBTabs.state || !Array.isArray(GBTabs.state.tabs)) return;
    // Tabs persisted from the previous session are KEPT as tiles: the user
    // gets their context back after a reload, but nothing connects or replays
    // until the tile is clicked (no boot socket storm). Tabs without a
    // session that were not created during this boot cannot represent a
    // conversation and are dropped.
    var seen = { "tab-default": true };
    var keep = [];
    // #1949 — a persisted strip used to come back with EVERY previous
    // conversation stacked above the fresh one (reported as ~5 duplicate
    // chats). Keep at most two most-recent distinct conversations besides the
    // default tab, plus whatever session this page load is actually on.
    var RESTORED_LIMIT = 2;
    var restored = 0;
    var currentSession = (window.ChatState && ChatState.currentSessionId) || null;
    GBTabs.state.tabs.forEach(function (tab) {
      if (tab.id === "tab-default") { keep.unshift(tab); return; }
      if (tab.fresh && tab.fresh > BOOT_AT + 1500) { keep.push(tab); return; }
      if (tab.sessionId && !seen[tab.sessionId]) {
        seen[tab.sessionId] = true;
        if (currentSession && tab.sessionId !== currentSession && restored >= RESTORED_LIMIT) return;
        if (!currentSession || tab.sessionId !== currentSession) restored++;
        tab.restored = true;
        keep.push(tab);
      }
    });
    if (keep.length === GBTabs.state.tabs.length &&
        keep.some(function (tab) { return tab.id === GBTabs.state.activeTabId; })) {
      GBTabs.renderStrip();
      return;
    }
    GBTabs.state.tabs = keep;
    if (!keep.some(function (tab) { return tab.id === GBTabs.state.activeTabId; })) {
      GBTabs.state.activeTabId = "tab-default";
      var def = keep.find(function (tab) { return tab.id === "tab-default"; });
      if (def) {
        setTimeout(function () {
          window.dispatchEvent(new CustomEvent("gb-tab-focused", { detail: { tab: def } }));
        }, 0);
      }
    }
    Object.keys(MC.tabs || {}).forEach(function (id) {
      if (id === "tab-default") return;
      if (!keep.some(function (tab) { return tab.id === id; })) {
        var rec = MC.tabs[id];
        if (rec && rec.ws) { try { rec.ws.close(); } catch (e) {} }
        delete MC.tabs[id];
      }
    });
    GBTabs.renderStrip();
  }

  MC.start = function () {
    MC.installStockRenderGuard();
    // Track the window injection we booted against: the WM re-injects the
    // chat fragment on every open WITHOUT a page reload, so module state
    // (including _started) survives. Re-bind whenever the #messages element
    // is a DIFFERENT node than the one we booted on.
    var msgsNow = MC.q("messages");
    if (!msgsNow) {
      // The chat reveal flow (chat-init connectWebSocket → showChatApp)
      // mounts #messages a beat after the fragment scripts execute. Retry
      // briefly so multi-chat bootstraps on every window open. Block tab
      // creation until the boot completes — a shift-click before #messages
      // exists would otherwise create a pane whose record start() then
      // wipes (MC.tabs reset), leaving an orphan pane and a broken strip.
      MC.blocked = true;
      var tries = 0;
      var iv = setInterval(function () {
        ++tries;
        if (MC.q("messages") || tries > 40) {
          clearInterval(iv);
          MC.blocked = false;
          MC.start();
        }
      }, 250);
      return;
    }
    if (_started && _bootId === msgsNow.__mcBoot) return;
    _started = true;
    _bootId = msgsNow.__mcBoot = "boot-" + Date.now();
    // A WM re-injection rebuilds the chat DOM without a page reload: drop the
    // previous injection's tab records AND any panes still attached to the
    // old DOM (orphans would stack invisibly and break surface swapping).
    if (MC.wrapper()) {
      MC.wrapper().querySelectorAll(".chat-pane").forEach(function (p) { p.remove(); });
    }
    MC.tabs = {};
    _started = true;
    // Track the ORIGINAL conversation as the first tab so it is never lost
    // when the user opens parallel ones. activate() alone creates the
    // canonical "tab-default" — creating ANOTHER tab here produced a
    // duplicate "Default" tab whose click could never restore the stock
    // surface (it was not in MC.tabs).
    var orig = GBTabs.defaultTab();
    if (!GBTabs.state.tabs.length) GBTabs.activate();
    // The first tab must be the canonical `tab-default`, not whichever
    // conversation happened to be first in the persisted order. Marking a
    // parallel tab as `usesStockDom` mixed its output with the stock session.
    var defaultIndex = -1;
    for (var di = 0; di < GBTabs.state.tabs.length; di++) {
      if (GBTabs.state.tabs[di].id === "tab-default") { defaultIndex = di; break; }
    }
    if (defaultIndex === -1) {
      GBTabs.state.tabs.unshift(orig);
    } else if (defaultIndex > 0) {
      GBTabs.state.tabs.splice(0, 0, GBTabs.state.tabs.splice(defaultIndex, 1)[0]);
    }
    var first = GBTabs.state.tabs[0] || GBTabs.createTab({ kind: "chat", title: orig.title, pinned: true });
    // The default tab represents the session the server minted for THIS page
    // load. A persisted tab kept the previous conversation's id, and the
    // ws.send hook stamps that id on every frame — so new messages were
    // written into the OLD conversation. The requested session (history
    // click) is already in ChatState because /api/auth was called with it.
    if (window.ChatState && ChatState.currentSessionId) {
      first.sessionId = ChatState.currentSessionId;
    }
    var t = MC.ensureTab(first.id, first.title, first.sessionId || null);
    // The canonical default tab always represents the STOCK surface, even if
    // it came back from a persisted store without a multichat record.
    t.usesStockDom = true;
    t.pane = null;
    // The original pane IS the existing #messages + footer structure: no
    // detaching needed — the stock pipeline owns it while this tab is active.
    t.usesStockDom = true;
    t.pane = null;
    MC.activeId = first.id;
    pruneRestoredTabs();
    var stockSurface = MC.ensureStockSurface();
    if (stockSurface && !stockSurface.childElementCount &&
        window.ChatState && ChatState.currentSessionId &&
        stockSurface.dataset.loadedSession !== ChatState.currentSessionId &&
        typeof loadSessionHistory === "function") {
      stockSurface.dataset.loadedSession = ChatState.currentSessionId;
      loadSessionHistory(ChatState.currentSessionId);
    }
    // Reconcile the DOM with the stock tab: a previous injection may have
    // left a parallel pane mounted while the record already said default.
    if (t && t.usesStockDom) MC.mountActive(t);
    var restored = MC.restore();
    MC.pruneTabs();
    MC.syncTabTitles();
    // The composer must be routed from the very first send after a reload,
    // not only after the user clicks a tab.
    MC.installSendRouter();
    // Boot always lands on the default tab. A restored parallel tab used to
    // come back as the active tab while ChatState carried the freshly minted
    // session; frames were then stamped with the OLD conversation id and new
    // messages were written into old history.
    if (GBTabs.state.activeTabId !== first.id) {
      GBTabs.state.activeTabId = first.id;
      MC.activeId = first.id;
      GBTabs.renderStrip();
    }
    // Self-heal: restore()/server merge can add tabs to GBTabs.state.tabs
    // AFTER our boot snapshot — any state tab without an MC record here is a
    // zombie (its pane/session died with the previous injection). Register a
    // record for it so clicks land on a real tab, or the tab is a ghost that
    // swallows clicks ("clicking a tab does nothing"). Only tab-default may
    // own the stock surface. A restored tab with NO session cannot restore
    // any conversation — pruning it instead of resurrecting prevents the
    // zombie-tab pileup (dozens of dead "Chat N" tiles).
    for (var i = GBTabs.state.tabs.length - 1; i >= 0; i--) {
      var st = GBTabs.state.tabs[i];
      if (MC.tabs[st.id]) continue;
      var isDefault = st.id === "tab-default";
      if (!isDefault && !st.sessionId) {
        GBTabs.state.tabs.splice(i, 1);
        continue;
      }
      var rec = MC.ensureTab(st.id, st.title, !isDefault && st.sessionId ? st.sessionId : null);
      if (isDefault) {
        rec.usesStockDom = true;
        rec.pane = null;
        if (MC.activeId !== rec.id && !MC.activeId) MC.activeId = rec.id;
      }
    }
    // The server merge resolves later and can re-add persisted tabs; prune
    // again so only this boot's Default and new tabs survive.
    setTimeout(pruneRestoredTabs, 2000);
    setTimeout(pruneRestoredTabs, 5000);

    // Drop MC records whose state tab vanished (cap pruning, remote delete).
    Object.keys(MC.tabs).forEach(function (id) {
      if (id === "tab-default") return;
      if (!GBTabs.state.tabs.some(function (st) { return st.id === id; })) {
        var rec = MC.tabs[id];
        if (rec && rec.pane && rec.pane.parentElement) rec.pane.parentElement.removeChild(rec.pane);
        if (rec && rec.ws && rec.ws.readyState < WebSocket.CLOSING) { try { rec.ws.close(); } catch (e) {} }
        delete MC.tabs[id];
        if (MC.activeId === id) MC.activeId = "tab-default";
      }
    });
    if (!restored) GBTabs.renderStrip();
    MC.installSendRouter();
  };

  // Late merge: restore()'s server fetch resolves AFTER start() — when it
  // swaps GBTabs.state.tabs, re-run the heal so new/removed tabs stay live.
  var _origRestore = null;
  function hookServerMerge() {
    if (!window.GBTabs || !GBTabs.restore || GBTabs.restore.__mcHooked) return;
    _origRestore = GBTabs.restore;
    GBTabs.restore = function () {
      var r = _origRestore.apply(this, arguments);
      setTimeout(function () { if (window.GBMultiChat) MC.start(); }, 800);
      return r;
    };
    GBTabs.restore.__mcHooked = true;
  }
  hookServerMerge();

  MC.active = function () { return MC.activeId; };

  // Route sends: when a PARALLEL tab is active, the composer must go to
  // that tab's own socket/session — never through the stock pipeline (whose
  // socket belongs to the default conversation). tab-default keeps the full
  // stock path (mentions, offline queue, TTS, file attach...).
  function installSendRouter() {
    if (window.sendMessage && !window.sendMessage.__mcRouted) {
      var stockSend = window.sendMessage;
      var routed = function (messageContent) {
        var t = MC.activeId && MC.tabs[MC.activeId];
        if (!t || t.usesStockDom) return stockSend.apply(this, arguments);
        var input = MC.q("messageInput");
        var fileInput = MC.q("messageFile");
        var files = fileInput && fileInput.files
          ? Array.prototype.slice.call(fileInput.files)
          : [];
        var text = messageContent || (input ? input.value : "");
        if (input && !messageContent) {
          input.value = "";
          input.focus();
        }
        if ((!text || !text.trim()) && !files.length) return;
        MC.sendInTab(t.id, text, files);
        if (fileInput) fileInput.value = "";
        var chip = MC.q("fileChip");
        if (chip) { chip.style.display = "none"; chip.textContent = ""; }
      };
      routed.__mcRouted = true;
      window.sendMessage = routed;
    }
  }

  // #2241 — allow the Default conversation to be reset when its session is
  // deleted via the sidebar: clear its history and rotate to a fresh session
  // instead of leaving stale content or an orphaned tab.
  window.addEventListener("gb-conversation-deleted", function (e) {
    var d = (e && e.detail) || {};
    var sid = d.sessionId;
    if (!sid) return;
    var defTab = (GBTabs && GBTabs.state && Array.isArray(GBTabs.state.tabs))
      ? GBTabs.state.tabs.find(function (t) { return t.id === "tab-default"; })
      : null;
    if (!defTab || String(defTab.sessionId || "") !== String(sid)) return;
    try {
      var msgs = document.getElementById("messages");
      if (msgs) msgs.innerHTML = "";
    } catch (err) {}
    try {
      defTab.sessionId = null;
      if (window.ChatState) ChatState.currentSessionId = null;
      GBTabs.state.activeTabId = "tab-default";
      GBTabs.renderStrip && GBTabs.renderStrip();
    } catch (err) {}
  });

  MC.ensureStockSurface = ensureStockSurface;
  MC.installSendRouter = installSendRouter;

  // Boot independently of 21_tabs_events' init: the fragment scripts execute
  // after the chat DOM is in place, and an eager retry closes the race where
  // a fast user click (or the test driver) creates a tab before start()'s
  // MC.tabs reset — which would orphan the new pane.
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () { MC.start(); });
  } else {
    MC.start();
  }
})();
