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
      .then(function (r) {
        if (!r.ok) {
          return r.json().catch(function () { return {}; }).then(function (body) {
            var msg = (body && (body.error || body.message)) ? String(body.error || body.message) : (t("recycle-load-failed", "Could not load trash.") + " (HTTP " + r.status + ")");
            throw new Error(msg);
          });
        }
        return r.json();
      })
      .then(function (items) {
        if (!Array.isArray(items)) {
          list.innerHTML = '<div class="recycle-empty">' + escapeHtml(t("recycle-load-failed", "Could not load trash.")) + "</div>";
          return;
        }
        if (!items.length) {
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
            restore(btn.dataset.id, btn);
          });
        });
      })
      .catch(function (err) {
        list.innerHTML = '<div class="recycle-empty">' + escapeHtml(t("recycle-load-failed", "Could not load trash.")) + "</div>";
        if (err && err.message && window.GBToasts && window.GBToasts.show) {
          // optional: show a brief toast for auth errors
        }
      });
  }

  function toast(bodyKey, fallback, kind) {
    if (window.GBToasts) window.GBToasts.show(t("recycle-title", "Recycle Bin"), t(bodyKey, fallback), kind);
  }

  function restore(id, btn) {
    if (btn) btn.disabled = true;
    fetch("/api/files/trash/restore", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: id }),
    })
      .then(function (r) {
        if (!r.ok) {
          return r.json().catch(function () { return {}; }).then(function (body) {
            var msg = (body && (body.error || body.message)) ? String(body.error || body.message) : t("recycle-restore-failed", "Restore failed.");
            throw new Error(msg);
          });
        }
        return r.json().catch(function () { return {}; });
      })
      .then(function () {
        toast("recycle-restored", "File restored.", "success");
        load();
      })
      .catch(function (err) {
        toast("recycle-restore-failed", (err && err.message) ? err.message : t("recycle-restore-failed", "Restore failed."), "error");
      })
      .finally(function () {
        if (btn) btn.disabled = false;
      });
  }

  function empty() {
    const question = t("recycle-empty-confirm", "Permanently delete all items in the Recycle Bin?");
    if (!(window.GBModal || { confirm: confirm }).confirm(question)) return;
    const btn = document.getElementById("recycleEmpty");
    if (btn) btn.disabled = true;
    fetch("/api/files/trash/empty", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: "{}",
    })
      .then(function (r) {
        if (!r.ok) {
          return r.json().catch(function () { return {}; }).then(function (body) {
            var msg = (body && (body.error || body.message)) ? String(body.error || body.message) : t("recycle-empty-failed", "Empty failed.");
            throw new Error(msg);
          });
        }
        return r.json().catch(function () { return {}; });
      })
      .then(function () {
        toast("recycle-emptied", "Trash emptied.", "success");
        load();
      })
      .catch(function (err) {
        toast("recycle-empty-failed", (err && err.message) ? err.message : t("recycle-empty-failed", "Empty failed."), "error");
      })
      .finally(function () {
        if (btn) btn.disabled = false;
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

  function bootRecycle() {
    const emptyBtn = document.getElementById("recycleEmpty");
    if (emptyBtn) emptyBtn.addEventListener("click", empty);
    const refresh = document.getElementById("recycleRefresh");
    if (refresh) refresh.addEventListener("click", load);
    load();
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bootRecycle, { once: true });
  } else {
    bootRecycle();
  }

  window.addEventListener("localeChanged", load);

  window.GBRecycle = { load: load, restore: restore, empty: empty };
})();
