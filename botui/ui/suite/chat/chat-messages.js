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

function stripMarkdownBlocks(content) {
  var htmlMatch = content.match(/^```(?:html|xml)?\s*\n([\s\S]+?)\n?```$/i);
  if (htmlMatch) return htmlMatch[1].trim();
  return content;
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
    if (msgId) {
      parsed = '<div class="streaming-loading"><span class="loading-dots">...</span></div>';
    } else if (hasHtmlTags) {
      parsed = cleanContent;
    } else {
      parsed = typeof marked !== "undefined" && marked.parse
        ? marked.parse(cleanContent)
        : escapeHtml(cleanContent);
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
    if (isTagBalanced(cleanContent) || (Date.now() - ChatState.lastRenderTime > 2000)) {
      msgContent.innerHTML = renderMentionInMessage(cleanContent);
      ChatState.lastRenderTime = Date.now();
      if (!ChatState.isUserScrolling) scrollToBottom(true);
    }
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
    var parsed = hasHtmlTags
      ? cleanContent
      : (typeof marked !== "undefined" && marked.parse
        ? marked.parse(cleanContent)
        : escapeHtml(cleanContent));
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
  } else {
    if (!ChatState.isStreaming) {
      ChatState.isStreaming = true;
      ChatState.streamingMessageId = "streaming-" + Date.now();
      ChatState.currentStreamingContent = data.content || "";
      addMessage("bot", ChatState.currentStreamingContent, ChatState.streamingMessageId);
      ChatState.lastRenderTime = Date.now();
    } else {
      ChatState.currentStreamingContent += data.content || "";
      var now = Date.now();
      if (now - ChatState.lastRenderTime > ChatState.renderInterval) {
        updateStreaming(ChatState.currentStreamingContent);
      }
    }
  }
}
