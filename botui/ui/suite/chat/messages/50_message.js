/* Chat messages module: addMessage: build the message bubble for user and bot turns. */
function chatNotice(message, type) {
  if (typeof window.showToast === "function") {
    window.showToast(message, type);
    return;
  }
  if (window.GBToasts && typeof window.GBToasts.show === "function") {
    window.GBToasts.show("Chat", message, type || "info");
    return;
  }
  console.info(message);
}
// #1797 — long user turns start collapsed so a pasted essay does not push the
// whole conversation off screen. Small turns are untouched.
var GB_USER_COLLAPSE_CHARS = 600;
var GB_USER_COLLAPSE_LINES = 8;

function shouldCollapseUserMessage(text) {
  if (!text) return false;
  return text.length > GB_USER_COLLAPSE_CHARS ||
    text.split("\n").length > GB_USER_COLLAPSE_LINES;
}

function collapseUserMessageIfNeeded(div, text) {
  var textBox = div && div.querySelector(".gb-message-text");
  if (!textBox || !shouldCollapseUserMessage(text)) return;
  var toggle = document.createElement("button");
  toggle.type = "button";
  toggle.className = "gb-message-toggle";
  toggle.textContent = "Show more";
  toggle.setAttribute("aria-expanded", "false");
  textBox.classList.add("gb-message-collapsed");
  toggle.addEventListener("click", function () {
    var collapsed = textBox.classList.toggle("gb-message-collapsed");
    toggle.textContent = collapsed ? "Show more" : "Show less";
    toggle.setAttribute("aria-expanded", collapsed ? "false" : "true");
  });
  textBox.parentNode.insertBefore(toggle, textBox.nextSibling);
}

function renderToolExecHeader(toolExec) {
  if (!toolExec) return "";
  var name = String(toolExec.tool_name || toolExec.name || toolExec.label || "tool");
  var desc = String(toolExec.tool_description || toolExec.description || "");
  var icon = String(toolExec.tool_icon || "").trim();
  var params = toolExec.tool_params || toolExec.params;
  var metaParts = [];
  if (icon) metaParts.push(icon);
  if (Array.isArray(params) && params.length) {
    metaParts.push(params.length + " param" + (params.length === 1 ? "" : "s"));
  }
  var detail = desc || metaParts.join(" · ");
  if (window.ChatCodeUX && typeof window.ChatCodeUX.toolCardHtml === "function") {
    return window.ChatCodeUX.toolCardHtml({
      id: toolExec.id || ("tool-exec-" + Date.now()),
      tool_name: name,
      status: "done",
      argsDetail: detail,
      summary: "Tool result",
    });
  }
  function esc(v) {
    if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(v);
    return String(v == null ? "" : v)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }
  return '<div class="gb-tool-card gb-tool-done" data-tool-kind="tool-exec">' +
    '<span class="gb-tool-icon gb-tool-done">✓</span>' +
    '<div class="gb-tool-info">' +
    '<div class="gb-tool-name">' + esc(name) + '</div>' +
    (detail ? '<div class="gb-tool-args">' + esc(detail) + '</div>' : '') +
    '<div class="gb-tool-summary">Tool result</div>' +
    '</div></div>';
}

/* #2349 — an empty conversation used to render nothing at all, which is
 * indistinguishable from a failed load. One card, shown only while the
 * transcript has no bubbles. */
function renderChatEmptyState() {
  var messages = document.getElementById("messages");
  if (!messages) return;
  if (messages.querySelector(".message")) return;
  var existing = document.getElementById("gb-chat-empty");
  if (existing) return;
  var botName = (ChatState.currentBotId || "your assistant");
  var card = document.createElement("div");
  card.id = "gb-chat-empty";
  card.className = "gb-chat-empty";
  card.setAttribute("role", "status");
  var title = document.createElement("p");
  title.className = "gb-chat-empty-title";
  title.textContent = "Start a conversation with " + botName;
  var hint = document.createElement("p");
  hint.className = "gb-chat-empty-hint";
  hint.textContent = "Type a message below, or pick a suggestion when your bot offers one.";
  card.appendChild(title);
  card.appendChild(hint);
  messages.appendChild(card);
}

function hideChatEmptyState() {
  var card = document.getElementById("gb-chat-empty");
  if (card && card.parentNode) card.parentNode.removeChild(card);
}

function addMessage(sender, content, msgId, reasoning, timestamp, meta) {
  var messages = ChatState.appendTarget || document.getElementById("messages");
  if (!messages) return;
  var liveMessages = document.getElementById("messages");
  if (sender === "bot") {
    // #2098 — an empty bot frame (thinking/status, tool boundary) must not
    // hide the progress element. Only visible content advances the turn; the
    // turn itself is closed by the completion frame in processMessage. History
    // replay renders bubbles too, so it never steers the live turn.
    var hasVisibleAnswer = (content && content.trim() !== "") ||
      (reasoning && reasoning.trim() !== "");
    if (hasVisibleAnswer && !ChatState.suppressPersist) {
      if (typeof window.clearChatPending === "function") window.clearChatPending();
      if (window.GBTurn) GBTurn.visible();
      else if (typeof hideThinkingIndicator === "function") hideThinkingIndicator();
    }
  }
  // Capture the user's position BEFORE the new bubble changes scrollHeight.
  // Measuring after the append made a long reply look like "user scrolled up"
  // and it never followed the newest message.
  var wasNearBottom = liveMessages &&
    (liveMessages.scrollHeight - liveMessages.scrollTop - liveMessages.clientHeight) < 140;
  // Do not re-persist messages that are being replayed from history: it
  // doubled the stored transcript on every reload (three identical greetings).
  // Streaming chunks are a live preview; the final answer is persisted once.
  var isStreamingChunk = msgId && String(msgId).indexOf("streaming-") === 0;
  if (content && sender && !ChatState.suppressPersist && !isStreamingChunk) {
    persistLocalMessage(sender, content);
  }

  var div = document.createElement("div");
  div.className = "message " + sender;
  if (msgId) div.id = msgId;
  // #2349 — every bubble is addressable. Ids used to be dropped at this
  // boundary (all call sites passed null), so no message could be targeted for
  // delete/edit/feedback or scoped auditing. Server ids are kept when present;
  // otherwise the bubble gets a stable local id.
  var addressableId = msgId ||
    ("msg-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 8));
  div.dataset.messageId = String(addressableId);
  div.dataset.role = sender;
  if (typeof content === "string") div.dataset.originalContent = content;
  // #2360 — structured failure signal only (never prose-matched): callers may
  // mark a bot turn explicitly retryable via meta or data attributes.
  var retryable = !!(meta && (
    meta.retryable || meta.error || meta.failed || meta.isError || meta.is_error ||
    meta.status === "error" || meta.status === "failed" ||
    meta.turn_status === "error" || meta.turn_status === "failed"
  ));
  if (!retryable && meta && typeof meta === "object" && meta.dataset && meta.dataset.gbRetryable === "1") retryable = true;
  if (retryable) div.dataset.gbRetryable = "1";
  hideChatEmptyState();
  // Hover timestamp. Replayed history keeps its own stored time; live turns
  // are stamped now. Server history currently carries no timestamp, so those
  // bubbles simply omit the hover label rather than inventing one.
  var messageTs = null;
  if (typeof timestamp === "number") {
    messageTs = timestamp;
  } else if (timestamp) {
    var parsedTs = Date.parse(timestamp);
    if (!isNaN(parsedTs)) messageTs = parsedTs;
  }
  if (!messageTs && !ChatState.suppressPersist) messageTs = Date.now();
  function stampMessage() {
    if (messageTs && typeof attachMessageTimestamp === "function") {
      attachMessageTimestamp(div, messageTs);
    }
  }

  if (sender !== "user") setChatStatus("");
  var toolExecMeta = (sender === "bot" && meta && meta.toolExec) ? meta.toolExec : null;
  if (toolExecMeta) {
    div.dataset.gbToolExecHeader = renderToolExecHeader(toolExecMeta);
  }
  var userAttachmentName = null;
  if (sender === "user") {
    // A new user turn always allows the next completion through, even if it
    // repeats the previous answer verbatim.
    ChatState.lastCompletedTurn = null;
    // #2360 — store the structured turn payload for retry (attachments/text).
    if (ChatState && ChatState.lastUserTurnPayload) {
      div._gbRetryPayload = ChatState.lastUserTurnPayload;
    }
    // #1641 — persisted user turns carry a trailing `📎 name`. Show it once,
    // as a chip, instead of printing the marker twice (the text plus the
    // restored chip used to duplicate the same filename).
    var shownContent = content || "";
    var attachmentMatch = shownContent.match(/(?:^|\n)📎\s+([^\n]+)\s*$/);
    if (attachmentMatch) {
      userAttachmentName = attachmentMatch[1].trim();
      shownContent = shownContent.replace(attachmentMatch[0], "").trim();
    }
    var processedContent = renderMentionInMessage(escapeHtml(shownContent));
    div.innerHTML = '<div class="message-content user-message"><div class="gb-message-text">' +
      processedContent + "</div></div>";
    collapseUserMessageIfNeeded(div, shownContent);
    if (userAttachmentName) {
      // Several files in one turn were joined as `📎 a.pdf, b.png`; show one
      // chip per file so each one can be previewed on its own.
      var attachmentNames = userAttachmentName.split(/,\s+/).filter(Boolean);
      attachmentNames.forEach(function (singleName, index) {
        var restored = document.createElement("div");
        restored.className = "gb-attachment-restored";
        var restoredLabel = document.createElement("span");
        restoredLabel.textContent = "📎 " + singleName;
        restored.appendChild(restoredLabel);
        var order = document.createElement("span");
        order.className = "gb-attachment-order";
        order.textContent = String(index + 1);
        restored.appendChild(order);
        var badge = document.createElement("span");
        badge.className = "gb-content-type-badge gb-type-badge";
        badge.textContent = (typeof vmContentTypeBadge === "function")
          ? vmContentTypeBadge(singleName)
          : String(singleName).split(".").pop().toUpperCase();
        restored.appendChild(badge);
        var desc = document.createElement("span");
        desc.className = "gb-attachment-desc gb-attachment-minor";
        desc.textContent = (typeof vmAttachmentDescription === "function")
          ? vmAttachmentDescription(singleName)
          : "Uploaded file";
        restored.appendChild(desc);
        if (/(\.docx|\.xlsx|\.pptx|\.pdf|\.png|\.jpe?g|\.gif|\.webp|\.txt|\.md|\.csv|\.json|\.log)$/i.test(singleName)) {
          var previewButton = document.createElement("button");
          previewButton.type = "button";
          previewButton.className = "gb-attachment-btn";
          previewButton.textContent = "Preview";
          previewButton.addEventListener("click", function () {
            toggleUserAttachmentPreview(restored, singleName);
          });
          restored.appendChild(previewButton);
        }
        div.querySelector(".message-content").appendChild(restored);
      });
    }
    enrichMentionIds(div);
  } else {
    var thinkingHtml = renderThinkingSection(reasoning);
    // Extract the app from the raw text FIRST: the stripper removes the very
    // tool-call JSON that carries the infographic HTML on history replay.
    var embeddedApp = extractEmbeddedApp(content);
    content = stripToolCallJson(stripClientPlan(content));
    // #1693 — when a fenced drawing is present, the live preview owns it: the
    // body text is rendered without the fence so the SVG is not shown twice.
    var livePreview = (sender === "bot" && window.GBLiveArtifact)
      ? window.GBLiveArtifact.lastBlock(content)
      : null;
    var cleanContent = stripMarkdownBlocks(
      livePreview ? stripLiveArtifactBlock(content) : content
    );
    if (embeddedApp) {
      div.innerHTML = '<div class="message-content bot-message">' + thinkingHtml + renderToolExecHeader(toolExecMeta) + "</div>";
      div.querySelector(".message-content").appendChild(
        renderInlineArtifact(embeddedApp.html, embeddedApp.title)
      );
      messages.appendChild(div);
      stampMessage();
      if (content && content.trim() !== "") div.appendChild(createCopyButton(content));
      if (ChatState.isUserAtBottom && !ChatState.suppressPersist) scrollToBottom(false);
      if (!ChatState.suppressPersist) stickToBottom(1400);
      return;
    }
    // #1693 — a fenced/complex drawing must not take the document path: it
    // would dump the fence itself into the artifact iframe ("```svg" printed
    // above the picture) and skip the live preview. The preview owns it.
    if (!livePreview && looksLikeHtmlDocument(content)) {
      div.innerHTML = '<div class="message-content bot-message">' + thinkingHtml + renderToolExecHeader(toolExecMeta) + "</div>";
      div.querySelector(".message-content").appendChild(renderInlineArtifact(content));
      messages.appendChild(div);
      stampMessage();
      if (content && content.trim() !== "") div.appendChild(createCopyButton(content));
      if (ChatState.isUserAtBottom && !ChatState.suppressPersist) scrollToBottom(false);
      return;
    }
    var parsed;
    if (typeof marked !== "undefined" && marked.parse) {
      parsed = marked.parse(cleanContent);
    } else {
      var hasHtmlTags = /<\/?[a-zA-Z][^>]*>|<!--|-->/i.test(cleanContent);
      parsed = hasHtmlTags ? cleanContent : escapeHtml(cleanContent);
    }
    // #2354 — model output is untrusted; sanitize before any transform adds
    // trusted markup (deep-link buttons carry an onclick by design).
    parsed = GBSanitize.html(parsed);
    parsed = convertNumberedLists(parsed);
    parsed = renderInlineMarkdown(parsed);
    parsed = parsed.replace(/<br\s*\/?>/gi, '');
    parsed = renderMentionInMessage(parsed);
    parsed = renderDeepLinks(parsed);
    if (typeof renderVmFileLinks === "function") parsed = renderVmFileLinks(parsed);
    div.innerHTML = '<div class="message-content bot-message">' + thinkingHtml + renderToolExecHeader(toolExecMeta) + parsed + "</div>";
    if (content && content.trim() !== "") {
      div.appendChild(createCopyButton(content));
      div.appendChild(createSpeakButton(content));
    }
  }

  messages.appendChild(div);
  if (window.GBChatVirtual && typeof window.GBChatVirtual.onAfterAppend === "function") {
    try { window.GBChatVirtual.onAfterAppend(div); } catch (e) { /* virtualization best-effort */ }
  }
  if (sender === "bot" && window.GBLiveArtifact) {
    window.GBLiveArtifact.render(div, content);
  }
  // Merge before polish (#2008): polishOutputBlocks moves the tool output into
  // a details card, which made mergeWorkspaceOutput fail to find the `<pre>`
  // and left a second "workspace run" card behind.
  var mergedOutput = typeof mergeWorkspaceOutput === "function" &&
    mergeWorkspaceOutput(div);
  if (!mergedOutput && sender !== "user") polishOutputBlocks(div);

  // BBSIA-CHAT-02 (#1582): VM-generated files become attachment cards.
  if (
    sender === "bot" &&
    !(msgId && String(msgId).indexOf("streaming-") === 0) &&
    typeof hydrateVmAttachments === "function"
  ) {
    hydrateVmAttachments(div);
  }
  // #2167 hard invariant: never leave a clickable vmfile: anchor on screen.
  if (sender === "bot" && typeof enforceNoVmFileLinks === "function") {
    enforceNoVmFileLinks(div);
  }
  if (sender === "bot" && window.ChatCodeUX) {
    ChatCodeUX.enhance(div);
    if (typeof ChatCodeUX.collapseLong === "function") ChatCodeUX.collapseLong(div);
    var toolCards = ChatCodeUX.extractToolCards(content);
    toolCards.forEach(function (tool) {
      if (tool.status) ChatCodeUX.appendCard(tool);
    });
  }
  // #2360 — retry must come from an explicit failure signal, never from
  // matching model prose (fragile under i18n and false-positives on any answer
  // that happens to contain those words). Structured retry rendering is wired
  // where a frame/message is explicitly marked retryable (e.g. data-error /
  // meta.retryable); absent that signal we intentionally show no retry here.
  if (sender === "bot") {
    var isRetryable = !!(div.dataset && div.dataset.gbRetryable === "1");
    if (isRetryable) {
      var retryBtn = document.createElement("button");
      retryBtn.type = "button";
      retryBtn.className = "gb-retry-btn";
      retryBtn.textContent = (window.i18n && typeof window.i18n.t === "function" && window.i18n.t("chat-retry") !== "chat-retry")
        ? window.i18n.t("chat-retry")
        : "Retry";
      retryBtn.addEventListener("click", function () {
        var sourceUser = null;
        var prev = div.previousElementSibling;
        while (prev) {
          if (prev.classList && prev.classList.contains("message") && prev.classList.contains("user")) {
            sourceUser = prev;
            break;
          }
          prev = prev.previousElementSibling;
        }
        if (!sourceUser) {
          var users = messages.querySelectorAll(".message.user");
          sourceUser = users.length ? users[users.length - 1] : null;
        }
        var payload = sourceUser && sourceUser._gbRetryPayload ? sourceUser._gbRetryPayload : null;
        if (payload && typeof window.gbResendChatPayload === "function") {
          window.gbResendChatPayload(payload);
          return;
        }
        // #2360 — fallback only when structured payload is unavailable
        // (e.g. history replay). Strip attachment markers so we never resend
        // scraped "📎 ..." placeholders as message content.
        var text = "";
        if (sourceUser && sourceUser.dataset && typeof sourceUser.dataset.originalContent === "string") {
          text = sourceUser.dataset.originalContent.trim();
        } else if (sourceUser) {
          text = (sourceUser.querySelector(".gb-message-text") || sourceUser.querySelector(".message-content") || sourceUser).innerText.trim();
        }
        if (text) {
          text = text.replace(/(?:^|\n)📎\s+[^\n]+$/gm, "").trim();
        }
        if (text && typeof window.sendMessage === "function") {
          window.sendMessage(text);
        }
      });
      var retryTarget = div.querySelector(".message-content");
      if (retryTarget) retryTarget.appendChild(retryBtn);
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

  // #2163 — adding a *user* message used to force the viewport to the bottom
  // even when the reader had scrolled up, which is what made Enter feel like a
  // jump. Follow the live edge only when the reader is already there; otherwise
  // leave the scroll alone and let the jump-to-bottom button advertise the new
  // message. `undefined` (first message of a session) counts as at-bottom.
  if (ChatState.isUserAtBottom !== false && !ChatState.suppressPersist) {
    scrollToBottom(false);
    stickToBottom(sender === "user" ? 300 : 1400);
  } else {
    updateScrollButton();
  }

  setupMentionClickHandlers(div);
  if (!ChatState.suppressPersist && typeof scheduleScrollbarMarkers === "function") scheduleScrollbarMarkers();
}

function isTagBalanced(html) {
  if (!html) return true;
  var lastChevronOpen = html.lastIndexOf('<');
  var lastChevronClose = html.lastIndexOf('>');
  if (lastChevronOpen > lastChevronClose) return false;
  return true;
}
