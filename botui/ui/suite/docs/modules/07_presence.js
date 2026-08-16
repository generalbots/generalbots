"use strict";
/* docs advanced module: 07_presence — remote collaborator cursors + selections.
 *
 * The docs WebSocket already broadcasts cursor/selection messages with real
 * user id/name/color, but the frontend never rendered them. This module draws
 * Google-Docs-style remote carets (colored bar + name tag) at each collaborator's
 * text position and translucent highlights over their selected ranges.
 *
 * Positions are *global character offsets* into the article's plain text
 * (matching getCaretCharacterOffsetWithin). Markers are fixed-positioned from
 * Range.getBoundingClientRect() and re-rendered on scroll/resize so they track
 * the document as it moves.
 *
 * Public API (window.DocsPresence):
 *   cursor(msg)      — place a caret at msg.position
 *   selection(msg)   — highlight the [start,end) range from msg.content
 *   sync(users)      — drop overlays for users no longer present
 *   clearAll()       — remove every overlay
 */
(function () {
  const cursors = new Map();     // userId -> { name, color, position }
  const selections = new Map();  // userId -> { name, color, start, end }
  let overlay = null;
  let rafId = null;

  function getArticle() {
    return document.querySelector("article[contenteditable]");
  }

  function ensureOverlay() {
    if (overlay && overlay.parentNode) return overlay;
    overlay = document.createElement("div");
    overlay.className = "docs-presence-overlay";
    overlay.style.cssText =
      "position:fixed;inset:0;pointer-events:none;z-index:60;overflow:hidden;";
    document.body.appendChild(overlay);
    return overlay;
  }

  // Resolve a [start,end) global character offset into a DOM Range within root.
  function resolveRange(root, start, end) {
    var doc = root.ownerDocument || document;
    var walker = doc.createTreeWalker(root, NodeFilter.SHOW_TEXT, null);
    var current = 0;
    var startNode = null, startOffset = 0;
    var endNode = null, endOffset = 0;
    var node;
    while ((node = walker.nextNode())) {
      var len = node.nodeValue.length;
      var nodeStart = current;
      var nodeEnd = current + len;
      if (startNode === null && start >= nodeStart && start <= nodeEnd) {
        startNode = node;
        startOffset = start - nodeStart;
      }
      if (endNode === null && end >= nodeStart && end <= nodeEnd) {
        endNode = node;
        endOffset = end - nodeStart;
      }
      if (startNode && endNode) break;
      current = nodeEnd;
    }
    if (!startNode) return null;
    if (!endNode) { endNode = startNode; endOffset = startOffset; }
    var range = doc.createRange();
    try {
      range.setStart(startNode, startOffset);
      range.setEnd(endNode, endOffset);
    } catch (e) { return null; }
    return range;
  }

  function buildTag(name, color) {
    var tag = document.createElement("div");
    tag.className = "docs-presence-tag";
    tag.textContent = name;
    tag.style.cssText =
      "position:absolute;top:-20px;left:0;padding:1px 6px;font-size:10px;line-height:16px;" +
      "font-weight:600;color:#fff;background:" + color + ";border-radius:3px 3px 3px 0;" +
      "white-space:nowrap;box-shadow:0 1px 2px rgba(0,0,0,.4);";
    return tag;
  }

  function render() {
    var article = getArticle();
    var layer = ensureOverlay();
    layer.innerHTML = "";
    if (!article) return;

    cursors.forEach(function (info, userId) {
      var range = resolveRange(article, info.position, info.position);
      if (!range) return;
      var rect = range.getBoundingClientRect();
      var caret = document.createElement("div");
      caret.className = "docs-presence-caret";
      caret.style.cssText =
        "position:absolute;width:2px;height:" + Math.max(14, rect.height || 18) + "px;" +
        "background:" + info.color + ";";
      caret.style.left = rect.left + "px";
      caret.style.top = rect.top + "px";
      caret.appendChild(buildTag(info.name, info.color));
      layer.appendChild(caret);
    });

    selections.forEach(function (info, userId) {
      var range = resolveRange(article, info.start, info.end);
      if (!range) return;
      var rects = range.getClientRects();
      var name = info.name, color = info.color;
      for (var i = 0; i < rects.length; i++) {
        var r = rects[i];
        if (!r || (r.width < 1 && r.height < 1)) continue;
        var hl = document.createElement("div");
        hl.className = "docs-presence-selection";
        hl.style.cssText =
          "position:absolute;background:" + color + ";opacity:0.22;border-radius:1px;";
        hl.style.left = r.left + "px";
        hl.style.top = r.top + "px";
        hl.style.width = r.width + "px";
        hl.style.height = r.height + "px";
        if (i === 0) hl.appendChild(buildTag(name, color));
        layer.appendChild(hl);
      }
    });
  }

  function scheduleRender() {
    if (rafId) return;
    rafId = requestAnimationFrame(function () {
      rafId = null;
      render();
    });
  }

  function clearUser(userId) {
    cursors.delete(userId);
    selections.delete(userId);
    scheduleRender();
  }

  window.DocsPresence = {
    cursor: function (msg) {
      if (!msg) return;
      if (typeof msg.position !== "number") { cursors.delete(msg.user_id); scheduleRender(); return; }
      cursors.set(msg.user_id, {
        name: msg.user_name || "User",
        color: msg.user_color || "#3b82f6",
        position: msg.position
      });
      scheduleRender();
    },

    selection: function (msg) {
      if (!msg) return;
      var sel = null;
      try { sel = typeof msg.content === "string" ? JSON.parse(msg.content) : (msg.content || null); }
      catch (e) { sel = null; }
      if (!sel || typeof sel.start !== "number" || typeof sel.end !== "number") {
        selections.delete(msg.user_id);
        scheduleRender();
        return;
      }
      if (sel.start === sel.end) {
        selections.delete(msg.user_id);
        cursors.set(msg.user_id, {
          name: msg.user_name || "User",
          color: msg.user_color || "#3b82f6",
          position: sel.start
        });
        scheduleRender();
        return;
      }
      selections.set(msg.user_id, {
        name: msg.user_name || "User",
        color: msg.user_color || "#3b82f6",
        start: Math.min(sel.start, sel.end),
        end: Math.max(sel.start, sel.end)
      });
      scheduleRender();
    },

    sync: function (users) {
      var active = new Set((users || []).map(function (u) { return u.user_id; }));
      cursors.forEach(function (_, id) { if (!active.has(id)) cursors.delete(id); });
      selections.forEach(function (_, id) { if (!active.has(id)) selections.delete(id); });
      scheduleRender();
    },

    clearAll: function () {
      cursors.clear();
      selections.clear();
      scheduleRender();
    },

    clearUser: clearUser
  };

  function bindScroll() {
    var content = document.getElementById("docs-content");
    if (content) content.addEventListener("scroll", scheduleRender, { passive: true });
    window.addEventListener("resize", scheduleRender);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bindScroll);
  } else {
    bindScroll();
  }
})();
