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

// Instant snap used after layout-changing content (history replay, inline
// artifact iframes). Smooth scrolling here finished before the iframe grew.
function snapToBottom() {
  var messages = document.getElementById("messages");
  if (messages) messages.scrollTop = messages.scrollHeight;
}

// Keep the live edge pinned for a short window while async layout (inline
// iframes, images, markdown) grows the last message. Without this the scroll
// landed before the artifact finished and the reply stayed below the fold.
function stickToBottom(duration) {
  var end = Date.now() + (duration || 1200);
  var timer = setInterval(function () {
    var messages = document.getElementById("messages");
    if (!messages) { clearInterval(timer); return; }
    messages.scrollTop = messages.scrollHeight;
    if (Date.now() > end) clearInterval(timer);
  }, 120);
}

// History replay appends messages whose inline artifacts and images load
// lazily, so the transcript keeps growing after the first scroll. Keep the
// newest turn in view until the layout settles, and stop the moment the
// reader scrolls away (their intent wins).
function followLiveEdge(maxMs) {
  var messages = document.getElementById("messages");
  if (!messages) return;
  var end = Date.now() + (maxMs || 12000);
  var cancelled = false;
  function cancel() { cancelled = true; cleanup(); }
  function cleanup() {
    messages.removeEventListener("wheel", cancel);
    messages.removeEventListener("touchstart", cancel);
    messages.removeEventListener("keydown", cancel);
  }
  messages.addEventListener("wheel", cancel, { passive: true });
  messages.addEventListener("touchstart", cancel, { passive: true });
  messages.addEventListener("keydown", cancel);
  function step() {
    if (cancelled) return;
    messages.scrollTop = messages.scrollHeight;
    if (Date.now() < end) requestAnimationFrame(step);
    else cleanup();
  }
  requestAnimationFrame(step);
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

// A model sometimes returns a complete HTML app instead of prose. It must
// never be injected into the chat DOM (styles leak, scripts execute, and the
// raw CSS/JS shows as garbage). Render it in an opaque sandboxed iframe.
// Remove a UI-plan JSON object the model printed instead of (or in addition
// to) the server consuming it. Handles both `__ui_plan__` and the common
// `ui_plan` misspelling, plus the plan tail if braces are split.
function stripClientPlan(text) {
  var raw = String(text || "");
  var marker = raw.search(/"_{0,2}ui_plan_{0,2}"\s*:/);
  if (marker === -1) return raw;
  var start = raw.lastIndexOf("{", marker);
  if (start === -1) return raw;
  var depth = 0, inString = false, escaped = false, end = -1;
  for (var i = start; i < raw.length; i++) {
    var c = raw[i];
    if (inString) {
      if (escaped) escaped = false;
      else if (c === "\\") escaped = true;
      else if (c === '"') inString = false;
      continue;
    }
    if (c === '"') inString = true;
    else if (c === "{") depth++;
    else if (c === "}") {
      depth--;
      if (depth === 0) { end = i + 1; break; }
    }
  }
  if (end === -1) return raw.slice(0, start).trim();
  return (raw.slice(0, start) + raw.slice(end)).replace(/\n{3,}/g, "\n\n").trim();
}

function looksLikeHtmlDocument(text) {
  var raw = String(text || "").trim();
  if (raw.length < 600) return false;
  if (/^<!doctype\s+html/i.test(raw) || /^<html[\s>]/i.test(raw) ||
      /^<head[\s>]/i.test(raw) || /^<body[\s>]/i.test(raw)) return true;
  if (/^<meta\s+charset/i.test(raw) && /<style[\s>]/i.test(raw)) return true;
  return /<style[\s>][\s\S]*<\/style>/i.test(raw) &&
    /<(div|section|canvas|svg)[\s>]/i.test(raw) && raw.length > 1500;
}

function inlineArtifactTitle(html) {
  var title = /<title[^>]*>([\s\S]*?)<\/title>/i.exec(html);
  if (title && title[1].trim()) return title[1].replace(/<[^>]*>/g, "").trim().slice(0, 80);
  var h1 = /<h1[^>]*>([\s\S]*?)<\/h1>/i.exec(html);
  if (h1 && h1[1].trim()) return h1[1].replace(/<[^>]*>/g, "").trim().slice(0, 80);
  return "Interactive app";
}

// Some models emit a `show_app` tool call as *text* instead of invoking it:
// a JSON object whose arguments carry the HTML app. Recover it and render the
// sandboxed app instead of printing the JSON (the "garbage on screen" case).
// The model sometimes prints a tool call as text instead of (or in addition
// to) the server executing it. `{"tool_call": true, "name": "propose_plan",
// "arguments": {...}}` must never reach the reader; the matching surface is
// already rendered from the CUSTOM frame.
function stripToolCallJson(text) {
  var raw = String(text || "");
  var names = "propose_plan|show_app|ask_user|show_surface|classify_media|vm_exec";
  var pattern = new RegExp('"tool_call"\\s*:\\s*true|"name"\\s*:\\s*"(' + names + ')"');
  for (var guard = 0; guard < 5; guard++) {
    var match = pattern.exec(raw);
    if (!match) break;
    var start = raw.lastIndexOf("{", match.index);
    if (start === -1) break;
    var depth = 0, inString = false, escaped = false, end = -1;
    for (var i = start; i < raw.length; i++) {
      var c = raw[i];
      if (inString) {
        if (escaped) escaped = false;
        else if (c === "\\") escaped = true;
        else if (c === '"') inString = false;
        continue;
      }
      if (c === '"') inString = true;
      else if (c === "{") depth++;
      else if (c === "}") {
        depth--;
        if (depth === 0) { end = i + 1; break; }
      }
    }
    if (end === -1) {
      raw = raw.slice(0, start).trim();
      break;
    }
    raw = (raw.slice(0, start) + raw.slice(end)).replace(/\n{3,}/g, "\n\n");
  }
  return raw.trim();
}

function extractEmbeddedApp(text) {
  var raw = String(text || "");
  var marker = raw.indexOf('"__tool_call__"');
  if (marker === -1) marker = raw.indexOf('"name":"show_app"');
  if (marker === -1) marker = raw.indexOf('"name": "show_app"');
  if (marker === -1) return null;
  var start = raw.lastIndexOf("{", marker);
  if (start === -1) return null;
  var depth = 0, inString = false, escaped = false, end = -1;
  for (var i = start; i < raw.length; i++) {
    var c = raw[i];
    if (inString) {
      if (escaped) escaped = false;
      else if (c === "\\") escaped = true;
      else if (c === '"') inString = false;
      continue;
    }
    if (c === '"') inString = true;
    else if (c === "{") depth++;
    else if (c === "}") {
      depth--;
      if (depth === 0) { end = i + 1; break; }
    }
  }
  if (end === -1) return null;
  var payload;
  try { payload = JSON.parse(raw.slice(start, end)); } catch (e) { return null; }
  var args = payload.arguments || payload.args || (payload.function && payload.function.arguments);
  if (typeof args === "string") {
    try { args = JSON.parse(args); } catch (e) { return null; }
  }
  if (!args || typeof args !== "object") return null;
  var html = args.html || args.content || args.body || args.app_html;
  if (!html || !looksLikeHtmlDocument(html)) return null;
  return {
    title: args.title || inlineArtifactTitle(html),
    html: html
  };
}

function renderInlineArtifact(html, titleOverride) {
  var box = document.createElement("div");
  box.className = "gb-inline-artifact";
  var head = document.createElement("div");
  head.className = "gb-inline-artifact-head";
  var title = document.createElement("span");
  title.className = "gb-inline-artifact-title";
  title.textContent = titleOverride || inlineArtifactTitle(html);
  head.appendChild(title);
  var expand = document.createElement("button");
  expand.type = "button";
  expand.className = "gb-inline-artifact-btn";
  expand.textContent = "Expand";
  expand.addEventListener("click", function () {
    box.classList.toggle("expanded");
    expand.textContent = box.classList.contains("expanded") ? "Collapse" : "Expand";
  });
  head.appendChild(expand);
  var frame = document.createElement("iframe");
  frame.className = "gb-inline-artifact-frame";
  // allow-scripts enables the interactive chart; no allow-same-origin keeps it
  // away from the suite DOM, cookies and storage.
  frame.setAttribute("sandbox", "allow-scripts allow-forms allow-modals");
  frame.setAttribute("referrerpolicy", "no-referrer");
  frame.setAttribute("loading", "lazy");
  frame.setAttribute("title", title.textContent);
  frame.srcdoc = html;
  frame.addEventListener("load", function () {
    setTimeout(function () { snapToBottom(); stickToBottom(900); }, 60);
  });
  box.appendChild(head);
  box.appendChild(frame);
  return box;
}

// User-uploaded attachments (#1673): the chip under a user turn offers a
// Preview button that streams the staged Drive inbox copy through the
// sanitized preview endpoint. Nothing is fetched until the button is pressed.
function userAttachmentPreviewUrl(name) {
  var bot =
    (window.ChatState && ChatState.currentBotName) ||
    window.__INITIAL_BOT_NAME__ ||
    "default";
  var url =
    "/api/chat/attachments/preview?bot=" +
    encodeURIComponent(bot) +
    "&name=" +
    encodeURIComponent(name);
  var token = "";
  try {
    token =
      (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      "";
  } catch (e) { /* storage unavailable */ }
  if (token) url += "&access_token=" + encodeURIComponent(token);
  return url;
}

function toggleUserAttachmentPreview(chip, name) {
  if (!chip) return;
  var existing = chip.nextElementSibling;
  if (existing && existing.classList && existing.classList.contains("gb-user-preview")) {
    existing.remove();
    return;
  }
  var box = document.createElement("div");
  box.className = "gb-attachment-inline gb-user-preview";

  var head = document.createElement("div");
  head.className = "gb-attachment-inline-head";
  var title = document.createElement("span");
  title.textContent = name;
  head.appendChild(title);

  var expand = document.createElement("button");
  expand.type = "button";
  expand.className = "gb-attachment-inline-btn";
  expand.textContent = "Expand";
  expand.addEventListener("click", function () {
    box.classList.toggle("expanded");
    expand.textContent = box.classList.contains("expanded") ? "Collapse" : "Expand";
  });
  head.appendChild(expand);

  var close = document.createElement("button");
  close.type = "button";
  close.className = "gb-attachment-inline-btn";
  close.setAttribute("aria-label", "Close preview");
  close.textContent = "×";
  close.addEventListener("click", function () { box.remove(); });
  head.appendChild(close);

  var body = document.createElement("div");
  body.className = "gb-attachment-inline-body";
  var frame = document.createElement("iframe");
  frame.src = userAttachmentPreviewUrl(name);
  frame.title = "Preview of " + name;
  frame.setAttribute("sandbox", "");
  frame.setAttribute("referrerpolicy", "no-referrer");
  frame.setAttribute("loading", "lazy");
  body.appendChild(frame);

  box.appendChild(head);
  box.appendChild(body);
  chip.after(box);
  if (typeof snapToBottom === "function") snapToBottom();
}

function addMessage(sender, content, msgId, reasoning) {
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
  if (content && sender && !ChatState.suppressPersist) persistLocalMessage(sender, content);

  var div = document.createElement("div");
  div.className = "message " + sender;
  if (msgId) div.id = msgId;
  // Hover timestamp. Replayed history keeps its own stored time, so only
  // stamp messages that are being added live.
  if (!ChatState.suppressPersist) {
    try {
      div.dataset.time = new Date().toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch (e) { /* Intl unavailable — no timestamp is fine */ }
  }

  if (sender !== "user") setChatStatus("");
  var userAttachmentName = null;
  if (sender === "user") {
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
    div.innerHTML = '<div class="message-content user-message">' + processedContent + "</div>";
    if (userAttachmentName) {
      // Several files in one turn were joined as `📎 a.pdf, b.png`; show one
      // chip per file so each one can be previewed on its own.
      var attachmentNames = userAttachmentName.split(/,\s+/).filter(Boolean);
      attachmentNames.forEach(function (singleName) {
        var restored = document.createElement("div");
        restored.className = "gb-attachment-restored";
        var restoredLabel = document.createElement("span");
        restoredLabel.textContent = "📎 " + singleName;
        restored.appendChild(restoredLabel);
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
    var cleanContent = stripMarkdownBlocks(content);
    if (embeddedApp) {
      div.innerHTML = '<div class="message-content bot-message">' + thinkingHtml + "</div>";
      div.querySelector(".message-content").appendChild(
        renderInlineArtifact(embeddedApp.html, embeddedApp.title)
      );
      messages.appendChild(div);
      if (!ChatState.isUserScrolling) scrollToBottom(true);
      stickToBottom(1400);
      return;
    }
    if (looksLikeHtmlDocument(content)) {
      div.innerHTML = '<div class="message-content bot-message">' + thinkingHtml + "</div>";
      div.querySelector(".message-content").appendChild(renderInlineArtifact(content));
      messages.appendChild(div);
      if (content && content.trim() !== "") div.appendChild(createCopyButton(content));
      if (!ChatState.isUserScrolling) scrollToBottom(true);
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

  if (sender === "user" || !ChatState.isUserScrolling || wasNearBottom) {
    scrollToBottom(sender !== "user");
    stickToBottom(sender === "user" ? 300 : 1400);
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
  if (!btn || btn.dataset.bound === "1") return;
  btn.dataset.bound = "1";
  function refresh() {
    btn.classList.toggle("active", TTSState.autoSpeak);
    btn.setAttribute("aria-pressed", TTSState.autoSpeak ? "true" : "false");
    btn.title = TTSState.autoSpeak ? "Stop auto-reading bot replies" : "Auto-read bot replies (TTS)";
    btn.setAttribute("aria-label", btn.title);
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

// The chat partial is injected by htmx after DOMContentLoaded, so the
// listener alone would never bind the toggle (#1693 follow-up).
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initTtsToggle);
} else {
  initTtsToggle();
}

// ── TTS (issue #708) — speak a bot message aloud ──
function createCopyButton(text) {
  var btn = document.createElement("button");
  btn.className = "copy-msg-btn";
  btn.type = "button";
  btn.title = "Copy message";
  btn.setAttribute("aria-label", "Copy message");
  btn.innerHTML = '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
  btn.addEventListener("click", function (e) {
    e.stopPropagation();
    var done = function () {
      btn.classList.add("copied");
      btn.title = "Copied";
      if (window.showToast) window.showToast("Message copied", "success");
      setTimeout(function () {
        btn.classList.remove("copied");
        btn.title = "Copy message";
      }, 1500);
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done).catch(function () {
        if (window.showToast) window.showToast("Copy failed", "warning");
      });
    } else {
      var area = document.createElement("textarea");
      area.value = text;
      document.body.appendChild(area);
      area.select();
      try { document.execCommand("copy"); done(); } catch (err) { /* ignore */ }
      area.remove();
    }
  });
  return btn;
}

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
  var streamBox = document.getElementById("messages");
  var wasNearBottom = streamBox &&
    (streamBox.scrollHeight - streamBox.scrollTop - streamBox.clientHeight) < 240;

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
  if (!ChatState.isUserScrolling || wasNearBottom) scrollToBottom(true);
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
    // Capture the accumulated stream before finalizeStreaming() clears it.
    // Completion frames often carry only reasoning; dropping the accumulated
    // content left a "Thinking" bubble with no visible answer (the persisted
    // message still had the answer).
    var finalContent =
      (data.content && data.content.trim() !== "")
        ? data.content
        : (ChatState.currentStreamingContent || "");
    var finalReasoning = data.reasoning || ChatState.currentReasoning || "";
    hideThinkingIndicator();
    if (ChatState.isStreaming) {
      finalizeStreaming();
    }
    if (finalContent && finalContent.trim() !== "") {
      addMessage("bot", finalContent, null, finalReasoning);
      if (TTSState.autoSpeak) {
        speakText(finalContent);
      }
    } else if (finalReasoning && finalReasoning.trim() !== "") {
      addMessage("bot", "", null, finalReasoning);
      if (TTSState.autoSpeak) {
        speakText(finalReasoning);
      }
    }
    ChatState.isStreaming = false;
    ChatState.currentReasoning = "";
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
