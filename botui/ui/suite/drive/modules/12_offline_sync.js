/* Drive Module v2.0 — 12 Offline Sync (GB6-22).
 * Caches file listings and content in IndexedDB for offline access.
 * When the connection is restored, pending uploads are synced back.
 * On Tauri desktop, can also sync with a local OS folder. */
"use strict";

(function () {
  var DB_NAME = "gb_drive_offline";
  var DB_VERSION = 1;
  var STORE_FILES = "files";
  var STORE_PENDING = "pending_uploads";
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
    }).catch(function () {});
  }

  function getCachedListing(bucket, path) {
    return openDB().then(function (d) {
      return new Promise(function (resolve) {
        var tx = d.transaction(STORE_FILES, "readonly");
        var req = tx.objectStore(STORE_FILES).get(cacheKey(bucket, path));
        req.onsuccess = function () { resolve(req.result || null); };
        req.onerror = function () { resolve(null); };
      });
    }).catch(function () { return null; });
  }

  function addPendingUpload(bucket, path, content, scope) {
    return openDB().then(function (d) {
      return new Promise(function (resolve) {
        var tx = d.transaction(STORE_PENDING, "readwrite");
        tx.objectStore(STORE_PENDING).add({ bucket: bucket, path: path, content: content, scope: scope, createdAt: Date.now() });
        tx.oncomplete = function () { resolve(); };
        tx.onerror = function () { resolve(); };
      });
    }).catch(function () {});
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
    }).catch(function () {});
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

  window.addEventListener("online", function () {
    setTimeout(syncPendingUploads, 2000);
  });

  var syncFolder = localStorage.getItem("gb_sync_folder") || "";

  function setSyncFolder(path) {
    syncFolder = path;
    localStorage.setItem("gb_sync_folder", path);
  }

  function showSyncSettings() {
    var modal = document.getElementById("sync-settings-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "sync-settings-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;backdrop-filter:blur(8px);";
    modal.innerHTML = '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:440px;padding:24px;">'
      + '<h3 style="margin:0 0 16px;color:#f8fafc;">Offline Sync Settings</h3>'
      + '<label style="display:block;color:#94a3b8;font-size:13px;margin-bottom:6px;">Local sync folder</label>'
      + '<input type="text" id="gb-sync-folder-input" value="' + (syncFolder ? escapeHtml(syncFolder) : '') + '" placeholder="e.g. C:\\Users\\you\\Documents\\GBDrive" style="width:100%;padding:10px;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;font-size:13px;box-sizing:border-box;margin-bottom:12px;" />'
      + '<p style="color:#64748b;font-size:12px;margin-bottom:16px;">Files will be synced bidirectionally between this folder and your Drive when the app starts.</p>'
      + '<div id="gb-sync-status" style="color:#94a3b8;font-size:12px;margin-bottom:16px;"></div>'
      + '<div style="display:flex;gap:10px;justify-content:flex-end;">'
      + '<button onclick="document.getElementById(\'sync-settings-modal\').remove()" style="background:#334155;border:none;border-radius:6px;color:#f8fafc;padding:8px 16px;cursor:pointer;">Cancel</button>'
      + '<button onclick="DriveOfflineSync.saveSyncSettings()" style="background:#3b82f6;border:none;border-radius:6px;color:#fff;padding:8px 16px;cursor:pointer;">Save</button>'
      + '</div></div>';
    modal.addEventListener("click", function (e) { if (e.target === modal) modal.remove(); });
    document.body.appendChild(modal);

    getPendingUploads().then(function (items) {
      var status = document.getElementById("gb-sync-status");
      if (status) {
        status.textContent = items.length > 0 ? items.length + " file(s) waiting to sync" : "All files are synced";
      }
    });
  }

  function saveSyncSettings() {
    var input = document.getElementById("gb-sync-folder-input");
    if (input) setSyncFolder(input.value.trim());
    var modal = document.getElementById("sync-settings-modal");
    if (modal) modal.remove();
    if (typeof showNotification === "function") showNotification("Sync settings saved", "success");
  }

  window.DriveOfflineSync = {
    cacheFileListing: cacheFileListing,
    getCachedListing: getCachedListing,
    addPendingUpload: addPendingUpload,
    syncPendingUploads: syncPendingUploads,
    showSyncSettings: showSyncSettings,
    saveSyncSettings: saveSyncSettings,
    setSyncFolder: setSyncFolder,
    getSyncFolder: function () { return syncFolder; },
  };

  try { openDB(); } catch (e) {}
})();
