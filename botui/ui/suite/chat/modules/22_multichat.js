"use strict";

/**
 * Vibe multi-chat (#1282/#1283/#1285-fe) — parallel chat tabs.
 *
 * Each tab is an INDEPENDENT conversation: its own WebSocket session, its
 * own message container, its own streaming state — so two LLM modification
 * sessions stream simultaneously (the active tab through the stock chat
 * pipeline, background tabs through a lightweight accumulator that renders
 * final messages and raises an unread badge).
 *
 * Mechanics: the chat pipeline renders through fixed ids (#messages,
 * #suggestions, #chatForm). When multi-chat is on, this module detaches the
 * original #messages into the first tab's pane and gives every tab its own
 * .chat-pane. On tab switch the tab's pane element is re-attached under the
 * wrapper with the canonical id — stock code keeps working unchanged.
 *
 * This module is additive: it only activates on the "+" new-tab path
 * (GBTabs.activate()), so the single-conversation flow is untouched.
 */

(function () {
  var MC = (window.GBMultiChat = {
    tabs: {}, // tabId -> record
    activeId: null,
    seq: 0,
  });

  var STORAGE_KEY = "gb.multichat.v1";

  function q(id) {
    return document.getElementById(id);
  }

  function wrapper() {
    return q("chatContentWrapper");
  }

  function strip() {
    return q("gbTabStrip");
  }

  function tabRecord(id) {
    return MC.tabs[id] || null;
  }

  function token() {
    return (
      (window.GBTabs && GBTabs.token && GBTabs.token()) ||
      (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      ""
    );
  }

  function esc(s) {
    var d = document.createElement("div");
    d.textContent = s == null ? "" : String(s);
    return d.innerHTML;
  }

  /* ── Persistence (localStorage fallback, #1285 AC3) ── */

  function persist() {
    try {
      var rec = { tabs: {}, activeId: MC.activeId };
      Object.keys(MC.tabs).forEach(function (id) {
        var t = MC.tabs[id];
        rec.tabs[id] = {
          title: t.title,
          sessionId: t.sessionId,
          scrollback: t.scrollback.slice(-100),
          unread: t.unread || 0,
        };
      });
      localStorage.setItem(STORAGE_KEY, JSON.stringify(rec));
    } catch (e) { /* storage unavailable */ }
  }

  var persistTimer = null;
  function schedulePersist() {
    if (persistTimer) return;
    persistTimer = setTimeout(function () {
      persistTimer = null;
      persist();
    }, 500);
  }

  function restore() {
    var rec = null;
    try {
      rec = JSON.parse(localStorage.getItem(STORAGE_KEY) || "null");
    } catch (e) { rec = null; }
    if (!rec || !rec.tabs) return false;
    var ids = Object.keys(rec.tabs);
    if (!ids.length) return false;
    ids.forEach(function (id) {
      var saved = rec.tabs[id];
      var tab = ensureTab(id, saved.title || "Chat", saved.sessionId || null);
      // The stock tab's conversation IS the stock DOM — never resurrect a
      // persisted scrollback for it (pre-fix versions accumulated one
      // greeting copy per window open here).
      if (tab.usesStockDom || id === "tab-default") {
        tab.scrollback = [];
      } else {
        tab.scrollback = Array.isArray(saved.scrollback) ? saved.scrollback.slice(-100) : [];
      }
      tab.needsHistoryReplay = tab.scrollback.length === 0 && !!tab.sessionId;
      tab.scrollback.forEach(function (m) {
        appendToContainer(tab.pane, m.sender === "user" ? "user" : "bot", m.content);
      });
    });
    // Restore the last active tab (default: the first non-original tab).
    var target = rec.activeId && MC.tabs[rec.activeId] ? rec.activeId : null;
    if (!target) {
      var ids2 = Object.keys(MC.tabs);
      target = ids2.length ? ids2[ids2.length - 1] : null;
    }
    if (target) switchTo(target);
    return true;
  }

  /* ── DOM construction ── */

  function makePane() {
    var pane = document.createElement("div");
    pane.className = "chat-pane";
    pane.style.cssText =
      "display:flex;flex-direction:column;flex:1;min-height:0;overflow:hidden;";
    var msgs = document.createElement("main");
    msgs.className = "chat-pane-messages";
    msgs.style.cssText = window.getComputedStyle(q("messages") || pane).cssText;
    // Minimal self-contained styles (computed cssText can be empty pre-layout):
    msgs.style.cssText =
      "flex:1;overflow-y:auto;overflow-x:hidden;padding:16px;display:flex;" +
      "flex-direction:column;gap:16px;scrollbar-width:thin;justify-content:flex-start;align-items:stretch;";
    var sugg = document.createElement("div");
    sugg.className = "suggestions-container";
    sugg.style.cssText = "display:flex;flex-wrap:wrap;gap:8px;";
    pane.appendChild(msgs);
    pane.appendChild(sugg);
    return pane;
  }

  function appendToContainer(container, sender, content, msgId) {
    if (!container) return null;
    var msgs = container.__msgs || container.querySelector(".chat-pane-messages");
    if (!msgs) return null;
    var div = document.createElement("div");
    div.className = "message " + (sender === "user" ? "user" : "bot");
    if (msgId) div.id = msgId;
    var inner = document.createElement("div");
    inner.className =
      "message-content " + (sender === "user" ? "user-message" : "bot-message");
    if (sender === "user") {
      var processed = typeof renderMentionInMessage === "function"
        ? renderMentionInMessage(esc(content))
        : esc(content);
      inner.innerHTML = processed;
    } else if (typeof marked !== "undefined" && marked.parse) {
      try {
        inner.innerHTML = marked.parse(content || "");
      } catch (e) {
        inner.textContent = content;
      }
    } else {
      inner.textContent = content;
    }
    div.appendChild(inner);
    msgs.appendChild(div);
    msgs.scrollTop = msgs.scrollHeight;
    return div;
  }

  /* ── Tab lifecycle ── */

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
    };
    return t;
  }

  function ensurePane(t) {
    if (t.pane && t.pane.isConnected !== false && t.pane.parentElement) return t.pane;
    t.pane = makePane();
    return t.pane;
  }

  /** Re-attach the tab's pane under the wrapper with the canonical ids so
   *  the stock chat pipeline (addMessage/processMessage/suggestions) renders
   *  into THIS tab. The previously active pane is detached intact. */
  function mountActive(t) {
    var wrap = wrapper();
    if (!wrap) return;
    // #1283 — the original (stock) surface owns no pane record: the FIRST
    // tab is the stock DOM itself, so switching to it means removing any
    // mounted parallel pane (restoring ids on the stock <main>) and NO-OPing
    // the mount. Only parallel tabs get pane mount/swap treatment.
    // NOTE: #messages resolves to the pane's INNER <main>; the __mcPane/
    // __mcTabId flags live on the pane's OUTER .chat-pane div, so resolve
    // the owner through closest('.chat-pane').
    var mountedEl = q("messages");
    var mountedPane = mountedEl ? mountedEl.closest(".chat-pane") : null;
    var mountedIsPane = !!(mountedPane && mountedPane.__mcPane);
    if (t.usesStockDom) {
      if (mountedIsPane) {
        var paneMsgs = mountedPane.querySelector(".chat-pane-messages");
        if (paneMsgs) paneMsgs.removeAttribute("id");
        var paneSugg = mountedPane.querySelector(".suggestions-container");
        if (paneSugg) paneSugg.removeAttribute("id");
        mountedPane.parentElement && mountedPane.parentElement.removeChild(mountedPane);
        var owner = MC.tabs[mountedPane.__mcTabId];
        if (owner) owner.pane = mountedPane;
      }
      // Restore the canonical ids on the STOCK surface (they were handed to
      // the parallel pane while it was active).
      var stockMsgs = wrap.querySelector("main[data-mc-stock]");
      if (stockMsgs) {
        stockMsgs.id = "messages";
        stockMsgs.removeAttribute("data-mc-stock");
        stockMsgs.style.display = ""; // unhide: the stock surface is active again
      }
      var stockSugg = wrap.querySelector("footer .suggestions-container[data-mc-stock]");
      if (stockSugg) {
        stockSugg.id = "suggestions";
        stockSugg.removeAttribute("data-mc-stock");
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
      }
    } else if (oldPane && oldPane.__mcTabId !== t.id) {
      // Detach the previous active pane (keep its canonical-id removal).
      var prev = MC.tabs[oldPane.__mcTabId];
      var prevMsgs = oldPane.querySelector(".chat-pane-messages");
      if (prevMsgs) prevMsgs.removeAttribute("id");
      var prevSugg = oldPane.querySelector(".suggestions-container");
      if (prevSugg) prevSugg.removeAttribute("id");
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
          appendToContainer(pane, m.sender === "user" ? "user" : "bot", m.content);
        });
      }
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
      connectTab(t);
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
    schedulePersist();
  }

  /* ── Per-tab WebSocket ── */

  function resolveBotName() {
    return (
      (window.GBResolveActiveBot && GBResolveActiveBot()) ||
      (window.ChatState && window.ChatState.currentBotName) ||
      window.__INITIAL_BOT_NAME__ ||
      "default"
    );
  }

  function connectTab(t) {
    var headers = {};
    var tok = token();
    if (tok) headers["Authorization"] = "Bearer " + tok;
    fetch("/api/auth?bot_name=" + encodeURIComponent(resolveBotName()), { headers: headers })
      .then(function (r) { return r.json(); })
      .then(function (auth) {
        if (!t || MC.tabs[t.id] !== t) return;
        if (t.sessionId && auth.session_id && requestedSessionRebind(t)) {
          /* keep stored binding */
        }
        if (!t.sessionId) {
          t.sessionId = auth.session_id;
          // Keep the GBTabs entry in sync so the ws.send hook stamps the
          // tab's OWN session on outgoing frames.
          GBTabs.state.tabs.forEach(function (st) {
            if (st.id === t.id) st.sessionId = t.sessionId;
          });
        }
        t.userId = auth.user_id;
        t.botId = auth.bot_id || "default";

        var proto = location.protocol === "https:" ? "wss://" : "ws://";
        var url =
          proto + location.host + "/ws?session_id=" + encodeURIComponent(t.sessionId) +
          "&user_id=" + encodeURIComponent(auth.user_id || "") +
          "&bot_name=" + encodeURIComponent(resolveBotName());
        var ws = new WebSocket(url);
        t.ws = ws;

        ws.onmessage = function (event) {
          try {
            var data = JSON.parse(event.data);
            if (data.type === "connected") return;
            if (data.event) return; // run events route elsewhere
            if (data.message_type === 2) handleBotFrame(t, data);
          } catch (e) { /* ignore malformed frame */ }
        };

        ws.onclose = function () {
          if (MC.tabs[t.id] !== t) return;
          // Bounded reconnect so a downed server cannot spin forever.
          t.reconnects = (t.reconnects || 0) + 1;
          if (t.reconnects <= 5) {
            setTimeout(function () {
              if (MC.tabs[t.id] === t && (!t.ws || t.ws.readyState >= WebSocket.CLOSING)) {
                connectTab(t);
              }
            }, 1000 * t.reconnects);
          }
        };

        ws.onopen = function () {
          t.reconnects = 0;
          if (t.needsHistoryReplay && t.sessionId) {
            t.needsHistoryReplay = false;
            replayHistory(t);
          }
        };
      })
      .catch(function () { /* tab stays local until retried */ });
  }

  function requestedSessionRebind() {
    // Reserved: a reconnected tab re-attaching to its conversation. The
    // session id is stable per tab, so no rebinding is needed today.
    return false;
  }

  function replayHistory(t) {
    var headers = {};
    var tok = token();
    if (tok) headers["Authorization"] = "Bearer " + tok;
    fetch("/api/chat/history/sessions/" + encodeURIComponent(t.sessionId) + "/messages", {
      headers: headers,
    })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        if (!data || !Array.isArray(data.messages)) return;
        var msgs = t.pane && t.pane.querySelector(".chat-pane-messages");
        if (msgs) msgs.innerHTML = "";
        t.scrollback = [];
        data.messages.forEach(function (m) {
          var sender = m.role === "user" ? "user" : "bot";
          pushScrollback(t, sender, m.content);
          appendToContainer(t.pane, sender, m.content);
        });
        schedulePersist();
      })
      .catch(function () { /* history replay is best-effort */ });
  }

  /* ── Background-tab frame handling ── */

  function handleBotFrame(t, data) {
    // Stock tab: the stock pipeline renders these frames — a parallel
    // record here would duplicate them (double greeting) and inflate
    // the persisted scrollback on every boot.
    if (t.usesStockDom) return;
    // Parallel tabs' frames ALWAYS arrive on their own socket (the stock
    // pipeline never sees them) — render unconditionally. When the tab is
    // the active one its pane IS #messages, so appendToContainer lands in
    // the visible surface; background tabs render into their parked pane.
    if (data.is_complete) {
      // Drop the live-streaming placeholder, if any.
      if (t.streamingEl && t.streamingEl.nodeType) {
        t.streamingEl.remove();
      }
      t.streamingEl = null;
      t.streamingContent = "";
      if (data.content && data.content.trim()) {
        appendToContainer(t.pane, "bot", data.content);
        pushScrollback(t, "bot", data.content);
      }
      bumpUnread(t);
      schedulePersist();
    } else if ((data.content && data.content.trim()) || data.reasoning) {
      t.streamingContent = (t.streamingContent || "") + (data.content || "");
      // Active + mounted: live-stream into the visible pane (Claude-Code-
      // style parallel agents must stream in their own tab, not appear all
      // at once on completion).
      var activeMounted = t.id === MC.activeId && t.pane &&
        q("messages") && q("messages").__mcTabId === t.id;
      if (activeMounted) {
        if (!t.streamingEl || !t.streamingEl.nodeType) {
          var holder = t.pane.querySelector(".chat-pane-messages");
          if (holder) {
            var div = document.createElement("div");
            div.className = "message bot";
            div.innerHTML = '<div class="message-content bot-message"></div>';
            holder.appendChild(div);
            t.streamingEl = div;
          }
        }
        if (t.streamingEl && t.streamingEl.nodeType) {
          var body = t.streamingEl.querySelector(".message-content");
          if (body) body.textContent = t.streamingContent;
          var msgsEl = t.pane.querySelector(".chat-pane-messages");
          if (msgsEl) msgsEl.scrollTop = msgsEl.scrollHeight;
        }
      } else {
        t.streamingEl = t.streamingEl || { placeholder: true };
      }
    }
  }

  function pushScrollback(t, sender, content) {
    t.scrollback.push({ sender: sender, content: content, at: Date.now() });
    if (t.scrollback.length > 100) t.scrollback.shift();
  }

  function bumpUnread(t) {
    t.unread = (t.unread || 0) + 1;
    GBTabs.unread[t.id] = true;
    if (typeof GBTabs.renderStrip === "function") GBTabs.renderStrip();
  }

  /* ── Sending from a specific tab ── */

  MC.sendInTab = function (tabId, text) {
    var t = MC.tabs[tabId];
    if (!t || !text || !text.trim()) return;
    text = text.trim();
    // Parallel tabs ALWAYS use their own socket + session: the stock
    // pipeline's socket is bound to the default conversation, so routing
    // through it sent the message (and received the reply) under the WRONG
    // session — parallel tabs shared one history. The tab's own socket is
    // connected with the tab's session id.
    appendToContainer(t.pane, "user", text);
    pushScrollback(t, "user", text);
    schedulePersist();
    var payload = {
      bot_id: t.botId || "default",
      user_id: t.userId || "",
      session_id: t.sessionId,
      channel: "web",
      content: text,
      message_type: 1,
      timestamp: new Date().toISOString(),
    };
    t.pending = t.pending || [];
    if (t.ws && t.ws.readyState === WebSocket.OPEN) {
      t.ws.send(JSON.stringify(payload));
    } else {
      t.pending.push(payload);
      if (!t.ws || t.ws.readyState >= WebSocket.CLOSING) connectTab(t);
    }
  };

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

  // Flush queued parallel-tab frames once their socket opens.
  (function () {
    // connectTab is private; wrap its socket-open behavior by observing the
    // pending queue on every state change instead of patching the closure.
    setInterval(function () {
      Object.keys(MC.tabs).forEach(function (id) {
        var t = MC.tabs[id];
        if (t && t.pending && t.pending.length && t.ws && t.ws.readyState === WebSocket.OPEN) {
          MC.sendPending(t);
        }
      });
    }, 1000);
  })();

  MC.tabByElement = function (el) {
    var id = el && el.getAttribute ? el.getAttribute("data-tab-id") : null;
    return id ? MC.tabs[id] || null : null;
  };

  MC.sendPending = function (t) {
    if (!t.pending || !t.pending.length) return;
    if (!t.ws || t.ws.readyState !== WebSocket.OPEN) return;
    var qd = t.pending.splice(0, t.pending.length);
    qd.forEach(function (p) {
      try { t.ws.send(JSON.stringify(p)); } catch (e) { t.pending.push(p); }
    });
  };

  /* ── Hook: "+" button creates a PARALLEL conversation tab ── */

  // The stock picker opens app entries / recent sessions; a plain "+" shift
  // click (or the keyboard shortcut) starts a fresh parallel chat instead.
  document.addEventListener("click", function (e) {
    if (!e.target.closest || !e.target.closest("#gbTabNew")) return;
    if (!(e.shiftKey || e.altKey)) return; // plain click = stock picker
    if (MC.blocked) return; // boot pending — ignore creates until surfaces are ready
    e.stopPropagation();
    e.preventDefault();
    var tab = GBTabs.createTab({ kind: "chat", title: "Chat " + ++MC.seq, faviconGlyph: "\u{1F4AC}" });
    var t = ensureTab(tab.id, tab.title, null);
    mountActive(t);
    connectTab(t);
    installSendRouter();
    var input = q("messageInput");
    if (input) input.focus();
  }, true);

  /* ── Public: called by 21_tabs_events after its own boot ── */

  var _started = false;
  var _bootId = null;
  MC.start = function () {
    installStockRenderGuard();
    // Track the window injection we booted against: the WM re-injects the
    // chat fragment on every open WITHOUT a page reload, so module state
    // (including _started) survives. Re-bind whenever the #messages element
    // is a DIFFERENT node than the one we booted on.
    var msgsNow = q("messages");
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
        if (q("messages") || tries > 40) {
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
    if (wrapper()) {
      wrapper().querySelectorAll(".chat-pane").forEach(function (p) { p.remove(); });
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
    var first = GBTabs.state.tabs[0] || GBTabs.createTab({ kind: "chat", title: orig.title, pinned: true });
    var t = ensureTab(first.id, first.title, window.ChatState ? ChatState.currentSessionId : null);
    // The canonical default tab always represents the STOCK surface, even if
    // it came back from a persisted store without a multichat record.
    t.usesStockDom = true;
    t.pane = null;
    // The original pane IS the existing #messages + footer structure: no
    // detaching needed — the stock pipeline owns it while this tab is active.
    t.usesStockDom = true;
    t.pane = null;
    MC.activeId = first.id;
    var restored = restore();
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
      var rec = ensureTab(st.id, st.title, !isDefault && st.sessionId ? st.sessionId : null);
      if (isDefault) {
        rec.usesStockDom = true;
        rec.pane = null;
        if (MC.activeId !== rec.id && !MC.activeId) MC.activeId = rec.id;
      }
    }
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

  MC.switchTo = switchTo;
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
        var input = q("messageInput");
        var text = messageContent || (input ? input.value : "");
        if (input && !messageContent) {
          input.value = "";
          input.focus();
        }
        if (text && text.trim()) MC.sendInTab(t.id, text);
      };
      routed.__mcRouted = true;
      window.sendMessage = routed;
    }
  }

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
