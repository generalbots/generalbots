/* Split from chat-init.js for #2305 (450-line limit). */
function installComposerFocusAfterLayoutChange() {
  if (window.__gbComposerFocusBound) return;
  window.__gbComposerFocusBound = true;
  var lastChatClickAt = 0;
  document.addEventListener("click", function (event) {
    var target = event.target;
    if (!target || !target.closest) return;
    if (target.closest('[id^="window-chat"], .gb-chat-pane, #chat-form, #chatForm')) {
      lastChatClickAt = Date.now();
    }
  }, true);
  document.addEventListener("gb-window-changed", function () {
    if (!lastChatClickAt || Date.now() - lastChatClickAt > 1500) return;
    lastChatClickAt = 0;
    refocusComposerAfterLayoutChange();
  });
}

function setupEventHandlers() {
var form = document.getElementById("chatForm");
var input = document.getElementById("messageInput");
var sendBtn = document.getElementById("sendBtn");

if (form) {
form.onsubmit = function (e) { e.preventDefault(); sendMessage(); return false; };
}

if (input) {
// BBSIA: grow the composer with its content, never past 180px (#2516).
// On a phone the 180px cap is taller than the space left once the keyboard
// is open, so the ceiling is the smaller of 180px and a third of the visible
// viewport (published as --gb-vv-height by mobile-viewport.js).
var composerMax = function () {
  var fixed = 180;
  var vv = window.visualViewport;
  var visible = vv && vv.height ? vv.height : window.innerHeight;
  if (visible && visible < 900) {
    fixed = Math.min(fixed, Math.round(visible * 0.3));
  }
  return Math.max(64, fixed);
};
var autoGrow = function () {
  if (input.tagName !== "TEXTAREA") return;
  input.style.height = "auto";
  input.style.height = Math.min(input.scrollHeight, composerMax()) + "px";
};
input.addEventListener("input", autoGrow);

// Only attach mention handlers if they exist
var mentionInputHandler = window.handleMentionInput;
var mentionKeydownHandler = window.handleMentionKeydown;

if (mentionInputHandler) {
input.addEventListener("input", mentionInputHandler);
}
if (mentionKeydownHandler) {
input.onkeydown = function (e) {
if (mentionKeydownHandler(e)) return;
if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); sendMessage(); }
if (e.key === "PageUp" || e.key === "PageDown") {
e.preventDefault();
var messages = document.getElementById("messages");
if (messages) {
var dir = e.key === "PageUp" ? -1 : 1;
messages.scrollBy({ top: dir * messages.clientHeight, behavior: "instant" });
}
}
};
} else {
input.onkeydown = function (e) {
if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); sendMessage(); }
if (e.key === "PageUp" || e.key === "PageDown") {
e.preventDefault();
var messages = document.getElementById("messages");
if (messages) {
var dir = e.key === "PageUp" ? -1 : 1;
messages.scrollBy({ top: dir * messages.clientHeight, behavior: "instant" });
}
}
};
}
}

if (sendBtn) {
sendBtn.onclick = function (e) { e.preventDefault(); sendMessage(); };
}

var scrollBtn = document.getElementById("scrollToBottom");
if (scrollBtn) {
scrollBtn.addEventListener("click", function () { scrollToBottom(true); ChatState.isUserScrolling = false; ChatState.isUserAtBottom = true; });
}

var messagesEl = document.getElementById("messages");
if (messagesEl) {
// Async layout (iframe artifacts, suggestions, footer resize) changes the
// viewport after the initial scroll. Stay pinned while the reader is near the
// live edge; a deliberate scroll further up is left alone.
if (window.ResizeObserver) {
  var liveEdgeObserver = new ResizeObserver(function () {
    var box = document.getElementById("messages");
    if (!box) return;
    if (ChatState.suppressPersist) return;
    if (!ChatState.isUserAtBottom) return;
    // Pin to live edge when content grows or layout shifts while the reader is
    // at the bottom. Do not run during history replay (suppressPersist) to
    // avoid fighting the bulk-load scroll restoration.
    box.scrollTop = box.scrollHeight;
  });
  liveEdgeObserver.observe(messagesEl);
}
if (window.MutationObserver) {
  var liveEdgeMutations = new MutationObserver(function () {
    requestAnimationFrame(function () {
      var box = document.getElementById("messages");
      if (!box) return;
      if (ChatState.suppressPersist) return;
      if (!ChatState.isUserAtBottom) return;
      box.scrollTop = box.scrollHeight;
    });
  });
  liveEdgeMutations.observe(messagesEl, { childList: true, subtree: true });
}
messagesEl.addEventListener("scroll", function () {
  var box = document.getElementById("messages");
  var dist = box ? (box.scrollHeight - box.scrollTop - box.clientHeight) : 0;
  // While a response is pending/streaming, be lenient: rapid content growth
  // can momentarily push the distance over the normal threshold even though
  // the reader never scrolled away. Flipping to "scrolled up" mid-turn made
  // the transcript stop following and look like it scrolled up (#2244).
  var pending = !!(ChatState && (ChatState.pendingResponseAt || ChatState.isStreaming));
  // Sticky bottom: once we're following the live edge, small layout shifts
  // (streaming/indicator growth, reflow) must not flip us to "scrolled up".
  // A real user scroll away produces a larger distance.
  var threshold = ChatState.isUserAtBottom ? (pending ? 400 : 120) : 40;
  ChatState.isUserAtBottom = dist < threshold;
  if (!pending) {
    ChatState.isUserScrolling = true;
    clearTimeout(messagesEl.scrollTimeout);
    messagesEl.scrollTimeout = setTimeout(function () { ChatState.isUserScrolling = false; }, 1000);
  }
  updateScrollButton();
});
}

document.addEventListener("click", function (e) {
if (!e.target.closest("#mentionDropdown") && !e.target.closest("#messageInput")) {
var hideMention = window.hideMentionDropdown;
if (hideMention) {
hideMention();
}
}
});
}

function initChat() {
  // #1753 — deleting the open conversation reinitializes through here.
  // Duplicate delete events (or a re-injected module binding again) must not
  // run two /api/auth rounds and two sockets for the SAME chat instance.
  // #2285 — the guard is per-window/instance (not page-global): opening a
  // second chat while the first is booting must still produce a working second
  // chat. A boot request that arrives while this instance is in flight is
  // coalesced into one queued re-run on release instead of being dropped.
  var root = document.getElementById("chat-app") || document.body;
  if (root.__gbInitChatBusy) {
    root.__gbInitChatQueued = true;
    return;
  }
  root.__gbInitChatBusy = true;
  root.__gbInitChatQueued = false;
  var release = function () {
    if (!root.__gbInitChatBusy) return;
    root.__gbInitChatBusy = false;
    if (root.__gbInitChatTimer) {
      clearTimeout(root.__gbInitChatTimer);
      root.__gbInitChatTimer = null;
    }
    if (root.__gbInitChatQueued) {
      root.__gbInitChatQueued = false;
      setTimeout(initChat, 0);
    }
  };
  root.__gbInitChatRelease = release;
  if (!root.__gbInitChatReleaseBound) {
    root.__gbInitChatReleaseBound = true;
    window.addEventListener("gb-chat-session-changed", function () {
      if (root.__gbInitChatRelease) root.__gbInitChatRelease();
    });
    window.addEventListener("gb-chat-connected", function () {
      if (root.__gbInitChatRelease) root.__gbInitChatRelease();
    });
  }
  if (root.__gbInitChatTimer) clearTimeout(root.__gbInitChatTimer);
  root.__gbInitChatTimer = setTimeout(release, 12000);
  if (window.GBAppLifecycle) GBAppLifecycle.begin("chat");
  if (typeof loadBotConfig === 'function') {
    loadBotConfig();
  }
  proceedWithChatInit();
  autoFocusInput();
  installComposerFocusAfterLayoutChange();
  // #2228 — after a period of inactivity, return focus to the composer.
  (function installIdleRefocus() {
    if (window.__gbIdleRefocusBound) return;
    window.__gbIdleRefocusBound = true;
    var idleTimer = null;
    function schedule() {
      if (idleTimer) clearTimeout(idleTimer);
      idleTimer = setTimeout(function () {
        var input = document.getElementById("messageInput");
        if (!input || !input.isConnected) return;
        var active = document.activeElement;
        var typingElsewhere = active && active !== document.body && active !== input &&
          (active.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(active.tagName));
        if (typingElsewhere) return;
        if (String(input.value || "").trim() !== "") return;
        try { input.focus({ preventScroll: true }); } catch (e) {}
      }, 15000);
    }
    ["mousemove","keydown","click","scroll","touchstart"].forEach(function (ev) {
      document.addEventListener(ev, schedule, { passive: true });
    });
    schedule();
  })();
}

function showChatApp() {
  var chatApp = document.getElementById("chat-app");
  if (chatApp) {
    chatApp.style.opacity = "1";
    chatApp.style.visibility = "visible";
  }
}

window.showChatApp = showChatApp;

// Wait for DOM to be ready before initializing
if (typeof document !== 'undefined') {
if (document.readyState === 'loading') {
(function(){ var __cb = function() {
setupEventHandlers();
initChat();
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); }
// #2349 — a conversation with no turns must say so instead of showing a blank
// pane; the card removes itself as soon as the first bubble is appended.
if (typeof renderChatEmptyState === "function") renderChatEmptyState();
})();
} else {
setupEventHandlers();
initChat();
}
}
