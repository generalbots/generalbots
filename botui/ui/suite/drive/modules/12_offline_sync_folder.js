/* Drive Module v2.0 — 12 Offline Sync (folder): optional local OS folder
 * mirror (File System Access API). Split from 12_offline_sync.js for #1731;
 * loaded right after the core file and extends window.DriveOfflineSync. */
"use strict";

(function () {
  var API = window.DriveOfflineSync;
  var openDB = API._openDB;
  var syncState = API._state;
  var announce = API._announce;
  var getPendingUploads = API._getPendingUploads;
  var STORE_HANDLES = API._STORE_HANDLES;

  var MANIFEST_KEY = "gb_sync_manifest_v1";
  var MAX_SYNC_BYTES = 20 * 1024 * 1024;

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
    try { return gbStoreGet(MANIFEST_KEY, {}); } catch (e) { return {}; }
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

  API.showSyncSettings = showSyncSettings;
  API.saveSyncSettings = saveSyncSettings;
  API.setSyncFolder = setSyncFolder;
  API.pickFolder = pickFolder;
  API.syncNow = syncNow;
  API.supportsFolderSync = supportsFolderSync;
  // GB7-22 — exposed for its unit test.
  API.conflictName = conflictName;

  restoreFolderHandle();
})();
