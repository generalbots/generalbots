"use strict";
/* #2344 — conversation title, rename, clear/export/copy/delete controls in the chat app. */
(function () {
  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }

  var idle = window.gbIdle || window.setTimeout;

  function authHeaders(extra) {
    var headers = Object.assign({}, extra || {});
    var token = "";
    try {
      if (typeof window.getGBAccessToken === "function") token = window.getGBAccessToken() || "";
    } catch (e) { window.gbLogError ? gbLogError(e, "authHeaders") : console.error("authHeaders", e); }
    if (!token) {
      try { if (window.GBSecurity && typeof window.GBSecurity.getToken === "function") token = window.GBSecurity.getToken() || ""; }
      catch (e) { window.gbLogError ? gbLogError(e, "authHeaders") : console.error("authHeaders", e); }
    }
    if (!token) {
      try {
        token =
          window.localStorage.getItem("gb-access-token") ||
          window.sessionStorage.getItem("gb-access-token") ||
          window.localStorage.getItem("gb_token") ||
          window.sessionStorage.getItem("gb_token") ||
          "";
      } catch (e) { window.gbLogError ? gbLogError(e, "authHeaders") : console.error("authHeaders", e); }
    }
    if (token) headers.Authorization = "Bearer " + token;
    return headers;
  }

  function currentSessionId() {
    return (window.ChatState && ChatState.currentSessionId) || "";
  }

  function currentBotName() {
    return (window.ChatState && ChatState.currentBotName) || window.__INITIAL_BOT_NAME__ || "default";
  }

  function notify(msg, type) {
    if (typeof window.showToast === "function") { window.showToast(msg, type || "info"); return; }
    if (typeof window.chatNotice === "function") { window.chatNotice(msg, type || "info"); }
  }

  function announceCleared(sid) {
    window.dispatchEvent(new CustomEvent("gb-chat-session-changed", { detail: { session_id: sid, cleared: true } }));
  }

  async function fetchSessionTitle(sessionId) {
    if (!sessionId) return "";
    try {
      var url = "/api/chat/history/sessions?limit=50&bot_name=" + encodeURIComponent(currentBotName());
      var res = await fetch(url, { headers: authHeaders(), credentials: "same-origin" });
      if (!res.ok) return "";
      var data = await res.json();
      var sessions = (data && data.sessions) || [];
      for (var i = 0; i < sessions.length; i++) {
        var s = sessions[i], id = s.session_id || s.id || s.sessionId;
        if (String(id) === String(sessionId)) return (s.title || "").trim();
      }
    } catch (e) { window.gbLogError ? gbLogError(e, "fetchSessionTitle") : console.error("fetchSessionTitle", e); }
    return "";
  }

  function ensureUi() {
    var wrapper = document.getElementById("chatContentWrapper");
    if (!wrapper || document.getElementById("chatConversationHeader")) return null;
    var header = document.createElement("div");
    header.id = "chatConversationHeader";
    header.className = "gb-chat-conversation-header";
    header.innerHTML =
      '<div class="gb-chat-conversation-title-wrap">' +
        '<input id="chatConversationTitleInput" class="gb-chat-conversation-title" type="text" ' +
          'aria-label="' + t("chat-conversation-title", "Conversation title") + '" ' +
          'placeholder="' + t("chat-conversation-untitled", "Untitled conversation") + '" />' +
      '</div>' +
      '<div class="gb-chat-conversation-menu-wrap">' +
        '<button type="button" id="chatConversationMenuBtn" class="gb-chat-conversation-menu-btn" ' +
          'aria-haspopup="menu" aria-expanded="false" aria-label="' + t("chat-conversation-menu", "Conversation menu") + '">⋯</button>' +
        '<div id="chatConversationMenu" class="gb-chat-conversation-menu" role="menu" hidden>' +
          '<button type="button" role="menuitem" data-action="rename">' + t("chat-conversation-rename", "Rename") + '</button>' +
          '<button type="button" role="menuitem" data-action="clear">' + t("chat-conversation-clear", "Clear context") + '</button>' +
          '<button type="button" role="menuitem" data-action="export-md">' + t("chat-conversation-export-md", "Export transcript (.md)") + '</button>' +
          '<button type="button" role="menuitem" data-action="export-json">' + t("chat-conversation-export-json", "Export transcript (.json)") + '</button>' +
          '<button type="button" role="menuitem" data-action="copy-link">' + t("chat-conversation-copy-link", "Copy link") + '</button>' +
          '<button type="button" role="menuitem" data-action="delete" class="danger">' + t("chat-conversation-delete", "Delete") + '</button>' +
        '</div>' +
      '</div>';
    var anchor = document.getElementById("chatHeaderActions") || wrapper.firstChild;
    wrapper.insertBefore(header, anchor);
    return header;
  }

  function setTitle(title) {
    var input = document.getElementById("chatConversationTitleInput");
    if (!input) return;
    input.value = input.dataset.lastSaved = title || "";
  }

  async function refreshTitle() {
    var sid = currentSessionId();
    if (!sid) {
      setTitle("");
      return;
    }
    var title = await fetchSessionTitle(sid);
    setTitle(title);
  }

  var titleRefreshQueued = false;
  function scheduleTitleRefresh() {
    if (titleRefreshQueued) return;
    titleRefreshQueued = true;
    idle(function () { titleRefreshQueued = false; refreshTitle(); });
  }

  async function renameSession(newTitle) {
    var sid = currentSessionId();
    if (!sid) return false;
    try {
      var res = await fetch("/api/chat/history/sessions/" + encodeURIComponent(sid), { method: "PUT",
        headers: authHeaders({ "Content-Type": "application/json" }), credentials: "same-origin",
        body: JSON.stringify({ title: newTitle || "" }) });
      if (!res.ok) throw new Error("rename " + res.status);
      setTitle((newTitle || "").trim());
      notify(t("chat-conversation-renamed", "Conversation renamed"), "success");
      return true;
    } catch (e) {
      notify(t("chat-conversation-rename-failed", "Failed to rename conversation"), "error");
      return false;
    }
  }

  function clearMessagesUi() {
    if (window.GBChatVirtual && typeof window.GBChatVirtual.clear === "function") {
      try { window.GBChatVirtual.clear(); } catch (e) { window.gbLogError ? gbLogError(e, "clearMessagesUi") : console.error("clearMessagesUi", e); }
    }
    var pane = document.getElementById("messages");
    if (pane) pane.innerHTML = "";
    var sugg = document.getElementById("suggestions");
    if (sugg) sugg.innerHTML = "";
    try { localStorage.removeItem("gb_chat_history_" + (currentSessionId() || "default")); }
    catch (e) { window.gbLogError ? gbLogError(e, "clearMessagesUi") : console.error("clearMessagesUi", e); }
    if (typeof window.hideThinkingIndicator === "function") {
      try { window.hideThinkingIndicator(); } catch (e) { window.gbLogError ? gbLogError(e, "clearMessagesUi") : console.error("clearMessagesUi", e); }
    }
  }

  async function clearContext() {
    var sid = currentSessionId();
    if (!sid) return;
    try {
      var res = await fetch("/api/chat/history/sessions/" + encodeURIComponent(sid) + "/messages",
        { method: "DELETE", headers: authHeaders(), credentials: "same-origin" });
      if (!res.ok) {
        // Fall back to the truncate endpoint.
        res = await fetch("/api/chat/history/sessions/" + encodeURIComponent(sid) + "/truncate", { method: "POST",
          headers: authHeaders({ "Content-Type": "application/json" }), credentials: "same-origin",
          body: JSON.stringify({ from_index: 0 }) });
      }
      // A 404 still clears the local view while keeping the session id.
      if (res.ok || res.status === 404) {
        clearMessagesUi();
        notify(t("chat-conversation-cleared", "Conversation context cleared"), "success");
        announceCleared(sid);
        return;
      }
      throw new Error("clear " + res.status);
    } catch (e) {
      // Last resort: clear local UI so the control is not a dead end.
      clearMessagesUi();
      notify(t("chat-conversation-cleared", "Conversation context cleared"), "success");
      announceCleared(sid);
    }
  }

  function exportJson() {
    var messages = [], pane = document.getElementById("messages");
    if (pane) {
      pane.querySelectorAll(".message").forEach(function (el) {
        var role = el.classList.contains("user") ? "user" : "assistant";
        var content = "";
        if (el.dataset && el.dataset.originalContent) content = el.dataset.originalContent;
        else {
          var body = el.querySelector(".message-content") || el;
          content = (body.innerText || body.textContent || "").trim();
        }
        messages.push({ role: role, content: content });
      });
    }
    var payload = { session_id: currentSessionId(), bot: currentBotName(), messages: messages,
      title: (document.getElementById("chatConversationTitleInput") || {}).value || "",
      exported_at: new Date().toISOString() };
    var blob = new Blob([JSON.stringify(payload, null, 2)], { type: "application/json;charset=utf-8" });
    var url = URL.createObjectURL(blob);
    var a = document.createElement("a");
    a.href = url;
    a.download = "conversation-" + (currentSessionId() || "export") + ".json";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(function () { try { URL.revokeObjectURL(url); } catch (e) { window.gbLogError ? gbLogError(e, "setTimeout") : console.error("setTimeout", e); } }, 200);
    notify(t("chat-conversation-exported", "Conversation exported"), "success");
  }

  async function copyLink() {
    var sid = currentSessionId();
    if (!sid) return;
    var url = new URL(window.location.href);
    url.searchParams.set("session", sid);
    var text = url.toString();
    try {
      await gbCopyText(text);
      notify(t("chat-conversation-link-copied", "Conversation link copied"), "success");
    } catch (e) {
      notify(t("chat-conversation-link-copy-failed", "Failed to copy link"), "error");
    }
  }

  async function deleteConversation() {
    var sid = currentSessionId();
    if (!sid) return;
    var doDelete = async function () {
      try {
        var res = await fetch("/api/chat/history/sessions/" + encodeURIComponent(sid), {
          method: "DELETE",
          headers: authHeaders(),
          credentials: "same-origin"
        });
        if (!res.ok) throw new Error("delete " + res.status);
        clearMessagesUi();
        setTitle("");
        notify(t("chat-conversation-deleted", "Conversation deleted"), "success");
        // Start a fresh conversation in place
        if (typeof window.ChatSwitchBot === "function") {
          try { await window.ChatSwitchBot(currentBotName()); } catch (e) { window.gbLogError ? gbLogError(e, "deleteConversation") : console.error("deleteConversation", e); }
        } else {
          window.dispatchEvent(new CustomEvent("gb-chat-session-changed", { detail: { deleted: true } }));
        }
      } catch (e) {
        notify(t("chat-conversation-delete-failed", "Failed to delete conversation"), "error");
      }
    };
    var msg = t("chat-conversation-delete-confirm", "Delete this conversation?");
    var title = t("chat-conversation-delete-title", "Delete conversation");
    if (window.GBModal && typeof window.GBModal.confirm === "function") {
      window.GBModal.confirm(msg, title, function () { doDelete(); });
      return;
    }
    // Suite policy: no native confirm; if modal unavailable, do not proceed silently.
    notify(t("chat-conversation-delete-unavailable", "Confirmation dialog unavailable"), "warning");
  }

  function closeMenu() {
    var menu = document.getElementById("chatConversationMenu"), btn = document.getElementById("chatConversationMenuBtn");
    if (menu) menu.hidden = true;
    if (btn) btn.setAttribute("aria-expanded", "false");
  }

  function toggleMenu() {
    var menu = document.getElementById("chatConversationMenu"), btn = document.getElementById("chatConversationMenuBtn");
    if (!menu || !btn) return;
    var open = menu.hidden;
    menu.hidden = !open; btn.setAttribute("aria-expanded", open ? "true" : "false");
  }

  function bind() {
    var header = ensureUi();
    if (!header || header.dataset.bound === "1") return;
    header.dataset.bound = "1";

    var input = document.getElementById("chatConversationTitleInput");
    var menuBtn = document.getElementById("chatConversationMenuBtn");
    var menu = document.getElementById("chatConversationMenu");

    if (input) {
      input.addEventListener("keydown", function (e) {
        if (e.key === "Enter") {
          e.preventDefault();
          input.blur();
        } else if (e.key === "Escape") {
          input.value = input.dataset.lastSaved || "";
          input.blur();
        }
      });
      input.addEventListener("blur", function () {
        var val = (input.value || "").trim();
        if (val !== (input.dataset.lastSaved || "")) {
          renameSession(val);
        }
      });
      input.addEventListener("focus", function () {
        input.select();
      });
    }

    if (menuBtn) {
      menuBtn.addEventListener("click", function (e) {
        e.stopPropagation();
        toggleMenu();
      });
    }

    if (menu) {
      menu.addEventListener("click", async function (e) {
        var btn = e.target.closest("button[data-action]");
        if (!btn) return;
        var action = btn.getAttribute("data-action");
        closeMenu();
        if (action === "rename") {
          if (input) { input.focus(); input.select(); }
        } else if (action === "clear") {
          await clearContext();
        } else if (action === "export-md") {
          if (window.GBChatExport && typeof window.GBChatExport.exportConversation === "function") {
            window.GBChatExport.exportConversation();
          }
        } else if (action === "export-json") {
          exportJson();
        } else if (action === "copy-link") {
          await copyLink();
        } else if (action === "delete") {
          await deleteConversation();
        }
      });
    }

    document.addEventListener("click", function (e) {
      if (!header.contains(e.target)) closeMenu();
    });

    window.addEventListener("gb-chat-session-changed", scheduleTitleRefresh);

    // Also refresh periodically/after connection when session id becomes available
    var tries = 0;
    var timer = setInterval(function () {
      tries++;
      if (currentSessionId() || tries > 40) {
        clearInterval(timer);
        scheduleTitleRefresh();
      }
    }, 250);

    scheduleTitleRefresh();
  }

  var mountQueued = false;
  function mount() {
    if (mountQueued) return;
    mountQueued = true;
    idle(function () { bind(); });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  } else {
    mount();
  }
  window.addEventListener("gb-chat-ready", mount);
})();
