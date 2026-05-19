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

function renderMentionInMessage(content) {
  return content.replace(/@(\w+):([^\s]+)/g, function (match, type, name) {
    var entityType = EntityTypes[type.toLowerCase()];
    if (entityType) {
      return '<span class="mention-tag" data-type="' + type + '" data-name="' + escapeHtml(name) + '">' +
        '<span class="mention-icon">' + entityType.icon + "</span>" +
        '<span class="mention-text">@' + type + ":" + escapeHtml(name) + "</span>" +
        "</span>";
    }
    return match;
  });
}

function stripThinkTags(content) {
  // R6: Remove <think>...</think> but do NOT trim — preserves leading '<' in HTML chunks
  return content.replace(/<think>[\s\S]*?(?:<\/think>|$)/gi, "");
}

function stripReasoningPrefix(content) {
  // Nemotron and reasoning models output chain-of-thought before actual response
  // Strip everything before the first HTML tag (<div, <p, <h, etc.)
  var htmlStart = content.search(/<[a-zA-Z]/);
  if (htmlStart > 0) {
    return content.substring(htmlStart);
  }
  return content;
}

function stripMarkdownBlocks(content) {
  var cleanContent = stripThinkTags(content);
  cleanContent = stripReasoningPrefix(cleanContent);
  var hasHtmlTags = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
  if (hasHtmlTags) return cleanContent;
  var htmlMatch = cleanContent.match(/^```(?:html|xml)?\s*\n([\s\S]+?)\n?```$/i);
  if (htmlMatch) return htmlMatch[1];
  return cleanContent;
}

function stripMarkdownBlocks(content) {
  var cleanContent = stripThinkTags(content);
  var hasHtmlTags = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
  if (hasHtmlTags) return cleanContent;
  var htmlMatch = cleanContent.match(/^```(?:html|xml)?\s*\n([\s\S]+?)\n?```$/i);
  if (htmlMatch) return htmlMatch[1];
  return cleanContent;
}

function addMessage(sender, content, msgId) {
var messages = document.getElementById("messages");
if (!messages) return;

var div = document.createElement("div");
div.className = "message " + sender;
if (msgId) div.id = msgId;

  if (sender === "user") {
    var processedContent = renderMentionInMessage(escapeHtml(content));
    div.innerHTML = '<div class="message-content user-message">' + processedContent + "</div>";
  } else {
    var cleanContent = stripMarkdownBlocks(content);
    var hasHtmlTags = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
    var parsed;
    if (hasHtmlTags) {
      // F3: HTML content from LLM — render raw via innerHTML (never textContent)
      parsed = cleanContent;
    } else if (msgId) {
      // Streaming message with no HTML yet — show placeholder
      parsed = "";
    } else {
      parsed = escapeHtml(cleanContent);
    }
    parsed = renderMentionInMessage(parsed);
    div.innerHTML = '<div class="message-content bot-message">' + parsed + "</div>";
  }

  messages.appendChild(div);

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

function updateStreaming(content) {
  var el = document.getElementById(ChatState.streamingMessageId);
  if (!el) return;

  var msgContent = el.querySelector(".message-content");
  var cleanContent = stripMarkdownBlocks(content);
  var isHtml = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);

  if (isHtml) {
    // F3+F5: Render HTML chunks directly via innerHTML += (never textContent/innerText)
    // For streaming HTML, set full accumulated content — partial tags won't render, but completed ones will
    var parsed = renderMentionInMessage(cleanContent);
    msgContent.innerHTML = parsed;
    if (!ChatState.isUserScrolling) scrollToBottom(true);
  } else {
    var parsed = typeof marked !== "undefined" && marked.parse
      ? marked.parse(cleanContent)
      : escapeHtml(cleanContent);
    parsed = renderMentionInMessage(parsed);
    msgContent.innerHTML = parsed;
    if (!ChatState.isUserScrolling) scrollToBottom(true);
  }
}

function finalizeStreaming() {
  var el = document.getElementById(ChatState.streamingMessageId);
  if (el) {
    var cleanContent = stripMarkdownBlocks(ChatState.currentStreamingContent);
    var hasHtmlTags = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
    var parsed;
    if (hasHtmlTags) {
      parsed = cleanContent;
    } else {
      parsed = typeof marked !== "undefined" && marked.parse
        ? marked.parse(cleanContent)
        : escapeHtml(cleanContent);
    }
    parsed = renderMentionInMessage(parsed);
    el.querySelector(".message-content").innerHTML = parsed;
    el.removeAttribute("id");
    setupMentionClickHandlers(el);
    if (!ChatState.isUserScrolling) scrollToBottom(true);
  }
  ChatState.streamingMessageId = null;
  ChatState.currentStreamingContent = "";
  ChatState.streamingBuffer = "";
}

function processMessage(data) {
  if (data.thinking) {
    showThinkingIndicator();
    return;
  }
  hideThinkingIndicator();
  if (data.is_complete) {
    if (ChatState.isStreaming) {
      finalizeStreaming();
    } else if (data.content && data.content.trim() !== "") {
      addMessage("bot", data.content);
    }
    ChatState.isStreaming = false;
    if (data.suggestions && Array.isArray(data.suggestions) && data.suggestions.length > 0) {
      renderSuggestions(data.suggestions);
    }
    if (data.switchers && Array.isArray(data.switchers) && data.switchers.length > 0) {
      renderBotSwitchers(data.switchers);
    }
  } else if (data.content && data.content.trim() !== "") {
    if (!ChatState.isStreaming) {
      ChatState.isStreaming = true;
      ChatState.streamingMessageId = "streaming-" + Date.now();
      ChatState.currentStreamingContent = data.content;
      addMessage("bot", ChatState.currentStreamingContent, ChatState.streamingMessageId);
      ChatState.lastRenderTime = Date.now();
    } else {
      ChatState.currentStreamingContent += data.content;
      var now = Date.now();
      if (now - ChatState.lastRenderTime > ChatState.renderInterval) {
        updateStreaming(ChatState.currentStreamingContent);
      }
    }
  }
}
