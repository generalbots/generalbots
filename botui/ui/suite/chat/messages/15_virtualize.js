/* Chat transcript virtualization/windowing (#2229).
 * Keeps long sessions in memory but renders only a window of message nodes
 * around the viewport so the transcript stays fast. Messages outside the
 * window are replaced by height-preserving placeholders and restored on scroll.
 * Minimal/safe: does not change message creation, streaming, or persistence —
 * only DOM presence of offscreen .message nodes is windowed. */
"use strict";

(function () {
  var V = (window.GBChatVirtual = window.GBChatVirtual || {});
  var ENABLE_AT = 120;      // start windowing above this many messages
  var WINDOW = 80;          // nominal messages to keep around viewport
  var BUFFER = 20;          // extra messages above/below viewport window
  var MAX_LIVE = 140;       // hard cap to keep DOM bounded

  var state = {
    container: null,
    enabled: false,
    items: [],              // { el, h, placeholder, live }
    raf: 0,
    ro: null,
    scrollBound: false,
  };

  function reset() {
    state.enabled = false;
    state.items = [];
  }

  function isMessageNode(n) {
    return n && n.nodeType === 1 && n.classList && n.classList.contains("message");
  }

  function measureGapApprox() {
    var c = state.container;
    if (!c) return 16;
    try {
      var cs = window.getComputedStyle(c);
      var g = parseFloat(cs.rowGap || cs.gap || "16");
      return isNaN(g) ? 16 : g;
    } catch (e) { return 16; }
  }

  function itemOuterHeight(it) {
    if (it.live && it.el && it.el.isConnected) {
      var h = it.el.offsetHeight;
      if (h > 0) it.h = h;
    }
    return (it.h || 48) + measureGapApprox();
  }

  function isPinned(it) {
    if (!it || !it.el) return false;
    try {
      if (typeof ChatState !== "undefined" && ChatState.streamingMessageId &&
          it.el.id === ChatState.streamingMessageId) return true;
      if (it.el.getAttribute && it.el.getAttribute("data-gb-vpin") === "1") return true;
      // Keep actively streaming/updating node live to avoid breaking getElementById consumers.
      if (it.el.id && /^streaming-/.test(it.el.id)) return true;
    } catch (e) {}
    return false;
  }

  function compactItems() {
    // Drop tracked nodes that were removed from the transcript externally.
    if (!state.container) return;
    var c = state.container;
    var changed = false;
    var out = [];
    for (var i = 0; i < state.items.length; i++) {
      var it = state.items[i];
      if (!it || !it.el) { changed = true; continue; }
      if (it.live) {
        if (it.el.isConnected && c.contains(it.el)) out.push(it);
        else { changed = true; if (state.ro) try { state.ro.unobserve(it.el); } catch (e) {} }
      } else {
        if (it.placeholder && it.placeholder.isConnected && c.contains(it.placeholder)) out.push(it);
        else if (it.el.isConnected && c.contains(it.el)) { it.live = true; out.push(it); changed = true; }
        else { changed = true; if (state.ro) try { state.ro.unobserve(it.el); } catch (e) {} }
      }
    }
    if (changed) state.items = out;
  }

  V.init = function (container) {
    if (!container) container = document.getElementById("messages");
    if (!container) return;
    if (state.container !== container) {
      reset();
      state.container = container;
      state.scrollBound = false;
    }
    if (!state.scrollBound) {
      container.addEventListener("scroll", function () { V.schedule(); }, { passive: true });
      state.scrollBound = true;
    }
    if (!state.ro && typeof ResizeObserver !== "undefined") {
      state.ro = new ResizeObserver(function () {
        if (!state.enabled) return;
        V.schedule();
      });
    }
  };

  V.register = function (el) {
    if (!el || !isMessageNode(el)) return;
    if (!state.container) V.init(document.getElementById("messages"));
    if (!state.container) return;
    // During batched history replay, messages are appended to a DocumentFragment
    // (ChatState.appendTarget) and the caller performs a full registerAll/rebuild
    // once attached — skip incremental registration to avoid stale/off-container
    // entries and redundant scheduling.
    try {
      if (typeof ChatState !== "undefined" && ChatState.appendTarget && el.parentNode === ChatState.appendTarget) {
        return;
      }
    } catch (e) {}
    // Dedup by identity (cheap tail check covers append-heavy path).
    for (var i = state.items.length - 1; i >= 0 && i >= state.items.length - 8; i--) {
      if (state.items[i].el === el) return;
    }
    if (el.__gbVirtIdx != null && state.items[el.__gbVirtIdx] && state.items[el.__gbVirtIdx].el === el) return;
    state.items.push({ el: el, h: (el.offsetHeight || 0), placeholder: null, live: true });
    if (state.ro) try { state.ro.observe(el); } catch (e) {}
    V.maybeEnable();
  };

  V.registerAll = function (container) {
    var c = container || state.container || document.getElementById("messages");
    if (!c) return;
    V.init(c);
    compactItems();
    state.items = [];
    state.enabled = false;
    var nodes = c.querySelectorAll(":scope > .message");
    for (var i = 0; i < nodes.length; i++) {
      var el = nodes[i];
      state.items.push({ el: el, h: (el.offsetHeight || 0), placeholder: null, live: true });
      if (state.ro) try { state.ro.observe(el); } catch (e) {}
    }
  };

  V.clear = function () {
    // Materialize tracked items before external clears/rebuilds when possible.
    try {
      if (state.enabled && state.container) V.materializeRange(0, state.items.length);
    } catch (e) {}
    reset();
  };

  V.maybeEnable = function () {
    if (state.enabled) return;
    if (state.items.length < ENABLE_AT) return;
    if (!state.container) V.init(document.getElementById("messages"));
    if (!state.container) return;
    // Avoid enabling mid-batch during history replay; afterHistoryRender finalizes.
    if (typeof ChatState !== "undefined" && ChatState.suppressPersist) return;
    state.enabled = true;
    V.schedule();
  };

  V.schedule = function () {
    if (!state.enabled) { V.maybeEnable(); return; }
    if (state.raf) return;
    state.raf = (typeof requestAnimationFrame === "function" ? requestAnimationFrame : function (cb) { return setTimeout(cb, 16); })(function () {
      state.raf = 0;
      V.update();
    });
  };

  function findRange() {
    var c = state.container;
    var items = state.items;
    var n = items.length;
    var viewTop = c.scrollTop;
    var viewBottom = viewTop + c.clientHeight;
    var acc = 0;
    var start = 0, end = n;
    var foundStart = false;
    for (var i = 0; i < n; i++) {
      var h = itemOuterHeight(items[i]);
      var top = acc;
      var bottom = acc + h;
      if (!foundStart && bottom >= viewTop - 80) { start = i; foundStart = true; }
      if (foundStart && top > viewBottom + 80) { end = i; break; }
      acc = bottom;
    }
    start = Math.max(0, start - BUFFER);
    end = Math.min(n, end + BUFFER);

    // Keep live set bounded.
    if (end - start > MAX_LIVE) {
      var distBottom = c.scrollHeight - c.scrollTop - c.clientHeight;
      if (distBottom < 300) start = Math.max(0, end - MAX_LIVE);
      else end = Math.min(n, start + MAX_LIVE);
    }

    // When near bottom (common live-chat case), bias window to tail so latest
    // messages, streaming updates, and auto-follow behavior remain intact.
    var dist = c.scrollHeight - c.scrollTop - c.clientHeight;
    if (dist < 300) {
      start = Math.max(0, n - Math.min(WINDOW + BUFFER, MAX_LIVE));
      end = n;
    }
    return { start: start, end: end };
  }

  V.materializeRange = function (start, end) {
    var c = state.container;
    if (!c) return;
    for (var i = 0; i < state.items.length; i++) {
      var it = state.items[i];
      if (!it) continue;
      var wantLive = (i >= start && i < end) || isPinned(it);
      if (wantLive) {
        if (!it.live) {
          if (it.placeholder && it.placeholder.parentNode === c) {
            try { c.replaceChild(it.el, it.placeholder); } catch (e) {}
          } else if (!it.el.isConnected || !c.contains(it.el)) {
            var before = null;
            for (var k = i + 1; k < state.items.length; k++) {
              var nk = state.items[k];
              var node = nk && (nk.live ? nk.el : nk.placeholder);
              if (node && node.parentNode === c) { before = node; break; }
            }
            try { c.insertBefore(it.el, before); } catch (e2) {}
          }
          it.live = true;
          it.placeholder = null;
          if (state.ro) try { state.ro.observe(it.el); } catch (e3) {}
        } else if (!it.el.isConnected || !c.contains(it.el)) {
          // Unexpected DOM loss; drop logical liveness so range logic can recover.
          it.live = false;
        }
      } else {
        if (it.live) {
          if (it.el && it.el.parentNode === c) {
            var h = it.el.offsetHeight;
            if (h > 0) it.h = h;
            var ph = document.createElement("div");
            ph.className = "gb-vplaceholder";
            ph.setAttribute("data-gb-vi", String(i));
            ph.setAttribute("aria-hidden", "true");
            ph.style.cssText = "height:" + Math.max(24, (it.h || 48)) + "px;flex:0 0 auto;pointer-events:none;contain:layout paint size;";
            it.placeholder = ph;
            try { c.replaceChild(ph, it.el); } catch (e4) { it.placeholder = null; continue; }
            it.live = false;
          } else if (it.placeholder && it.placeholder.parentNode === c) {
            it.live = false;
          } else {
            it.live = false;
          }
        } else if (it.placeholder && it.placeholder.parentNode === c) {
          // Keep placeholder height in sync with best-known height.
          var hh = Math.max(24, (it.h || 48));
          if (parseInt(it.placeholder.style.height, 10) !== hh) it.placeholder.style.height = hh + "px";
        }
      }
    }
  };

  V.update = function () {
    if (!state.enabled || !state.container) return;
    var c = state.container;
    compactItems();
    if (state.items.length < ENABLE_AT) {
      // Fell below threshold (e.g. clear/rebuild races) — keep everything live.
      V.materializeRange(0, state.items.length);
      c.dataset.gbVirtual = "0";
      c.dataset.gbVirtualTotal = String(state.items.length);
      c.dataset.gbVirtualLive = String(state.items.length);
      state.enabled = false;
      return;
    }

    // Refresh cached heights from live items.
    for (var j = 0; j < state.items.length; j++) {
      var it = state.items[j];
      if (it && it.live && it.el && it.el.isConnected) {
        var h = it.el.offsetHeight;
        if (h > 0) it.h = h;
      }
    }

    var r = findRange();

    // Anchor-based scroll compensation: keep the viewport's first visible live
    // message stable across detach/attach so reading position does not jump.
    var anchor = null, anchorTop = 0;
    var crect = c.getBoundingClientRect();
    for (var a = 0; a < state.items.length; a++) {
      var ait = state.items[a];
      if (ait && ait.live && ait.el && ait.el.isConnected) {
        var ar = ait.el.getBoundingClientRect();
        if (ar.bottom > crect.top + 1) { anchor = ait.el; anchorTop = ar.top; break; }
      }
    }

    V.materializeRange(r.start, r.end);

    if (anchor && anchor.isConnected) {
      var newTop = anchor.getBoundingClientRect().top;
      var delta = newTop - anchorTop;
      if (Math.abs(delta) > 1 && Math.abs(delta) < c.clientHeight) c.scrollTop += delta;
    }

    V.updateDebug();
  };

  V.updateDebug = function () {
    var c = state.container;
    if (!c) return;
    var live = 0;
    for (var i = 0; i < state.items.length; i++) if (state.items[i] && state.items[i].live) live++;
    c.dataset.gbVirtual = state.enabled ? "1" : "0";
    c.dataset.gbVirtualTotal = String(state.items.length);
    c.dataset.gbVirtualLive = String(live);
  };

  V.onAfterAppend = function (el) {
    V.register(el);
    V.schedule();
  };

  V.afterHistoryRender = function () {
    V.registerAll(state.container || document.getElementById("messages"));
    if (state.items.length >= ENABLE_AT) {
      state.enabled = true;
      V.schedule();
      // Re-run after layout settles (fonts/media/heights) to stabilize anchors.
      setTimeout(function () { V.schedule(); }, 120);
      setTimeout(function () { V.schedule(); }, 500);
    } else {
      V.updateDebug();
    }
  };

  V.ensureVisible = function (el) {
    if (!el) return;
    if (!state.enabled) return;
    for (var i = 0; i < state.items.length; i++) {
      if (state.items[i].el === el) {
        if (!state.items[i].live) {
          V.materializeRange(Math.max(0, i - BUFFER), Math.min(state.items.length, i + BUFFER + 1));
          V.updateDebug();
        }
        return;
      }
    }
  };

  V.ensureById = function (id) {
    if (!id) return null;
    var el = document.getElementById(id);
    if (el) return el;
    // Search tracked items (including detached ones).
    for (var i = 0; i < state.items.length; i++) {
      var it = state.items[i];
      if (it && it.el && it.el.id === id) {
        V.ensureVisible(it.el);
        return it.el;
      }
    }
    return null;
  };

  V.remove = function (el) {
    if (!el) return;
    for (var i = 0; i < state.items.length; i++) {
      if (state.items[i].el === el) {
        if (state.ro) try { state.ro.unobserve(el); } catch (e) {}
        if (state.items[i].placeholder && state.items[i].placeholder.parentNode) {
          try { state.items[i].placeholder.parentNode.removeChild(state.items[i].placeholder); } catch (e2) {}
        }
        state.items.splice(i, 1);
        V.updateDebug();
        V.schedule();
        return;
      }
    }
  };

  V.isEnabled = function () { return !!state.enabled; };
  V.count = function () { return state.items.length; };
  V.liveCount = function () { var n = 0; for (var i = 0; i < state.items.length; i++) if (state.items[i].live) n++; return n; };
  V._items = function () { return state.items; };
  V.forceUpdate = function () { V.update(); };
})();