/* Chat messages module: Scroll helpers: pin the transcript to the live edge. */
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
