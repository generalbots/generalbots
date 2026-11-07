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

// #1797 — thinking state: an animated icon plus elapsed seconds so a slow
// provider never looks frozen. The timer is module-scoped so hide()/show()
// cannot leak intervals across turns or bot switches.
var gbThinkingTimer = null;
var gbThinkingStartedAt = 0;

function gbUpdateThinkingElapsed() {
  var el = document.querySelector("#thinking-indicator .thinking-elapsed");
  if (!el) return;
  var seconds = Math.max(0, Math.floor((Date.now() - gbThinkingStartedAt) / 1000));
  el.textContent = seconds + "s";
}

function gbStartThinkingTimer(div, startedAt) {
  gbThinkingStartedAt = startedAt || Date.now();
  if (gbThinkingTimer) clearInterval(gbThinkingTimer);
  gbUpdateThinkingElapsed();
  gbThinkingTimer = setInterval(function () {
    if (!div.isConnected) {
      clearInterval(gbThinkingTimer);
      gbThinkingTimer = null;
      return;
    }
    gbUpdateThinkingElapsed();
  }, 1000);
}

function showThinkingIndicator() {
  var existing = document.getElementById("thinking-indicator");
  if (existing) {
    if (!gbThinkingTimer) gbStartThinkingTimer(existing, gbThinkingStartedAt || Date.now());
    return;
  }
  var messages = document.getElementById("messages");
  if (!messages) return;
  var div = document.createElement("div");
  div.id = "thinking-indicator";
  div.className = "message bot";
  div.innerHTML = '<div class="message-content bot-message"><div class="thinking-indicator" role="status" aria-live="polite">' +
    '<span class="thinking-spinner" aria-hidden="true"></span>' +
    '<span class="thinking-label">Thinking</span>' +
    '<span class="thinking-elapsed" aria-hidden="true">0s</span>' +
    '</div></div>';
  messages.appendChild(div);
  gbStartThinkingTimer(div, Date.now());
  scrollToBottom(true);
  if (typeof scheduleScrollbarMarkers === "function") scheduleScrollbarMarkers();
}

function hideThinkingIndicator() {
  if (gbThinkingTimer) {
    clearInterval(gbThinkingTimer);
    gbThinkingTimer = null;
  }
  gbThinkingStartedAt = 0;
  var el = document.getElementById("thinking-indicator");
  if (el) el.remove();
}

/* #1797 — conversation markers along the right edge of the transcript.
 * One tick per user turn marks where a new thread starts; the overlay is
 * pinned to the viewport while the message pane scrolls underneath it. */
var gbMarkersRefreshQueued = false;

function gbMarkerOverlay(messages) {
  var overlay = messages.querySelector(".scrollbar-markers");
  if (!overlay) {
    overlay = document.createElement("div");
    overlay.className = "scrollbar-markers";
    overlay.setAttribute("aria-hidden", "true");
    messages.appendChild(overlay);
  }
  return overlay;
}

function gbSyncMarkerOverlay() {
  var messages = document.getElementById("messages");
  if (!messages) return;
  var overlay = messages.querySelector(".scrollbar-markers");
  if (!overlay) return;
  // Counteract the transcript's scroll offset so the ticks stay glued to the
  // visible scrollbar instead of riding the content upwards.
  overlay.style.top = messages.scrollTop + "px";
  overlay.style.height = messages.clientHeight + "px";
}

function gbMarkerLabel(message) {
  var body = message.querySelector(".message-content") || message;
  var text = (body.innerText || body.textContent || "").replace(/\s+/g, " ").trim();
  if (text.length > 64) text = text.slice(0, 61) + "\u2026";
  var ts = parseInt(message.getAttribute("data-ts") || "0", 10);
  var when = ts ? formatRelativeTime(ts) : "";
  if (text && when) return text + " \u00b7 " + when;
  return text || when || "Message";
}

function gbRefreshScrollbarMarkers() {
  var messages = document.getElementById("messages");
  if (!messages) return;
  var overlay = gbMarkerOverlay(messages);
  gbSyncMarkerOverlay();
  overlay.innerHTML = "";
  var total = Math.max(messages.scrollHeight, 1);
  var height = Math.max(messages.clientHeight, 1);
  var turns = messages.querySelectorAll(".message.user");
  for (var i = 0; i < turns.length; i++) {
    var message = turns[i];
    var top = Math.max(0, Math.min(height - 4, (message.offsetTop / total) * height));
    var marker = document.createElement("button");
    marker.type = "button";
    marker.className = "scrollbar-marker user-marker";
    marker.style.top = top + "px";
    var label = gbMarkerLabel(message);
    marker.setAttribute("aria-label", "Jump to message: " + label);
    marker.title = label;
    var tooltip = document.createElement("span");
    tooltip.className = "scrollbar-marker-tooltip";
    tooltip.textContent = label;
    marker.appendChild(tooltip);
    (function (target, tick) {
      tick.addEventListener("click", function () {
        messages.scrollTo({ top: Math.max(0, target.offsetTop - 8), behavior: "smooth" });
      });
    })(message, marker);
    overlay.appendChild(marker);
  }
}

function scheduleScrollbarMarkers() {
  if (gbMarkersRefreshQueued) return;
  gbMarkersRefreshQueued = true;
  requestAnimationFrame(function () {
    gbMarkersRefreshQueued = false;
    gbRefreshScrollbarMarkers();
  });
}

function initScrollbarMarkers() {
  var messages = document.getElementById("messages");
  if (!messages || messages.dataset.gbMarkers === "1") return;
  messages.dataset.gbMarkers = "1";
  messages.addEventListener("scroll", gbSyncMarkerOverlay, { passive: true });
  window.addEventListener("resize", scheduleScrollbarMarkers);
  if (window.MutationObserver) {
    new MutationObserver(function (entries) {
      // Ignore the marker overlay's own churn; only real message boundaries
      // (new turns, history pages) need a rebuild.
      var changed = entries.some(function (entry) {
        if (entry.type !== "childList") return false;
        var nodes = Array.prototype.slice.call(entry.addedNodes)
          .concat(Array.prototype.slice.call(entry.removedNodes));
        return nodes.some(function (node) {
          return node.nodeType === 1 && node.classList && node.classList.contains("message");
        });
      });
      if (changed) scheduleScrollbarMarkers();
    }).observe(messages, { childList: true });
  }
  scheduleScrollbarMarkers();
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initScrollbarMarkers);
} else {
  initScrollbarMarkers();
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
