"use strict";
/* Sheet advanced module: 21_presence — remote collaborator cursors + typing.
 *
 * The shell (sheet.js) already sends `cursor` / `typing_start` over the
 * GBCollab WebSocket, but never rendered what other users were doing. This
 * module draws colored cell outlines with name tags for every remote cursor
 * and a small "is typing…" pill, all inside the virtual grid's scrollable
 * body so they pan with the sheet like Google Sheets.
 *
 * Public API (window.SheetPresence):
 *   cursor(msg)      — place/update a remote cursor from a collab message
 *   typing(msg)      — show a "typing" pill for a user
 *   clearTyping(id)  — hide the typing pill for a user
 *   sync(users)      — drop cursors/typing for users no longer present
 *   clearAll()       — remove every overlay
 */
(function () {
  const DEFAULT_ROW_HEIGHT = 24;
  let layer = null;
  const cursors = new Map(); // userId -> { el }
  const typing = new Map();  // userId -> { el }

  function grid() {
    if (window.SheetVirtualGrid) return window.SheetVirtualGrid;
    return null;
  }

  function rowHeight() {
    const g = grid();
    if (g && g.bodyInner) {
      const cell = g.bodyInner.querySelector(".vg-cell");
      if (cell) {
        const h = parseFloat(cell.style.height);
        if (!isNaN(h) && h > 0) return h;
      }
    }
    return DEFAULT_ROW_HEIGHT;
  }

  function colXOf(c) {
    const g = grid();
    if (g && g.colXOf) return g.colXOf(c);
    return 48 + c * 96;
  }

  function colWidthOf(c) {
    const g = grid();
    if (g && g.colWidthOf) return g.colWidthOf(c);
    return 96;
  }

  function ensureLayer() {
    const g = grid();
    if (!g || !g.bodyInner) return null;
    // The grid is rebuilt on every loadSheet/newSheet; if our cached layer is
    // no longer inside the current bodyInner, discard it and start fresh.
    if (layer && layer.parentNode === g.bodyInner) return layer;
    cursors.clear();
    typing.clear();
    layer = document.createElement("div");
    layer.className = "cursor-indicators";
    layer.style.cssText =
      "position:absolute;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:20;";
    g.bodyInner.appendChild(layer);
    return layer;
  }

  function placeCursor(userId, name, color, row, col) {
    const host = ensureLayer();
    if (!host) return;
    removeCursor(userId);

    const border = document.createElement("div");
    border.className = "ss-remote-cursor";
    border.style.cssText =
      "position:absolute;box-sizing:border-box;border:2px solid " + color + ";" +
      "border-radius:2px;pointer-events:none;";
    border.style.left = colXOf(col) + "px";
    border.style.top = (row * rowHeight()) + "px";
    border.style.width = colWidthOf(col) + "px";
    border.style.height = rowHeight() + "px";

    const tag = document.createElement("div");
    tag.className = "ss-remote-cursor-tag";
    tag.textContent = name;
    tag.style.cssText =
      "position:absolute;left:0;top:-16px;padding:1px 6px;font-size:10px;line-height:14px;" +
      "font-weight:600;color:#fff;background:" + color + ";border-radius:3px 3px 3px 0;" +
      "white-space:nowrap;pointer-events:none;box-shadow:0 1px 2px rgba(0,0,0,.4);";
    border.appendChild(tag);

    host.appendChild(border);
    cursors.set(userId, { el: border });
  }

  function removeCursor(userId) {
    const c = cursors.get(userId);
    if (c && c.el && c.el.parentNode) c.el.parentNode.removeChild(c.el);
    cursors.delete(userId);
  }

  function showTyping(userId, name, color, row, col) {
    const host = ensureLayer();
    if (!host) return;
    clearTyping(userId);

    const pill = document.createElement("div");
    pill.className = "ss-remote-typing";
    pill.textContent = name + " is typing…";
    pill.style.cssText =
      "position:absolute;padding:2px 8px;font-size:11px;line-height:16px;font-style:italic;" +
      "color:#f8fafc;background:" + color + ";border-radius:999px;white-space:nowrap;" +
      "pointer-events:none;box-shadow:0 2px 6px rgba(0,0,0,.45);";
    pill.style.left = colXOf(col) + "px";
    pill.style.top = (row * rowHeight() + rowHeight() + 2) + "px";

    host.appendChild(pill);
    typing.set(userId, { el: pill });
  }

  function clearTyping(userId) {
    const t = typing.get(userId);
    if (t && t.el && t.el.parentNode) t.el.parentNode.removeChild(t.el);
    typing.delete(userId);
  }

  // A collab message may carry A1 coordinates directly or a `position` index.
  function resolveCell(msg) {
    if (msg.row !== undefined && msg.col !== undefined) {
      return { row: msg.row, col: msg.col };
    }
    if (msg.position !== undefined) {
      const g = grid();
      const cols = (g && g.totalCols) || 16384;
      return { row: Math.floor(msg.position / cols), col: msg.position % cols };
    }
    return null;
  }

  window.SheetPresence = {
    cursor: function (msg) {
      if (!msg) return;
      const cell = resolveCell(msg);
      if (!cell) return;
      const name = msg.user_name || msg.userId || "User";
      const color = msg.user_color || "#3b82f6";
      // textContent is used below, so the name is safe without escaping.
      placeCursor(msg.user_id, name, color, cell.row, cell.col);
    },

    typing: function (msg) {
      if (!msg) return;
      const cell = resolveCell(msg);
      if (!cell) return;
      const name = msg.user_name || msg.userId || "User";
      const color = msg.user_color || "#3b82f6";
      showTyping(msg.user_id, name, color, cell.row, cell.col);
    },

    clearTyping: clearTyping,

    // Remove overlays for users that are no longer in the presence list.
    sync: function (users) {
      const active = new Set((users || []).map(function (u) { return u.user_id; }));
      cursors.forEach(function (_, id) { if (!active.has(id)) removeCursor(id); });
      typing.forEach(function (_, id) { if (!active.has(id)) clearTyping(id); });
    },

    clearAll: function () {
      cursors.forEach(function (_, id) { removeCursor(id); });
      typing.forEach(function (_, id) { clearTyping(id); });
    }
  };
})();
