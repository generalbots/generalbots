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
//
// #2014 — the pin must never fight the reader: it only runs while they are at
// the bottom edge, and any deliberate scroll (wheel, keys, scrollbar drag) ends
// it for good. The 250 ms run-timeline heartbeat would otherwise re-arm this on
// every tick during a VM cold start and drag the viewport down each second.
// Only one stick-to-bottom timer may run at a time: during history replay
// (hundreds of messages) the old code armed a new 1.4 s interval per message,
// which fought the reader and made the transcript "dance". Re-arming now
// extends the existing window instead of stacking intervals.
var gbStickTimer = null;
function stickToBottom(duration) {
  var messages = document.getElementById("messages");
  if (!messages) return;
  if (ChatState.isUserAtBottom === false) return;
  if (gbStickTimer) { clearInterval(gbStickTimer); gbStickTimer = null; }
  var intentAt = gbPinCancelAt;
  var end = Date.now() + (duration || 1200);
  gbStickTimer = setInterval(function () {
    if (!messages.isConnected) { clearInterval(gbStickTimer); gbStickTimer = null; return; }
    if (ChatState.suppressPersist) { clearInterval(gbStickTimer); gbStickTimer = null; return; }
    if (gbPinCancelAt > intentAt || ChatState.isUserAtBottom === false) {
      clearInterval(gbStickTimer); gbStickTimer = null; return;
    }
    messages.scrollTop = messages.scrollHeight;
    if (Date.now() > end) { clearInterval(gbStickTimer); gbStickTimer = null; }
  }, 120);
}

/* #2014 — reader intent. Any of these means "the viewport is mine": a wheel,
 * a keyboard page, or a scrollbar drag (pointerdown/mousedown on the track or
 * thumb). The transcript's own pins land at the bottom, so a scroll event that
 * leaves the reader far from the bottom can only be theirs. */
var gbPinCancelAt = 0;

function gbMarkReaderIntent() {
  gbPinCancelAt = Date.now();
}

function gbAttachReaderIntent(messages) {
  if (!messages || messages.dataset.gbReaderIntent === "1") return;
  messages.dataset.gbReaderIntent = "1";
  ["wheel", "touchstart", "pointerdown", "mousedown", "keydown"].forEach(function (name) {
    messages.addEventListener(name, gbMarkReaderIntent, { passive: true });
  });
  messages.addEventListener("scroll", function () {
    // Do not treat scroll drift during an active turn as reader intent.
    var pending = !!(ChatState && (ChatState.pendingResponseAt || ChatState.isStreaming));
    if (!pending && ChatState.isUserAtBottom === false) gbMarkReaderIntent();
  }, { passive: true });
}

// History replay appends messages whose inline artifacts and images load
// lazily, so the transcript keeps growing after the first scroll. Keep the
// newest turn in view until the layout settles, and stop the moment the
// reader scrolls away (their intent wins).
function followLiveEdge(maxMs) {
  var messages = document.getElementById("messages");
  if (!messages) return;
  gbAttachReaderIntent(messages);
  var intentAt = gbPinCancelAt;
  var end = Date.now() + (maxMs || 3000);
  var cancelled = false;
  function cancel() { cancelled = true; cleanup(); }
  function cleanup() {
    messages.removeEventListener("wheel", cancel);
    messages.removeEventListener("touchstart", cancel);
    messages.removeEventListener("pointerdown", cancel);
    messages.removeEventListener("mousedown", cancel);
    messages.removeEventListener("keydown", cancel);
  }
  messages.addEventListener("wheel", cancel, { passive: true });
  messages.addEventListener("touchstart", cancel, { passive: true });
  messages.addEventListener("pointerdown", cancel, { passive: true });
  messages.addEventListener("mousedown", cancel, { passive: true });
  messages.addEventListener("keydown", cancel);
  function step() {
    if (cancelled) return;
    if (ChatState.suppressPersist) { cancelled = true; cleanup(); return; }
    if (gbPinCancelAt > intentAt || ChatState.isUserAtBottom === false) {
      cancelled = true;
      cleanup();
      return;
    }
    messages.scrollTop = messages.scrollHeight;
    if (Date.now() < end) requestAnimationFrame(step);
    else cleanup();
  }
  requestAnimationFrame(step);
}

// #1797 / #2098 — thinking state. The visible element and its elapsed timer
// are owned by the turn state machine (chat-turn-state.js): visibility follows
// the turn state (queued → thinking → tool → streaming → done/error) instead of
// ad-hoc show/hide calls, and every turn is keyed by tab id. These two names
// stay as the compatibility surface older modules call.
function showThinkingIndicator(opts) {
  if (window.GBTurn) window.GBTurn.start("", opts);
}

function hideThinkingIndicator(opts) {
  if (window.GBTurn) window.GBTurn.forceHide(opts);
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
  // #222x — during heavy history replay scrollHeight keeps growing, so the
  // ratios stored on the first pass drift and the green ticks land in the
  // wrong place. Drop the cached ratios whenever the transcript grew.
  var lastH = parseInt(overlay.getAttribute("data-gb-last-h") || "0", 10);
  if (lastH && lastH !== messages.scrollHeight) {
    messages.querySelectorAll(".message.user").forEach(function (m) {
      m.removeAttribute("data-gb-marker-ratio");
    });
  }
  overlay.setAttribute("data-gb-last-h", String(messages.scrollHeight));
  gbSyncMarkerOverlay();
  overlay.innerHTML = "";
  var total = Math.max(messages.scrollHeight, 1);
  var height = Math.max(messages.clientHeight, 1);

  // #2229 — when virtualized, user turns may be detached; read from the
  // virtualizer's item list so markers remain accurate and jump can rematerialize.
  var V = window.GBChatVirtual;
  if (V && V.isEnabled && V.isEnabled() && V._items) {
    var items = V._items();
    var gap = (function(){ try { var cs=getComputedStyle(messages); var g=parseFloat(cs.rowGap||cs.gap||"16"); return isNaN(g)?16:g; } catch(e){return 16;} })();
    var acc = 0;
    for (var vi = 0; vi < items.length; vi++) {
      var it = items[vi];
      var h = ((it.h && it.h > 0) ? it.h : (it.el ? it.el.offsetHeight : 48)) + gap;
      if (it.el && it.el.classList && it.el.classList.contains("user")) {
        var ratio = acc / total;
        var top = Math.max(0, Math.min(height - 4, ratio * height));
        var marker = document.createElement("button");
        marker.type = "button";
        marker.className = "scrollbar-marker user-marker";
        marker.style.top = top + "px";
        var label = (typeof gbMarkerLabel === "function") ? gbMarkerLabel(it.el) : "Message";
        marker.setAttribute("aria-label", "Jump to message: " + label);
        marker.title = label;
        var tooltip = document.createElement("span");
        tooltip.className = "scrollbar-marker-tooltip";
        tooltip.textContent = label;
        marker.appendChild(tooltip);
        (function (target, targetAcc, tick) {
          tick.addEventListener("click", function () {
            if (V && V.ensureVisible) V.ensureVisible(target);
            var top = targetAcc;
            if (target && target.isConnected) {
              // Prefer live geometry after materialization to avoid drift from estimated heights.
              var crect = messages.getBoundingClientRect();
              var trect = target.getBoundingClientRect();
              top = messages.scrollTop + (trect.top - crect.top);
            }
            messages.scrollTo({ top: Math.max(0, top - 8), behavior: "smooth" });
          });
        })(it.el, acc, marker);
        overlay.appendChild(marker);
      }
      acc += h;
    }
    return;
  }

  var turns = messages.querySelectorAll(".message.user");
  for (var i = 0; i < turns.length; i++) {
    var message = turns[i];
    var top;
    var storedRatio = message.getAttribute("data-gb-marker-ratio");
    if (storedRatio !== null) {
      top = Math.max(0, Math.min(height - 4, parseFloat(storedRatio) * height));
    } else {
      var ratio = message.offsetTop / total;
      message.setAttribute("data-gb-marker-ratio", String(ratio));
      top = Math.max(0, Math.min(height - 4, ratio * height));
    }
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
      if (ChatState.suppressPersist) return;
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
  var dist = messages.scrollHeight - messages.scrollTop - messages.clientHeight;
  // Button appears only when not near bottom yet. Use a slightly larger
  // threshold than "at bottom" tracking so the button doesn't flicker.
  var isNearBottom = dist < 100;
  if (isNearBottom) {
    scrollBtn.classList.remove("visible");
  } else {
    scrollBtn.classList.add("visible");
  }
}
