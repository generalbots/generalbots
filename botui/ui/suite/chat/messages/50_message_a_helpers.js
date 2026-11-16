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
  if (window.gbLog && typeof window.gbLog.info === "function") {
    window.gbLog.info(message);
    return;
  }
}

function gbChatText(key, fallback) {
  return typeof gbT === "function" ? gbT(key, fallback) : fallback;
}

function getChatMessageText(div) {
  if (!div) return "";
  if (div.dataset && typeof div.dataset.originalContent === "string" && div.dataset.originalContent) {
    return div.dataset.originalContent;
  }
  var textEl = div.querySelector(".gb-message-text");
  if (textEl) return (textEl.innerText || textEl.textContent || "").trim();
  var content = div.querySelector(".message-content");
  return (content ? (content.innerText || content.textContent || "") : (div.innerText || div.textContent || "")).trim();
}

function getPreviousUserMessage(div) {
  if (!div) return null;
  var prev = div.previousElementSibling;
  while (prev) {
    if (prev.classList && prev.classList.contains("message") && prev.classList.contains("user")) return prev;
    prev = prev.previousElementSibling;
  }
  var msgs = document.querySelectorAll("#messages .message.user");
  return msgs.length ? msgs[msgs.length - 1] : null;
}

function gbChatConfirm(message, title, onConfirm) {
  if (window.GBModal && typeof window.GBModal.confirm === "function") {
    try {
      window.GBModal.confirm(message, title || gbChatText("chat-confirm-title", "Confirm"), function () { onConfirm && onConfirm(); });
      return;
    } catch (e) {}
  }
  // No native confirm() fallback per suite policy (#2297/#1395).
  chatNotice(gbChatText("chat-confirm-unavailable", "Confirmation unavailable"), "warning");
}

function attachMessageActions(div, sender, content) {
  if (!div || !div.classList || !div.classList.contains("message")) return;
  if (div.querySelector(".message-actions")) return;
  var text = (typeof content === "string" && content.trim()) ? content : getChatMessageText(div);
  var bar = document.createElement("div");
  bar.className = "message-actions";
  bar.setAttribute("role", "toolbar");
  bar.setAttribute("aria-label", gbChatText("chat-message-actions", "Message actions"));

  function makeBtn(action, label, handler) {
    var b = document.createElement("button");
    b.type = "button";
    b.className = "message-action-btn message-action-" + action;
    b.textContent = label;
    b.title = label;
    b.setAttribute("aria-label", label);
    b.addEventListener("click", function (e) {
      e.preventDefault();
      e.stopPropagation();
      handler(b);
    });
    bar.appendChild(b);
    return b;
  }

  makeBtn("copy", gbChatText("chat-copy", "Copy"), function (btn) {
    var t = getChatMessageText(div) || text || "";
    gbCopyText(t).then(function (ok) {
      if (ok) {
        chatNotice(gbChatText("chat-message-copied", "Message copied"), "success");
        btn.dataset.copied = "1";
        setTimeout(function(){ try{ delete btn.dataset.copied; }catch(e){} }, 1200);
      } else {
        chatNotice(gbChatText("chat-copy-failed", "Copy failed"), "warning");
      }
    });
  });

  makeBtn("edit", gbChatText("chat-edit", "Edit"), function () {
    var t = getChatMessageText(div) || text || "";
    if (!t) return;
    var input = document.getElementById("messageInput");
    if (input) {
      input.value = t;
      try { input.focus(); } catch (e) {}
      try { input.dispatchEvent(new Event("input", { bubbles: true })); } catch (e) {}
      try { input.setSelectionRange(input.value.length, input.value.length); } catch (e) {}
    } else {
      chatNotice(gbChatText("chat-edit-unavailable", "Composer unavailable"), "warning");
    }
  });

  makeBtn("regenerate", gbChatText("chat-regenerate", "Regenerate"), function () {
    var sourceUser = sender === "user" ? div : getPreviousUserMessage(div);
    if (!sourceUser) {
      chatNotice(gbChatText("chat-regenerate-no-source", "No previous user message to regenerate"), "warning");
      return;
    }
    var payload = sourceUser._gbRetryPayload || null;
    if (payload && typeof window.gbResendChatPayload === "function") {
      window.gbResendChatPayload(payload);
      return;
    }
    var t = getChatMessageText(sourceUser).replace(/(?:^|\n)📎\s+[^\n]+$/gm, "").trim();
    if (t && typeof window.sendMessage === "function") {
      window.sendMessage(t);
    } else {
      chatNotice(gbChatText("chat-regenerate-unavailable", "Cannot regenerate this turn"), "warning");
    }
  });

  makeBtn("delete", gbChatText("chat-delete", "Delete"), function () {
    gbChatConfirm(
      gbChatText("chat-delete-message-confirm", "Delete this message from the conversation?"),
      gbChatText("chat-delete-title", "Delete message"),
      function () {
        if (div && div.parentNode) div.parentNode.removeChild(div);
        // best-effort local transcript cleanup; server-side per-message delete endpoint may not exist
        if (window.ChatState && div.dataset && div.dataset.messageId) {
          try {
            if (Array.isArray(ChatState.localMessages)) {
              ChatState.localMessages = ChatState.localMessages.filter(function (m) { return m && m.id !== div.dataset.messageId; });
            }
          } catch (e) {}
        }
        if (typeof updateScrollButton === "function") try { updateScrollButton(); } catch (e) {}
      }
    );
  });

  // Prefer placing actions inside message-content for layout; fallback to bubble.
  var host = div.querySelector(".message-content") || div;
  host.appendChild(bar);
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
  var showMoreLabel = (typeof gbT === "function") ? gbT("chat-show-more", "Show more") : "Show more";
  var showLessLabel = (typeof gbT === "function") ? gbT("chat-show-less", "Show less") : "Show less";
  var toggle = document.createElement("button");
  toggle.type = "button";
  toggle.className = "gb-message-toggle";
  toggle.textContent = showMoreLabel;
  toggle.setAttribute("aria-expanded", "false");
  textBox.classList.add("gb-message-collapsed");
  toggle.addEventListener("click", function () {
    var collapsed = textBox.classList.toggle("gb-message-collapsed");
    toggle.textContent = collapsed ? showMoreLabel : showLessLabel;
    toggle.setAttribute("aria-expanded", collapsed ? "false" : "true");
  });
  textBox.parentNode.insertBefore(toggle, textBox.nextSibling);
}

function renderToolExecHeader(toolExec) {
  if (!toolExec) return "";
  var toolLabel = (typeof gbT === "function") ? gbT("chat-tool", "tool") : "tool";
  var toolResultLabel = (typeof gbT === "function") ? gbT("chat-tool-result", "Tool result") : "Tool result";
  var paramLabel = (typeof gbT === "function") ? gbT("chat-param", "param") : "param";
  var paramsLabel = (typeof gbT === "function") ? gbT("chat-params", "params") : "params";
  var name = String(toolExec.tool_name || toolExec.name || toolExec.label || toolLabel);
  var desc = String(toolExec.tool_description || toolExec.description || "");
  var icon = String(toolExec.tool_icon || "").trim();
  var params = toolExec.tool_params || toolExec.params;
  var metaParts = [];
  if (icon) metaParts.push(icon);
  if (Array.isArray(params) && params.length) {
    metaParts.push(params.length + " " + (params.length === 1 ? paramLabel : paramsLabel));
  }
  var detail = desc || metaParts.join(" · ");
  if (window.ChatCodeUX && typeof window.ChatCodeUX.toolCardHtml === "function") {
    return window.ChatCodeUX.toolCardHtml({
      id: toolExec.id || ("tool-exec-" + Date.now()),
      tool_name: name,
      status: "done",
      argsDetail: detail,
      summary: toolResultLabel,
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
    '<div class="gb-tool-summary">' + esc(toolResultLabel) + '</div>' +
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

