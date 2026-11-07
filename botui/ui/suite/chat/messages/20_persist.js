/* Chat messages module: Local transcript persistence and the live chat status line. */
function persistLocalMessage(sender, content) {
  try {
    var key = "gb_chat_history_" + (ChatState.currentSessionId || "default");
    var history = JSON.parse(localStorage.getItem(key) || "[]");
    history.push({ sender: sender, content: content, ts: Date.now() });
    if (history.length > 500) history = history.slice(-500);
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
