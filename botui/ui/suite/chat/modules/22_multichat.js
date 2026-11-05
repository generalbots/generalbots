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
        // Replayed messages must render vmfile links as attachment cards too;
        // the stock addMessage path does this conversion, appendToContainer
        // (used by history replay and parallel tabs) did not.
        if (typeof renderVmFileLinks === "function") {
          inner.innerHTML = renderVmFileLinks(inner.innerHTML);
        }
        if (typeof polishOutputBlocks === "function") {
          polishOutputBlocks(inner);
        }
      } catch (e) {
        inner.textContent = content;
      }
    } else {
      inner.textContent = content;
    }
    div.appendChild(inner);
    msgs.appendChild(div);
    if (typeof mergeWorkspaceOutput === "function") mergeWorkspaceOutput(div);
    msgs.scrollTop = msgs.scrollHeight;
    if (sender !== "user") {
      var cards = div.querySelectorAll(".gb-attachment");
      for (var ci = 0; ci < cards.length; ci++) {
        if (typeof vmBindAttachment === "function") vmBindAttachment(cards[ci]);
      }
    }
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
    // The composer must follow the active tab on every switch (including
    // restored sessions), otherwise sends fall through to the stock surface.
    installSendRouter();
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
    if (!t) return;
    // A healthy socket must not be replaced: every connect used to mint a new
    // /api/auth session, so the tab's record and its live socket diverged and
    // the reply was delivered to a stale conversation.
    if (t.ws && (t.ws.readyState === WebSocket.OPEN || t.ws.readyState === WebSocket.CONNECTING) && t.sessionId) {
      return;
    }
    // One socket per tab. Calling connect twice (switch + reconnect + boot)
    // used to leave the previous socket alive, so both handlers rendered the
    // same reply and output looked mixed between conversations.
    var generation = (t.wsGeneration || 0) + 1;
    t.wsGeneration = generation;
    if (t.ws) {
      try {
        t.ws.onmessage = null;
        t.ws.onclose = null;
        t.ws.close();
      } catch (e) { /* already closed */ }
      t.ws = null;
    }
    var headers = {};
    var tok = token();
    if (tok) headers["Authorization"] = "Bearer " + tok;
    // Reuse the tab's session on reconnect; without session_id the auth
    // endpoint mints a fresh conversation and the old one is orphaned.
    var authUrl = "/api/auth?bot_name=" + encodeURIComponent(resolveBotName());
    if (t.sessionId) authUrl += "&session_id=" + encodeURIComponent(t.sessionId);
    fetch(authUrl, { headers: headers })
      .then(function (r) { return r.json(); })
      .then(function (auth) {
        if (!t || MC.tabs[t.id] !== t || t.wsGeneration !== generation) return;
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
          if (t.wsGeneration !== generation) return;
          try {
            var data = JSON.parse(event.data);
            if (data.type === "connected") return;
            clearTabAck(t);
            if (data.event) return; // run events route elsewhere
            if (data.message_type === 2) {
              if (typeof window.clearChatPending === "function") window.clearChatPending();
              handleBotFrame(t, data);
            }
            // The stock socket executes UI plans in chat-websocket.js; a
            // parallel tab has its own socket and used to drop the frame, so
            // "open Weather" answered but never opened the window (#1691).
            if (data.message_type === 9 && data.plan && window.GBUiOrchestrator) {
              window.GBUiOrchestrator.executePlan(data.plan);
            }
            // Structured surfaces and HTML apps arrive as CUSTOM frames. The
            // stock socket handled them; a parallel tab used to drop them, so
            // show_app produced an upload but nothing appeared in the chat.
            if (data.type === "CUSTOM" && data.data) {
              if (typeof window.clearChatPending === "function") window.clearChatPending();
              var paneRoot = t.pane && t.pane.querySelector(".chat-pane-messages");
              var name = data.data.name;
              if (name === "gb.app" && window.AppFrame) {
                window.AppFrame.renderFromFrame(data, paneRoot);
              } else if ((name === "gb.surface" || name === "gb.plan") && window.SurfaceRenderer) {
                window.SurfaceRenderer.renderFromFrame(data, paneRoot);
              } else if (name === "gb.question" && window.SurfaceRenderer) {
                window.SurfaceRenderer.renderQuestion(data, paneRoot);
              } else if (name === "gb.surface_update" && window.SurfaceRenderer) {
                window.SurfaceRenderer.applyUpdate(data);
              }
            }
          } catch (e) { /* ignore malformed frame */ }
        };

        ws.onclose = function () {
          if (t.wsGeneration !== generation) return;
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
          if (t.wsGeneration !== generation) return;
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
    // #1682 — locked conversations need the short-lived unlock grant.
    var grant = sessionStorage.getItem("gb-unlock:" + t.sessionId);
    var url = "/api/chat/history/sessions/" + encodeURIComponent(t.sessionId) + "/messages";
    if (grant) url += "?unlock_token=" + encodeURIComponent(grant);
    fetch(url, {
      headers: headers,
    })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        if (!data || !Array.isArray(data.messages)) return;
        var msgs = t.pane && t.pane.querySelector(".chat-pane-messages");
        if (msgs) msgs.innerHTML = "";
        t.scrollback = [];
        var previousRole = "";
        var previousContent = "";
        data.messages.forEach(function (m) {
          var sender = m.role === "user" ? "user" : "bot";
          if (sender === "bot" && previousRole === "bot" && previousContent === m.content) {
            return;
          }
          previousRole = sender;
          previousContent = m.content;
          pushScrollback(t, sender, m.content);
          appendToContainer(t.pane, sender, m.content);
        });
        // Bind the cards and list any output/ file the model did not link.
        if (typeof hydrateVmAttachments === "function") {
          hydrateVmAttachments(t.pane);
        }
        if (typeof snapToBottom === "function") {
          setTimeout(snapToBottom, 250);
        }
        if (typeof stickToBottom === "function") {
          stickToBottom(2500);
        }
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
      // The send path shows a thinking indicator as soon as Enter is pressed;
      // a parallel tab's completion never cleared it, so the dots stayed next
      // to the finished answer.
      var indicator = t.pane && t.pane.querySelector && t.pane.querySelector("#thinking-indicator");
      if (indicator) indicator.remove();
      // Drop the live-streaming placeholder, if any.
      if (t.streamingEl && t.streamingEl.nodeType) {
        t.streamingEl.remove();
      }
      t.streamingEl = null;
      t.streamingContent = "";
      if (data.content && data.content.trim()) {
        // The server can emit the same completion frame twice (stream end +
        // final message). Keep one bubble: only dedupe an immediately
        // preceding identical bot turn, never a legitimate repeat after a
        // user message.
        var previous = t.scrollback[t.scrollback.length - 1];
        var duplicate = previous &&
          previous.sender === "bot" &&
          previous.content === data.content &&
          Date.now() - (previous.at || 0) < 4000;
        if (!duplicate) {
          appendToContainer(t.pane, "bot", data.content);
          pushScrollback(t, "bot", data.content);
        }
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

  function clearTabAck(t) {
    if (!t) return;
    if (t.ackTimer) {
      clearTimeout(t.ackTimer);
      t.ackTimer = null;
    }
    t.pendingAck = null;
  }

  // A tab socket can look OPEN after a server restart and still drop the
  // frame. Arm a short delivery guard: if no frame comes back, reconnect and
  // resend once so a message never disappears silently.
  function armTabAck(t, payload) {
    if (!t) return;
    clearTabAck(t);
    var ack = { payload: payload, attempts: 1 };
    t.pendingAck = ack;
    function expire() {
      if (t.pendingAck !== ack) return;
      // An OPEN socket with no frame is usually a slow model, not a lost
      // message. Reconnecting here resent the prompt and produced two
      // answers. Only retry when the socket is actually closed.
      if (t.ws && t.ws.readyState === WebSocket.OPEN) {
        t.pendingAck = null;
        return;
      }
      if (ack.attempts >= 2) {
        t.pendingAck = null;
        if (typeof notify === "function") {
          notify("Message not delivered — reconnecting. Please try again.", "error");
        }
        return;
      }
      ack.attempts += 1;
      try {
        if (t.ws && t.ws.readyState < WebSocket.CLOSING) t.ws.close();
      } catch (e) { /* already closed */ }
      t.pending = t.pending || [];
      t.pending.push(ack.payload);
      connectTab(t);
      t.ackTimer = setTimeout(expire, 8000);
    }
    t.ackTimer = setTimeout(expire, 7000);
  }

  MC.sendInTab = function (tabId, text, file) {
    var t = MC.tabs[tabId];
    if (!t) return;
    text = (text || "").trim();
    if (!text && !file) return;
    // Parallel tabs ALWAYS use their own socket + session: the stock
    // pipeline's socket is bound to the default conversation, so routing
    // through it sent the message (and received the reply) under the WRONG
    // session — parallel tabs shared one history. The tab's own socket is
    // connected with the tab's session id.
    var display = text || ("📎 " + file.name);
    if (typeof window.armChatPending === "function") {
      window.armChatPending(text || (file && file.name) || "");
    }
    appendToContainer(t.pane, "user", display);
    if (typeof showThinkingIndicator === "function") showThinkingIndicator();
    pushScrollback(t, "user", display);
    schedulePersist();
    var payload = {
      bot_id: t.botId || "default",
      user_id: t.userId || "",
      session_id: t.sessionId,
      channel: "web",
      content: text,
      message_type: 1,
      timestamp: new Date().toISOString(),
      client_capabilities: window.gbClientCapabilities
        ? window.gbClientCapabilities()
        : undefined,
    };
    function dispatch() {
      t.pending = t.pending || [];
      if (t.ws && t.ws.readyState === WebSocket.OPEN) {
        try {
          t.ws.send(JSON.stringify(payload));
          armTabAck(t, payload);
        } catch (e) {
          t.pending.push(payload);
          connectTab(t);
        }
      } else {
        t.pending.push(payload);
        if (!t.ws || t.ws.readyState >= WebSocket.CLOSING) connectTab(t);
      }
    }
    // Audio recordings and uploaded files in a parallel tab used to be
    // dropped: the router only forwarded text, so the attachment never
    // reached the tab's socket. Read it like the stock path does and send the
    // same `file` payload.
    if (file) {
      var reader = new FileReader();
      reader.onerror = function () {
        notify("Could not read the attachment. Attach it again.", "error");
      };
      reader.onload = function () {
        payload.file = {
          name: file.name,
          content_base64: String(reader.result).split(",")[1] || reader.result,
        };
        dispatch();
      };
      reader.readAsDataURL(file);
    } else {
      dispatch();
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
      if (def) mountActive(def);
      if (MC.activeId !== tab.id) switchTo(tab.id);
      return;
    }
    var t = MC.tabs[tab.id];
    if (!t) {
      t = ensureTab(tab.id, tab.title, tab.sessionId || null);
      t.kind = tab.kind || "chat";
      mountActive(t);
      connectTab(t);
      if (tab.sessionId) replayHistory(t);
      installSendRouter();
      return;
    }
    if (MC.activeId !== tab.id) switchTo(tab.id);
    else mountActive(t);
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
    var stock = ensureStockSurface();
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
    var t = ensureTab(tab.id, tab.title, null);
    mountActive(t);
    connectTab(t);
    installSendRouter();
    var input = q("messageInput");
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
      headers: { Authorization: "Bearer " + token() },
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
    var wrap = wrapper();
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
    GBTabs.state.tabs.forEach(function (tab) {
      if (tab.id === "tab-default") { keep.unshift(tab); return; }
      if (tab.fresh && tab.fresh > BOOT_AT + 1500) { keep.push(tab); return; }
      if (tab.sessionId && !seen[tab.sessionId]) {
        seen[tab.sessionId] = true;
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
    var t = ensureTab(first.id, first.title, first.sessionId || null);
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
    var stockSurface = ensureStockSurface();
    if (stockSurface && !stockSurface.childElementCount &&
        window.ChatState && ChatState.currentSessionId &&
        stockSurface.dataset.loadedSession !== ChatState.currentSessionId &&
        typeof loadSessionHistory === "function") {
      stockSurface.dataset.loadedSession = ChatState.currentSessionId;
      loadSessionHistory(ChatState.currentSessionId);
    }
    // Reconcile the DOM with the stock tab: a previous injection may have
    // left a parallel pane mounted while the record already said default.
    if (t && t.usesStockDom) mountActive(t);
    var restored = restore();
    pruneTabs();
    syncTabTitles();
    // The composer must be routed from the very first send after a reload,
    // not only after the user clicks a tab.
    installSendRouter();
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
      var rec = ensureTab(st.id, st.title, !isDefault && st.sessionId ? st.sessionId : null);
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
    installSendRouter();
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
        var fileInput = q("messageFile");
        var file = fileInput && fileInput.files && fileInput.files[0];
        var text = messageContent || (input ? input.value : "");
        if (input && !messageContent) {
          input.value = "";
          input.focus();
        }
        if ((!text || !text.trim()) && !file) return;
        MC.sendInTab(t.id, text, file);
        if (fileInput) fileInput.value = "";
        var chip = q("fileChip");
        if (chip) { chip.style.display = "none"; chip.textContent = ""; }
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
