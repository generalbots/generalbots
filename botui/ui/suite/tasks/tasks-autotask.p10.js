"use strict";
/**
 * AutoTask items list (reform #1505).
 *
 * Every created automation is a row of `auto_tasks`; this renders those rows
 * in the Tasks app with a ✏️ action that opens the `.bas` the item produced
 * (the file lives in the bot's repository — edit → Source Control commit →
 * the git-pull monitor compiles it). A row without a resolved source shows
 * the reason instead of a dead button.
 */
(function () {
  var TOKEN = null;

  function authHeaders() {
    var token =
      (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
      localStorage.getItem("gb-access-token") ||
      localStorage.getItem("management_token") ||
      sessionStorage.getItem("gb-access-token") ||
      "";
    return token ? { Authorization: "Bearer " + token } : {};
  }

  function esc(s) {
    var d = document.createElement("div");
    d.textContent = s == null ? "" : String(s);
    return d.innerHTML;
  }

  function listEl() {
    return document.getElementById("autotask-items-list");
  }

  function resolveBotId() {
    return (
      window.__INITIAL_BOT_ID__ ||
      (window.GBSecurity && window.GBSecurity.botId) ||
      null
    );
  }

  function loadItems() {
    var list = listEl();
    if (!list) return;
    var botId = resolveBotId();
    var url = "/api/autotask/tasks" + (botId ? "?bot_id=" + encodeURIComponent(botId) : "");
    fetch(url, { headers: authHeaders() })
      .then(function (r) { return r.json(); })
      .then(function (rows) {
        if (!list.isConnected) return;
        if (!Array.isArray(rows) || !rows.length) {
          list.innerHTML =
            '<div class="autotask-items-empty">No AutoTask items yet — create one above.</div>';
          return;
        }
        list.innerHTML = rows
          .map(function (row) {
            var title = row.title || "automation";
            var status = row.status || "pending";
            var when = row.created_at ? new Date(row.created_at).toLocaleString() : "";
            return (
              '<div class="autotask-item" data-item-id="' + esc(row.id) + '">' +
              '<span class="autotask-item-status st-' + esc(status) + '">' + esc(status) + "</span>" +
              '<span class="autotask-item-title" title="' + esc(row.intent || "") + '">' + esc(title) + "</span>" +
              '<span class="autotask-item-date">' + esc(when) + "</span>" +
              '<button class="autotask-item-edit" title="Edit the .bas this item produced" data-item-edit="' +
              esc(row.id) + '">✏️</button>' +
              "</div>"
            );
          })
          .join("");
        list.querySelectorAll("[data-item-edit]").forEach(function (btn) {
          btn.addEventListener("click", function () {
            editItemSource(btn.getAttribute("data-item-edit"));
          });
        });
      })
      .catch(function () {
        if (list.isConnected) {
          list.innerHTML = '<div class="autotask-items-empty">Could not load AutoTask items.</div>';
        }
      });
  }

  /// Row action: resolve the item's `.bas` + owning project and open it in
  /// the suite editor's project mode (commit via Source Control → monitor
  /// compiles). One request resolves everything the editor needs.
  function editItemSource(itemId) {
    fetch("/api/autotask/items/" + encodeURIComponent(itemId) + "/source", { headers: authHeaders() })
      .then(function (r) { return r.json(); })
      .then(function (data) {
        if (!data || !data.success) {
          alert((data && data.message) || "Could not resolve this item's source.");
          return;
        }
        if (!window.WindowManager || !data.project_id) {
          alert("Editing needs the bot's project (no git project found).");
          return;
        }
        var ts = Date.now();
        window.__gbAppParams__ = Object.assign({}, window.__gbAppParams__ || {}, { project: data.project_id });
        window.__EDITOR_VIBE_PATH = ".gbdialog/" + data.name;
        window.WindowManager.open("editor-" + ts, data.name, "");
        fetch("/suite/editor.html")
          .then(function (r) { return r.text(); })
          .then(function (html) {
            window.WindowManager._injectBodyContent("editor-" + ts, html);
          });
      })
      .catch(function () {
        alert("Could not load this item's source.");
      });
  }

  // Refresh when the app mounts and whenever a task-created event fires
  // (create-and-execute records the row after the sources are committed).
  document.addEventListener("DOMContentLoaded", function () {
    loadItems();
    document.addEventListener("taskCreated", function () {
      setTimeout(loadItems, 1500);
    });
  });
  if (document.readyState !== "loading") {
    loadItems();
  }
})();
