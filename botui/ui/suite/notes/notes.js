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
          '<textarea data-i="' + i + '" placeholder="Write…">' + escapeHtml(n.text) + "</textarea>" +
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

  function add() {
    const list = read();
    list.unshift({
      text: "",
      color: COLORS[Math.floor(Math.random() * COLORS.length)],
      ts: Date.now(),
    });
    write(list);
    render();
  }

  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  document.addEventListener("DOMContentLoaded", function () {
    const addBtn = document.getElementById("notesAdd");
    if (addBtn) addBtn.addEventListener("click", add);
    render();
  });

  window.GBNotes = { add: add, render: render };
})();