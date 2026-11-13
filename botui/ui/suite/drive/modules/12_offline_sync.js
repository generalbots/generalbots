/* Drive Module v2.0 — 12 Offline Sync (core): IndexedDB cache, queued
 * uploads and the online/offline indicator.
 *
 * Split from 12_offline_sync.js for #1731. The local-folder mirror lives in
 * 12_offline_sync_folder.js and shares this module state through
 * window.DriveOfflineSync (created at the end of this file). */
"use strict";

(function () {
  var DB_NAME = "gb_drive_offline";
  var DB_VERSION = 2;
  var STORE_FILES = "files";
  var STORE_PENDING = "pending_uploads";
  var STORE_HANDLES = "handles";
  var db = null;

  function openDB() {
    return new Promise(function (resolve, reject) {
      if (db) { resolve(db); return; }
      var req = indexedDB.open(DB_NAME, DB_VERSION);
      req.onupgradeneeded = function (e) {
        var d = e.target.result;
        if (!d.objectStoreNames.contains(STORE_FILES)) {
          d.createObjectStore(STORE_FILES, { keyPath: "cacheKey" });
        }
        if (!d.objectStoreNames.contains(STORE_PENDING)) {
          d.createObjectStore(STORE_PENDING, { keyPath: "id", autoIncrement: true });
        }
        if (!d.objectStoreNames.contains(STORE_HANDLES)) {
          d.createObjectStore(STORE_HANDLES, { keyPath: "name" });
        }
      };
      req.onsuccess = function (e) { db = e.target.result; resolve(db); };
      req.onerror = function () { reject(new Error("IndexedDB open failed")); };
    });
  }

  function cacheKey(bucket, path) {
    return (bucket || "") + ":" + (path || "");
  }

  function cacheFileListing(bucket, path, files) {
    return openDB().then(function (d) {
      return new Promise(function (resolve) {
        var tx = d.transaction(STORE_FILES, "readwrite");
        var store = tx.objectStore(STORE_FILES);
        store.put({ cacheKey: cacheKey(bucket, path), bucket: bucket, path: path, files: files, cachedAt: Date.now() });
        tx.oncomplete = function () { resolve(); };
        tx.onerror = function () { resolve(); };
      });
    }).catch(function (err) {
      /* Best-effort offline cache write; online drive remains the source of truth. */
      console.debug("[gb-drive] Offline cache write failed:", err);
    });
  }

  function getCachedListing(bucket, path) {
    return openDB().then(function (d) {
      return new Promise(function (resolve) {
        var tx = d.transaction(STORE_FILES, "readonly");
        var req = tx.objectStore(STORE_FILES).get(cacheKey(bucket, path));
        req.onsuccess = function () { resolve(req.result || null); };
        req.onerror = function () { resolve(null); };
      });
    }).catch(function (err) {
      /* Best-effort offline cache read; online drive remains the source of truth. */
      console.debug("[gb-drive] Offline cache read failed:", err);
      return null;
    });
  }

  function addPendingUpload(bucket, path, content, scope) {
    return openDB().then(function (d) {
      return new Promise(function (resolve) {
        var tx = d.transaction(STORE_PENDING, "readwrite");
        tx.objectStore(STORE_PENDING).add({ bucket: bucket, path: path, content: content, scope: scope, createdAt: Date.now() });
        tx.oncomplete = function () { resolve(); };
        tx.onerror = function () { resolve(); };
      });
    }).catch(function (err) {
      /* Best-effort offline queue write; upload may require manual retry. */
      console.debug("[gb-drive] Offline queue write failed:", err);
    });
  }

  function getPendingUploads() {
    return openDB().then(function (d) {
      return new Promise(function (resolve) {
        var tx = d.transaction(STORE_PENDING, "readonly");
        var req = tx.objectStore(STORE_PENDING).getAll();
        req.onsuccess = function () { resolve(req.result || []); };
        req.onerror = function () { resolve([]); };
      });
    }).catch(function () { return []; });
  }

  function removePendingUpload(id) {
    return openDB().then(function (d) {
      return new Promise(function (resolve) {
        var tx = d.transaction(STORE_PENDING, "readwrite");
        tx.objectStore(STORE_PENDING).delete(id);
        tx.oncomplete = function () { resolve(); };
        tx.onerror = function () { resolve(); };
      });
    }).catch(function (err) {
      /* Best-effort offline queue cleanup. */
      console.debug("[gb-drive] Offline queue cleanup failed:", err);
    });
  }

  function syncPendingUploads() {
    if (!navigator.onLine) return Promise.resolve();
    return getPendingUploads().then(function (items) {
      if (items.length === 0) return;
      if (typeof showNotification === "function") showNotification("Syncing " + items.length + " offline file(s)...", "info");
      var chain = Promise.resolve();
      items.forEach(function (item) {
        chain = chain.then(function () {
          return apiRequest("/write", {
            method: "POST",
            body: JSON.stringify({ bucket: item.bucket, path: item.path, content: item.content, scope: item.scope }),
          }).then(function () {
            return removePendingUpload(item.id);
          }).catch(function (err) {
            console.warn("[offline-sync] Failed to sync " + item.path + ":", err);
          });
        });
      });
      return chain.then(function () {
        if (typeof showNotification === "function") showNotification("Offline files synced", "success");
        if (typeof loadFiles === "function") loadFiles(currentPath, currentBucket);
      });
    });
  }

  var syncState = {
    state: navigator.onLine ? "idle" : "offline",
    folder: localStorage.getItem("gb_sync_folder") || "",
    lastSync: localStorage.getItem("gb_sync_last") || null,
    uploaded: 0,
    downloaded: 0,
    conflicts: 0,
    error: null,
  };

  window.addEventListener("online", function () {
    syncState.state = "idle";
    announce();
    setTimeout(syncPendingUploads, 2000);
  });
  window.addEventListener("offline", function () {
    syncState.state = "offline";
    announce();
  });

  function indicatorColor(state) {
    if (state === "synced") return "#22c55e";
    if (state === "syncing" || state === "conflict") return "#f59e0b";
    if (state === "error") return "#ef4444";
    if (state === "unsupported" || state === "no-folder" || state === "idle") return "#64748b";
    if (state === "offline") return "#94a3b8";
    return "#64748b";
  }

  function updateIndicator() {
    var dot = document.getElementById("gbSyncDot");
    if (dot) dot.style.background = indicatorColor(syncState.state);
    var button = document.getElementById("gbSyncStatusBtn");
    if (button) {
      button.title = "Offline sync: " + syncState.state +
        (syncState.folder ? " (" + syncState.folder + ")" : "") +
        (syncState.lastSync ? " · last " + gbFormatDateTime(new Date(syncState.lastSync)) : "");
    }
  }

  function announce() {
    updateIndicator();
    document.dispatchEvent(new CustomEvent("gb-offline-sync-status", { detail: Object.assign({}, syncState) }));
  }

  window.DriveOfflineSync = {
    cacheFileListing: cacheFileListing,
    getCachedListing: getCachedListing,
    addPendingUpload: addPendingUpload,
    syncPendingUploads: syncPendingUploads,
    getStatus: function () { return Object.assign({}, syncState); },
    getSyncFolder: function () { return syncState.folder; },
    // Shared with 12_offline_sync_folder.js (same split); not public API.
    _openDB: openDB,
    _STORE_HANDLES: STORE_HANDLES,
    _getPendingUploads: getPendingUploads,
    _state: syncState,
    _announce: announce,
    _updateIndicator: updateIndicator,
  };

  try { openDB().then(updateIndicator); } catch (e) {}
  document.addEventListener("DOMContentLoaded", updateIndicator);
})();
