/*
 * gb-shared.js — single implementation of the frontend primitives every app
 * used to duplicate (issue #1740).
 *
 * Loaded before app scripts (`/suite/js/gb-shared.js` from the suite shell,
 * `/cloud/js/gb-shared.js` from the cloud pages). Pages that do not load this
 * file keep a local delegate whose fallback keeps them working standalone.
 *
 * Storage keys are intentionally unchanged: `gb-access-token` (canonical,
 * localStorage then sessionStorage) plus the documented legacy fallbacks
 * `token`, `id_token` and `management_token`. Token precedence matches the
 * implementations this kit replaces; do not reorder without checking callers.
 */
(function () {
  "use strict";

  if (window.GBShared && window.GBShared.html && window.GBShared.auth) return;

  var HTML_ESCAPES = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;"
  };

  // Escapes & < > " ' and preserves 0 / false (the `if (!value)` guards that
  // used to swallow them are gone on purpose).
  function escapeHtml(value) {
    return String(value == null ? "" : value).replace(/[&<>"']/g, function (ch) {
      return HTML_ESCAPES[ch];
    });
  }

  // Reads a JSON value stored by the suite (#2310). localStorage outlives a
  // deploy, so a truncated or hand-edited key used to throw at boot and brick
  // the desktop. A value that cannot be parsed is dropped once, logged, and
  // replaced by the caller's fallback, so the app always starts.
  function storeGet(key, fallback) {
    var raw;
    try {
      raw = window.localStorage.getItem(key);
    } catch (error) {
      return fallback;
    }
    if (raw === null || raw === "") return fallback;
    try {
      var parsed = JSON.parse(raw);
      return parsed === null ? fallback : parsed;
    } catch (error) {
      try {
        window.localStorage.removeItem(key);
      } catch (dropError) {
        // Nothing else to do: the fallback keeps this boot alive.
      }
      if (window.console && window.console.warn) {
        window.console.warn("[gb-shared] dropped unreadable localStorage key:", key);
      }
      return fallback;
    }
  }

  function read(storage, key) {
    try {
      return (storage && storage.getItem(key)) || "";
    } catch (error) {
      return "";
    }
  }

  function runtimeToken() {
    try {
      return (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) || "";
    } catch (error) {
      return "";
    }
  }

  // options.legacy     -> token, id_token before gb-access-token (vibe apps)
  // options.management -> management_token between local and session storage
  //                       (tasks autotask API)
  function token(options) {
    var opts = options || {};
    var found = runtimeToken();
    if (found) return found;

    if (opts.legacy) {
      found = read(window.localStorage, "token") || read(window.localStorage, "id_token");
      if (found) return found;
    }

    found = read(window.localStorage, "gb-access-token");
    if (found) return found;

    if (opts.management) {
      found = read(window.localStorage, "management_token");
      if (found) return found;
    }

    return read(window.sessionStorage, "gb-access-token");
  }

  function headers(extra, options) {
    var out = {};
    var key;
    var source = extra || {};
    for (key in source) {
      if (Object.prototype.hasOwnProperty.call(source, key)) out[key] = source[key];
    }
    var found = token(options);
    if (found) out.Authorization = "Bearer " + found;
    return out;
  }

  // #1834 — the suite runs inside a Tauri WebView in the desktop edition,
  // where `<a download>` and blob URLs are ignored by WebKitGTK. Route those
  // downloads through the native Save dialog; the web build keeps the plain
  // anchor fallback.
  function isDesktop() {
    return !!(window.__TAURI__ || window.__TAURI_INTERNALS__);
  }

  function tauriInvoke(command, args) {
    var tauri = window.__TAURI__;
    if (tauri && tauri.core && typeof tauri.core.invoke === "function") {
      return tauri.core.invoke(command, args);
    }
    if (
      window.__TAURI_INTERNALS__ &&
      typeof window.__TAURI_INTERNALS__.invoke === "function"
    ) {
      return window.__TAURI_INTERNALS__.invoke(command, args);
    }
    return Promise.reject(new Error("Tauri invoke is not available"));
  }

  function absoluteUrl(url) {
    try {
      return new URL(String(url || ""), window.location.href).href;
    } catch (error) {
      return String(url || "");
    }
  }

  function notify(message, kind) {
    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show("Download", message, kind);
    } else if (kind === "error") {
      console.error(message);
    }
  }

  function anchorDownload(url, fileName) {
    var link = document.createElement("a");
    link.href = url;
    link.download = fileName || "";
    link.rel = "noopener";
    document.body.appendChild(link);
    link.click();
    link.remove();
  }

  function reportSaved(saved, fileName) {
    if (!saved) return null;
    notify(fileName + " saved to " + saved, "success");
    return saved;
  }

  // `url` may be relative; `fileName` is the suggested Save-dialog name.
  function download(url, fileName) {
    var name = fileName || "";
    if (!isDesktop()) {
      anchorDownload(url, name);
      return Promise.resolve(null);
    }
    return tauriInvoke("save_download", {
      fileName: name,
      url: absoluteUrl(url),
      bytes: null,
    })
      .then(function (saved) {
        return reportSaved(saved, name);
      })
      .catch(function (error) {
        notify(
          "Download failed: " +
            (error && error.message ? error.message : String(error)),
          "error"
        );
        throw error;
      });
  }

  // A Blob (POST download endpoints, object URLs) uses the same Save dialog.
  function saveBlob(blob, fileName) {
    var name = fileName || "";
    if (!isDesktop()) {
      var objectUrl = URL.createObjectURL(blob);
      anchorDownload(objectUrl, name);
      setTimeout(function () {
        URL.revokeObjectURL(objectUrl);
      }, 1000);
      return Promise.resolve(null);
    }
    return blob
      .arrayBuffer()
      .then(function (buffer) {
        return tauriInvoke("save_download", {
          fileName: name,
          url: null,
          bytes: Array.from(new Uint8Array(buffer)),
        });
      })
      .then(function (saved) {
        return reportSaved(saved, name);
      })
      .catch(function (error) {
        notify(
          "Download failed: " +
            (error && error.message ? error.message : String(error)),
          "error"
        );
        throw error;
      });
  }

  var html = { escape: escapeHtml };
  var auth = { token: token, headers: headers };
  var downloadKit = { download: download, saveBlob: saveBlob, isDesktop: isDesktop };

  window.GBHtml = window.GBHtml || html;
  window.GBShared = {
    html: html,
    auth: auth,
    download: download,
    saveBlob: saveBlob,
    storeGet: storeGet
  };
  window.gbStoreGet = storeGet;
  window.GBDownload = window.GBDownload || downloadKit;
  // auth-service.js owns the rest of window.GBAuth; only add our two helpers.
  window.GBAuth = Object.assign(window.GBAuth || {}, auth);
  // Legacy global used by few remaining call sites; a page-local declaration
  // still hoists over this and wins, which is the pre-existing behaviour.
  if (typeof window.escapeHtml !== "function") window.escapeHtml = escapeHtml;
})();
