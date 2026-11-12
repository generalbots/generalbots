"use strict";

// Vibe multi-chat (#1282/#1283/#1285-fe) — per-tab WebSocket, history replay and frame handling.
// Split out of the former 22_multichat module for #1731 (450-line limit). Mechanical split:
// same functions, globals and DOM/network behavior. Load order matters — the
// module namespace lives on window.GBMultiChat, created by 01_state.js.

(function () {
  var MC = window.GBMultiChat;

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
    var tok = MC.token();
    if (tok) headers["Authorization"] = "Bearer " + tok;
    // Reuse the tab's session on reconnect; without session_id the auth
    // endpoint mints a fresh conversation and the old one is orphaned.
    var authUrl = "/api/auth?bot_name=" + encodeURIComponent(resolveBotName());
    if (t.sessionId) authUrl += "&session_id=" + encodeURIComponent(t.sessionId);
    // A tab created with "+" must OWN its conversation. Without `new=1` the
    // auth endpoint resolves to the user's most recent session (2h window), so
    // every parallel tab silently shared one conversation and side-by-side
    // chats wrote into each other (#2219-family navigation review).
    else if (t.freshSession) authUrl += "&new=1";
    fetch(authUrl, { headers: headers })
      .then(function (r) { return r.json(); })
      .then(function (auth) {
        if (!t || MC.tabs[t.id] !== t || t.wsGeneration !== generation) return;
        if (t.sessionId && auth.session_id && requestedSessionRebind(t)) {
          /* keep stored binding */
        }
        if (!t.sessionId) {
          t.sessionId = auth.session_id;
          t.freshSession = false;
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
            MC.clearTabAck(t);
            if (data.event) return; // run events route elsewhere
            // #2098 — progress frames keep this tab's turn alive (step/tool
            // label), scoped to the tab's own pane.
            if (window.GBTurn) {
              try { GBTurn.observe(data, { tabId: t.id, pane: t.pane }); }
              catch (e) { /* progress is best-effort */ }
            }
            if (data.message_type === 2) {
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
    var tok = MC.token();
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
          MC.appendToContainer(t.pane, sender, m.content);
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
        MC.schedulePersist();
      })
      .catch(function () { /* history replay is best-effort */ });
  }

  /* ── Background-tab frame handling ── */

  function handleBotFrame(t, data) {
    // Stock tab: the stock pipeline renders these frames — a parallel
    // record here would duplicate them (double greeting) and inflate
    // the persisted scrollback on every boot.
    if (t.usesStockDom) return;
    // A reply in a parallel tab is what lets the server name that
    // conversation, so refresh the strip labels once the turn lands: the
    // addMessage hook only covers the stock surface, which left every
    // parallel tab stuck on "Chat N" (#2219-family chat navigation review).
    if (data.is_complete && t.sessionId && window.GBTabs) {
      setTimeout(function () {
        if (MC.syncTabTitles) MC.syncTabTitles(true);
      }, 2000);
    }
    // Parallel tabs' frames ALWAYS arrive on their own socket (the stock
    // pipeline never sees them) — render unconditionally. When the tab is
    // the active one its pane IS #messages, so appendToContainer lands in
    // the visible surface; background tabs render into their parked pane.
    if (data.is_complete) {
      // #2098 — the turn for THIS tab ends here: the progress element is
      // removed only after the final bubble is appended, and a duplicate/late
      // completion from the previous turn cannot end the live one.
      if (window.GBTurn) {
        GBTurn.complete(data, { tabId: t.id, pane: t.pane });
      }
      // The send path shows a thinking indicator as soon as Enter is pressed;
      // a parallel tab's completion never cleared it, so the dots stayed next
      // to the finished answer.
      var indicator = t.pane && t.pane.querySelector && t.pane.querySelector("#thinking-indicator");
      if (indicator) indicator.remove();
      if (typeof window.clearChatPending === "function") window.clearChatPending();
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
          MC.appendToContainer(t.pane, "bot", data.content);
          pushScrollback(t, "bot", data.content);
        }
      }
      var tabSugg = t.pane && t.pane.querySelector
        ? t.pane.querySelector(".suggestions-container")
        : null;
      if (tabSugg && data.suggestions && Array.isArray(data.suggestions) &&
          data.suggestions.length && typeof renderSuggestions === "function") {
        renderSuggestions(data.suggestions, tabSugg);
      }
      // #1753 — switcher chips belong to the tab that received the frame.
      if (data.switchers && Array.isArray(data.switchers) &&
          data.switchers.length && typeof renderBotSwitchers === "function") {
        renderBotSwitchers(data.switchers, t.id);
      }
      bumpUnread(t);
      MC.schedulePersist();
    } else if ((data.content && data.content.trim()) || data.reasoning) {
      // Visible content in this tab: the bubble is the progress element now,
      // the turn stays open until the completion frame (#2098).
      if (window.GBTurn) GBTurn.visible({ tabId: t.id, pane: t.pane });
      t.streamingContent = (t.streamingContent || "") + (data.content || "");
      // Active + mounted: live-stream into the visible pane (Claude-Code-
      // style parallel agents must stream in their own tab, not appear all
      // at once on completion).
      var activeMounted = t.id === MC.activeId && t.pane &&
        MC.q("messages") && MC.q("messages").__mcTabId === t.id;
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
  /* ── Published for the sibling multichat modules ── */

  MC.connectTab = connectTab;
  MC.replayHistory = replayHistory;
  MC.pushScrollback = pushScrollback;
})();
