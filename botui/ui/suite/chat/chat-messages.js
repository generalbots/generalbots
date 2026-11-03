function scrollToBottom(animate) {
  var messages = document.getElementById("messages");
  if (messages) {
    if (animate) {
      messages.scrollTo({ top: messages.scrollHeight, behavior: "smooth" });
    } else {
      messages.scrollTop = messages.scrollHeight;
    }
  }
}

function showThinkingIndicator() {
  var existing = document.getElementById("thinking-indicator");
  if (existing) return;
  var messages = document.getElementById("messages");
  if (!messages) return;
  var div = document.createElement("div");
  div.id = "thinking-indicator";
  div.className = "message bot";
  div.innerHTML = '<div class="message-content bot-message"><div class="thinking-indicator"><div class="thinking-dots"><div class="thinking-dot"></div><div class="thinking-dot"></div><div class="thinking-dot"></div></div></div></div>';
  messages.appendChild(div);
  scrollToBottom(true);
}

function hideThinkingIndicator() {
  var el = document.getElementById("thinking-indicator");
  if (el) el.remove();
}

function updateScrollButton() {
  var messages = document.getElementById("messages");
  var scrollBtn = document.getElementById("scrollToBottom");
  if (!messages || !scrollBtn) return;
  var isNearBottom = messages.scrollHeight - messages.scrollTop - messages.clientHeight < 100;
  if (isNearBottom) {
    scrollBtn.classList.remove("visible");
  } else {
    scrollBtn.classList.add("visible");
  }
}

function persistLocalMessage(sender, content) {
  try {
    var key = "gb_chat_history_" + (ChatState.currentSessionId || "default");
    var history = JSON.parse(localStorage.getItem(key) || "[]");
    history.push({ sender: sender, content: content, ts: Date.now() });
    if (history.length > 200) history = history.slice(-200);
    localStorage.setItem(key, JSON.stringify(history));
  } catch (e) {}
}

function loadLocalHistory() {
  try {
    var key = "gb_chat_history_" + (ChatState.currentSessionId || "default");
    return JSON.parse(localStorage.getItem(key) || "[]");
  } catch (e) { return []; }
}

function listLocalSessions() {
  var sessions = [];
  try {
    for (var i = 0; i < localStorage.length; i++) {
      var k = localStorage.key(i);
      if (k && k.indexOf("gb_chat_history_") === 0) {
        var id = k.replace("gb_chat_history_", "");
        var msgs = JSON.parse(localStorage.getItem(k) || "[]");
        if (msgs.length > 0) {
          sessions.push({ id: id, last_message: msgs[msgs.length - 1].content || "", ts: msgs[msgs.length - 1].ts || 0, count: msgs.length });
        }
      }
    }
    sessions.sort(function (a, b) { return b.ts - a.ts; });
  } catch (e) {}
  return sessions;
}

// #1547 — live status line while the assistant works (search decision,
// lookup, sources). Cleared as soon as the answer starts.
function setChatStatus(text) {
  var messages = document.getElementById("messages");
  if (!messages) return;
  var box = document.getElementById("gbChatStatus");
  if (!text) {
    if (box) box.remove();
    return;
  }
  if (!box) {
    box = document.createElement("div");
    box.id = "gbChatStatus";
    box.className = "chat-status";
    box.setAttribute("role", "status");
    box.setAttribute("aria-live", "polite");
    messages.appendChild(box);
  }
  box.innerHTML = '<span class="chat-status-dot"></span><span class="chat-status-text"></span>';
  box.querySelector(".chat-status-text").textContent = text;
  messages.scrollTop = messages.scrollHeight;
}

function addMessage(sender, content, msgId, reasoning) {
  var messages = document.getElementById("messages");
  if (!messages) return;
  if (content && sender) persistLocalMessage(sender, content);

  var div = document.createElement("div");
  div.className = "message " + sender;
  if (msgId) div.id = msgId;

  if (sender !== "user") setChatStatus("");
  if (sender === "user") {
    var processedContent = renderMentionInMessage(escapeHtml(content));
    div.innerHTML = '<div class="message-content user-message">' + processedContent + "</div>";
    enrichMentionIds(div);
  } else {
    var thinkingHtml = renderThinkingSection(reasoning);
    var cleanContent = stripMarkdownBlocks(content);
    var parsed;
    if (typeof marked !== "undefined" && marked.parse) {
      parsed = marked.parse(cleanContent);
    } else {
      var hasHtmlTags = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
      parsed = hasHtmlTags ? cleanContent : escapeHtml(cleanContent);
    }
    parsed = convertNumberedLists(parsed);
    parsed = renderInlineMarkdown(parsed);
    parsed = parsed.replace(/<br\s*\/?>/gi, '');
    parsed = renderMentionInMessage(parsed);
    parsed = renderDeepLinks(parsed);
    if (typeof renderVmFileLinks === "function") parsed = renderVmFileLinks(parsed);
    div.innerHTML = '<div class="message-content bot-message">' + thinkingHtml + parsed + "</div>";
    polishOutputBlocks(div);
    if (content && content.trim() !== "") {
      div.appendChild(createSpeakButton(content));
    }
  }

  messages.appendChild(div);

  reexecuteScripts(div);
  // BBSIA-CHAT-02 (#1582): VM-generated files become attachment cards.
  if (
    sender === "bot" &&
    !(msgId && String(msgId).indexOf("streaming-") === 0) &&
    typeof hydrateVmAttachments === "function"
  ) {
    hydrateVmAttachments(div);
  }
  if (sender === "bot" && window.ChatCodeUX) {
    ChatCodeUX.enhance(div);
    var toolCards = ChatCodeUX.extractToolCards(content);
    toolCards.forEach(function (tool) {
      if (tool.status) ChatCodeUX.appendCard(tool);
    });
  }
  // Polish: a failed turn offers a one-click retry instead of dead text.
  if (
    sender === "bot" &&
    /(could not process|couldn't process|não consegui processar|nao consegui processar)/i.test(content)
  ) {
    var retryBtn = document.createElement("button");
    retryBtn.type = "button";
    retryBtn.className = "gb-retry-btn";
    retryBtn.textContent = "Retry";
    retryBtn.addEventListener("click", function () {
      var users = messages.querySelectorAll(".message.user");
      var last = users.length ? users[users.length - 1] : null;
      var text = last ? (last.querySelector(".message-content") || last).innerText.trim() : "";
      if (text && typeof window.sendMessage === "function") {
        window.sendMessage(text);
      }
    });
    var retryTarget = div.querySelector(".message-content");
    if (retryTarget) retryTarget.appendChild(retryBtn);
  }

  // #1641 — the persisted user turn carries `📎 name`; render it as a chip so
  // reopening the conversation still shows the attachment on the message.
  if (sender === "user" && content) {
    var attachment = content.match(/📎\s+([^\n]+)/);
    if (attachment) {
      var restored = document.createElement("div");
      restored.className = "gb-attachment-restored";
      restored.textContent = "📎 " + attachment[1].trim();
      var contentEl = div.querySelector(".message-content");
      if (contentEl) contentEl.appendChild(restored);
    }
  }

  if (sender === "bot" && content) {
    var artifact = window.GBArtifacts && window.GBArtifacts.detect(content);
    if (artifact) {
      var artifactBtn = document.createElement("button");
      artifactBtn.className = "gb-artifact-btn";
      artifactBtn.textContent = "Open as document";
      artifactBtn.addEventListener("click", function () {
        var msgContent = div.querySelector(".message-content");
        openArtifactPanel(content, msgContent);
      });
      var msgContent = div.querySelector(".message-content");
      if (msgContent) msgContent.appendChild(artifactBtn);
    }
  }

  if (!ChatState.isUserScrolling) {
    scrollToBottom(true);
  } else {
    updateScrollButton();
  }

  setupMentionClickHandlers(div);
}

function isTagBalanced(html) {
  if (!html) return true;
  var lastChevronOpen = html.lastIndexOf('<');
  var lastChevronClose = html.lastIndexOf('>');
  if (lastChevronOpen > lastChevronClose) return false;
  return true;
}

// ── TTS state + auto-read toggle (issue #708) ──
var TTSState = {
  autoSpeak: localStorage.getItem("gb_tts_auto") === "1"
};

function initTtsToggle() {
  var btn = document.getElementById("ttsToggle");
  if (!btn) return;
  function refresh() {
    btn.classList.toggle("active", TTSState.autoSpeak);
    btn.title = TTSState.autoSpeak ? "Stop auto-reading bot replies" : "Auto-read bot replies (TTS)";
  }
  btn.addEventListener("click", function () {
    TTSState.autoSpeak = !TTSState.autoSpeak;
    localStorage.setItem("gb_tts_auto", TTSState.autoSpeak ? "1" : "0");
    refresh();
    if (!TTSState.autoSpeak && "speechSynthesis" in window) {
      window.speechSynthesis.cancel();
    }
  });
  refresh();
}

document.addEventListener("DOMContentLoaded", initTtsToggle);

// ── TTS (issue #708) — speak a bot message aloud ──
function createSpeakButton(text) {
  var btn = document.createElement("button");
  btn.className = "tts-speak-btn";
  btn.type = "button";
  btn.title = "Listen (TTS)";
  btn.setAttribute("aria-label", "Listen to this message");
  btn.innerHTML = '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/></svg>';
  btn.addEventListener("click", function (e) {
    e.stopPropagation();
    speakText(text);
  });
  return btn;
}

function speakText(text) {
  try {
    if (!("speechSynthesis" in window)) {
      window.showToast && window.showToast("Text-to-speech is not supported in this browser", "warning");
      return;
    }
    window.speechSynthesis.cancel();
    var clean = text.replace(/[*_#`~]/g, "").replace(/\s+/g, " ").trim();
    if (!clean) return;
    var utterance = new SpeechSynthesisUtterance(clean);
    var preferred = "pt-BR";
    var voices = window.speechSynthesis.getVoices();
    if (voices.length) {
      var match = voices.filter(function (v) { return v.lang === preferred; })[0] || voices[0];
      utterance.voice = match;
    }
    utterance.rate = 1;
    utterance.pitch = 1;
    window.speechSynthesis.speak(utterance);
  } catch (err) {
    console.error("TTS failed:", err);
  }
}

function updateStreaming(content) {
  var el = document.getElementById(ChatState.streamingMessageId);
  if (!el) return;

  var msgContent = el.querySelector(".message-content");
  var cleanContent = stripMarkdownBlocks(content);
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
  if (window.ChatCodeUX) {
    ChatCodeUX.enhance(msgContent);
    var toolCards = ChatCodeUX.extractToolCards(content);
    toolCards.forEach(function (tool) {
      if (tool.status) ChatCodeUX.appendCard(tool);
    });
  }
  if (!ChatState.isUserScrolling) scrollToBottom(true);
}

function finalizeStreaming() {
  var el = document.getElementById(ChatState.streamingMessageId);
  if (el) {
    el.remove();
  }
  ChatState.streamingMessageId = null;
  ChatState.currentStreamingContent = "";
  ChatState.currentReasoning = "";
  ChatState.streamingBuffer = "";
}

function processMessage(data) {
  if (data.thinking) {
    if (!ChatState.isStreaming) {
      showThinkingIndicator();
    }
    return;
  }

  if (data.is_complete) {
    hideThinkingIndicator();
    if (ChatState.isStreaming) {
      finalizeStreaming();
    }
    if (data.content && data.content.trim() !== "") {
      addMessage("bot", data.content, null, data.reasoning || ChatState.currentReasoning);
      if (TTSState.autoSpeak) {
        speakText(data.content);
      }
    } else if ((data.reasoning || ChatState.currentReasoning) && (data.reasoning || ChatState.currentReasoning).trim() !== "") {
      addMessage("bot", "", null, data.reasoning || ChatState.currentReasoning);
      if (TTSState.autoSpeak) {
        speakText(data.reasoning);
      }
    }
    ChatState.isStreaming = false;
    ChatState.currentReasoning = "";
    if (data.suggestions && Array.isArray(data.suggestions) && data.suggestions.length > 0) {
      renderSuggestions(data.suggestions);
    }
    if (data.switchers && Array.isArray(data.switchers) && data.switchers.length > 0) {
      renderBotSwitchers(data.switchers);
    }
  } else if ((data.content && data.content.trim() !== "") || (data.reasoning && data.reasoning.trim() !== "")) {
    if (!ChatState.isStreaming) {
      ChatState.isStreaming = true;
      ChatState.streamingMessageId = "streaming-" + Date.now();
      ChatState.currentStreamingContent = data.content || "";
      ChatState.currentReasoning = data.reasoning || "";
      addMessage("bot", ChatState.currentStreamingContent, ChatState.streamingMessageId, ChatState.currentReasoning);
      ChatState.lastRenderTime = Date.now();
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
