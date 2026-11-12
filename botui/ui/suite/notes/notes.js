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
      return;
    }
    board.innerHTML = list
      .map(function (n, i) {
        return (
          '<div class="note-card" style="background:' + n.color + '" data-i="' + i + '">' +
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