"use strict";
/* Slides advanced module: 05_presence — remote collaborator cursors + typing.
 *
 * The slides WebSocket already broadcasts cursor/selection/typing messages with
 * the real user id/name/color, but the frontend never rendered them. This
 * module draws a colored outline + name tag on the element each remote user is
 * focused on, and a small "is typing…" pill while they edit a text element.
 *
 * Public API (window.SlidesPresence):
 *   cursor(msg)      — outline + name tag on a remote user's focused element
 *   selection(msg)   — same, from a selection message (first element)
 *   typing(msg)      — show a "typing" pill on the element
 *   clearTyping(id)  — hide the typing pill for a user
 *   sync(users)      — drop overlays for users no longer present
 *   clearAll()       — remove every overlay
 */
(function () {
  const cursors = new Map(); // userId -> { overlay, elementId }
  const typings = new Map(); // userId -> { pill, elementId }
  let onlineUsers = [];      // most recent presence snapshot

  function elById(id) {
    return id ? document.querySelector('[data-id="' + id + '"]') : null;
  }

  function removeCursor(userId) {
    const c = cursors.get(userId);
    if (c && c.overlay && c.overlay.parentNode) c.overlay.parentNode.removeChild(c.overlay);
    cursors.delete(userId);
  }

  function showCursor(userId, name, color, elementId) {
    removeCursor(userId);
    const el = elById(elementId);
    if (!el) return;
    const overlay = document.createElement("div");
    overlay.className = "sl-remote-cursor";
    overlay.style.cssText =
      "position:absolute;inset:0;border:2px solid " + color + ";border-radius:2px;" +
      "pointer-events:none;z-index:6;box-shadow:0 0 0 1px rgba(255,255,255,.2);";
    const tag = document.createElement("div");
    tag.textContent = name;
    tag.style.cssText =
      "position:absolute;top:-18px;left:0;padding:1px 6px;font-size:10px;line-height:14px;" +
      "font-weight:600;color:#fff;background:" + color + ";border-radius:3px 3px 3px 0;" +
      "white-space:nowrap;pointer-events:none;box-shadow:0 1px 2px rgba(0,0,0,.4);";
    overlay.appendChild(tag);
    el.appendChild(overlay);
    cursors.set(userId, { overlay: overlay, elementId: elementId });
  }

  function clearTyping(userId) {
    const t = typings.get(userId);
    if (t && t.pill && t.pill.parentNode) t.pill.parentNode.removeChild(t.pill);
    typings.delete(userId);
  }

  function showTyping(userId, name, color, elementId) {
    clearTyping(userId);
    const el = elById(elementId);
    if (!el) return;
    const pill = document.createElement("div");
    pill.className = "sl-remote-typing";
    pill.textContent = name + " is typing…";
    pill.style.cssText =
      "position:absolute;top:-20px;left:0;padding:2px 8px;font-size:11px;line-height:16px;" +
      "font-style:italic;color:#f8fafc;background:" + color + ";border-radius:999px;" +
      "white-space:nowrap;pointer-events:none;box-shadow:0 2px 6px rgba(0,0,0,.45);z-index:7;";
    el.appendChild(pill);
    typings.set(userId, { pill: pill, elementId: elementId });
  }

  window.SlidesPresence = {
    cursor: function (msg) {
      if (!msg || !msg.element_id) { if (msg) removeCursor(msg.user_id); return; }
      showCursor(msg.user_id, msg.user_name || "User", msg.user_color || "#3b82f6", msg.element_id);
    },

    selection: function (msg) {
      if (!msg || !msg.data) return;
      const ids = Array.isArray(msg.data.element_ids) ? msg.data.element_ids : [];
      if (!ids.length) { removeCursor(msg.user_id); return; }
      // Single-select canvas: anchor on the first selected element.
      showCursor(msg.user_id, msg.user_name || "User", msg.user_color || "#3b82f6", ids[0]);
    },

    typing: function (msg) {
      if (!msg || !msg.element_id) return;
      showTyping(msg.user_id, msg.user_name || "User", msg.user_color || "#3b82f6", msg.element_id);
    },

    clearTyping: clearTyping,

    sync: function (users) {
      onlineUsers = users || [];
      const active = new Set(onlineUsers.map(function (u) { return u.user_id; }));
      cursors.forEach(function (_, id) { if (!active.has(id)) removeCursor(id); });
      typings.forEach(function (_, id) { if (!active.has(id)) clearTyping(id); });
    },

    list: function () {
      return onlineUsers.slice();
    },

    follow: function (userId) {
      const c = cursors.get(userId);
      if (!c) return;
      const el = elById(c.elementId);
      if (el && el.scrollIntoView) el.scrollIntoView({ behavior: "smooth", block: "center" });
    },

    clearAll: function () {
      cursors.forEach(function (_, id) { removeCursor(id); });
      typings.forEach(function (_, id) { clearTyping(id); });
    }
  };
})();
