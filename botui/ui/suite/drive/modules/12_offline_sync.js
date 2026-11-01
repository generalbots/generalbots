/* Drive Module v2.0 — 12 Offline Sync (GB6-22).
 * Caches file listings and content in IndexedDB for offline access.
 * When the connection is restored, pending uploads are synced back.
 * On Tauri desktop, can also sync with a local OS folder. */
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
    syncState.state = "idle";
    announce();
    setTimeout(syncPendingUploads, 2000);
  });
  window.addEventListener("offline", function () {
    syncState.state = "offline";
    announce();
  });

  // ── Local folder sync (GB6-22) ─────────────────────────────────────
  // The browser can only reach a real folder through the File System Access
  // API. The handle is persisted in IndexedDB (it survives reloads); syncing is
  // explicit and reports its state, so it never pretends to be live when the
  // platform does not support it.
  var MANIFEST_KEY = "gb_sync_manifest_v1";
  var MAX_SYNC_BYTES = 20 * 1024 * 1024;
  var syncState = {
    state: navigator.onLine ? "idle" : "offline",
    folder: localStorage.getItem("gb_sync_folder") || "",
    lastSync: localStorage.getItem("gb_sync_last") || null,
    uploaded: 0,
    downloaded: 0,
    conflicts: 0,
    error: null,
  };

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
        (syncState.lastSync ? " · last " + new Date(syncState.lastSync).toLocaleString() : "");
    }
  }

  function announce() {
    updateIndicator();
    document.dispatchEvent(new CustomEvent("gb-offline-sync-status", { detail: Object.assign({}, syncState) }));
  }

  function supportsFolderSync() {
    return typeof window.showDirectoryPicker === "function";
  }

  function putHandle(handle) {
    return openDB().then(function (d) {
      return new Promise(function (resolve, reject) {
        var tx = d.transaction(STORE_HANDLES, "readwrite");
        tx.objectStore(STORE_HANDLES).put({ name: "sync", handle: handle });
        tx.oncomplete = function () { resolve(); };
        tx.onerror = function () { reject(new Error("could not store the folder handle")); };
      });
    });
  }

  function getHandle() {
    return openDB().then(function (d) {
      return new Promise(function (resolve) {
        var req = d.transaction(STORE_HANDLES, "readonly").objectStore(STORE_HANDLES).get("sync");
        req.onsuccess = function () { resolve((req.result && req.result.handle) || null); };
        req.onerror = function () { resolve(null); };
      });
    }).catch(function () { return null; });
  }

  async function pickFolder() {
    if (!supportsFolderSync()) {
      syncState.state = "unsupported";
      syncState.error = "This platform does not expose a local folder picker.";
      announce();
      if (typeof showNotification === "function") {
        showNotification("Local folder sync is not available in this build", "error");
      }
      return null;
    }
    try {
      var handle = await window.showDirectoryPicker({ mode: "readwrite", id: "gb-drive-sync" });
      await putHandle(handle);
      syncState.folder = handle.name;
      syncState.error = null;
      localStorage.setItem("gb_sync_folder", handle.name);
      announce();
      return handle;
    } catch (error) {
      if (error && error.name !== "AbortError") {
        syncState.error = error.message || "folder selection failed";
        announce();
      }
      return null;
    }
  }

  async function ensurePermission(handle) {
    var options = { mode: "readwrite" };
    if ((await handle.queryPermission(options)) === "granted") return true;
    return (await handle.requestPermission(options)) === "granted";
  }

  function readManifest() {
    try { return JSON.parse(localStorage.getItem(MANIFEST_KEY) || "{}"); } catch (e) { return {}; }
  }

  function saveManifest(manifest) {
    try { localStorage.setItem(MANIFEST_KEY, JSON.stringify(manifest)); } catch (e) {}
  }

  function fileToBase64(file) {
    return new Promise(function (resolve, reject) {
      var reader = new FileReader();
      reader.onload = function () {
        var result = String(reader.result || "");
        resolve(result.indexOf(",") >= 0 ? result.split(",")[1] : result);
      };
      reader.onerror = function () { reject(reader.error || new Error("read failed")); };
      reader.readAsDataURL(file);
    });
  }

  function base64ToBytes(base64) {
    var binary = atob(base64);
    var bytes = new Uint8Array(binary.length);
    for (var i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    return bytes;
  }

  function conflictName(name, when) {
    var dot = name.lastIndexOf(".");
    var stamp = when.toISOString().replace(/[:.]/g, "-").slice(0, 19);
    return dot > 0
      ? name.slice(0, dot) + " (conflict " + stamp + ")" + name.slice(dot)
      : name + " (conflict " + stamp + ")";
  }

  async function listDrive(path) {
    var query = "/list?bucket=" + encodeURIComponent(getEffectiveBucket() || "") +
      "&path=" + encodeURIComponent(path || "") +
      "&scope=" + encodeURIComponent(currentScope || "");
    var items = await apiRequest(query, { method: "GET" });
    return Array.isArray(items) ? items : [];
  }

  async function uploadLocal(fileHandle, remotePath) {
    var file = await fileHandle.getFile();
    if (file.size > MAX_SYNC_BYTES) return false;
    var content = await fileToBase64(file);
    await apiRequest("/write", {
      method: "POST",
      body: JSON.stringify({
        bucket: getEffectiveBucket(),
        path: remotePath,
        content: content,
        scope: currentScope,
      }),
    });
    return true;
  }

  async function downloadRemote(item, dirHandle) {
    if (!item || !item.name || item.size > MAX_SYNC_BYTES) return false;
    var payload = await apiRequest("/download", {
      method: "POST",
      body: JSON.stringify({
        bucket: getEffectiveBucket(),
        path: item.path,
        scope: currentScope,
      }),
    });
    var fileHandle = await dirHandle.getFileHandle(item.name, { create: true });
    var writable = await fileHandle.createWritable();
    await writable.write(base64ToBytes(payload.content || ""));
    await writable.close();
    return true;
  }

  async function syncNow() {
    var handle = await getHandle();
    if (!handle) {
      syncState.state = "no-folder";
      announce();
      if (typeof showNotification === "function") showNotification("Choose a local folder first", "info");
      return syncState;
    }
    if (!(await ensurePermission(handle))) {
      syncState.state = "permission";
      announce();
      return syncState;
    }
    syncState.state = "syncing";
    syncState.uploaded = 0;
    syncState.downloaded = 0;
    syncState.conflicts = 0;
    syncState.error = null;
    announce();

    var manifest = readManifest();
    var folderPath = currentPath || "";
    try {
      var remoteItems = (await listDrive(folderPath)).filter(function (item) { return item && !item.is_dir; });
      var remoteByName = {};
      remoteItems.forEach(function (item) { remoteByName[item.name] = item; });
      var localNames = {};

      for await (var entry of handle.values()) {
        if (entry.kind !== "file" || entry.name.startsWith(".")) continue;
        localNames[entry.name] = true;
        var file = await entry.getFile();
        if (file.size > MAX_SYNC_BYTES) continue;
        var remote = remoteByName[entry.name];
        var remotePath = folderPath ? folderPath.replace(/\/$/, "") + "/" + entry.name : entry.name;
        var key = (getEffectiveBucket() || "") + ":" + remotePath;
        var known = manifest[key] || null;
        var localChanged = !known || file.lastModified !== known.local;
        var remoteChanged = remote && (!known || Date.parse(remote.modified || 0) !== known.remote);

        if (!remote) {
          if (await uploadLocal(entry, remotePath)) syncState.uploaded++;
        } else if (localChanged && remoteChanged) {
          // Both sides moved since the last sync: keep the local work under a
          // conflict name, then take the Drive revision as the shared truth.
          var copy = await handle.getFileHandle(conflictName(entry.name, new Date()), { create: true });
          var writable = await copy.createWritable();
          await writable.write(await file.arrayBuffer());
          await writable.close();
          await downloadRemote(remote, handle);
          syncState.conflicts++;
          syncState.downloaded++;
        } else if (localChanged) {
          if (await uploadLocal(entry, remotePath)) syncState.uploaded++;
        } else if (remoteChanged) {
          if (await downloadRemote(remote, handle)) syncState.downloaded++;
        }
        var current = await entry.getFile();
        manifest[key] = {
          local: current.lastModified,
          remote: remote ? Date.parse(remote.modified || 0) : Date.now(),
        };
      }

      for (var index = 0; index < remoteItems.length; index++) {
        var item = remoteItems[index];
        if (localNames[item.name]) continue;
        if (await downloadRemote(item, handle)) syncState.downloaded++;
        var key = (getEffectiveBucket() || "") + ":" + item.path;
        manifest[key] = { local: Date.now(), remote: Date.parse(item.modified || 0) };
      }

      saveManifest(manifest);
      syncState.state = syncState.conflicts > 0 ? "conflict" : "synced";
      syncState.lastSync = new Date().toISOString();
      localStorage.setItem("gb_sync_last", syncState.lastSync);
      if (typeof showNotification === "function") {
        showNotification(
          "Sync finished: " + syncState.uploaded + " up, " + syncState.downloaded + " down" +
            (syncState.conflicts ? ", " + syncState.conflicts + " conflict(s)" : ""),
          syncState.conflicts ? "warning" : "success",
        );
      }
    } catch (error) {
      syncState.state = "error";
      syncState.error = error && error.message ? error.message : "sync failed";
    }
    announce();
    if (typeof loadFiles === "function") loadFiles(currentPath, currentBucket);
    return syncState;
  }

  function setSyncFolder(path) {
    syncState.folder = path;
    localStorage.setItem("gb_sync_folder", path);
  }

  function showSyncSettings() {
    var modal = document.getElementById("sync-settings-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "sync-settings-modal";
    modal.style.cssText = "position:fixed;top:0;left:0;width:100%;height:100%;background:rgba(15,23,42,0.85);display:flex;align-items:center;justify-content:center;z-index:9999;backdrop-filter:blur(8px);";
    var supported = supportsFolderSync();
    modal.innerHTML = '<div style="background:#1e293b;border:1px solid #334155;border-radius:12px;width:90%;max-width:460px;padding:24px;">'
      + '<h3 style="margin:0 0 16px;color:#f8fafc;">Offline Sync</h3>'
      + '<p style="color:#94a3b8;font-size:13px;margin:0 0 12px;">Chosen folder: <strong style="color:#e2e8f0;">' + (syncState.folder ? escapeHtml(syncState.folder) : "none") + '</strong></p>'
      + '<p style="color:#64748b;font-size:12px;margin:0 0 16px;">' + (supported
          ? "The folder is mirrored both ways with the currently open Drive folder. Changed files keep both copies: the Drive revision stays canonical and your local edit is saved next to it as a conflict copy."
          : "This platform does not expose a local folder picker, so folder sync is unavailable. Offline caching and queued uploads keep working.") + '</p>'
      + '<div id="gb-sync-status" style="color:#94a3b8;font-size:12px;margin-bottom:16px;">' + escapeHtml(syncState.state) + (syncState.lastSync ? " · last sync " + escapeHtml(new Date(syncState.lastSync).toLocaleString()) : "") + '</div>'
      + '<div style="display:flex;gap:10px;justify-content:flex-end;flex-wrap:wrap;">'
      + '<button type="button" data-sync-action="choose" style="background:#334155;border:none;border-radius:6px;color:#f8fafc;padding:8px 16px;cursor:pointer;"' + (supported ? "" : " disabled") + '>Choose folder</button>'
      + '<button type="button" data-sync-action="sync" style="background:#3b82f6;border:none;border-radius:6px;color:#fff;padding:8px 16px;cursor:pointer;"' + (syncState.folder ? "" : " disabled") + '>Sync now</button>'
      + '</div></div>';
    modal.addEventListener("click", async function (e) {
      if (e.target === modal) { modal.remove(); return; }
      var button = e.target.closest ? e.target.closest("[data-sync-action]") : null;
      if (!button) return;
      var action = button.getAttribute("data-sync-action");
      if (action === "choose") {
        await pickFolder();
        modal.remove();
        showSyncSettings();
      } else if (action === "sync") {
        button.disabled = true;
        button.textContent = "Syncing…";
        await syncNow();
        modal.remove();
        showSyncSettings();
      }
    });
    document.body.appendChild(modal);
    getPendingUploads().then(function (items) {
      var status = document.getElementById("gb-sync-status");
      if (status && items.length) {
        status.textContent += " · " + items.length + " upload(s) queued offline";
      }
    });
  }

  function saveSyncSettings() {
    var modal = document.getElementById("sync-settings-modal");
    if (modal) modal.remove();
    if (typeof showNotification === "function") showNotification("Sync settings saved", "success");
  }

  async function restoreFolderHandle() {
    var handle = await getHandle();
    if (handle) {
      syncState.folder = handle.name;
      announce();
    }
  }
  restoreFolderHandle();

  window.DriveOfflineSync = {
    cacheFileListing: cacheFileListing,
    getCachedListing: getCachedListing,
    addPendingUpload: addPendingUpload,
    syncPendingUploads: syncPendingUploads,
    showSyncSettings: showSyncSettings,
    saveSyncSettings: saveSyncSettings,
    setSyncFolder: setSyncFolder,
    pickFolder: pickFolder,
    syncNow: syncNow,
    supportsFolderSync: supportsFolderSync,
    getStatus: function () { return Object.assign({}, syncState); },
    getSyncFolder: function () { return syncState.folder; },
  };

  try { openDB().then(updateIndicator); } catch (e) {}
  document.addEventListener("DOMContentLoaded", updateIndicator);
})();
