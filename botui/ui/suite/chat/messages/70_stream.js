/* Chat messages module: Streaming render/finalize and the websocket message dispatcher. */

/* #1849 — chat is the most-used window, so it publishes its turn state to the
 * shared status bar contract instead of leaving the generic "Ready". The bar
 * is optional (standalone chat page, older shells): failures are ignored. */
function gbChatStatus(payload) {
  try {
    if (window.GBStatus && typeof window.GBStatus.set === "function") {
      window.GBStatus.set("chat", payload);
    }
  } catch (e) { /* status bar unavailable */ }
}

function gbChatStatusText(key, fallback) {
  return window.gbT ? window.gbT(key, fallback) : fallback;
}
function updateStreaming(content) {
  var el = document.getElementById(ChatState.streamingMessageId);
  if (!el && window.GBChatVirtual && typeof window.GBChatVirtual.ensureById === "function") {
    el = window.GBChatVirtual.ensureById(ChatState.streamingMessageId);
  }
  if (!el) return;
  var streamBox = document.getElementById("messages");
  var wasNearBottom = streamBox &&
    (streamBox.scrollHeight - streamBox.scrollTop - streamBox.clientHeight) < 240;

  var msgContent = el.querySelector(".message-content");
  // #1693 — the open fence belongs to the live preview; keep its raw source
  // out of the bubble while it streams.
  var previewContent = (window.GBLiveArtifact &&
    window.GBLiveArtifact.lastBlock(content))
    ? stripLiveArtifactBlock(content)
    : content;
  var cleanContent = stripMarkdownBlocks(previewContent);
  var parsed;
  if (typeof marked !== "undefined" && marked.parse) {
    parsed = marked.parse(cleanContent);
  } else {
    var isHtml = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
    parsed = isHtml ? cleanContent : escapeHtml(cleanContent);
  }
  parsed = convertNumberedLists(parsed);
  parsed = renderInlineMarkdown(parsed);
  parsed = parsed.replace(/<br\s*\/?>/gi, '');
  parsed = renderMentionInMessage(parsed);
  var thinkingHtml = renderThinkingSection(ChatState.currentReasoning);
  msgContent.innerHTML = thinkingHtml + parsed;
  // #1693 — an open ```svg/```html fence is previewed as it grows. The frame
  // lives on the message (not inside .message-content) so the next streaming
  // re-render does not destroy and reload it every 200 ms.
  if (window.GBLiveArtifact) window.GBLiveArtifact.render(el, content);
  if (window.ChatCodeUX) {
    ChatCodeUX.enhance(msgContent);
    var toolCards = ChatCodeUX.extractToolCards(content);
    toolCards.forEach(function (tool) {
      if (tool.status) ChatCodeUX.appendCard(tool);
    });
  }
  if (ChatState.isUserAtBottom) scrollToBottom(false);
}

function finalizeStreaming() {
  var sid = ChatState.streamingMessageId;
  var el = sid ? document.getElementById(sid) : null;
  if (!el && sid && window.GBChatVirtual && typeof window.GBChatVirtual.ensureById === "function") {
    el = window.GBChatVirtual.ensureById(sid);
  }
  if (el) {
    if (window.GBChatVirtual && typeof window.GBChatVirtual.remove === "function") {
      try { window.GBChatVirtual.remove(el); } catch (e) {}
    }
    el.remove();
  } else if (sid && window.GBChatVirtual && typeof window.GBChatVirtual.remove === "function") {
    // Defensive cleanup: tracked-but-detached streaming node.
    try {
      var items = window.GBChatVirtual._items ? window.GBChatVirtual._items() : [];
      for (var i = 0; i < items.length; i++) {
        if (items[i] && items[i].el && items[i].el.id === sid) {
          window.GBChatVirtual.remove(items[i].el);
          break;
        }
      }
    } catch (e) {}
  }
  ChatState.streamingMessageId = null;
  ChatState.currentStreamingContent = "";
  ChatState.currentReasoning = "";
  ChatState.streamingBuffer = "";
}

/* #2098 — one place ends a turn: the indicator is removed only after the
 * final bubble is in the transcript, and the pending watchdog is cleared with
 * it. `completed` carries the answer actually rendered (the frame can be an
 * empty completion that reuses the accumulated stream). */
function gbFinishTurn(data, completed) {
  if (window.GBTurn) {
    GBTurn.complete(data, {
      content: completed ? completed.content : "",
      reasoning: completed ? completed.reasoning : "",
    });
  } else if (typeof hideThinkingIndicator === "function") {
    hideThinkingIndicator();
  }
  if (typeof window.clearChatPending === "function") window.clearChatPending();
}

function processMessage(data) {
  if (data.thinking) {
    // A thinking/keepalive frame keeps the progress element alive; it never
    // hides it (#2098).
    if (window.GBTurn) GBTurn.thinking();
    else if (!ChatState.isStreaming) showThinkingIndicator();
    gbChatStatus({ state: "running", text: gbChatStatusText("status-thinking", "Thinking…") });
    return;
  }

  if (data.is_complete) {
    // A late completion from the previous turn (or from another conversation)
    // must not end the live one.
    if (window.GBTurn && GBTurn.stale(data)) return;
    // Capture the accumulated stream before finalizeStreaming() clears it.
    // Completion frames often carry only reasoning; dropping the accumulated
    // content left a "Thinking" bubble with no visible answer (the persisted
    // message still had the answer).
    var finalContent =
      (data.content && data.content.trim() !== "")
        ? data.content
        : (ChatState.currentStreamingContent || "");
    var finalReasoning = data.reasoning || ChatState.currentReasoning || "";
    var hasAnswer = (finalContent && finalContent.trim() !== "") ||
      (finalReasoning && finalReasoning.trim() !== "");
    if (!hasAnswer && !ChatState.isStreaming) {
      // An empty completion (tool boundary, retry in flight) is not the end of
      // the turn: keep the progress element — and its current step label —
      // instead of going blank.
      if (window.GBTurn) GBTurn.touch();
      return;
    }
    if (ChatState.isStreaming) {
      finalizeStreaming();
    }
    // The server can emit the same completion more than once (stream end +
    // final frame + retry). Without this guard each copy became a separate
    // bubble and a duplicate local-history entry.
    var completed = finalContent && finalContent.trim() !== ""
      ? { content: finalContent, reasoning: "" }
      : { content: "", reasoning: finalReasoning };
    var previous = ChatState.lastCompletedTurn;
    var duplicateCompletion = previous &&
      previous.content === completed.content &&
      previous.reasoning === completed.reasoning &&
      // Retries can land several seconds apart; a new user turn clears this
      // marker, so a long window only suppresses the server's own duplicates.
      Date.now() - previous.at < 60000;
    if (duplicateCompletion) {
      gbFinishTurn(data, completed);
      return;
    }
    if (completed.content) {
      addMessage("bot", completed.content, null, finalReasoning);
      if (TTSState.autoSpeak) speakText(completed.content);
    } else if (completed.reasoning && completed.reasoning.trim() !== "") {
      addMessage("bot", "", null, completed.reasoning);
      if (TTSState.autoSpeak) speakText(completed.reasoning);
    }
    ChatState.lastCompletedTurn = {
      content: completed.content,
      reasoning: completed.reasoning,
      at: Date.now(),
    };
    // The final bubble can be rendered by a pane renderer that skips the
    // addMessage hydration; re-run it on the visible last bot turn so raw
    // `[name](vmfile:...)` text always becomes attachment cards.
    try {
      var bots = document.querySelectorAll("#messages .message.bot");
      var lastBot = bots.length ? bots[bots.length - 1] : null;
      if (lastBot && typeof hydrateVmAttachments === "function") {
        hydrateVmAttachments(lastBot);
      }
      if (lastBot && typeof enforceNoVmFileLinks === "function") {
        enforceNoVmFileLinks(lastBot);
      }
      if (lastBot && window.ChatCodeUX && typeof ChatCodeUX.collapseLong === "function") {
        ChatCodeUX.collapseLong(lastBot);
      }
    } catch (e) { /* hydration is best-effort */ }
    ChatState.isStreaming = false;
    ChatState.currentReasoning = "";
    gbChatStatus({ state: "ready" });
    gbFinishTurn(data, completed);
    // Consumers that react to a finished turn (voice/full-duplex, proactivity)
    // must fire once, on completion — not on the first streaming chunk.
    try {
      window.dispatchEvent(new CustomEvent("gb-chat-bot-complete", {
        detail: { content: finalContent || finalReasoning || "" },
      }));
    } catch (e) { /* CustomEvent unavailable */ }
    if (data.suggestions && Array.isArray(data.suggestions) && data.suggestions.length > 0) {
      renderSuggestions(data.suggestions);
    }
    if (data.switchers && Array.isArray(data.switchers) && data.switchers.length > 0) {
      // The stock socket speaks for the default conversation even while a
      // parallel pane is mounted — render into the stock strip, not the pane.
      renderBotSwitchers(data.switchers, "tab-default");
    }
  } else if ((data.content && data.content.trim() !== "") || (data.reasoning && data.reasoning.trim() !== "")) {
    // First visible content: the turn is streaming and the bubble replaces the
    // spinner, but the turn stays open until the completion frame (#2098).
    if (window.GBTurn) GBTurn.visible();
    if (!ChatState.isStreaming) {
      ChatState.isStreaming = true;
      ChatState.streamingMessageId = "streaming-" + Date.now();
      ChatState.currentStreamingContent = data.content || "";
      ChatState.currentReasoning = data.reasoning || "";
      addMessage("bot", ChatState.currentStreamingContent, ChatState.streamingMessageId, ChatState.currentReasoning);
      ChatState.lastRenderTime = Date.now();
      gbChatStatus({ state: "running", text: gbChatStatusText("status-generating", "Generating…") });
    } else {
      if (data.reasoning) {
        ChatState.currentReasoning = (ChatState.currentReasoning || "") + data.reasoning;
      }
      if (data.content) {
        ChatState.currentStreamingContent += data.content;
      }
      var now = Date.now();
      if (now - ChatState.lastRenderTime > ChatState.renderInterval) {
        updateStreaming(ChatState.currentStreamingContent);
      }
    }
  }
}
