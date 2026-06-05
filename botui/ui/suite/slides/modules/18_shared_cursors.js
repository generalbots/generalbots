"use strict";

/**
 * Module 18: Shared cursors and selections for Slides.
 * Renders remote-user cursors and selection highlights on the slide
 * canvas, based on CollaborationCursor / CollaborationSelection
 * messages received via WebSocket. Each user is assigned a unique
 * color (server-generated); we render a small dot with the user's
 * name as a label, and a colored highlight rectangle over their
 * selection. Cursor and selection indicators are removed when a
 * user disconnects (heartbeat timeout).
 *
 * Public API: window.SlidesCollabCursors = { handleMessage,
 *   ensureOverlay, setUserColor, removeUser }.
 */

(function () {
  const COLORS = ["#1a73e8", "#ea4335", "#34a853", "#fbbc04", "#9334e6", "#00acc1", "#f06292", "#ff7043"];
  let overlay = null;
  const userColors = {};
  const cursors = {};
  const selections = {};
  const lastSeen = {};
  const HEARTBEAT_MS = 8000;

  function colorFor(userId) {
    if (userColors[userId]) return userColors[userId];
    let hash = 0;
    for (let i = 0; i < userId.length; i++) hash = ((hash << 5) - hash) + userId.charCodeAt(i);
    const c = COLORS[Math.abs(hash) % COLORS.length];
    userColors[userId] = c;
    return c;
  }

  function ensureOverlay() {
    if (overlay) return overlay;
    overlay = document.createElement("div");
    overlay.id = "slidesCollabOverlay";
    overlay.style.cssText = "position:absolute;inset:0;pointer-events:none;z-index:9990;";
    const canvas = document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas, #canvasContainer");
    if (canvas) {
      canvas.style.position = canvas.style.position || "relative";
      canvas.appendChild(overlay);
    } else {
      document.body.appendChild(overlay);
    }
    setInterval(cleanupStale, 2000);
    return overlay;
  }

  function renderCursor(userId, name, x, y) {
    ensureOverlay();
    let cur = cursors[userId];
    if (!cur) {
      cur = document.createElement("div");
      cur.className = "collab-cursor";
      cur.style.cssText = "position:absolute;width:12px;height:12px;border-radius:50%;border:2px solid #fff;box-shadow:0 0 4px rgba(0,0,0,0.4);transition:left 0.1s,top 0.1s;";
      const label = document.createElement("div");
      label.className = "collab-cursor-label";
      label.style.cssText = "position:absolute;top:14px;left:14px;background:" + colorFor(userId) + ";color:#fff;padding:2px 6px;border-radius:3px;font-size:11px;white-space:nowrap;font-family:Arial,sans-serif;";
      cur.appendChild(label);
      overlay.appendChild(cur);
      cursors[userId] = { el: cur, name: name || userId };
    }
    cur.style.background = colorFor(userId);
    cur.style.left = (x - 6) + "px";
    cur.style.top = (y - 6) + "px";
    cur.querySelector(".collab-cursor-label").textContent = name || userId;
    lastSeen[userId] = Date.now();
  }

  function renderSelection(userId, name, rect) {
    ensureOverlay();
    let sel = selections[userId];
    if (!sel) {
      sel = document.createElement("div");
      sel.className = "collab-selection";
      sel.style.cssText = "position:absolute;border:2px dashed transparent;background:transparent;pointer-events:none;";
      overlay.appendChild(sel);
      selections[userId] = { el: sel, name: name || userId };
    }
    const c = colorFor(userId);
    sel.el.style.left = rect.x + "px";
    sel.el.style.top = rect.y + "px";
    sel.el.style.width = rect.width + "px";
    sel.el.style.height = rect.height + "px";
    sel.el.style.borderColor = c;
    sel.el.style.background = c.replace(")", ", 0.1)").replace("rgb", "rgba");
    if (!sel.el.style.background.includes("rgba")) sel.el.style.background = c + "1a";
    lastSeen[userId] = Date.now();
  }

  function removeUser(userId) {
    if (cursors[userId]) { cursors[userId].el.remove(); delete cursors[userId]; }
    if (selections[userId]) { selections[userId].el.remove(); delete selections[userId]; }
  }

  function cleanupStale() {
    const now = Date.now();
    for (const uid in lastSeen) {
      if (now - lastSeen[uid] > HEARTBEAT_MS) removeUser(uid);
    }
  }

  function handleMessage(msg) {
    if (!msg || !msg.type) return;
    if (msg.type === "cursor" && msg.userId) {
      renderCursor(msg.userId, msg.userName, msg.x || 0, msg.y || 0);
    } else if (msg.type === "selection" && msg.userId && msg.rect) {
      renderSelection(msg.userId, msg.userName, msg.rect);
    } else if (msg.type === "presence-leave" && msg.userId) {
      removeUser(msg.userId);
    } else if (msg.type === "presence" && Array.isArray(msg.users)) {
      for (const u of msg.users) {
        if (u.cursor) renderCursor(u.userId, u.userName, u.cursor.x, u.cursor.y);
        if (u.selection) renderSelection(u.userId, u.userName, u.selection);
      }
    }
  }

  function broadcastCursor(x, y) {
    const s = getState();
    if (!s || !s.botId) return;
    if (typeof s.sendWS !== "function") return;
    s.sendWS({
      type: "cursor",
      botId: s.botId,
      presentationId: s.presentationId || s.id,
      userId: s.currentUserId || "anon",
      userName: s.currentUserName || "Anonymous",
      x, y,
    });
  }

  function getState() { return window.state || null; }

  function attach() {
    ensureOverlay();
    const canvas = document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas, #canvasContainer");
    if (canvas) {
      canvas.addEventListener("mousemove", function (e) {
        const r = canvas.getBoundingClientRect();
        broadcastCursor(e.clientX - r.left, e.clientY - r.top);
      });
    }
    if (window.state && typeof window.state.onWSMessage === "function") {
      const orig = window.state.onWSMessage;
      window.state.onWSMessage = function (msg) {
        orig(msg);
        handleMessage(msg);
      };
    } else {
      document.addEventListener("wsMessage", function (e) { handleMessage(e.detail); });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 100);
  }

  window.SlidesCollabCursors = {
    handleMessage, ensureOverlay, setUserColor: colorFor, removeUser,
  };
})();
