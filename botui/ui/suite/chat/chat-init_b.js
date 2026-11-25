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
// #2933 — one live-edge pass per frame for resize + mutation bursts.
var gbLiveEdgeQueued = false;
function gbPinLiveEdge() {
  gbLiveEdgeQueued = false;
  var box = document.getElementById("messages");
  if (!box || ChatState.suppressPersist || !ChatState.isUserAtBottom) return;
  var bottom = box.scrollHeight; // batch the read before the write
  box.scrollTop = bottom;
}
function gbScheduleLiveEdge() {
  if (gbLiveEdgeQueued) return;
  gbLiveEdgeQueued = true;
  requestAnimationFrame(gbPinLiveEdge);
}
// #2933 — one geometry pass per frame; the scroll listener never reads layout.
var gbScrollFrameQueued = false;
function gbOnScrollFrame() {
  gbScrollFrameQueued = false;
  var box = document.getElementById("messages");
  if (!box) return;
  var dist = box.scrollHeight - box.scrollTop - box.clientHeight; // batched reads
  var pending = !!(ChatState && (ChatState.pendingResponseAt || ChatState.isStreaming));
  var threshold = ChatState.isUserAtBottom ? (pending ? 400 : 120) : 40;
  ChatState.isUserAtBottom = dist < threshold;
  if (!pending) {
    ChatState.isUserScrolling = true;
    clearTimeout(box.scrollTimeout);
    box.scrollTimeout = setTimeout(function () { ChatState.isUserScrolling = false; }, 1000);
  }
  updateScrollButton(dist);
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
// On a phone the cap is min(180, visibleHeight * 0.3): visualViewport height
// comes from modules/mobile_viewport.js (falling back to innerHeight).
var composerMax = function () {
  var fixed = 180;
  var visible = window.GBChatVisibleHeight ? window.GBChatVisibleHeight() : window.innerHeight;
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
input.addEventListener("gb-chat-viewport", autoGrow);
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
// Async layout shifts the viewport; one coalesced frame pin per burst.
// A real user scroll away produces a larger distance than streaming growth.
if (window.ResizeObserver) {
  var liveEdgeObserver = new ResizeObserver(gbScheduleLiveEdge);
  liveEdgeObserver.observe(messagesEl);
}
if (window.MutationObserver) {
  var liveEdgeMutations = new MutationObserver(gbScheduleLiveEdge);
  liveEdgeMutations.observe(messagesEl, { childList: true, subtree: true });
}
messagesEl.addEventListener("scroll", function () {
  if (gbScrollFrameQueued) return;
  gbScrollFrameQueued = true;
  requestAnimationFrame(gbOnScrollFrame);
}, { passive: true });
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
  // #1753/#2285 — per-instance guard: coalesce duplicate boot requests into
  // one queued re-run so two sockets never bind to the same chat instance.
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
        if (String(input.value || "").trim() !== "") return;
        gbScheduleComposerFocus();
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
// #2349 card is decoration; #2933 paint on idle.
window.gbIdle(function(){ if (typeof renderChatEmptyState === "function") renderChatEmptyState(); });
}; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); }
})();
} else {
setupEventHandlers();
initChat();
}
}
