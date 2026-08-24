"use strict";
/* Recycle Bin (#1154): list, restore and permanently empty trashed Drive
   files via the /api/files/trash endpoints. */

(function () {
  if (window.GBRecycle) return;

  function load() {
    const list = document.getElementById("recycleList");
    if (!list) return;
    list.innerHTML = '<div class="recycle-empty">Loading…</div>';
    fetch("/api/files/trash?scope=user&_=" + Date.now())
      .then(function (r) { return r.json(); })
      .then(function (items) {
        if (!items || !items.length) {
          list.innerHTML = '<div class="recycle-empty">Trash is empty. 🎉</div>';
          return;
        }
        list.innerHTML = items
          .map(function (item) {
            const name = item.original_path ? item.original_path.split("/").pop() || item.path : item.path;
            return (
              '<div class="recycle-item">' +
              '<span class="recycle-icon">🗑</span>' +
              '<span class="recycle-name" title="' + escapeAttr(item.original_path || item.path) + '">' + escapeHtml(name) + "</span>" +
              '<span class="recycle-time">' + (item.deleted_at ? "Deleted " + escapeHtml(item.deleted_at) : "") + "</span>" +
              '<button class="recycle-restore" data-id="' + escapeAttr(item.id) + '">↩ Restore</button>' +
              "</div>"
            );
          })
          .join("");
        Array.from(list.querySelectorAll(".recycle-restore")).forEach(function (btn) {
          btn.addEventListener("click", function () {
            restore(btn.dataset.id);
          });
        });
      })
      .catch(function () {
        list.innerHTML = '<div class="recycle-empty">Could not load trash.</div>';
      });
  }

  function restore(id) {
    fetch("/api/files/trash/restore", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id }),
    })
      .then(function (r) { return r.json(); })
      .then(function () {
        if (window.GBToasts) window.GBToasts.show("Recycle Bin", "File restored.", "success");
        load();
      })
      .catch(function () {
        if (window.GBToasts) window.GBToasts.show("Recycle Bin", "Restore failed.", "error");
      });
  }

  function empty() {
    if (!window.confirm("Permanently delete all items in the Recycle Bin?")) return;
    fetch("/api/files/trash/empty", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: "{}",
    })
      .then(function (r) { return r.json(); })
      .then(function () {
        if (window.GBToasts) window.GBToasts.show("Recycle Bin", "Trash emptied.", "success");
        load();
      })
      .catch(function () {
        if (window.GBToasts) window.GBToasts.show("Recycle Bin", "Empty failed.", "error");
      });
  }

  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function escapeAttr(s) {
    return String(s).replace(/"/g, "&quot;");
  }

  document.addEventListener("DOMContentLoaded", function () {
    const emptyBtn = document.getElementById("recycleEmpty");
    if (emptyBtn) emptyBtn.addEventListener("click", empty);
    const refresh = document.getElementById("recycleRefresh");
    if (refresh) refresh.addEventListener("click", load);
    load();
  });

  window.GBRecycle = { load: load, restore: restore, empty: empty };
})();