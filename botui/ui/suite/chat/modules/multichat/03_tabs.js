"use strict";

// Vibe multi-chat (#1282/#1283/#1285-fe) — tab records, pane mount/switch lifecycle.
// Split out of the former 22_multichat module for #1731 (450-line limit). Mechanical split:
// same functions, globals and DOM/network behavior. Load order matters — the
// module namespace lives on window.GBMultiChat, created by 01_state.js.

(function () {
  var MC = window.GBMultiChat;

  function ensureTab(id, title, sessionId) {
    var t = MC.tabs[id];
    if (t) return t;
    t = MC.tabs[id] = {
      id: id,
      title: title || "Chat",
      sessionId: sessionId || null,
      pane: null,
      ws: null,
      scrollback: [],
      streamingEl: null,
      streamingContent: "",
      unread: 0,
      needsHistoryReplay: false,
      // #1753 — per-tab response-format switchers (the stock tab keeps
      // ChatState.activeSwitchers / ChatState.switcherDefinitions).
      switchers: new Set(),
      switcherDefinitions: [],
    };
    return t;
  }

  function ensurePane(t) {
    if (t.pane && t.pane.isConnected !== false && t.pane.parentElement) return t.pane;
    t.pane = MC.makePane();
    return t.pane;
  }

  /** Re-attach the tab's pane under the wrapper with the canonical ids so
   *  the stock chat pipeline (addMessage/processMessage/suggestions) renders
   *  into THIS tab. The previously active pane is detached intact. */
  function mountActive(t) {
    var wrap = MC.wrapper();
    if (!wrap) return;
    // #1283 — the original (stock) surface owns no pane record: the FIRST
    // tab is the stock DOM itself, so switching to it means removing any
    // mounted parallel pane (restoring ids on the stock <main>) and NO-OPing
    // the mount. Only parallel tabs get pane mount/swap treatment.
    // NOTE: #messages resolves to the pane's INNER <main>; the __mcPane/
    // __mcTabId flags live on the pane's OUTER .chat-pane div, so resolve
    // the owner through closest('.chat-pane').
    var mountedEl = MC.q("messages");
    var mountedPane = mountedEl ? mountedEl.closest(".chat-pane") : null;
    var mountedIsPane = !!(mountedPane && mountedPane.__mcPane);
    if (t.usesStockDom) {
      if (mountedIsPane) {
        var paneMsgs = mountedPane.querySelector(".chat-pane-messages");
        if (paneMsgs) paneMsgs.removeAttribute("id");
        var paneSugg = mountedPane.querySelector(".suggestions-container");
        if (paneSugg) paneSugg.removeAttribute("id");
        var paneSwitchers = mountedPane.querySelector(".switchers-container");
        if (paneSwitchers) paneSwitchers.removeAttribute("id");
        var paneSwitcherChips = mountedPane.querySelector(".switchers-chips");
        if (paneSwitcherChips) paneSwitcherChips.removeAttribute("id");
        mountedPane.parentElement && mountedPane.parentElement.removeChild(mountedPane);
        var owner = MC.tabs[mountedPane.__mcTabId];
        if (owner) owner.pane = mountedPane;
      }
      // Restore the canonical ids on the STOCK surface (they were handed to
      // the parallel pane while it was active).
      // Exactly one stock message area may carry id="messages". A fragment
      // re-injection or interrupted switch used to leave a second orphan main;
      // both then received flex:1 and split the height 50/50, which showed as
      // a half-height chat with a blank first pane.
      var allMessageMains = wrap.querySelectorAll("main#messages");
      for (var dup = allMessageMains.length - 1; dup > 0; dup--) {
        allMessageMains[dup].remove();
      }
      var stockMsgs = wrap.querySelector("main#messages") ||
        wrap.querySelector("main[data-mc-stock]");
      if (!stockMsgs) {
        var mains = wrap.querySelectorAll("main");
        for (var mi = 0; mi < mains.length; mi++) {
          if (!mains[mi].classList.contains("chat-pane-messages") &&
              mains[mi].id !== "messages") {
            stockMsgs = mains[mi];
            break;
          }
        }
      }
      // Drop any remaining orphan main that is neither the stock area nor a
      // pane's message list (it would still consume flex space).
      var leftovers = wrap.querySelectorAll("main");
      for (var li = 0; li < leftovers.length; li++) {
        if (leftovers[li] !== stockMsgs &&
            !leftovers[li].classList.contains("chat-pane-messages") &&
            leftovers[li].id !== "messages") {
          leftovers[li].remove();
        }
      }
      if (!stockMsgs) {
        stockMsgs = document.createElement("main");
        var footerHost = wrap.querySelector("footer");
        if (footerHost) wrap.insertBefore(stockMsgs, footerHost);
        else wrap.appendChild(stockMsgs);
      }
      stockMsgs.id = "messages";
      stockMsgs.removeAttribute("data-mc-stock");
      stockMsgs.style.display = ""; // unhide: the stock surface is active again
      // Switching back to Default must bring its conversation back. The stock
      // DOM can be empty after a parallel pane owned #messages during boot;
      // reload the stock session once instead of showing a blank page.
      if (!stockMsgs.childElementCount &&
          window.ChatState && ChatState.currentSessionId &&
          stockMsgs.dataset.loadedSession !== ChatState.currentSessionId &&
          typeof loadSessionHistory === "function") {
        stockMsgs.dataset.loadedSession = ChatState.currentSessionId;
        loadSessionHistory(ChatState.currentSessionId);
      }
      var stockSugg = wrap.querySelector("footer .suggestions-container[data-mc-stock]") ||
        wrap.querySelector("footer .suggestions-container:not([id])");
      if (stockSugg) {
        stockSugg.id = "suggestions";
        stockSugg.removeAttribute("data-mc-stock");
        stockSugg.style.display = "";
      }
      // Restore the STOCK tab's own switcher strip (parked while a parallel
      // pane owned the canonical ids).
      var stockSwitchers = wrap.querySelector("footer .switchers-container[data-mc-stock]") ||
        wrap.querySelector("footer .switchers-container");
      if (stockSwitchers) {
        stockSwitchers.id = "switchers";
        stockSwitchers.removeAttribute("data-mc-stock");
        var stockSwitcherChips = stockSwitchers.querySelector(".switchers-chips");
        if (stockSwitcherChips) stockSwitcherChips.id = "switcherChips";
        stockSwitchers.style.display =
          stockSwitcherChips && stockSwitcherChips.childElementCount ? "" : "none";
      }
      MC.activeId = t.id;
      return;
    }
    var oldPane = mountedPane;
    if (!oldPane && mountedEl && mountedEl.tagName === "MAIN") {
      // Stock surface is active: park its ids on data attributes AND hide it,
      // otherwise the stock <main> and the parallel pane both render as flex
      // rows — the two conversations appear STACKED instead of swapped.
      mountedEl.removeAttribute("id");
      mountedEl.setAttribute("data-mc-stock", "1");
      mountedEl.style.display = "none";
      var oldFooterSugg = wrap.querySelector("footer .suggestions-container");
      if (oldFooterSugg) {
        oldFooterSugg.removeAttribute("id");
        oldFooterSugg.setAttribute("data-mc-stock", "1");
        // Parking the stock surface removed its id but left its chips
        // visible below the parallel pane's own suggestions.
        oldFooterSugg.style.display = "none";
      }
      // Same for the stock response-format strip: it belongs to the stock
      // conversation, not the pane we are about to mount.
      var oldFooterSwitchers = wrap.querySelector("footer .switchers-container");
      if (oldFooterSwitchers) {
        var oldFooterChips = oldFooterSwitchers.querySelector(".switchers-chips");
        if (oldFooterChips) oldFooterChips.removeAttribute("id");
        oldFooterSwitchers.removeAttribute("id");
        oldFooterSwitchers.setAttribute("data-mc-stock", "1");
        oldFooterSwitchers.style.display = "none";
      }
    } else if (oldPane && oldPane.__mcTabId !== t.id) {
      // Detach the previous active pane (keep its canonical-id removal).
      var prev = MC.tabs[oldPane.__mcTabId];
      var prevMsgs = oldPane.querySelector(".chat-pane-messages");
      if (prevMsgs) prevMsgs.removeAttribute("id");
      var prevSugg = oldPane.querySelector(".suggestions-container");
      if (prevSugg) prevSugg.removeAttribute("id");
      var prevSwitchers = oldPane.querySelector(".switchers-container");
      if (prevSwitchers) prevSwitchers.removeAttribute("id");
      var prevSwitcherChips = oldPane.querySelector(".switchers-chips");
      if (prevSwitcherChips) prevSwitcherChips.removeAttribute("id");
      oldPane.parentElement && oldPane.parentElement.removeChild(oldPane);
      if (prev) prev.pane = oldPane;
    }
    var pane = ensurePane(t);
    pane.__mcTabId = t.id;
    pane.__mcPane = true;
    pane.id = "mc-pane-holder";
    var msgs = pane.querySelector(".chat-pane-messages");
    if (msgs) msgs.id = "messages";
    var sugg = pane.querySelector(".suggestions-container");
    if (sugg) sugg.id = "suggestions";
    // Hand the canonical switcher ids to THIS pane so stock-style lookups
    // (and the legacy #switchers rule) resolve inside the active tab.
    var paneSwitchers = pane.querySelector(".switchers-container");
    if (paneSwitchers) paneSwitchers.id = "switchers";
    var paneSwitcherChips = pane.querySelector(".switchers-chips");
    if (paneSwitcherChips) paneSwitcherChips.id = "switcherChips";
    // Insert right after the connection status (before footer).
    var footer = wrap.querySelector("footer");
    if (footer) wrap.insertBefore(pane, footer);
    else wrap.appendChild(pane);
    pane.id = "";
    // Scrollback restored from localStorage (#1285) for a background tab could
    // not be appended at boot time (no pane existed yet) — render it now, once,
    // before the tab becomes interactive.
    if (t.scrollback && t.scrollback.length) {
      var paneMsgsEl = pane.querySelector(".chat-pane-messages");
      if (paneMsgsEl && !paneMsgsEl.childElementCount) {
        t.scrollback.forEach(function (m) {
          MC.appendToContainer(pane, m.sender === "user" ? "user" : "bot", m.content);
        });
      }
    }
    // Switchers received while the tab was parked were recorded but could
    // not be painted (no mounted container); paint them now.
    if (t.switcherDefinitions && t.switcherDefinitions.length &&
        typeof renderSwitcherChips === "function") {
      renderSwitcherChips(t.id);
      if (paneSwitchers) paneSwitchers.style.display = "";
    }
    MC.activeId = t.id;
  }

  function switchTo(id) {
    var t = MC.tabs[id];
    if (!t) return;
    mountActive(t);
    // The stock tab rides the stock pipeline's socket (ChatState) — giving
    // it a second WS on the same session made every server frame arrive
    // twice (duplicated greetings rendered AND recorded into scrollback,
    // piling up across window re-opens).
    if (t.usesStockDom) {
      // keep stock socket
    } else if (t.ws && t.ws.readyState === WebSocket.OPEN) {
      /* already live */
    } else if (!t.ws || t.ws.readyState >= WebSocket.CLOSING) {
      MC.connectTab(t);
    }
    // Session routing: parallel tabs send on their OWN socket (sendInTab),
    // so ChatState.currentSessionId must stay bound to the STOCK session —
    // #1288: rebinding it to a parallel tab made a stock reconnect dial the
    // TAB's session, whose server channel was already owned by the tab's
    // socket — the two sockets then closed each other in an endless flap.
    // The send router (installSendRouter) and the ws.send tab-stamp already
    // direct parallel-tab traffic to the right session without touching
    // ChatState.
    if (window.ChatState && (t.usesStockDom || t.id === "tab-default")) {
      t.sessionId = t.sessionId || ChatState.currentSessionId;
      ChatState.currentSessionId = t.sessionId;
    }
    // Mirror the session onto the GBTabs state entry: the #1168 ws.send
    // hook stamps outgoing frames with GBTabs.activeTab().sessionId.
    GBTabs.state.tabs.forEach(function (st) { if (st.id === t.id) st.sessionId = t.sessionId; });
    GBTabs.focusTab(id);
    // The composer must follow the active tab on every switch (including
    // restored sessions), otherwise sends fall through to the stock surface.
    MC.installSendRouter();
    MC.schedulePersist();
  }
  /* ── Published for the sibling multichat modules ── */

  MC.ensureTab = ensureTab;
  MC.mountActive = mountActive;
  MC.switchTo = switchTo;
})();
