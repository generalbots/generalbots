"use strict";
/* Recycle Bin (#1154): list, restore and permanently empty trashed Drive
   files via the /api/files/trash endpoints. */

(function () {
  if (window.GBRecycle) return;

  function t(key, fallback) {
    return window.gbT ? window.gbT(key, fallback) : fallback;
  }

  function load() {
    const list = document.getElementById("recycleList");
    if (!list) return;
    list.innerHTML = '<div class="recycle-empty">' + escapeHtml(t("label-loading", "Loading…")) + "</div>";
    fetch("/api/files/trash?scope=user&_=" + Date.now())
      .then(function (r) { return r.json(); })
      .then(function (items) {
        if (!items || !items.length) {
          list.innerHTML = '<div class="recycle-empty">' + escapeHtml(t("recycle-empty-state", "Trash is empty. 🎉")) + "</div>";
          return;
        }
        const deletedPrefix = t("recycle-deleted-at", "Deleted");
        list.innerHTML = items
          .map(function (item) {
            const name = item.original_path ? item.original_path.split("/").pop() || item.path : item.path;
            return (
              '<div class="recycle-item">' +
              '<span class="recycle-icon">🗑</span>' +
              '<span class="recycle-name" title="' + escapeAttr(item.original_path || item.path) + '">' + escapeHtml(name) + "</span>" +
              '<span class="recycle-time">' + (item.deleted_at ? escapeHtml(deletedPrefix + " " + item.deleted_at) : "") + "</span>" +
              '<button class="recycle-restore" data-id="' + escapeAttr(item.id) + '">↩ ' + escapeHtml(t("recycle-restore", "Restore")) + "</button>" +
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
        list.innerHTML = '<div class="recycle-empty">' + escapeHtml(t("recycle-load-failed", "Could not load trash.")) + "</div>";
      });
  }

  function toast(bodyKey, fallback, kind) {
    if (window.GBToasts) window.GBToasts.show(t("recycle-title", "Recycle Bin"), t(bodyKey, fallback), kind);
  }

  function restore(id) {
    fetch("/api/files/trash/restore", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id }),
    })
      .then(function (r) { return r.json(); })
      .then(function () {
        toast("recycle-restored", "File restored.", "success");
        load();
      })
      .catch(function () {
        toast("recycle-restore-failed", "Restore failed.", "error");
      });
  }

  function empty() {
    const question = t("recycle-empty-confirm", "Permanently delete all items in the Recycle Bin?");
    if (!(window.GBModal || { confirm: confirm }).confirm(question)) return;
    fetch("/api/files/trash/empty", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: "{}",
    })
      .then(function (r) { return r.json(); })
      .then(function () {
        toast("recycle-emptied", "Trash emptied.", "success");
        load();
      })
      .catch(function () {
        toast("recycle-empty-failed", "Empty failed.", "error");
      });
  }

  function escapeHtml(value) {
    if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
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

  window.addEventListener("localeChanged", load);

  window.GBRecycle = { load: load, restore: restore, empty: empty };
})();
