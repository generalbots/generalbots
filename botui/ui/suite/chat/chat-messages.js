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

function stripSectorInfo(content) {
  var c = content.replace(/[-–—]\s*(Setor|Departamento|Cargo|Se[cç]ão|Enfermaria|Administra[çc][aã]o)\s*[^.<>]*/gi, "");
  return c.replace(/Ramal(\d)/g, "Ramal $1");
}

function stripMarkdownBlocks(content) {
  var cleanContent = stripThinkTags(content);
  cleanContent = stripReasoningPrefix(cleanContent);
  cleanContent = stripSectorInfo(cleanContent);
  // Unwrap markdown-language fences embedded in prose (before trailing-fence strip,
  // which would otherwise remove the closing ``` and leave the block unwrapped)
  cleanContent = cleanContent.replace(/(^|\n)```(?:markdown|md|text|plain)\s*\n([\s\S]*?)\n?```\s*(\n|$)/gi, "$1$2$3");
  // Unwrap whole-message code fences (HTML/XML/markdown/text) so tables and markup
  // render as HTML instead of displaying as raw markdown
  cleanContent = cleanContent.replace(/^```(?:html|xml|markdown|md|text|plain)?\s*\n?/gi, "").replace(/\n?```\s*$/gi, "");
  var hasHtmlTags = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
  if (hasHtmlTags) {
    cleanContent = stripSectorInfo(cleanContent);
    return cleanContent;
  }
  var htmlMatch = cleanContent.match(/^```(?:html|xml|markdown|md|text|plain)?\s*\n([\s\S]+?)\n?```$/i);
  if (htmlMatch) return htmlMatch[1];
  return cleanContent;
}

function reexecuteScripts(container) {
  var scripts = container.querySelectorAll('script');
  for (var i = 0; i < scripts.length; i++) {
    try {
      var text = scripts[i].textContent || '';
      if (text.trim()) {
        var fn = new Function(text);
        fn();
      }
    } catch (e) {
      console.warn('Script exec error:', e);
    }
  }
}

function renderThinkingSection(reasoning) {
  if (!reasoning || reasoning.trim() === "") return "";
  var safeReasoning = escapeHtml(reasoning);
  return '<details class="thinking-section">' +
    '<summary><span class="thinking-summary-icon">\u{1F9E0}</span> Thinking</summary>' +
    '<div class="thinking-content">' + safeReasoning + '</div>' +
    '</details>';
}

/* v3-indicator-fixed */

function convertNumberedLists(html) {
  var lines = html.split('\n');
  var result = [];
  var group = null;

  for (var i = 0; i < lines.length; i++) {
    var trimmed = lines[i].trim();
    var m = trimmed.match(/^(\d+)\.\s+([\s\S]*)/);

    if (m) {
      if (!group) group = [];
      group.push({ num: parseInt(m[1], 10), content: m[2] });
    } else {
      if (group) {
        if (group.length >= 2) flushGroup();
        else result.push(group.map(function(g) { return g.num + '. ' + g.content; }).join('\n'));
        group = null;
      }
      result.push(lines[i]);
    }
  }
  if (group) {
    if (group.length >= 2) flushGroup();
    else result.push(group.map(function(g) { return g.num + '. ' + g.content; }).join('\n'));
  }

  function flushGroup() {
    var startAttr = group[0].num !== 1 ? ' start="' + group[0].num + '"' : '';
    var lis = group.map(function(it) { return '<li>' + it.content.trim() + '</li>'; }).join('');
    result.push('<ol' + startAttr + '>' + lis + '</ol>');
  }

  return result.join('\n');
}

function addMessage(sender, content, msgId, reasoning) {
  var messages = document.getElementById("messages");
  if (!messages) return;

  var div = document.createElement("div");
  div.className = "message " + sender;
  if (msgId) div.id = msgId;

  if (sender === "user") {
    var processedContent = renderMentionInMessage(escapeHtml(content));
    div.innerHTML = '<div class="message-content user-message">' + processedContent + "</div>";
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
    parsed = parsed.replace(/<br\s*\/?>/gi, '');
    parsed = renderMentionInMessage(parsed);
    div.innerHTML = '<div class="message-content bot-message">' + thinkingHtml + parsed + "</div>";
  }

  messages.appendChild(div);

  reexecuteScripts(div);

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
  var parsed;
  if (typeof marked !== "undefined" && marked.parse) {
    parsed = marked.parse(cleanContent);
  } else {
    var isHtml = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
    parsed = isHtml ? cleanContent : escapeHtml(cleanContent);
  }
  parsed = convertNumberedLists(parsed);
  parsed = parsed.replace(/<br\s*\/?>/gi, '');
  parsed = renderMentionInMessage(parsed);
  var thinkingHtml = renderThinkingSection(ChatState.currentReasoning);
  msgContent.innerHTML = thinkingHtml + parsed;
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
    } else if ((data.reasoning || ChatState.currentReasoning) && (data.reasoning || ChatState.currentReasoning).trim() !== "") {
      addMessage("bot", "", null, data.reasoning || ChatState.currentReasoning);
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
