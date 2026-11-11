/* Chat transcript virtualization/windowing (#2229).
 * Keeps the default (and long) session in memory but renders only a window of
 * message nodes around the viewport so the transcript stays fast. Messages
 * outside the window are replaced by height-preserving spacers and restored on
 * scroll. Designed to be safe/minimal: no change to message creation, streaming
 * targets, or history persistence — only DOM presence is windowed. */
"use strict";

(function () {
  var V = (window.GBChatVirtual = window.GBChatVirtual || {});
  var ENABLE_AT = 120;      // start windowing above this many messages
  var WINDOW = 80;          // messages to keep materialized around viewport
  var BUFFER = 20;          // extra messages above/below viewport window
  var MIN_RUN = 8;          // never create spacers for tiny runs

  var state = {
    container: null,
    enabled: false,
    items: [],              // { el, h, placeholder, live }
    topSpacer: null,
    bottomSpacer: null,
    raf: 0,
    ro: null,
    scrollBound: false,
  };

  function reset() {
    state.enabled = false;
    state.items = [];
    state.topSpacer = null;
    state.bottomSpacer = null;
  }

  function ensureSpacers() {
    var c = state.container;
    if (!c) return;
    if (!state.topSpacer) {
      state.topSpacer = document.createElement("div");
      state.topSpacer.className = "gb-vspacer gb-vspacer-top";
      state.topSpacer.setAttribute("aria-hidden", "true");
      state.topSpacer.style.cssText = "flex:0 0 auto;height:0px;overflow:hidden;pointer-events:none;";
    }
    if (!state.bottomSpacer) {
      state.bottomSpacer = document.createElement("div");
      state.bottomSpacer.className = "gb-vspacer gb-vspacer-bottom";
      state.bottomSpacer.setAttribute("aria-hidden", "true");
      state.bottomSpacer.style.cssText = "flex:0 0 auto;height:0px;overflow:hidden;pointer-events:none;";
    }
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
    if (it.live && it.el && it.el.parentNode === state.container) {
      var h = it.el.offsetHeight;
      if (h > 0) it.h = h;
    }
    return (it.h || 48) + measureGapApprox();
  }

  function totalHeight() {
    var sum = 0;
    for (var i = 0; i < state.items.length; i++) sum += itemOuterHeight(state.items[i]);
    return sum;
  }

  function isMessageNode(n) {
    return n && n.nodeType === 1 && n.classList && n.classList.contains("message");
  }

  V.init = function (container) {
    if (!container) container = document.getElementById("messages");
    if (!container) return;
    if (state.container !== container) {
      reset();
      state.container = container;
    }
    if (!state.scrollBound) {
      container.addEventListener("scroll", function () { V.schedule(); }, { passive: true });
      state.scrollBound = true;
    }
    if (!state.ro && window.ResizeObserver) {
      state.ro = new ResizeObserver(function () {
        if (!state.enabled) return;
        // Heights of materialized items may have changed (iframes, expand, etc.)
        V.schedule();
      });
    }
  };

  V.register = function (el) {
    if (!el) return;
    if (!state.container) V.init(document.getElementById("messages"));
    // Only track real message bubbles in the main transcript.
    if (state.container && el.parentNode !== state.container && el.parentNode !== null) {
      // could be inside a fragment during replay; accept if it will be in container
    }
    if (!isMessageNode(el)) return;
    // Avoid duplicates
    for (var i = state.items.length - 1; i >= Math.max(0, state.items.length - 5); i--) {
      if (state.items[i].el === el) return;
    }
    state.items.push({ el: el, h: 0, placeholder: null, live: true });
    if (state.ro) try { state.ro.observe(el); } catch (e) {}
    V.maybeEnable();
  };

  V.registerAll = function (container) {
    var c = container || state.container || document.getElementById("messages");
    if (!c) return;
    V.init(c);
    // Rebuild tracking to match current DOM order (safe after prepends/rebuilds).
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
    // Materialize everything back before external clears/rebuilds.
    if (state.enabled && state.container) {
      V.materializeRange(0, state.items.length);
    }
    reset();
  };

  V.maybeEnable = function () {
    if (state.enabled) return;
    if (state.items.length < ENABLE_AT) return;
    if (!state.container) V.init(document.getElementById("messages"));
    if (!state.container) return;
    // Allow enable during replay only when explicitly finalized later; avoid flicker mid-batch.
    if (typeof ChatState !== "undefined" && ChatState.suppressPersist) return;
    state.enabled = true;
    ensureSpacers();
    V.schedule();
  };

  V.schedule = function () {
    if (!state.enabled) { V.maybeEnable(); return; }
    if (state.raf) return;
    state.raf = requestAnimationFrame(function () {
      state.raf = 0;
      V.update();
    });
  };

  function findRange() {
    var c = state.container;
    var scrollTop = c.scrollTop;
    var viewTop = scrollTop;
    var viewBottom = scrollTop + c.clientHeight;
    var acc = 0;
    var start = 0, end = state.items.length;
    var foundStart = false;
    for (var i = 0; i < state.items.length; i++) {
      var h = itemOuterHeight(state.items[i]);
      var top = acc;
      var bottom = acc + h;
      if (!foundStart && bottom >= viewTop - 40) { start = i; foundStart = true; }
      if (foundStart && top > viewBottom + 40) { end = i; break; }
      acc = bottom;
    }
    start = Math.max(0, start - BUFFER);
    end = Math.min(state.items.length, end + BUFFER);
    // When near bottom (common case), bias window to tail so live chat is intact.
    var dist = c.scrollHeight - c.scrollTop - c.clientHeight;
    if (dist < 300) {
      start = Math.max(0, state.items.length - WINDOW - BUFFER);
      end = state.items.length;
    }
    return { start: start, end: end };
  }

  function heightOfRange(from, to) {
    var sum = 0;
    for (var i = from; i < to; i++) sum += itemOuterHeight(state.items[i]);
    return sum;
  }

  V.materializeRange = function (start, end) {
    var c = state.container;
    if (!c) return;
    // Detach outside range
    for (var i = 0; i < state.items.length; i++) {
      var it = state.items[i];
      var wantLive = (i >= start && i < end);
      if (wantLive) {
        if (!it.live) {
          // replace placeholder with element
          if (it.placeholder && it.placeholder.parentNode === c) {
            try { c.replaceChild(it.el, it.placeholder); } catch (e) {}
          } else if (it.el && it.el.parentNode !== c) {
            // If placeholder missing, try to insert before next live/placeholder node
            var before = null;
            for (var k = i + 1; k < state.items.length; k++) {
              var nk = state.items[k];
              var node = nk.live ? nk.el : nk.placeholder;
              if (node && node.parentNode === c) { before = node; break; }
            }
            try { c.insertBefore(it.el, before); } catch (e) {}
          }
          it.live = true;
          it.placeholder = null;
          if (state.ro) try { state.ro.observe(it.el); } catch (e) {}
        }
      } else {
        if (it.live) {
          if (it.el && it.el.parentNode === c) {
            // measure before detach
            var h = it.el.offsetHeight;
            if (h > 0) it.h = h;
            var ph = document.createElement("div");
            ph.className = "gb-vplaceholder";
            ph.setAttribute("data-gb-vi", String(i));
            ph.setAttribute("aria-hidden", "true");
            ph.style.cssText = "height:" + Math.max(24, (it.h || 48)) + "px;flex:0 0 auto;pointer-events:none;";
            it.placeholder = ph;
            try { c.replaceChild(ph, it.el); } catch (e) {}
            it.live = false;
          } else if (it.placeholder && it.placeholder.parentNode === c) {
            it.live = false;
          } else {
            // element not in container (unexpected) - keep logical state but don't break
            it.live = false;
          }
        }
      }
    }
  };

  function firstLiveIndex() {
    for (var i = 0; i < state.items.length; i++) if (state.items[i].live) return i;
    return -1;
  }

  V.update = function () {
    if (!state.enabled || !state.container) return;
    var c = state.container;
    if (state.items.length < ENABLE_AT) return;
    // Refresh cached heights from live items
    for (var j = 0; j < state.items.length; j++) {
      var it = state.items[j];
      if (it.live && it.el && it.el.parentNode === c) {
        var h = it.el.offsetHeight;
        if (h > 0) it.h = h;
      }
    }
    var r = findRange();
    // Anchor-based scroll compensation: keep first live element stable when possible.
    var anchor = null, anchorTop = 0;
    var fli = firstLiveIndex();
    if (fli >= 0) {
      anchor = state.items[fli].el;
      if (anchor && anchor.parentNode === c) anchorTop = anchor.getBoundingClientRect().top;
    }
    V.materializeRange(r.start, r.end);
    if (anchor && anchor.parentNode === c) {
      var newTop = anchor.getBoundingClientRect().top;
      var delta = newTop - anchorTop;
      if (Math.abs(delta) > 1) c.scrollTop += delta;
    } else if (fli < 0) {
      // if nothing was live, bias to tail when near bottom
      var dist = c.scrollHeight - c.scrollTop - c.clientHeight;
      if (dist < 300) c.scrollTop = c.scrollHeight;
    }
    V.updateDebug();
  };

  V.updateDebug = function () {
    var c = state.container;
    if (!c) return;
    var live = 0;
    for (var i = 0; i < state.items.length; i++) if (state.items[i].live) live++;
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
      ensureSpacers();
      // Prefer keeping the tail (newest messages) live after history load.
      V.schedule();
      setTimeout(function(){ V.schedule(); }, 120);
    }
  };

  V.ensureVisible = function (el) {
    if (!state.enabled || !el) return;
    // If element is a tracked message and detached, materialize around it.
    for (var i = 0; i < state.items.length; i++) {
      if (state.items[i].el === el && !state.items[i].live) {
        V.materializeRange(Math.max(0, i - BUFFER), Math.min(state.items.length, i + BUFFER + 1));
        return;
      }
    }
  };

  V.isEnabled = function () { return !!state.enabled; };
  V.count = function () { return state.items.length; };
  V._items = function () { return state.items; };
  V.forceUpdate = function () { V.update(); };
})();
