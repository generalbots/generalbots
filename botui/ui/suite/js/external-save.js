"use strict";

/* #2047 — one save-target menu for every surface that can store a file.
 *
 * The external drives were read-only, so "Save" always meant General Bots
 * Drive. This module turns it into a menu: the internal drive stays the
 * default, and one entry appears per connected provider whose recorded
 * consent includes a write scope (`can_write`). Capability is read from the
 * server payload, never guessed from a provider id.
 *
 * Surfaces that use it: chat artifact panel, VM output folder and the Drive
 * app's External tab. The last chosen target is remembered per browser; when
 * it disappears the menu falls back to internal Drive and says so.
 */

window.GBExternalSave = (function () {
  var CONNECTIONS_URL = "/api/external-drives/connections";
  var UPLOAD_URL = "/api/external-drives/upload";
  var STORAGE_KEY = "gb.save.target";
  var cache = null;
  var cacheAt = 0;

  function token() {
    try {
      return (
        (window.getGBAccessToken && window.getGBAccessToken()) ||
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        ""
      );
    } catch (e) {
      return "";
    }
  }

  function authHeaders(extra) {
    var headers = extra || {};
    var value = token();
    if (value) headers.Authorization = "Bearer " + value;
    return headers;
  }

  function toast(title, message, kind) {
    if (window.GBToasts && GBToasts.show) GBToasts.show(title, message, kind || "success");
    else if (window.showToast) window.showToast(message, kind || "success");
    else if (window.console) console.log(title + ": " + message);
  }

  function icon(name) {
    // Inline local marks (no CDN, no icon-font dependency).
    if (name === "gdrive") {
      return (
        '<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">' +
        '<path fill="#4285F4" d="M8.4 3h7.2l7.2 12.5H15.6z"/>' +
        '<path fill="#0F9D58" d="M1.2 15.5 4.8 9.3 8.4 3l3.6 6.2-7.2 12.4z"/>' +
        '<path fill="#F4B400" d="M5.1 21h13.5l2.4-4.2H7.5z"/>' +
        "</svg>"
      );
    }
    if (name === "onedrive") {
      return (
        '<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">' +
        '<path fill="#0364B8" d="M9.3 7.4a5.4 5.4 0 0 1 9.5 2.5 3.6 3.6 0 0 1 .6 7.1H6.9a4.2 4.2 0 0 1-.6-8.2 5.4 5.4 0 0 1 3-1.4z"/>' +
        "</svg>"
      );
    }
    return '<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true"><path fill="currentColor" d="M4 4h9l3 3h4v13H4z"/></svg>';
  }

  function connections(force) {
    var now = Date.now();
    if (!force && cache && now - cacheAt < 60000) return Promise.resolve(cache);
    return fetch(CONNECTIONS_URL, { headers: authHeaders({}) })
      .then(function (res) {
        if (!res.ok) throw new Error("HTTP " + res.status);
        return res.json();
      })
      .then(function (data) {
        cache = (data && data.providers) || [];
        cacheAt = Date.now();
        return cache;
      })
      .catch(function () {
        return [];
      });
  }

  function writeTargets() {
    return connections().then(function (providers) {
      return providers
        .filter(function (p) {
          return p && p.configured && p.connected && p.can_write;
        })
        .map(function (p) {
          return {
            id: p.provider,
            label: p.display_name || p.provider,
            icon: icon(p.provider),
          };
        });
    });
  }

  function remembered() {
    try {
      return localStorage.getItem(STORAGE_KEY) || "internal";
    } catch (e) {
      return "internal";
    }
  }

  function remember(id) {
    try {
      localStorage.setItem(STORAGE_KEY, id);
    } catch (e) { /* storage unavailable: the menu still works, without memory */ }
  }

  function closeMenu() {
    var open = document.querySelector(".gb-save-menu");
    if (open && open.parentNode) open.parentNode.removeChild(open);
  }

  /* Builds the menu next to the control. Resolves with the chosen target id
   * (`internal` or a provider slug), or `null` when dismissed. */
  function menu(anchor, targets) {
    closeMenu();
    return new Promise(function (resolve) {
      var box = document.createElement("div");
      box.className = "gb-save-menu";
      box.setAttribute("role", "menu");
      var options = [
        { id: "internal", label: "General Bots Drive", icon: icon("internal") },
      ].concat(targets);
      options.forEach(function (option) {
        var item = document.createElement("button");
        item.type = "button";
        item.className = "gb-save-menu-item";
        item.setAttribute("role", "menuitem");
        item.dataset.target = option.id;
        item.innerHTML = option.icon + '<span class="gb-save-menu-label"></span>';
        item.querySelector(".gb-save-menu-label").textContent =
          "Save to " + option.label;
        item.title = "Save to " + option.label;
        item.addEventListener("click", function () {
          closeMenu();
          resolve(option.id);
        });
        box.appendChild(item);
      });
      var rect = anchor && anchor.getBoundingClientRect
        ? anchor.getBoundingClientRect()
        : { left: 40, top: 40, bottom: 60 };
      box.style.position = "fixed";
      box.style.left = Math.max(8, Math.min(rect.left, window.innerWidth - 240)) + "px";
      box.style.top = Math.min(rect.bottom + 6, window.innerHeight - 160) + "px";
      box.style.zIndex = "12000";
      document.body.appendChild(box);
      var dismiss = function (event) {
        if (!box.contains(event.target)) {
          closeMenu();
          document.removeEventListener("mousedown", dismiss);
          resolve(null);
        }
      };
      setTimeout(function () {
        document.addEventListener("mousedown", dismiss);
      }, 0);
    });
  }

  /* Resolves the target for a control: internal immediately when no external
   * provider can be written to, otherwise the menu. A remembered provider that
   * no longer can write falls back to internal and says so. */
  function choose(anchor) {
    return writeTargets().then(function (targets) {
      if (!targets.length) {
        return "internal";
      }
      var last = remembered();
      if (last !== "internal") {
        var still = targets.some(function (t) { return t.id === last; });
        if (!still) {
          remember("internal");
          toast(
            "Save target",
            "The previous external drive is unavailable; saving to General Bots Drive.",
            "info"
          );
          return "internal";
        }
      }
      return menu(anchor, targets).then(function (picked) {
        if (!picked) return null;
        if (picked !== last) remember(picked);
        return picked;
      });
    });
  }

  function base64(bytes) {
    var view = new Uint8Array(bytes);
    var parts = [];
    var chunk = 0x8000;
    for (var i = 0; i < view.length; i += chunk) {
      parts.push(String.fromCharCode.apply(null, view.subarray(i, i + chunk)));
    }
    return btoa(parts.join(""));
  }

  function mimeFor(name) {
    var ext = String(name || "").split(".").pop().toLowerCase();
    var types = {
      pdf: "application/pdf",
      png: "image/png",
      jpg: "image/jpeg",
      jpeg: "image/jpeg",
      gif: "image/gif",
      webp: "image/webp",
      svg: "image/svg+xml",
      txt: "text/plain",
      md: "text/markdown",
      csv: "text/csv",
      json: "application/json",
      xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      pptx: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    };
    return types[ext] || "application/octet-stream";
  }

  /* Uploads one already-encoded file. Returns { web_view_link, name, ... }. */
  function upload(provider, options) {
    var body = {
      provider: provider,
      name: options.name,
      mime_type: options.mimeType || mimeFor(options.name),
      content: options.base64,
      path: options.folder || "",
    };
    if (options.parentId) body.folder_id = options.parentId;
    if (options.replaceRemoteId) body.replace_remote_id = options.replaceRemoteId;
    return fetch(UPLOAD_URL, {
      method: "POST",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify(body),
    }).then(function (res) {
      return res.json().then(function (data) {
        if (!res.ok) {
          var error = new Error((data && data.error) || "HTTP " + res.status);
          error.kind = data && data.kind;
          throw error;
        }
        return data;
      });
    });
  }

  /* Reads a workspace file as bytes through the signed VM endpoint. */
  function fetchWorkspaceFile(conversationId, path) {
    var url =
      "/api/agent/vm/file?conversation_id=" +
      encodeURIComponent(conversationId) +
      "&path=" +
      encodeURIComponent(path) +
      "&raw=1";
    var value = token();
    if (value) url += "&access_token=" + encodeURIComponent(value);
    return fetch(url).then(function (res) {
      if (!res.ok) throw new Error("HTTP " + res.status + " reading " + path);
      return res.arrayBuffer();
    });
  }

  function report(provider, saved) {
    var label = provider === "gdrive" ? "Google Drive" : provider === "onedrive" ? "OneDrive" : provider;
    var first = saved && saved[0];
    var message =
      "Saved " + saved.length + " file" + (saved.length === 1 ? "" : "s") + " to " + label;
    if (first && first.web_view_link) message += " · " + first.web_view_link;
    toast("Saved", message, "success");
    return message;
  }

  return {
    connections: connections,
    writeTargets: writeTargets,
    choose: choose,
    remember: remember,
    remembered: remembered,
    upload: upload,
    base64: base64,
    mimeFor: mimeFor,
    fetchWorkspaceFile: fetchWorkspaceFile,
    report: report,
    icon: icon,
    closeMenu: closeMenu,
  };
})();
