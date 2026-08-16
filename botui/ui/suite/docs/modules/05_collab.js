"use strict";
/* docs collab — WebSocket collaboration */

function initCollab() {
  if (!window.GBCollab) return;
  var connStatus = document.getElementById("gb-conn-status");
  var docId = (document.getElementById("docTitle") && document.getElementById("docTitle").value) || "current";
  var typingEl = document.getElementById("typing-indicator");
  window.GBCollab.connect({
    app: "docs",
    docId: docId,
    collaboratorsEl: document.getElementById("collaborators"),
    typingEl: typingEl,
    onConnect: function () {
      if (connStatus) { connStatus.className = "gb-connection-status online"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "online"; }
    },
    onDisconnect: function () {
      if (connStatus) { connStatus.className = "gb-connection-status offline"; connStatus.style.display = "inline-flex"; connStatus.querySelector(".label").textContent = "offline"; }
      if (window.DocsPresence) window.DocsPresence.clearAll();
    },
    onMessage: function (msg) {
      if (!msg || !window.DocsPresence) return;
      if (msg.msg_type === "cursor") window.DocsPresence.cursor(msg);
    },
    onSelection: function (msg) {
      if (window.DocsPresence) window.DocsPresence.selection(msg);
    },
    onPresence: function (users) {
      if (window.DocsPresence) window.DocsPresence.sync(users);
    },
    onTyping: function (msg) {
      var map = (window.__gbTypingUsers = window.__gbTypingUsers || new Map());
      if (msg.msg_type === "typing_start") map.set(msg.user_id, msg);
      else map.delete(msg.user_id);
      var arr = Array.from(map.values()).filter(function (m) { return Date.now() - (m.timestamp || 0) < 5000; });
      if (window.GBCollab && window.GBCollab.helpers) {
        window.GBCollab.helpers.renderTypingIndicator(typingEl, arr);
      }
    },
    onEdit: function (msg) {
      if (!msg) return;
      var article = document.querySelector("article[contenteditable]");
      if (!article || article.dataset.suppressRemote) return;
      var hasDelta = typeof msg.position === "number" && (typeof msg.removeLength === "number" || typeof msg.length === "number");
      article.dataset.suppressRemote = "1";
      if (hasDelta) {
        var pos = Math.max(0, msg.position | 0);
        var removeLength = typeof msg.removeLength === "number" ? msg.removeLength : (msg.length && msg.length > 0 && msg.content === "" ? msg.length : 0);
        applyDeltaEdit(article, pos, msg.content, removeLength);
      } else if (typeof msg.content === "string") {
        var offset = getCaretCharacterOffsetWithin(article);
        article.innerHTML = msg.content;
        setCaretPosition(article, offset);
      }
      article.dataset.suppressRemote = "";
      updatePageCount();
    }
  });
}
