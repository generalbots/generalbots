/* Assistant parity: Drive file actions inside chat's existing Drive picker.
 * The picker (chat-drive-picker.js) intentionally exposes only open(); this
 * module wraps it and decorates its rows without replacing any picker code. */
"use strict";
(function () {
  var P = (window.GBChatAssistantParity = window.GBChatAssistantParity || {});
  var MORE = "gb-parity-drive-more";
  var CTX = "gb-parity-drive-ctx";
  var DIALOG = "gb-parity-drive-confirm";
  var STATUS = "gb-parity-drive-status";
  var ctxEl = null;
  var dialogEl = null;
  var observer = null;
  var listenersReady = false;

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function logError(error, where) {
    var label = "assistant-parity:drive-actions:" + where;
    if (typeof window.gbLogError === "function") window.gbLogError(error, label);
    else if (window.console && console.error) console.error(label, error);
  }
  function closest(target, selector) {
    return target && typeof target.closest === "function" ? target.closest(selector) : null;
  }
  function token() {
    try {
      if (typeof window.getChatAccessToken === "function") return window.getChatAccessToken() || "";
      if (window.GBSecurity && typeof window.GBSecurity.getToken === "function") {
        return window.GBSecurity.getToken() || "";
      }
      return sessionStorage.getItem("gb-access-token") || "";
    } catch (error) {
      return "";
    }
  }
  function headers(extra) {
    var value = token();
    var base = extra || {};
    if (value) base.Authorization = "Bearer " + value;
    return base;
  }
  function botName() {
    return (window.ChatState && window.ChatState.currentBotName) ||
      window.__INITIAL_BOT_NAME__ || "default";
  }
  function pickerOverlay() {
    return document.querySelector(".gb-drive-picker-overlay");
  }
  function inPicker(element) {
    var overlay = pickerOverlay();
    return !!(overlay && element && overlay.contains(element));
  }
  function injectStyles() {
    if (document.getElementById("gb-parity-drive-styles")) return;
    var style = document.createElement("style");
    style.id = "gb-parity-drive-styles";
    style.textContent =
      "." + MORE + "{margin-left:6px;flex:0 0 auto;border:1px solid rgba(128,128,128,.45);background:transparent;border-radius:4px;line-height:1;cursor:pointer;padding:0 5px;font-size:14px;color:inherit}" +
      "." + CTX + "{position:fixed;z-index:var(--z-top);min-width:160px;padding:4px;background:var(--gb-panel,#fff);color:var(--gb-text,#222);border:1px solid rgba(128,128,128,.4);border-radius:6px;box-shadow:0 6px 18px rgba(0,0,0,.2);display:flex;flex-direction:column}" +
      "." + CTX + "-item{display:block;width:100%;text-align:left;border:0;background:transparent;color:inherit;padding:7px 10px;border-radius:4px;cursor:pointer;font:inherit}" +
      "." + CTX + "-item:hover{background:rgba(128,128,128,.15)}" + "." + CTX + "-danger{color:#dc2626}" +
      "." + STATUS + "{margin:4px 10px;font-size:12px;color:var(--gb-muted,#666)}" +
      "." + DIALOG + "{position:fixed;inset:0;z-index:var(--z-top);background:rgba(0,0,0,.45);display:flex;align-items:center;justify-content:center}" +
      "." + DIALOG + "-box{background:var(--gb-panel,#fff);color:var(--gb-text,#222);padding:18px;border-radius:8px;min-width:280px;max-width:420px;box-shadow:0 10px 30px rgba(0,0,0,.3)}" +
      "." + DIALOG + "-title{font-weight:600;margin-bottom:8px}" + "." + DIALOG + "-text{margin-bottom:10px;word-break:break-word}" +
      ".gb-parity-drive-input{width:100%;box-sizing:border-box;padding:8px;margin-bottom:12px;border:1px solid rgba(128,128,128,.5);border-radius:4px;background:transparent;color:inherit}" +
      "." + DIALOG + "-actions{display:flex;justify-content:flex-end;gap:8px}" +
      ".gb-parity-drive-btn{border:1px solid rgba(128,128,128,.5);background:transparent;color:inherit;padding:7px 12px;border-radius:4px;cursor:pointer}" +
      ".gb-parity-drive-btn-primary{background:#2563eb;border-color:#2563eb;color:#fff}";
    (document.head || document.body).appendChild(style);
  }
  function ensureStatus(overlay) {
    var status = overlay.querySelector("." + STATUS);
    if (!status) {
      status = document.createElement("div");
      status.className = STATUS;
      status.setAttribute("role", "status");
      status.setAttribute("aria-live", "polite");
      status.setAttribute("aria-atomic", "true");
      status.hidden = true;
      overlay.appendChild(status);
    }
    return status;
  }
  function setStatus(message) {
    var overlay = pickerOverlay();
    if (!overlay) return;
    var status = ensureStatus(overlay);
    status.textContent = message || "";
    status.hidden = !message;
  }
  function setDialogText(message) {
    if (!dialogEl) return;
    var text = dialogEl.querySelector("." + DIALOG + "-text");
    if (text) text.textContent = message;
  }
  function errorText(key, fallback, error) {
    if (error && error.unavailable) return error.message;
    var detail = error && error.message ? error.message : "";
    return t(key, fallback) + (detail ? ": " + detail : "");
  }
  function decorateRows() {
    var overlay = pickerOverlay();
    if (!overlay) return;
    ensureStatus(overlay);
    var rows = overlay.querySelectorAll(".gb-drive-picker-row:not(.is-dir)");
    Array.prototype.forEach.call(rows, function (row) {
      if (row.querySelector("." + MORE)) return;
      var button = document.createElement("button");
      button.type = "button";
      button.className = MORE;
      button.textContent = "\u22ef";
      button.setAttribute("aria-label", t("chat-drive-parity-more", "More file actions"));
      button.title = t("chat-drive-parity-more", "More file actions");
      button.__gbParityRow = row;
      row.appendChild(button);
    });
  }
  function observe() {
    if (observer || typeof MutationObserver !== "function") return;
    observer = new MutationObserver(function () { decorateRows(); });
    observer.observe(document.body, { childList: true, subtree: true });
  }
  function rowPath(name) {
    var overlay = pickerOverlay();
    var parts = [];
    if (overlay) {
      var crumbs = overlay.querySelectorAll(".gb-drive-picker-crumb-btn");
      for (var i = 1; i < crumbs.length; i++) {
        parts.push((crumbs[i].textContent || "").trim());
      }
    }
    var folder = parts.join("/");
    return folder ? folder + "/" + name : name;
  }
  function refresh() {
    var overlay = pickerOverlay();
    if (!overlay) return;
    var crumbs = overlay.querySelectorAll(".gb-drive-picker-crumb-btn");
    var last = crumbs.length ? crumbs[crumbs.length - 1] : null;
    if (last && typeof last.click === "function") last.click();
    else if (window.GBDrivePicker && typeof window.GBDrivePicker.open === "function") {
      window.GBDrivePicker.open();
    }
  }
  function closeMenu() {
    if (!ctxEl) return;
    if (ctxEl.parentNode) ctxEl.parentNode.removeChild(ctxEl);
    ctxEl = null;
  }
  function openMenu(event, anchor, row) {
    if (!row) return;
    closeMenu();
    var nameEl = row.querySelector(".gb-drive-picker-name");
    var name = nameEl ? (nameEl.textContent || "").trim() : "";
    if (!name) return;
    ctxEl = document.createElement("div");
    ctxEl.className = CTX;
    ctxEl.setAttribute("role", "menu");
    var defs = [
      ["use-in-chat", "chat-drive-parity-use-in-chat", "Use in conversation"], ["rename", "chat-drive-parity-rename", "Rename"],
      ["download", "chat-drive-parity-download", "Download"], ["delete", "chat-drive-parity-delete", "Delete"],
      ["preview", "chat-drive-parity-preview", "Preview"]
    ];
    defs.forEach(function (def) {
      var item = document.createElement("button");
      item.type = "button";
      item.className = CTX + "-item" + (def[0] === "delete" ? " " + CTX + "-danger" : "");
      item.setAttribute("role", "menuitem");
      item.setAttribute("data-action", def[0]);
      item.textContent = t(def[1], def[2]);
      item.__gbParityCtx = { row: row, name: name, path: rowPath(name) };
      ctxEl.appendChild(item);
    });
    document.body.appendChild(ctxEl);
    ctxEl.__gbParityRow = row;
    var x = event && event.clientX ? event.clientX : 0;
    var y = event && event.clientY ? event.clientY : 0;
    if ((!x || !y) && anchor && typeof anchor.getBoundingClientRect === "function") {
      var rect = anchor.getBoundingClientRect();
      x = x || rect.right;
      y = y || rect.bottom;
    }
    ctxEl.style.left = Math.max(4, Math.min(x, window.innerWidth - (ctxEl.offsetWidth || 170) - 8)) + "px";
    ctxEl.style.top = Math.max(4, Math.min(y, window.innerHeight - (ctxEl.offsetHeight || 180) - 8)) + "px";
  }
  function post(path, body) {
    if (typeof window.fetch !== "function") {
      return Promise.reject(new Error(t("chat-drive-parity-unavailable", "This action is not available on this server.")));
    }
    var payload = Object.assign({}, body || {}, { bot: botName() });
    return window.fetch(path, { method: "POST", headers: headers({ "Content-Type": "application/json" }), body: JSON.stringify(payload) }).then(function (response) {
      if (response.status === 404 || response.status === 405) {
        var error = new Error(t("chat-drive-parity-unavailable", "This action is not available on this server."));
        error.unavailable = true;
        throw error;
      }
      if (!response.ok) throw new Error("HTTP " + response.status);
      return response;
    });
  }
  function saveBlob(blob, name) {
    if (window.GBShared && typeof window.GBShared.saveBlob === "function") {
      window.GBShared.saveBlob(blob, name);
      return;
    }
    if (typeof URL === "undefined" || typeof URL.createObjectURL !== "function") return;
    var url = URL.createObjectURL(blob);
    var link = document.createElement("a");
    link.href = url; link.download = name;
    document.body.appendChild(link); link.click();
    if (link.parentNode) link.parentNode.removeChild(link);
    setTimeout(function () { URL.revokeObjectURL(url); }, 30000);
  }
  function closeDialog() {
    if (!dialogEl) return;
    dialogEl.removeEventListener("click", onDialogClick);
    if (dialogEl.parentNode) dialogEl.parentNode.removeChild(dialogEl);
    dialogEl = null;
  }
  function onDialogClick(event) {
    if (!dialogEl) return;
    if (event.target === dialogEl) { closeDialog(); return; }
    var action = closest(event.target, "[data-act]");
    if (!action || !dialogEl.contains(action)) return;
    if (action.getAttribute("data-act") === "cancel") { closeDialog(); return; }
    if (action.disabled) return;
    action.disabled = true;
    try {
      var result = dialogEl.__gbParityOnOk();
      if (result && typeof result.then === "function") {
        result.then(function () { closeDialog(); }, function () { action.disabled = false; });
      }
    } catch (error) {
      logError(error, "dialog");
      action.disabled = false;
    }
  }
  function openDialog(options) {
    closeDialog();
    dialogEl = document.createElement("div");
    dialogEl.className = DIALOG;
    dialogEl.setAttribute("role", "dialog");
    dialogEl.setAttribute("aria-modal", "true");
    dialogEl.setAttribute("aria-labelledby", "gb-parity-drive-dialog-title");
    var box = document.createElement("div");
    box.className = DIALOG + "-box";
    var title = document.createElement("div");
    title.className = DIALOG + "-title"; title.id = "gb-parity-drive-dialog-title"; title.textContent = options.title;
    var text = document.createElement("div");
    text.className = DIALOG + "-text"; text.textContent = options.message;
    box.appendChild(title); box.appendChild(text);
    var input = null;
    if (typeof options.inputValue === "string") {
      input = document.createElement("input");
      input.type = "text"; input.className = "gb-parity-drive-input";
      input.value = options.inputValue; input.maxLength = 200;
      input.setAttribute("aria-label", t("chat-drive-parity-new-name", "New name"));
      box.appendChild(input);
    }
    var actions = document.createElement("div");
    actions.className = DIALOG + "-actions";
    var ok = document.createElement("button");
    ok.type = "button"; ok.className = "gb-parity-drive-btn gb-parity-drive-btn-primary";
    ok.setAttribute("data-act", "ok"); ok.textContent = options.okLabel;
    var cancel = document.createElement("button");
    cancel.type = "button"; cancel.className = "gb-parity-drive-btn";
    cancel.setAttribute("data-act", "cancel"); cancel.textContent = t("chat-drive-parity-cancel", "Cancel");
    actions.appendChild(ok); actions.appendChild(cancel);
    box.appendChild(actions); dialogEl.appendChild(box);
    dialogEl.__gbParityOnOk = function () { return options.onOk(input ? input.value : ""); };
    document.body.appendChild(dialogEl);
    dialogEl.addEventListener("click", onDialogClick);
    if (input) { input.focus(); input.select(); } else { cancel.focus(); }
  }
  function rename(data) {
    openDialog({
      title: t("chat-drive-parity-rename-title", "Rename file"),
      message: data.name,
      inputValue: data.name,
      okLabel: t("chat-drive-parity-rename", "Rename"),
      onOk: function (value) {
        var name = String(value || "").trim();
        if (!name || name === data.name) {
          setStatus("");
          return Promise.resolve();
        }
        var slash = data.path.lastIndexOf("/");
        var parent = slash >= 0 ? data.path.substring(0, slash) : "";
        var dest = parent ? parent + "/" + name : name;
        setStatus(t("chat-drive-parity-renaming", "Renaming…") + " " + data.name);
        return post("/api/files/move", { source_path: data.path, dest_path: dest, scope: "bot" }).then(function () {
          setStatus(t("chat-drive-parity-renamed", "Renamed to") + " " + name);
          refresh();
        }).catch(function (error) {
          var message = errorText("chat-drive-parity-rename-failed", "Rename failed", error);
          setStatus(message);
          setDialogText(message);
          throw error;
        });
      }
    });
  }
  function remove(data) {
    openDialog({
      title: t("chat-drive-parity-delete-title", "Delete file"),
      message: t("chat-drive-parity-delete-question", "Delete this file?") + " " + data.name,
      okLabel: t("chat-drive-parity-delete", "Delete"),
      onOk: function () {
        setStatus(t("chat-drive-parity-deleting", "Deleting…") + " " + data.name);
        return post("/api/files/trash/move", { path: data.path, scope: "bot" })
          .then(function () {
            setStatus(t("chat-drive-parity-deleted", "Deleted") + " " + data.name);
            refresh();
          }).catch(function (error) {
            var message = errorText("chat-drive-parity-delete-failed", "Delete failed", error);
            setStatus(message);
            setDialogText(message);
            throw error;
          });
      }
    });
  }
  function download(data) {
    setStatus(t("chat-drive-parity-downloading", "Downloading…") + " " + data.name);
    post("/api/files/download-binary", { path: data.path, scope: "bot" })
      .then(function (response) { return response.blob(); })
      .then(function (blob) {
        saveBlob(blob, data.name);
        setStatus("");
      }).catch(function (error) {
        setStatus(errorText("chat-drive-parity-download-failed", "Download failed", error));
      });
  }
  function preview(data) {
    if (typeof URL === "undefined" || typeof URL.createObjectURL !== "function") {
      setStatus(t("chat-drive-parity-unavailable", "This action is not available on this server."));
      return;
    }
    setStatus(t("chat-drive-parity-previewing", "Opening preview…") + " " + data.name);
    post("/api/files/download-inline?bot=" + encodeURIComponent(botName()), {
      path: data.path,
      scope: "bot"
    }).then(function (response) { return response.blob(); })
      .then(function (blob) {
        var url = URL.createObjectURL(blob);
        var win = window.open(url, "_blank", "noopener,noreferrer");
        if (!win) saveBlob(blob, data.name);
        setTimeout(function () { URL.revokeObjectURL(url); }, 60000);
        setStatus("");
      }).catch(function (error) {
        setStatus(errorText("chat-drive-parity-preview-failed", "Preview failed", error));
      });
  }
  function runAction(action, data) {
    if (!data) return;
    if (action === "use-in-chat") {
      if (data.row && typeof data.row.click === "function") {
        setStatus(t("chat-drive-parity-attaching", "Attaching…") + " " + data.name);
        data.row.click();
      }
    } else if (action === "rename") rename(data);
    else if (action === "download") download(data);
    else if (action === "delete") remove(data);
    else if (action === "preview") preview(data);
  }
  function onClick(event) {
    var more = closest(event.target, "." + MORE);
    if (more && inPicker(more)) {
      event.preventDefault();
      event.stopPropagation();
      var row = more.__gbParityRow;
      if (ctxEl && ctxEl.__gbParityRow === row) closeMenu();
      else openMenu(event, more, row);
      return;
    }
    var item = closest(event.target, "." + CTX + "-item");
    if (item && ctxEl && ctxEl.contains(item)) {
      event.preventDefault();
      event.stopPropagation();
      var data = item.__gbParityCtx;
      closeMenu();
      runAction(item.getAttribute("data-action"), data);
      return;
    }
    if (ctxEl && !closest(event.target, "." + CTX)) closeMenu();
    if (dialogEl && event.target === dialogEl) closeDialog();
  }
  function onContextMenu(event) {
    var row = closest(event.target, ".gb-drive-picker-row");
    if (!row || !inPicker(row) || row.classList.contains("is-dir")) return;
    event.preventDefault();
    event.stopPropagation();
    openMenu(event, row, row);
  }
  function onKeyDown(event) {
    if (event.key !== "Escape") return;
    if (dialogEl) {
      event.preventDefault();
      event.stopPropagation();
      closeDialog();
    } else if (ctxEl) {
      event.preventDefault();
      event.stopPropagation();
      closeMenu();
    }
  }
  function ensureListeners() {
    if (listenersReady) return;
    listenersReady = true;
    document.addEventListener("click", onClick, true);
    document.addEventListener("contextmenu", onContextMenu, true);
    document.addEventListener("keydown", onKeyDown, true);
    observe();
  }
  function wrap() {
    var picker = window.GBDrivePicker;
    if (!picker || typeof picker.open !== "function" || picker.__gbParityDriveWrapped) return;
    var original = picker.open;
    picker.open = function () {
      var result = original.apply(picker, arguments);
      try { decorateRows(); } catch (error) { logError(error, "decorate"); }
      return result;
    };
    picker.__gbParityDriveWrapped = true;
  }
  P.driveActions = {
    mount: function () {
      try {
        injectStyles();
        ensureListeners();
        if (window.GBDrivePicker && typeof window.GBDrivePicker.open === "function") wrap();
        var tries = 0;
        var timer = setInterval(function () {
          if (window.GBDrivePicker && typeof window.GBDrivePicker.open === "function") {
            clearInterval(timer);
            wrap();
          } else if (++tries >= 20) {
            clearInterval(timer);
          }
        }, 100);
      } catch (error) {
        logError(error, "mount");
      }
    }
  };
})();
