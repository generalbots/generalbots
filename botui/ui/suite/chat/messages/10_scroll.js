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

// Instant snap used after layout-changing content (history replay, iframes).
function snapToBottom() {
  var messages = document.getElementById("messages");
  if (messages) messages.scrollTop = messages.scrollHeight;
}

// #2014 — one timer: re-arming extends the pin window instead of stacking.
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

/* #2014 — reader intent: wheel/key/drag. A scroll far from the bottom is theirs. */
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
    messages.removeEventListener("keydown", cancel);
  }
  messages.addEventListener("wheel", cancel, { passive: true });
  messages.addEventListener("touchstart", cancel, { passive: true });
  messages.addEventListener("pointerdown", cancel, { passive: true });
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

// #1797 / #2098 — thinking state is owned by the turn state machine.
function showThinkingIndicator(opts) {
  if (window.GBTurn) window.GBTurn.start("", opts);
}

function hideThinkingIndicator(opts) {
  if (window.GBTurn) window.GBTurn.forceHide(opts);
}

/* #2933 — #1797 conversation markers: one tick per user turn. */
var gbMarkersRefreshQueued = false;
var gbMarkerSyncQueued = false;
// #2933 — the boot idle callback only arms this flag; every geometry read is
// deferred to the first real scroll (one rAF) or mutation (coalesced idle pass).
var gbMarkersBootPending = true;

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

// textContent (never innerText) — innerText forces a synchronous layout.
function gbMarkerLabel(message) {
  var body = message.querySelector(".message-content") || message;
  var text = (body.textContent || "").replace(/\s+/g, " ").trim();
  if (text.length > 64) text = text.slice(0, 61) + "\u2026";
  var ts = parseInt(message.getAttribute("data-ts") || "0", 10);
  var when = ts ? formatRelativeTime(ts) : "";
  if (text && when) return text + " \u00b7 " + when;
  return text || when || "Message";
}

// Marker jumps are user-initiated; layout reads stay out of the boot path.
function gbMarkerJump(messages, target, top, live) {
  if (live && target && window.GBChatVirtual && window.GBChatVirtual.ensureVisible) {
    window.GBChatVirtual.ensureVisible(target);
  }
  if (live && target && target.isConnected) {
    var c = messages.getBoundingClientRect();
    var t = target.getBoundingClientRect();
    top = messages.scrollTop + (t.top - c.top);
  }
  messages.scrollTo({ top: Math.max(0, top - 8), behavior: "smooth" });
}

/* #2933 — marker rebuild: every layout read is captured in the read phase,
 * then the overlay is cleared, sized and repopulated in a write-only phase. */
function gbRefreshScrollbarMarkers() {
  var messages = document.getElementById("messages");
  if (!messages) return;
  // READ PHASE — no writes until every geometry/label read is captured.
  var rawH = messages.scrollHeight;
  var total = Math.max(rawH, 1);
  var height = Math.max(messages.clientHeight, 1);
  var top = messages.scrollTop;
  var V = window.GBChatVirtual;
  var virtual = !!(V && V.isEnabled && V.isEnabled() && V._items);
  var gap = 16;
  if (virtual) {
    try { var cs = getComputedStyle(messages); var g = parseFloat(cs.rowGap || cs.gap || "16"); gap = isNaN(g) ? 16 : g; } catch (e) { gap = 16; }
  }
  var specs = [];
  var acc = 0;
  if (virtual) {
    var items = V._items();
    for (var vi = 0; vi < items.length; vi++) {
      var it = items[vi];
      if (it.el && it.el.classList && it.el.classList.contains("user")) {
        specs.push({ el: it.el, top: Math.max(0, Math.min(height - 4, (acc / total) * height)), acc: acc, live: true });
      }
      acc += ((it.h && it.h > 0) ? it.h : (it.el ? it.el.offsetHeight : 48)) + gap;
    }
  } else {
    var turns = messages.querySelectorAll(".message.user");
    for (var i = 0; i < turns.length; i++) {
      var m = turns[i];
      var sr = m.getAttribute("data-gb-marker-ratio");
      var anchor = m.offsetTop;
      var ratio = sr !== null ? parseFloat(sr) : anchor / total;
      specs.push({ el: m, top: Math.max(0, Math.min(height - 4, ratio * height)), acc: anchor, live: false, ratio: ratio, cache: sr === null });
    }
  }
  for (var s = 0; s < specs.length; s++) specs[s].label = gbMarkerLabel(specs[s].el);
  // WRITE PHASE — no layout reads below this line.
  var overlay = gbMarkerOverlay(messages);
  var lastH = parseInt(overlay.getAttribute("data-gb-last-h") || "0", 10);
  if (lastH && lastH !== rawH && !virtual) {
    messages.querySelectorAll(".message.user").forEach(function (m) { m.removeAttribute("data-gb-marker-ratio"); });
  }
  overlay.setAttribute("data-gb-last-h", String(rawH));
  overlay.style.top = top + "px";
  overlay.style.height = height + "px";
  overlay.innerHTML = "";
  for (var k = 0; k < specs.length; k++) {
    var sp = specs[k];
    if (sp.cache) sp.el.setAttribute("data-gb-marker-ratio", String(sp.ratio));
    var marker = document.createElement("button");
    marker.type = "button";
    marker.className = "scrollbar-marker user-marker";
    marker.textContent = ""; // aria-label is the only accessible name
    marker.style.top = sp.top + "px";
    marker.setAttribute("aria-label", gbT("chat-str-jump-to-message", "Jump to message: ") + sp.label);
    marker.title = gbT("chat-str-jump-to-message", "Jump to message: ") + sp.label;
    var tooltip = document.createElement("span");
    tooltip.className = "scrollbar-marker-tooltip";
    tooltip.textContent = sp.label;
    marker.appendChild(tooltip);
    (function (target, at, live) {
      marker.addEventListener("click", function () { gbMarkerJump(messages, target, at, live); });
    })(sp.el, sp.acc, sp.live);
    overlay.appendChild(marker);
  }
}

// Scroll path: one coalesced geometry read/write per frame, never per event.
function gbSyncMarkerOverlay() {
  var messages = document.getElementById("messages");
  if (!messages) return;
  var overlay = messages.querySelector(".scrollbar-markers");
  if (!overlay) {
    // #2933 — boot skipped geometry. The first real scroll builds the markers
    // in this one rAF pass, never on the boot idle callback.
    if (!gbMarkersBootPending) gbRefreshScrollbarMarkers();
    return;
  }
  var top = messages.scrollTop;
  var height = messages.clientHeight;
  overlay.style.top = top + "px";
  overlay.style.height = height + "px";
}

function scheduleMarkerSync() {
  if (gbMarkerSyncQueued) return;
  gbMarkerSyncQueued = true;
  requestAnimationFrame(function () { gbMarkerSyncQueued = false; gbSyncMarkerOverlay(); });
}

/* #2933 — idle scheduler for chat's non-critical paint; setTimeout fallback. */
window.gbIdle = function (fn) {
  if (typeof requestIdleCallback === "function") return requestIdleCallback(fn, { timeout: 800 });
  return setTimeout(fn, 1);
};
function scheduleScrollbarMarkers() {
  if (gbMarkersRefreshQueued) return;
  gbMarkersRefreshQueued = true;
  if (gbMarkersBootPending) {
    // #2933 — boot idle callback stays cheap: arm marker geometry with a data
    // flag only. No style/box read runs here; the first scroll or mutation
    // triggers the real rebuild off the boot path.
    window.gbIdle(function () {
      gbMarkersRefreshQueued = false;
      var box = document.getElementById("messages");
      if (box) box.dataset.gbMarkersArmed = "1";
      gbMarkersBootPending = false;
    });
    return;
  }
  // #2933 — marker rebuild was a 129-158 ms rAF handler; run it on idle.
  window.gbIdle(function () { gbMarkersRefreshQueued = false; gbRefreshScrollbarMarkers(); });
}

function initScrollbarMarkers() {
  var messages = document.getElementById("messages");
  if (!messages || messages.dataset.gbMarkers === "1") return;
  messages.dataset.gbMarkers = "1";
  messages.addEventListener("scroll", scheduleMarkerSync, { passive: true });
  window.addEventListener("resize", scheduleScrollbarMarkers);
  if (window.MutationObserver) {
    new MutationObserver(function (entries) {
      if (ChatState.suppressPersist) return;
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

function updateScrollButton(dist) {
  var messages = document.getElementById("messages");
  var scrollBtn = document.getElementById("scrollToBottom");
  if (!messages || !scrollBtn) return;
  if (dist === undefined) dist = messages.scrollHeight - messages.scrollTop - messages.clientHeight;
  var isNearBottom = dist < 100;
  if (isNearBottom) {
    scrollBtn.classList.remove("visible");
  } else {
    scrollBtn.classList.add("visible");
  }
}
