/* Chat send path: composer payload, privacy gate and offline queue entry.
 * Split out of chat-init.js so every chat script stays under 450 lines. */
"use strict";

// #1691 — the server needs the client's real surfaces and the Preview-mode
// switch to advertise only the applications this session can actually use.
// Visible waiting state + one retry hint. The provider can return 503 and
// retry for a while; without this the chat looked dead.
function clearChatPending() {
  ChatState.pendingResponseAt = null;
  if (ChatState.responseWatchdog) {
    clearTimeout(ChatState.responseWatchdog);
    ChatState.responseWatchdog = null;
  }
  if (typeof setChatStatus === "function") setChatStatus("");
}
window.clearChatPending = clearChatPending;

function armChatPending(text) {
  clearChatPending();
  ChatState.pendingResponseAt = Date.now();
  ChatState.pendingText = text || "";
  ChatState.responseWatchdog = setTimeout(function () {
    if (!ChatState.pendingResponseAt || ChatState.isStreaming) return;
    if (typeof setChatStatus === "function") {
      setChatStatus("Waiting for the model… the provider may be busy.");
    }
  }, 12000);
}

function clientCapabilities() {
  return {
    surfaces: ["text", "ui_plan"],
    input: "text,file,audio,camera",
    locale: (function () {
      try { return localStorage.getItem("gb-locale") || navigator.language || "en"; }
      catch (e) { return "en"; }
    })(),
    preview_mode: !!(window.GBPreviewMode && window.GBPreviewMode.isOn()),
    max_live_apps: 2,
  };
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

  if (ChatState.isStreaming && ChatState.streamingMessageId) {
    finalizeStreaming();
    ChatState.isStreaming = false;
  }

  if (!messageContent) {
    hideMentionDropdown();
    input.value = "";
    if (input.tagName === "TEXTAREA") input.style.height = "auto";
    input.focus();
  }

  var display = content || ("📎 " + fileNames);
  if (content && fileNames) display += "\n\n📎 " + fileNames;
  addMessage("user", display);
  // Immediate feedback: the provider may take seconds (or minutes) before the
  // first thinking frame, and the chat used to look frozen after Enter.
  if (typeof showThinkingIndicator === "function") showThinkingIndicator();
  // Sending always returns the view to the live edge.
  ChatState.isUserScrolling = false;
  scrollToBottom(false);

  var selectedMentions = Array.isArray(ChatState.selectedMentions) ? ChatState.selectedMentions : [];
  ChatState.selectedMentions = [];

  var driveFiles = [];
  try {
    if (window.DriveModule && typeof DriveModule.getRecentFileNames === "function") {
      driveFiles = DriveModule.getRecentFileNames();
    } else if (window.__driveRecentFiles) {
      driveFiles = window.__driveRecentFiles;
    }
  } catch (e) {}

  var modelPicker = document.getElementById("modelPicker");
  var selectedModel = modelPicker ? modelPicker.value : "auto";

  var isAnonymous = !!(window.__BOT_IS_PUBLIC__ && !(window.GBSecurity && GBSecurity.isAuthenticated && GBSecurity.isAuthenticated()));

  var payload = {
    bot_id: ChatState.currentBotId,
    user_id: ChatState.currentUserId,
    session_id: ChatState.currentSessionId,
    channel: "web",
    content: content,
    message_type: MessageType.USER,
    active_switchers: Array.from(ChatState.activeSwitchers),
    drive_context: (!isAnonymous && driveFiles.length > 0 && (!window.GBPrivacy || GBPrivacy.getSettings().save_memory !== false)) ? { source: "user_drive", untrusted: true, files: driveFiles.slice(0, 20) } : undefined,
    // #1545: Drive is part of the prompt context by default; the setting is
    // stored as a string so a future Settings toggle can flip it.
    index_all_drive: (function () {
      try { return localStorage.getItem("gb-index-all-drive") !== "false"; }
      catch (e) { return true; }
    })(),
    model_preference: selectedModel !== "auto" ? selectedModel : undefined,
    client_capabilities: clientCapabilities(),
    // #1593 — explicit per-conversation web-search switch. When false the
    // server skips the search pipeline entirely; default is on.
    web_search: window.ChatWebSearch ? window.ChatWebSearch.isEnabled() : true,
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
  // The vibe shell knows the running project even when the mention picker
  // never loaded its catalog: a typed (or toolbar pre-filled) `@<project>`
  // resolves against the SELECTED project, so the message carries
  // project_context and the server routes it deterministically to the vibe
  // agent instead of the bot's generic LLM answering with a clarification.
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

  var finalSend = function () {
    var sentPayload = payload;
    ChatState.ws.send(JSON.stringify(sentPayload));
    // `file` is not defined in this scope: referencing it threw a
    // ReferenceError right after ws.send and skipped the delivery guard.
    armChatPending(content || "");
    // Delivery guard: a send that never reaches the server used to look like a
    // bot that ignores the user. Watch for any frame of this conversation for
    // a few seconds; if nothing arrives, reopen the socket and resend once.
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
        notify("Message not delivered — reconnecting", "error");
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
    // GB6-15: the attachment leaves the composer with a visible outcome
    // instead of silently disappearing.
    if (files.length) {
      notify((files.length === 1 ? "Attachment sent: " : "Attachments sent: ") + fileNames, "success");
    }
    if (fileInput) { fileInput.value = ""; }
    var chip = document.getElementById("fileChip");
    if (chip) { chip.style.display = "none"; chip.textContent = ""; }
    window.dispatchEvent(new CustomEvent("gb-chat-message-sent", {
      detail: {
        session_id: ChatState.currentSessionId,
        // GB6-25 — the input is cleared before this event fires, so the usage
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
      activeChip.textContent = "⏳ Preparing " + fileNames + "…";
      activeChip.style.display = "inline-flex";
    }
    readAttachments(files).then(function (attachments) {
      payload.files = attachments;
      payload.file = attachments[0];
      finalSend();
    }).catch(function (error) {
      if (activeChip) activeChip.textContent = "⚠ Could not read " + error.message;
      notify("Could not read an attachment. Attach it again.", "error");
    });
  } else if (ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
    finalSend();
  } else if (files.length) {
    // #1275 — reconnect may still be in flight; read every attachment now
    // and queue the send for the reopened socket.
    notify("Connection is back shortly — attachment queued", "info");
    readAttachments(files).then(function (attachments) {
      payload.files = attachments;
      payload.file = attachments[0];
      queueOfflineMessage(payload);
    }).catch(function () {
      notify("Could not read an attachment. Attach it again.", "error");
    });
  } else {
    // #1275 — a server restart kills the socket while the user keeps
    // typing. Queue instead of dropping the message on the floor; the
    // reconnect handler flushes the queue once the socket reopens.
    queueOfflineMessage(payload);
  }
}

// #1275 — messages composed while the socket is down are kept here (most
// recent last) and flushed in order after the WebSocket reconnects. The
// queue is capped so a stuck socket cannot grow memory without bound.
// Shared by the composer chip and the send path: it used to live inside the
// file-button IIFE, so `sendMessage` threw "formatAttachSize is not defined"
// as soon as a file was attached and the message never left the browser.
function formatAttachSize(bytes) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
}

function queueOfflineMessage(payload) {
  if (!Array.isArray(ChatState.offlineQueue)) ChatState.offlineQueue = [];
  ChatState.offlineQueue.push(payload);
  if (ChatState.offlineQueue.length > 50) ChatState.offlineQueue.shift();
  notify("Reconnecting — your message will be sent when the connection is back.", "info");
  if (typeof updateConnectionStatus === "function") updateConnectionStatus("connecting");
  if (ChatState.reconnectAttempts >= ChatState.maxReconnectAttempts) {
    // Reconnect budget was exhausted before the user tried to send: restart
    // it so the queued message actually goes out.
    ChatState.reconnectAttempts = 0;
    if (typeof connectWebSocket === "function") connectWebSocket();
  }
}

window.sendMessage = sendMessage;

// #1670 — the surface renderer submits forms and plan approvals here.
window.ChatSend = window.ChatSend || {};
window.ChatSend.sendStructured = function (detail) {
  detail = detail || {};
  if (detail.action === "approve_plan") {
    var steps = Array.isArray(detail.steps) ? detail.steps : [];
    var titles = steps.map(function (step) { return step.title || ""; }).filter(Boolean);
    sendMessage("Approved plan: " + titles.join(" → "), {
      plan_approval: { surface_id: detail.surface_id || null, steps: steps },
    });
    return;
  }
  var values = detail.values || {};
  var text = Object.keys(values).map(function (key) {
    return key + ": " + values[key];
  }).join("\n");
  sendMessage(text || "Continue.", {
    question_answers: values,
    question_id: detail.surface_id || null,
  });
};
