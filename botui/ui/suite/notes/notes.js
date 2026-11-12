"use strict";
/* Sticky Notes (#1154): colorful notes persisted in localStorage. */

(function () {
  if (window.GBNotes) return;

  const KEY = "gb-sticky-notes";
  const COLORS = ["#fef3c7", "#fde68a", "#d9f99d", "#a7f3d0", "#bfdbfe", "#fbcfe8"];

  function read() {
    try {
      return JSON.parse(localStorage.getItem(KEY) || "[]");
    } catch (e) {
      return [];
    }
  }

  function write(list) {
    try {
      localStorage.setItem(KEY, JSON.stringify(list));
    } catch (e) {}
  }

  function render() {
    const board = document.getElementById("notesBoard");
    if (!board) return;
    const list = read();
    const count = document.getElementById("notesCount");
    if (count) count.textContent = list.length + " note" + (list.length === 1 ? "" : "s");
    if (!list.length) {
      board.innerHTML = '<div class="notes-empty">No notes yet — click “+ New Note”.</div>';
      board.style.position = "";
      board.style.minHeight = "";
      return;
    }
    const hasPos = list.some(function (n) { return n && typeof n.x === "number" && typeof n.y === "number"; });
    if (hasPos) {
      board.style.position = "relative";
      board.style.minHeight = "600px";
    } else {
      board.style.position = "";
      board.style.minHeight = "";
    }
    board.innerHTML = list
      .map(function (n, i) {
        var posStyle = "";
        if (n && typeof n.x === "number" && typeof n.y === "number") {
          posStyle = "position:absolute;left:" + n.x + "px;top:" + n.y + "px;width:220px;z-index:" + (10 + i) + ";cursor:grab;";
        }
        return (
          '<div class="note-card" style="background:' + n.color + ';' + posStyle + '" data-i="' + i + '" data-ts="' + n.ts + '">' +
          '<textarea data-i="' + i + '" data-gb-field="text" placeholder="Write…">' + escapeHtml(n.text) + "</textarea>" +
          '<div class="note-card-footer">' +
          '<span class="note-time">' + new Date(n.ts).toLocaleString() + "</span>" +
          '<button class="note-del" data-del="' + i + '" title="Delete">✕</button>' +
          "</div>" +
          "</div>"
        );
      })
      .join("");
    Array.from(board.querySelectorAll("textarea")).forEach(function (ta) {
      ta.addEventListener("input", function () {
        const list2 = read();
        list2[parseInt(ta.dataset.i, 10)].text = ta.value;
        write(list2);
      });
    });
    Array.from(board.querySelectorAll("[data-del]")).forEach(function (btn) {
      btn.addEventListener("click", function () {
        const list2 = read();
        list2.splice(parseInt(btn.dataset.del, 10), 1);
        write(list2);
        render();
      });
    });
    Array.from(board.querySelectorAll(".note-card")).forEach(function (card) {
      card.addEventListener("pointerdown", function (e) {
        if (e.target.closest("textarea") || e.target.closest("button")) return;
        e.preventDefault();
        // #2261 — address the note by its `ts` key, never by its index in the
        // list snapshot: a note added/removed by the chat delta path while the
        // drag is in flight would otherwise be resurrected or deleted when the
        // snapshot was written back.
        var key = card.getAttribute("data-ts");
        var start = read();
        var idx = indexOfKey(start, key);
        if (idx === -1) return;
        var rect = card.getBoundingClientRect();
        var brect = board.getBoundingClientRect();
        // Absolute coordinates are relative to the board's padding box, so the
        // border width must be subtracted or the card jumps on the first drag.
        if (typeof start[idx].x !== "number" || typeof start[idx].y !== "number") {
          start[idx].x = Math.max(0, rect.left - brect.left - board.clientLeft + board.scrollLeft);
          start[idx].y = Math.max(0, rect.top - brect.top - board.clientTop + board.scrollTop);
        }
        var startX = e.clientX;
        var startY = e.clientY;
        var origX = start[idx].x;
        var origY = start[idx].y;
        board.style.position = "relative";
        board.style.minHeight = "600px";
        card.style.position = "absolute";
        card.style.left = origX + "px";
        card.style.top = origY + "px";
        card.style.width = rect.width + "px";
        card.style.zIndex = "1000";
        card.style.cursor = "grabbing";
        try { card.setPointerCapture(e.pointerId); } catch (err) {}
        function moveTo(x, y) {
          const list2 = read();
          const at = indexOfKey(list2, key);
          if (at === -1) return;
          list2[at].x = Math.max(0, x);
          list2[at].y = Math.max(0, y);
          write(list2);
        }
        var lastX = origX;
        var lastY = origY;
        function onMove(ev) {
          lastX = Math.max(0, origX + (ev.clientX - startX));
          lastY = Math.max(0, origY + (ev.clientY - startY));
          card.style.left = lastX + "px";
          card.style.top = lastY + "px";
        }
        function onUp() {
          card.removeEventListener("pointermove", onMove);
          card.removeEventListener("pointerup", onUp);
          card.removeEventListener("pointercancel", onUp);
          card.style.cursor = "grab";
          // Persisted on release (not only as a transient inline style) and
          // merged into the current list so the position survives a reload.
          moveTo(lastX, lastY);
          render();
        }
        card.addEventListener("pointermove", onMove);
        card.addEventListener("pointerup", onUp);
        card.addEventListener("pointercancel", onUp);
      });
    });
  }

  function add(text, color) {
    // #2225 — callers include a bare click listener and chat/automation hooks;
    // only a string is note text (a click Event must never become the body).
    const body = typeof text === "string" ? text : "";
    // NOTE-01: honor a requested color name when present in the phrase.
    var chosen = (typeof color === "string") ? color : null;
    if (!chosen && body) {
      var m = body.match(/(?:^|\s)(yellow|blue|green|pink|amber|cyan|purple)(?:\s|$)/i);
      if (m) {
        var map = { yellow: "#fef3c7", blue: "#bfdbfe", green: "#d9f99d", pink: "#fbcfe8", amber: "#fde68a", cyan: "#a7f3d0", purple: "#fbcfe8" };
        chosen = map[m[1].toLowerCase()] || null;
      }
    }
    const list = read();
    // Two notes created in the same millisecond must stay distinguishable:
    // the timestamp doubles as the note key for patch/delete.
    const newest = list.reduce(function (max, n) { return Math.max(max, Number(n.ts) || 0); }, 0);
    const ts = Math.max(Date.now(), newest + 1);
    list.unshift({
      text: body.replace(/(?:^|\s)(yellow|blue|green|pink|amber|cyan|purple)(?:\s|$)/i, " ").trim(),
      color: chosen || COLORS[Math.floor(Math.random() * COLORS.length)],
      ts: ts,
    });
    write(list);
    render();
    return ts;
  }

  // #2225 — a chat/automation stream addresses a note by the key `add`
  // returned; unknown keys are a no-op rather than a wrong-note update.
  function indexOfKey(list, key) {
    for (let i = 0; i < list.length; i += 1) {
      if (String(list[i].ts) === String(key)) return i;
    }
    return -1;
  }

  function patch(key, text) {
    if (typeof text !== "string") return false;
    const list = read();
    const at = indexOfKey(list, key);
    if (at === -1) return false;
    list[at].text = text;
    write(list);
    render();
    return true;
  }

  function remove(key) {
    const list = read();
    const at = indexOfKey(list, key);
    if (at === -1) return false;
    list.splice(at, 1);
    write(list);
    render();
    return true;
  }

  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  // #1981 — Notes is injected into an already-loaded desktop window, where
  // DOMContentLoaded never fires again; boot directly in that case so the
  // Add button and the restored notes work.
  function boot() {
    const addBtn = document.getElementById("notesAdd");
    if (addBtn) addBtn.addEventListener("click", add);
    render();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }

  window.GBNotes = { add: add, patch: patch, remove: remove, render: render };
})();