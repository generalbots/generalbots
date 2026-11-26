/* Chat send path: composer payload, privacy gate and offline queue entry. */
"use strict";

// #1691 — advertise the client's real surfaces and Preview mode; keep a visible
// waiting state + retry hint so a 503 storm never looks like a dead chat.
function clearChatPending() {
  ChatState.pendingResponseAt = null;
  if (ChatState.responseWatchdog) {
    clearTimeout(ChatState.responseWatchdog);
    ChatState.responseWatchdog = null;
  }
  if (ChatState.pendingHardTimeout) {
    clearTimeout(ChatState.pendingHardTimeout);
    ChatState.pendingHardTimeout = null;
  }
  if (typeof setChatStatus === "function") setChatStatus("");
}
window.clearChatPending = clearChatPending;
// A private bot or a failed probe dials no socket: clear the stuck turn so the composer stays usable.
function clearBlockedChatTurn() { if (window.GBTurn && GBTurn.forceHide) GBTurn.forceHide(); if (typeof clearChatPending === "function") clearChatPending(); }
window.clearBlockedChatTurn = clearBlockedChatTurn;
// #2244 — on tab focus, drop a stale "Waiting for the model…" status.
(function () {
  function clearStale() {
    if (ChatState && ChatState.pendingResponseAt) return;
    if (typeof setChatStatus === "function") setChatStatus("");
  }
  if (typeof document !== "undefined" && document.addEventListener) {
    document.addEventListener("visibilitychange", function () {
      if (document.visibilityState === "visible") clearStale();
    });
  }
  if (typeof window !== "undefined" && window.addEventListener) {
    window.addEventListener("focus", clearStale);
  }
})();

// A terminal probe can strand a composed message after the composer was cleared.
function gbRestoreBlockedComposer() { var i = document.getElementById("messageInput"), p = ChatState.lastUserTurnPayload; if (!i || !p || !p.content || i.value) return false; i.value = p.content; if (Array.isArray(ChatState.offlineQueue)) ChatState.offlineQueue = ChatState.offlineQueue.filter(function (q) { return q !== p; }); ChatState.lastUserTurnPayload = null; return true; }
window.gbRestoreBlockedComposer = gbRestoreBlockedComposer;

function armChatPending(text) {
  clearChatPending();
  ChatState.pendingResponseAt = Date.now();
  ChatState.pendingText = text || "";
  var slowAfter = (window.GBTurn && GBTurn.SLOW_AFTER_MS) || 12000;
  ChatState.responseWatchdog = setTimeout(function () {
    if (!ChatState.pendingResponseAt) return;
    // #2098 — keep the SAME progress element; the status bar alone looked frozen.
    if (window.GBTurn && GBTurn.slow()) return;
    if (ChatState.isStreaming) return;
    if (typeof setChatStatus === "function") {
      setChatStatus("Waiting for the model… the provider may be busy.");
    }
  }, slowAfter);
  // #2244 — no terminal frame must leave the pending turn stuck forever.
  if (ChatState.pendingHardTimeout) clearTimeout(ChatState.pendingHardTimeout);
  ChatState.pendingHardTimeout = setTimeout(function () {
    if (!ChatState.pendingResponseAt) return;
    clearChatPending();
  }, 180000);
}

function clientCapabilities() {
  var controls = (window.GBChatControls && typeof window.GBChatControls.getState === "function")
    ? window.GBChatControls.getState()
    : { web: true, mediaType: "auto", agent: true, appAutomation: true };
  var surfaces = ["text"];
  if (controls.appAutomation !== false) surfaces.push("ui_plan");
  var caps = {
    surfaces: surfaces,
    input: "text,file,audio,camera",
    locale: (function () {
      try { return localStorage.getItem("gb-locale") || navigator.language || "en"; }
      catch (e) { return "en"; }
    })(),
    preview_mode: !!(window.GBPreviewMode && window.GBPreviewMode.isOn()),
    max_live_apps: 2,
    ui_automation: controls.appAutomation !== false,
  };
  if (controls.mediaType && controls.mediaType !== "auto") caps.media_type = controls.mediaType;
  return caps;
}
window.gbClientCapabilities = clientCapabilities;

function sendMessage(messageContent, extra) {
  var input = document.getElementById("messageInput");
  if (!input) return;

  var content = messageContent || input.value.trim();

  var fileInput = document.getElementById("messageFile");
  var files = fileInput && fileInput.files
    ? Array.prototype.slice.call(fileInput.files)
    : [];
  var fileNames = files.map(function (file) { return file.name; }).join(", ");

  if (!content && !files.length) return;

  // A private bot or a failed probe never dials a socket: fail fast instead of
  // painting "Sending…" and queueing with only a console warning.
  var blocked = ChatState.botAccess === "auth_required" ? "auth" : ChatState.botAccess === "probe_error" ? "probe" : "";
  if (blocked) {
    clearBlockedChatTurn();
    if (!messageContent) { var blockedInput = document.getElementById("messageInput"); if (blockedInput) blockedInput.value = content; }
    if (blocked === "auth") notify(chatPrivateSigninMessage(), "error");
    return;
  }

  // #1947 — base64 attachments fail the frame silently when oversized; refuse up front.
  var MAX_ATTACH_BYTES = 50 * 1024 * 1024;
  var tooBig = files.filter(function (file) { return file.size > MAX_ATTACH_BYTES; });
  if (tooBig.length) {
    var names = tooBig.map(function (file) { return file.name; }).join(", ");
    if (typeof notify === "function") {
      notify(gbT("chat-notify-attachment-too-large-50-mb-per-file-limit", "Attachment too large (50 MB per file limit): ") + names, "error");
    }
    if (fileInput) fileInput.value = "";
    return;
  }

  if (ChatState.isStreaming && ChatState.streamingMessageId) {
    finalizeStreaming();
    ChatState.isStreaming = false;
  }

  if (!messageContent) {
    hideMentionDropdown();
    input.value = "";
    if (input.tagName === "TEXTAREA") input.style.height = "auto";
    // #2163 — preventScroll stops the whole window jumping after Enter.
    input.focus({ preventScroll: true });
  }

  // #2163 — capture scroll position before the bubble changes scroll height.
  var wasAtBottom = true;
  var pane = document.getElementById("messages");
  if (pane) wasAtBottom = pane.scrollHeight - pane.scrollTop - pane.clientHeight < 80;

  var display = content || ("📎 " + fileNames);
  if (content && fileNames) display += "\n\n📎 " + fileNames;

  var selectedMentions = Array.isArray(ChatState.selectedMentions) ? ChatState.selectedMentions : [];
  ChatState.selectedMentions = [];

  var driveFiles = [];
  try {
    if (window.DriveModule && typeof DriveModule.getRecentFileNames === "function") {
      driveFiles = DriveModule.getRecentFileNames();
    } else if (window.__driveRecentFiles) {
      driveFiles = window.__driveRecentFiles;
    }
  } catch (e) { window.gbLogError ? gbLogError(e, "sendMessage") : console.error("sendMessage", e); }

  var modelPicker = document.getElementById("modelPicker");
  var selectedModel = modelPicker ? modelPicker.value : "auto";
  // #1753 — the switcher set follows the active conversation. Parallel tabs
  // resolve to their own set; the stock tab keeps ChatState.activeSwitchers.
  var outgoingSwitchers = typeof activeSwitcherIds === "function"
    ? activeSwitcherIds()
    : Array.from(ChatState.activeSwitchers || []);

  var controls = (window.GBChatControls && typeof window.GBChatControls.getState === "function")
    ? window.GBChatControls.getState()
    : { web: true, mediaType: "auto", agent: true, appAutomation: true };

  var isAnonymous = !!(window.__BOT_IS_PUBLIC__ && !(window.GBSecurity && GBSecurity.isAuthenticated && GBSecurity.isAuthenticated()));

  function generateTurnMessageId() {
    try {
      if (typeof crypto !== "undefined" && crypto && typeof crypto.randomUUID === "function") {
        return crypto.randomUUID();
      }
    } catch (e) { window.gbLogError ? gbLogError(e, "generateTurnMessageId") : console.error("generateTurnMessageId", e); }
    return "m-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 10);
  }

  var payload = {
    bot_id: ChatState.currentBotId,
    user_id: ChatState.currentUserId,
    session_id: ChatState.currentSessionId,
    channel: "web",
    content: content,
    message_type: MessageType.USER,
    message_id: generateTurnMessageId(),
    client_id: ChatState.clientId,
    active_switchers: outgoingSwitchers,
    drive_context: (!isAnonymous && driveFiles.length > 0 && (!window.GBPrivacy || GBPrivacy.getSettings().save_memory !== false)) ? { source: "user_drive", untrusted: true, files: driveFiles.slice(0, 20) } : undefined,
    // #1545: Drive is part of the prompt context by default; the setting is
    // stored as a string so a future Settings toggle can flip it.
    index_all_drive: (function () {
      try { return localStorage.getItem("gb-index-all-drive") !== "false"; }
      catch (e) { return true; }
    })(),
    model_preference: selectedModel !== "auto" ? selectedModel : undefined,
    client_capabilities: clientCapabilities(),
    // #1593/#2238 — explicit per-conversation switches.
    web_search: (typeof controls.web === "boolean") ? controls.web : (window.ChatWebSearch ? window.ChatWebSearch.isEnabled() : true),
    agent_enabled: controls.agent !== false,
    ui_automation: controls.appAutomation !== false,
    media_type: controls.mediaType || "auto",
    // #1642 — the prompt reads chat_preferences server-side; the old
    // `user_memory` payload was ignored by the server and is gone.
    mentions: selectedMentions.map(function (mention) {
      return {
        kind: mention.kind,
        id: mention.id,
        label: mention.name,
        project_id: mention.project_id || null,
      };
    }),
    timestamp: new Date().toISOString(),
  };

  // #1670 — structured answers ride with the same turn so the server can
  // record the field ids and stop asking them again.
  if (extra && typeof extra === "object") {
    Object.keys(extra).forEach(function (key) {
      if (extra[key] !== undefined) payload[key] = extra[key];
    });
  }

  var projectMention = selectedMentions.find(function (mention) {
    return mention.kind === "project" && mention.id;
  });
  // Also resolve a manually typed `@calculator` when its project list was
  // already loaded by the mention picker.
  if (!projectMention && Array.isArray(ChatState.projectCatalog)) {
    var directMatch = content.match(/(?:^|\s)@([A-Za-z0-9][A-Za-z0-9_.-]*)/);
    if (directMatch) {
      var directName = directMatch[1].toLowerCase();
      var directProject = ChatState.projectCatalog.find(function (project) {
        return String(project.name || "").toLowerCase() === directName;
      });
      if (directProject) {
        projectMention = {
          id: directProject.id || directProject.project_id,
          name: directProject.name,
        };
      }
    }
  }
  // The vibe shell resolves a typed `@<project>` against the SELECTED project
  // even without a loaded catalog, so the server routes it to the vibe agent.
  if (!projectMention && typeof window.currentProjectId !== "undefined" && window.currentProjectId) {
    var vibeName = String(window.currentProject || "").trim();
    var vibeMatch = content.match(/(?:^|\s)@([A-Za-z0-9][A-Za-z0-9_.-]*)/);
    if (vibeMatch && vibeName && vibeMatch[1].toLowerCase() === vibeName.toLowerCase()) {
      projectMention = { id: window.currentProjectId, name: vibeName };
    }
  }
  if (projectMention) {
    payload.project_context = {
      project_id: String(projectMention.id),
      project_name: String(projectMention.name || ""),
    };
    // Keep flat aliases for older server versions; the nested object is the
    // canonical field used by the current chat pipeline.
    payload.project_id = payload.project_context.project_id;
    payload.project_name = payload.project_context.project_name;
  }

  // #2360 — keep the structured payload so a failed turn retries without prose.
  ChatState.lastUserTurnPayload = payload;

  addMessage("user", display);
  // #2098 — the turn state machine keeps feedback until the final answer.
  if (window.GBTurn) GBTurn.start(content || fileNames);
  else if (typeof showThinkingIndicator === "function") showThinkingIndicator();
  // Sending returns to the live edge only when the reader was already there.
  if (wasAtBottom) {
    ChatState.isUserScrolling = false;
    ChatState.isUserAtBottom = true;
    scrollToBottom(false);
  }

  var finalSend = function () {
    var sentPayload = payload;
    ChatState.ws.send(JSON.stringify(sentPayload));
    // A stray `file` reference here threw and skipped the delivery guard.
    armChatPending(content || "");
    // Delivery guard: no server receipt within seconds means reopen and resend once.
    ChatState.pendingSend = {
      payload: sentPayload,
      at: Date.now(),
      attempts: 1,
      acked: false,
    };
    setTimeout(function () {
      var pending = ChatState.pendingSend;
      if (!pending || pending.payload !== sentPayload) return;
      // Only the server's own receipt proves the turn was accepted; another
      // frame (welcome, suggestions) can arrive while the message was lost.
      if (pending.acked) {
        ChatState.pendingSend = null;
        return;
      }
      if (pending.attempts >= 3) {
        ChatState.pendingSend = null;
        notify((typeof gbT === "function") ? gbT("chat-message-not-delivered", "Message not delivered — reconnecting") : "Message not delivered — reconnecting", "error");
        return;
      }
      ChatState.pendingSend = {
        payload: sentPayload,
        at: Date.now(),
        attempts: pending.attempts + 1,
        acked: false,
      };
      if (!Array.isArray(ChatState.offlineQueue)) ChatState.offlineQueue = [];
      ChatState.offlineQueue.push(sentPayload);
      if (typeof connectWebSocket === "function") connectWebSocket();
    }, 6000);
    // GB7-15: the attachment leaves the composer with a visible outcome
    // instead of silently disappearing.
    if (files.length) {
      var attachSent = (typeof gbT === "function")
        ? (files.length === 1 ? gbT("chat-attachment-sent", "Attachment sent: ") : gbT("chat-attachments-sent", "Attachments sent: "))
        : (files.length === 1 ? "Attachment sent: " : "Attachments sent: ");
      notify(attachSent + fileNames, "success");
    }
    if (fileInput) { fileInput.value = ""; }
    var chip = document.getElementById("fileChip");
    if (chip) { chip.style.display = "none"; chip.textContent = ""; }
    window.dispatchEvent(new CustomEvent("gb-chat-message-sent", {
      detail: {
        session_id: ChatState.currentSessionId,
        // GB7-25 — the input is cleared before this event fires, so the usage
        // panel must receive the text length (and model) with the event.
        content_length: (content || "").length,
        model: selectedModel,
      },
    }));
  };

  function readAttachments(list) {
    return Promise.all(list.map(function (attachment) {
      return new Promise(function (resolve, reject) {
        var reader = new FileReader();
        reader.onerror = function () { reject(new Error(attachment.name)); };
        reader.onload = function () {
          resolve({
            name: attachment.name,
            content_base64: String(reader.result).split(",")[1] || reader.result,
          });
        };
        reader.readAsDataURL(attachment);
      });
    }));
  }

  if (files.length && ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
    var activeChip = document.getElementById("fileChip");
    if (activeChip) {
      activeChip.textContent = gbT("chat-str-preparing", "⏳ Preparing ") + fileNames + "…";
      activeChip.style.display = "inline-flex";
    }
    readAttachments(files).then(function (attachments) {
      payload.files = attachments;
      payload.file = attachments[0];
      finalSend();
    }).catch(function (error) {
      if (activeChip) activeChip.textContent = gbT("chat-str-could-not-read", "⚠ Could not read ") + error.message;
      notify((typeof gbT === "function") ? gbT("chat-attachment-read-failed", "Could not read an attachment. Attach it again.") : "Could not read an attachment. Attach it again.", "error");
    });
  } else if (ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
    finalSend();
  } else if (files.length) {
    // #1275 — reconnect may still be in flight; read every attachment now
    // and queue the send for the reopened socket.
    notify((typeof gbT === "function") ? gbT("chat-attachment-queued", "Connection is back shortly — attachment queued") : "Connection is back shortly — attachment queued", "info");
    readAttachments(files).then(function (attachments) {
      payload.files = attachments;
      payload.file = attachments[0];
      queueOfflineMessage(payload);
    }).catch(function () {
      notify((typeof gbT === "function") ? gbT("chat-attachment-read-failed", "Could not read an attachment. Attach it again.") : "Could not read an attachment. Attach it again.", "error");
    });
  } else {
    // #1275 — a server restart kills the socket while the user keeps
    // typing. Queue instead of dropping the message on the floor; the
    // reconnect handler flushes the queue once the socket reopens.
    queueOfflineMessage(payload);
  }
}

// #1275 — messages composed while the socket is down are kept here, flushed in order on
// reconnect and capped at 50 so a stuck socket cannot grow memory without bound.
