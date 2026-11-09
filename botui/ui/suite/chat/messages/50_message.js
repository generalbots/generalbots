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

function addMessage(sender, content, msgId, reasoning, timestamp) {
  var messages = document.getElementById("messages");
  if (!messages) return;
  if (sender === "bot") {
    if (typeof window.clearChatPending === "function") window.clearChatPending();
    if (typeof hideThinkingIndicator === "function") hideThinkingIndicator();
  }
  // Capture the user's position BEFORE the new bubble changes scrollHeight.
  // Measuring after the append made a long reply look like "user scrolled up"
  // and it never followed the newest message.
  var wasNearBottom = (messages.scrollHeight - messages.scrollTop - messages.clientHeight) < 140;
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
  var userAttachmentName = null;
  if (sender === "user") {
    // A new user turn always allows the next completion through, even if it
    // repeats the previous answer verbatim.
    ChatState.lastCompletedTurn = null;
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
      div.innerHTML = '<div class="message-content bot-message">' + thinkingHtml + "</div>";
      div.querySelector(".message-content").appendChild(
        renderInlineArtifact(embeddedApp.html, embeddedApp.title)
      );
      messages.appendChild(div);
      stampMessage();
      if (ChatState.isUserAtBottom) scrollToBottom(true);
      stickToBottom(1400);
      return;
    }
    if (looksLikeHtmlDocument(content)) {
      div.innerHTML = '<div class="message-content bot-message">' + thinkingHtml + "</div>";
      div.querySelector(".message-content").appendChild(renderInlineArtifact(content));
      messages.appendChild(div);
      stampMessage();
      if (content && content.trim() !== "") div.appendChild(createCopyButton(content));
      if (ChatState.isUserAtBottom) scrollToBottom(true);
      return;
    }
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
      div.appendChild(createCopyButton(content));
      div.appendChild(createSpeakButton(content));
    }
  }

  messages.appendChild(div);
  if (sender === "bot" && window.GBLiveArtifact) {
    window.GBLiveArtifact.render(div, content);
  }
  if (typeof mergeWorkspaceOutput === "function") mergeWorkspaceOutput(div);

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

  if (sender === "user" || ChatState.isUserAtBottom) {
    scrollToBottom(sender !== "user");
    stickToBottom(sender === "user" ? 300 : 1400);
  } else {
    updateScrollButton();
  }

  setupMentionClickHandlers(div);
  if (typeof scheduleScrollbarMarkers === "function") scheduleScrollbarMarkers();
}

function isTagBalanced(html) {
  if (!html) return true;
  var lastChevronOpen = html.lastIndexOf('<');
  var lastChevronClose = html.lastIndexOf('>');
  if (lastChevronOpen > lastChevronClose) return false;
  return true;
}
