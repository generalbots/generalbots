/* Chat messages module: Local transcript persistence and the live chat status line. */

// #1797 — hover timestamps. Messages carry an epoch in `data-ts`; the visible
// label is recomputed when the pointer enters so it stays relative ("3m ago").
function formatRelativeTime(ts) {
  var then = typeof ts === "number" ? ts : Date.parse(ts);
  if (!then || isNaN(then)) return "";
  var seconds = Math.max(0, Math.floor((Date.now() - then) / 1000));
  if (seconds < 10) return "just now";
  if (seconds < 60) return seconds + "s ago";
  var minutes = Math.floor(seconds / 60);
  if (minutes < 60) return minutes + "m ago";
  var hours = Math.floor(minutes / 60);
  if (hours < 24) return hours + "h ago";
  var days = Math.floor(hours / 24);
  if (days < 7) return days + "d ago";
  try {
    return new Date(then).toLocaleDateString();
  } catch (e) {
    return "";
  }
}

function attachMessageTimestamp(message, ts) {
  if (!message) return;
  var then = typeof ts === "number" ? ts : Date.parse(ts);
  if (!then || isNaN(then)) return;
  message.setAttribute("data-ts", String(then));
  try {
    message.dataset.time = new Date(then).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch (e) { /* Intl unavailable — relative label still works */ }
  var stamp = message.querySelector(":scope > .gb-message-time");
  if (!stamp) {
    stamp = document.createElement("span");
    stamp.className = "gb-message-time";
    stamp.setAttribute("aria-hidden", "true");
    message.appendChild(stamp);
    message.addEventListener("mouseenter", function () {
      stamp.textContent = formatRelativeTime(then);
      stamp.title = new Date(then).toLocaleString();
    });
  }
  stamp.textContent = formatRelativeTime(then);
  stamp.title = new Date(then).toLocaleString();
}

function persistLocalMessage(sender, content) {
  try {
    var key = "gb_chat_history_" + (ChatState.currentSessionId || "default");
    var history = gbStoreGet(key, []);
    history.push({ sender: sender, content: content, ts: Date.now() });
    if (history.length > 500) history = history.slice(-500);
    localStorage.setItem(key, JSON.stringify(history));
  } catch (e) {}
}

function loadLocalHistory() {
  try {
    var key = "gb_chat_history_" + (ChatState.currentSessionId || "default");
    return gbStoreGet(key, []);
  } catch (e) { return []; }
}

function listLocalSessions() {
  var sessions = [];
  try {
    for (var i = 0; i < localStorage.length; i++) {
      var k = localStorage.key(i);
      if (k && k.indexOf("gb_chat_history_") === 0) {
        var id = k.replace("gb_chat_history_", "");
        var msgs = gbStoreGet(k, []);
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
