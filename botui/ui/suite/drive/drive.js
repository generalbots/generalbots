/* Drive Module v1.0 - Full API Integration */

(function () {
  "use strict";

  const API_BASE = "/api/files";

  let currentBucket = "";
  let currentPath = "";
  let currentScope = sessionStorage.getItem("drive-scope") || "user";
  let availableBuckets = [];
  let selectedFiles = new Set();
  let viewMode = "list";
  let clipboardFiles = [];
  let clipboardOperation = null;
  let retryCount = 0;
  const MAX_RETRIES = 3;
  const RETRY_DELAYS = [1000, 3000, 10000]; // Exponential backoff: 1s, 3s, 10s

  function escapeHtml(str) {
    if (!str) return "";
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function escapeJs(str) {
    if (!str) return "";
    return String(str)
      .replace(/\\/g, "\\\\")
      .replace(/'/g, "\\'")
      .replace(/"/g, '\\"');
  }

  function formatFileSize(bytes) {
    if (!bytes || bytes === 0) return "0 B";
    const units = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + " " + units[i];
  }

  function formatDate(dateStr) {
    if (!dateStr) return "";
    const d = new Date(dateStr);
    return d.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }

  function getFileTypeClass(filename) {
    const ext = (filename || "").split(".").pop().toLowerCase();
    const types = {
      document: ["doc", "docx", "pdf", "txt", "rtf", "odt"],
      image: ["jpg", "jpeg", "png", "gif", "svg", "webp", "bmp"],
      video: ["mp4", "avi", "mov", "mkv", "webm"],
      audio: ["mp3", "wav", "ogg", "flac", "aac"],
      archive: ["zip", "rar", "7z", "tar", "gz"],
      code: [
        "js",
        "ts",
        "py",
        "rs",
        "go",
        "java",
        "c",
        "cpp",
        "h",
        "html",
        "css",
        "json",
        "xml",
      ],
    };
    for (const [type, exts] of Object.entries(types)) {
      if (exts.includes(ext)) return type;
    }
    return "file";
  }

  function getFolderIcon() {
    return '<svg width="20" height="20" viewBox="0 0 24 24" fill="#5f6368" stroke="none"><path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg>';
  }

  function getFileIcon(filename) {
    const ext = (filename || "").split(".").pop().toLowerCase();
    const colors = {
      pdf: "#ea4335",
      doc: "#4285f4",
      docx: "#4285f4",
      xls: "#0f9d58",
      xlsx: "#0f9d58",
      ppt: "#fbbc04",
      pptx: "#fbbc04",
    };
    const color = colors[ext] || "#5f6368";
    return `<svg width="20" height="20" viewBox="0 0 24 24" fill="${color}" stroke="none"><path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z"/></svg>`;
  }

  function showNotification(message, type) {
    const existing = document.querySelector(".drive-notification");
    if (existing) existing.remove();

    const notification = document.createElement("div");
    notification.className = `drive-notification notification-${type || "info"}`;
    notification.textContent = message;
    notification.style.cssText =
      "position:fixed;bottom:20px;right:20px;padding:12px 20px;border-radius:8px;background:#333;color:#fff;z-index:9999;animation:slideIn 0.3s ease;";

    if (type === "error") notification.style.background = "#ef4444";
    if (type === "success") notification.style.background = "#22c55e";
    if (type === "warning") notification.style.background = "#f59e0b";

    document.body.appendChild(notification);
    setTimeout(() => notification.remove(), 4000);
  }

  async function init() {
    bindNavigation();
    bindViewToggle();
    bindDragAndDrop();
    bindContextMenu();
    bindKeyboardShortcuts();
    bindUploadButton();
    bindNewFolderButton();
    bindSearchInput();
    bindScopeTabs();

    await discoverBuckets();
    loadStorageInfo();
    loadFiles();
  }

  async function discoverBuckets() {
    try {
      const buckets = await apiRequest("/buckets");
      availableBuckets = buckets || [];
      retryCount = 0; // Reset on success

      const gbai = availableBuckets.find((b) => b.is_gbai);
      if (gbai) {
        currentBucket = gbai.name;
      } else if (availableBuckets.length > 0) {
        currentBucket = availableBuckets[0].name;
      }

      updateBucketSelector();

      if (!currentBucket) {
        const content =
          document.getElementById("drive-content") ||
          document.getElementById("file-grid");
        if (content) {
          content.innerHTML = `<div class="empty-state"><svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg><h3>No drive storage found</h3><p>Please contact your administrator to set up storage.</p></div>`;
        }
      }
    } catch (err) {
      console.error("Failed to discover buckets:", err);
      const content =
        document.getElementById("drive-content") ||
        document.getElementById("file-grid");
      if (content) {
        const canRetry = retryCount < MAX_RETRIES;
        const retryMsg = canRetry
          ? `<button class="btn-primary" onclick="DriveModule.retryWithBackoff()">Retry</button>`
          : `<p class="text-muted">Max retries reached. Please refresh the page.</p>`;
        content.innerHTML = `<div class="empty-state"><svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg><h3>Drive connection error</h3><p>${escapeHtml(err.message)}</p>${retryMsg}</div>`;
      }
    }
  }

  async function retryWithBackoff() {
    if (retryCount >= MAX_RETRIES) {
      showNotification(
        "Max retries reached. Please refresh the page.",
        "error",
      );
      return;
    }

    const delay =
      RETRY_DELAYS[retryCount] || RETRY_DELAYS[RETRY_DELAYS.length - 1];
    retryCount++;

    const content =
      document.getElementById("drive-content") ||
      document.getElementById("file-grid");
    if (content) {
      content.innerHTML = `<div class="empty-state"><div class="spinner"></div><p>Retrying in ${delay / 1000}s... (attempt ${retryCount}/${MAX_RETRIES})</p></div>`;
    }

    await new Promise((resolve) => setTimeout(resolve, delay));
    await init();
  }

  function updateBucketSelector() {
    const selector = document.getElementById("bucket-selector");
    if (!selector) return;

    if (availableBuckets.length <= 1) {
      selector.style.display = "none";
      return;
    }

    selector.style.display = "block";
    selector.innerHTML = availableBuckets
      .map(
        (b) =>
          `<option value="${escapeHtml(b.name)}" ${b.name === currentBucket ? "selected" : ""}>${escapeHtml(b.is_gbai ? b.name.replace(".gbai", "") : b.name)}</option>`,
      )
      .join("");

    selector.removeEventListener("change", handleBucketChange);
    selector.addEventListener("change", handleBucketChange);
  }

  function handleBucketChange(e) {
    currentBucket = e.target.value;
    currentPath = "";
    loadFiles();
  }

  async function apiRequest(endpoint, options = {}) {
    const url = `${API_BASE}${endpoint}`;

    // Use global ApiClient if available for automatic auth token injection
    if (window.ApiClient) {
      try {
        return await window.ApiClient.request(url, options);
      } catch (err) {
        console.error(`API Error [${endpoint}]:`, err);
        throw err;
      }
    }

    // Fallback if ApiClient not loaded
    const defaultHeaders = { "Content-Type": "application/json" };

    // Try to get auth token from storage
    const token =
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token");
    if (token) {
      defaultHeaders["Authorization"] = `Bearer ${token}`;
    }

    try {
      const response = await fetch(url, {
        headers: { ...defaultHeaders, ...options.headers },
        ...options,
      });

      if (!response.ok) {
        const error = await response
          .json()
          .catch(() => ({ error: response.statusText }));
        throw new Error(error.error || "Request failed");
      }

      return response.json();
    } catch (err) {
      console.error(`API Error [${endpoint}]:`, err);
      throw err;
    }
  }

  async function loadFiles(path, bucket) {
    if (path !== undefined) currentPath = path;
    if (bucket !== undefined) currentBucket = bucket;

    if (!currentBucket) {
      await discoverBuckets();
      if (!currentBucket) return;
    }

    const content =
      document.getElementById("drive-content") ||
      document.getElementById("file-grid");
    if (!content) return;

    content.innerHTML =
      '<div class="loading-state"><div class="spinner"></div><p>Loading files...</p></div>';
    updateBreadcrumb();

    try {
      const params = new URLSearchParams();
      if (currentBucket) params.set("bucket", currentBucket);
      if (currentPath) params.set("path", currentPath);
      params.set("scope", currentScope);

      const files = await apiRequest(`/list?${params.toString()}`);
      renderFiles(files);
    } catch (err) {
      content.innerHTML = `<div class="empty-state"><h3>Failed to load files</h3><p>${escapeHtml(err.message)}</p><button class="btn-primary" onclick="DriveModule.loadFiles()">Retry</button></div>`;
    }
  }

  async function loadStorageInfo() {
    try {
      const quota = await apiRequest("/quota");
      const usedEl = document.getElementById("storage-used");
      const fillEl = document.getElementById("storage-fill");
      const detailEl = document.getElementById("storage-detail");

      if (usedEl)
        usedEl.textContent = `${formatFileSize(quota.used_bytes)} of ${formatFileSize(quota.total_bytes)}`;
      if (fillEl) fillEl.style.width = `${quota.percentage_used || 0}%`;
      if (detailEl)
        detailEl.textContent = `${formatFileSize(quota.available_bytes)} available`;
    } catch (err) {
      // Silently fail - don't retry storage info, it's not critical
      console.error("Failed to load storage info:", err);
    }
  }
  function renderFiles(files) {
    const content =
      document.getElementById("drive-content") ||
      document.getElementById("file-grid");
    if (!content) return;

    if (!files || files.length === 0) {
      content.innerHTML = `<div class="empty-state"><svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg><h3>This folder is empty</h3><p>Upload files or create a new folder to get started</p></div>`;
      return;
    }

    const folders = files
      .filter((f) => f.is_dir)
      .sort((a, b) => a.name.localeCompare(b.name));
    const regularFiles = files
      .filter((f) => !f.is_dir)
      .sort((a, b) => a.name.localeCompare(b.name));
    const sorted = [...folders, ...regularFiles];

    if (viewMode === "grid") {
      content.innerHTML = `<div class="file-grid">${sorted.map((f) => renderFileCard(f)).join("")}</div>`;
    } else {
      content.innerHTML = `<div class="file-list"><div class="file-list-header"><div class="file-col file-name-col">Name</div><div class="file-col file-modified-col">Modified</div><div class="file-col file-size-col">Size</div><div class="file-col file-actions-col"></div></div>${sorted.map((f) => renderFileRow(f)).join("")}</div>`;
    }

    bindFileEvents();
    updateSelectionBar();
  }

  function renderFileCard(file) {
    const iconClass = file.is_dir ? "folder" : getFileTypeClass(file.name);
    const iconSvg = file.is_dir ? getFolderIcon() : getFileIcon(file.name);
    const sizeText = file.is_dir ? "" : formatFileSize(file.size);
    const checked = selectedFiles.has(file.path) ? "checked" : "";
    const selected = selectedFiles.has(file.path) ? "selected" : "";

    const kbTag = file.is_kb ? `<span class="kb-tag ${file.is_public ? "public" : "private"}" title="${file.is_public ? "Public KB" : "Restricted KB"}">KB</span>` : "";
    return `<div class="file-card ${selected}" data-path="${escapeHtml(file.path)}" data-name="${escapeHtml(file.name)}" data-type="${file.is_dir ? "folder" : "file"}" data-size="${file.size || 0}"><input type="checkbox" class="file-checkbox" ${checked} onchange="DriveModule.toggleSelection('${escapeJs(file.path)}')"><div class="file-card-preview ${iconClass}">${iconSvg}${kbTag}</div><div class="file-card-info"><div class="file-card-name" title="${escapeHtml(file.name)}">${escapeHtml(file.name)}</div><div class="file-card-meta">${sizeText}</div></div></div>`;
  }

  function renderFileRow(file) {
    const iconSvg = file.is_dir ? getFolderIcon() : getFileIcon(file.name);
    const sizeText = file.is_dir ? "—" : formatFileSize(file.size);
    const modifiedText = file.modified ? formatDate(file.modified) : "—";
    const checked = selectedFiles.has(file.path) ? "checked" : "";
    const selected = selectedFiles.has(file.path) ? "selected" : "";

    const downloadIcon = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>`;
    const moreIcon = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="1"></circle><circle cx="19" cy="12" r="1"></circle><circle cx="5" cy="12" r="1"></circle></svg>`;

    const downloadBtn = !file.is_dir
      ? `<button class="btn-icon-sm" title="Download" onclick="event.stopPropagation(); DriveModule.downloadFile('${escapeJs(file.path)}')">${downloadIcon}</button>`
      : "";

    const kbTag = file.is_kb ? `<span class="kb-tag ${file.is_public ? "public" : "private"}" title="${file.is_public ? "Public KB" : "Restricted KB"}">${file.is_public ? "🔓" : "🔒"} KB</span>` : "";

    return `<div class="drive-file-item ${file.is_dir ? "folder" : ""} ${selected}" data-path="${escapeHtml(file.path)}" data-name="${escapeHtml(file.name)}" data-type="${file.is_dir ? "folder" : "file"}" data-size="${file.size || 0}"><div class="file-col file-name-col"><input type="checkbox" class="file-checkbox" ${checked} onclick="event.stopPropagation()" onchange="DriveModule.toggleSelection('${escapeJs(file.path)}')">${iconSvg}<span>${escapeHtml(file.name)}</span>${kbTag}</div><div class="file-col file-modified-col">${modifiedText}</div><div class="file-col file-size-col">${sizeText}</div><div class="file-col file-actions-col">${downloadBtn}<button class="btn-icon-sm" title="More" onclick="event.stopPropagation(); DriveModule.showContextMenuFor(event, '${escapeJs(file.path)}')">${moreIcon}</button></div></div>`;
  }

  function bindFileEvents() {
    document.querySelectorAll(".file-card, .drive-file-item").forEach((el) => {
      el.addEventListener("click", function (e) {
        if (
          e.target.closest(".file-checkbox") ||
          e.target.closest(".btn-icon-sm")
        )
          return;
        const path = this.dataset.path;
        const type = this.dataset.type;

        if (e.ctrlKey || e.metaKey) {
          toggleSelection(path);
        } else {
          // Single click just selects, doesn't open
          toggleSelection(path);
        }
      });

      el.addEventListener("dblclick", function (e) {
        if (e.target.closest(".file-checkbox")) return;
        const path = this.dataset.path;
        const type = this.dataset.type;
        if (type === "folder") {
          loadFiles(path, currentBucket);
        } else {
          openFile(path);
        }
      });
    });
  }

  function toggleSelection(path) {
    if (selectedFiles.has(path)) selectedFiles.delete(path);
    else selectedFiles.add(path);

    const el = document.querySelector(`[data-path="${CSS.escape(path)}"]`);
    if (el) {
      el.classList.toggle("selected", selectedFiles.has(path));
      const checkbox = el.querySelector(".file-checkbox");
      if (checkbox) checkbox.checked = selectedFiles.has(path);
    }
    updateSelectionBar();
  }

  function selectAll() {
    document.querySelectorAll(".file-card, .drive-file-item").forEach((el) => {
      selectedFiles.add(el.dataset.path);
      el.classList.add("selected");
      const checkbox = el.querySelector(".file-checkbox");
      if (checkbox) checkbox.checked = true;
    });
    updateSelectionBar();
  }

  function clearSelection() {
    selectedFiles.clear();
    document
      .querySelectorAll(".file-card.selected, .drive-file-item.selected")
      .forEach((el) => {
        el.classList.remove("selected");
        const checkbox = el.querySelector(".file-checkbox");
        if (checkbox) checkbox.checked = false;
      });
    updateSelectionBar();
  }

  function updateSelectionBar() {
    const bar = document.getElementById("selection-bar");
    const countEl = document.getElementById("selected-count");
    if (bar) bar.style.display = selectedFiles.size > 0 ? "flex" : "none";
    if (countEl) countEl.textContent = selectedFiles.size;
  }

  function updateBreadcrumb() {
    const breadcrumb = document.querySelector(".breadcrumb, .drive-breadcrumb");
    if (!breadcrumb) return;

    const parts = currentPath ? currentPath.split("/").filter(Boolean) : [];
    const scopeLabel = currentScope === "bot" ? "Bot Files" : "My Files";
    let html = `<button class="breadcrumb-item" onclick="DriveModule.loadFiles('', '${currentBucket}')">${escapeHtml(scopeLabel)}</button>`;

    let cumulativePath = "";
    parts.forEach((part, idx) => {
      cumulativePath += (cumulativePath ? "/" : "") + part;
      const isLast = idx === parts.length - 1;
      html += `<span class="breadcrumb-sep">/</span>`;
      html += isLast
        ? `<span class="breadcrumb-current">${escapeHtml(part)}</span>`
        : `<button class="breadcrumb-item" onclick="DriveModule.loadFiles('${escapeJs(cumulativePath)}', '${currentBucket}')">${escapeHtml(part)}</button>`;
    });

    breadcrumb.innerHTML = html;
  }
  function bindNavigation() {
    document.querySelectorAll(".drive-nav-item, .nav-item").forEach((item) => {
      item.addEventListener("click", function () {
        document
          .querySelectorAll(".drive-nav-item, .nav-item")
          .forEach((i) => i.classList.remove("active"));
        this.classList.add("active");

        const view = this.dataset.view || this.dataset.filter;
        if (view === "my-drive" || !view) loadFiles("", currentBucket);
        else if (view === "recent") loadRecentFiles();
        else if (view === "starred" || view === "favorite") loadStarredFiles();
        else if (view === "shared") loadSharedFiles();
        else if (view === "trash") loadTrashFiles();
      });
    });
  }

  async function loadRecentFiles() {
    const content =
      document.getElementById("drive-content") ||
      document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML =
      '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    try {
      const params = new URLSearchParams();
      params.set("scope", currentScope);
      if (currentBucket) params.set("bucket", currentBucket);
      const files = await apiRequest(`/recent?${params.toString()}`);
      renderFiles(files);
    } catch (err) {
      content.innerHTML = `<div class="empty-state"><h3>No recent files</h3></div>`;
    }
  }

  async function loadStarredFiles() {
    const content =
      document.getElementById("drive-content") ||
      document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML =
      '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    try {
      const params = new URLSearchParams();
      params.set("scope", currentScope);
      const files = await apiRequest(`/favorite?${params.toString()}`);
      renderFiles(files);
    } catch (err) {
      content.innerHTML = `<div class="empty-state"><h3>No starred files</h3></div>`;
    }
  }

  async function loadSharedFiles() {
    const content =
      document.getElementById("drive-content") ||
      document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML =
      '<div class="loading-state"><div class="spinner"></div><p>Loading...</p></div>';
    try {
      const params = new URLSearchParams();
      params.set("scope", currentScope);
      const files = await apiRequest(`/shared?${params.toString()}`);
      renderFiles(files);
    } catch (err) {
      content.innerHTML = `<div class="empty-state"><h3>No shared files</h3></div>`;
    }
  }

  async function loadTrashFiles() {
    const content =
      document.getElementById("drive-content") ||
      document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = `<div class="empty-state"><h3>Trash is empty</h3></div>`;
  }

  function bindViewToggle() {
    document.querySelectorAll(".view-toggle, .view-btn").forEach((btn) => {
      btn.addEventListener("click", function () {
        const view = this.dataset.view;
        if (view) {
          viewMode = view;
          document
            .querySelectorAll(".view-toggle, .view-btn")
            .forEach((b) => b.classList.remove("active"));
          this.classList.add("active");
          loadFiles(currentPath, currentBucket);
        }
      });
    });
  }

  function bindUploadButton() {
    const uploadBtn = document.getElementById("upload-btn");
    if (uploadBtn) uploadBtn.addEventListener("click", triggerUpload);
    window.uploadFile = triggerUpload;

    let fileInput = document.getElementById("file-input");
    if (!fileInput) {
      fileInput = document.createElement("input");
      fileInput.type = "file";
      fileInput.id = "file-input";
      fileInput.multiple = true;
      fileInput.style.display = "none";
      document.body.appendChild(fileInput);
    }
    fileInput.addEventListener("change", handleFileInputChange);
  }

  function triggerUpload() {
    const input = document.getElementById("file-input");
    if (input) input.click();
  }

  function handleFileInputChange(e) {
    const files = e.target.files;
    if (files && files.length > 0) uploadFiles(Array.from(files));
    e.target.value = "";
  }

  async function uploadFiles(files) {
    showNotification(`Uploading ${files.length} file(s)...`, "info");

    let uploaded = 0;
    let failed = 0;

    for (const file of files) {
      try {
        const content = await readFileAsBase64(file);
        const filePath = currentPath
          ? `${currentPath}/${file.name}`
          : file.name;

        await apiRequest("/write", {
          method: "POST",
          body: JSON.stringify({
            bucket: currentBucket,
            path: filePath,
            content: content,
            scope: currentScope,
          }),
        });
        uploaded++;
      } catch (err) {
        console.error("Upload error:", err);
        failed++;
      }
    }

    if (failed === 0)
      showNotification(`Uploaded ${uploaded} file(s)`, "success");
    else showNotification(`Uploaded ${uploaded}, ${failed} failed`, "warning");

    loadFiles(currentPath, currentBucket);
    loadStorageInfo();
  }

  function readFileAsBase64(file) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const base64 = reader.result.split(",")[1] || reader.result;
        resolve(base64);
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }

  function bindDragAndDrop() {
    const container = document.querySelector(".drive-container, .drive-main");
    if (!container) return;

    ["dragenter", "dragover", "dragleave", "drop"].forEach((eventName) => {
      container.addEventListener(eventName, (e) => {
        e.preventDefault();
        e.stopPropagation();
      });
    });

    container.addEventListener("dragenter", () => {
      container.classList.add("drag-active");
      const overlay = document.getElementById("drop-overlay");
      if (overlay) overlay.classList.add("visible");
    });

    container.addEventListener("dragleave", (e) => {
      if (!container.contains(e.relatedTarget)) {
        container.classList.remove("drag-active");
        const overlay = document.getElementById("drop-overlay");
        if (overlay) overlay.classList.remove("visible");
      }
    });

    container.addEventListener("drop", (e) => {
      container.classList.remove("drag-active");
      const overlay = document.getElementById("drop-overlay");
      if (overlay) overlay.classList.remove("visible");
      const files = e.dataTransfer.files;
      if (files && files.length > 0) uploadFiles(Array.from(files));
    });
  }
  function bindContextMenu() {
    document.addEventListener("contextmenu", (e) => {
      const fileEl = e.target.closest(".file-card, .drive-file-item");
      if (fileEl) {
        e.preventDefault();
        showContextMenu(
          e.clientX,
          e.clientY,
          fileEl.dataset.path,
          fileEl.dataset.type,
        );
      }
    });

    document.addEventListener("click", (e) => {
      const menu = document.getElementById("context-menu");
      if (menu && !menu.contains(e.target)) {
        menu.classList.add("hidden");
        menu.style.display = "none";
      }
    });
  }

  function showContextMenu(x, y, path, type) {
    let menu = document.getElementById("context-menu");
    if (!menu) {
      menu = document.createElement("div");
      menu.id = "context-menu";
      menu.className = "context-menu";
      document.body.appendChild(menu);
    }

    const isFolder = type === "folder";
    const ep = escapeJs(path);

    const icons = {
      open: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>`,
      download: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>`,
      edit: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>`,
      copy: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>`,
      cut: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="6" cy="6" r="3"></circle><circle cx="6" cy="18" r="3"></circle><line x1="20" y1="4" x2="8.12" y2="15.88"></line><line x1="14.47" y1="14.48" x2="20" y2="20"></line><line x1="8.12" y1="8.12" x2="12" y2="12"></line></svg>`,
      rename: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path></svg>`,
      delete: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>`,
    };

    const hideMenu = `document.getElementById('context-menu').style.display='none';`;

    menu.innerHTML = `
      ${
        isFolder
          ? `<div class="context-menu-item" onclick="${hideMenu}DriveModule.loadFiles('${ep}', '${currentBucket}')">${icons.open}<span>Open</span></div>`
          : `<div class="context-menu-item" onclick="${hideMenu}DriveModule.openFile('${ep}')">${icons.open}<span>Open</span></div>
             <div class="context-menu-item" onclick="${hideMenu}DriveModule.previewFile('${ep}')">${icons.open}<span>Preview</span></div>
             <div class="context-menu-item" onclick="${hideMenu}DriveModule.downloadFile('${ep}')">${icons.download}<span>Download</span></div>
             <div class="context-menu-item" onclick="${hideMenu}DriveModule.shareFile('${ep}')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="margin-right:8px;"><path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"></path><polyline points="16 6 12 2 8 6"></polyline><line x1="12" y1="2" x2="12" y2="15"></line></svg><span>Share</span></div>`
      }
      <div class="context-menu-divider"></div>
      <div class="context-menu-item" onclick="${hideMenu}DriveModule.copyToClipboard('${ep}')">${icons.copy}<span>Copy</span></div>
      <div class="context-menu-item" onclick="${hideMenu}DriveModule.cutToClipboard('${ep}')">${icons.cut}<span>Cut</span></div>
      <div class="context-menu-item" onclick="${hideMenu}DriveModule.renameItem('${ep}')">${icons.rename}<span>Rename</span></div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item danger" onclick="${hideMenu}DriveModule.deleteItem('${ep}')">${icons.delete}<span>Delete</span></div>
    `;

    menu.style.display = "block";
    menu.classList.remove("hidden");

    const rect = menu.getBoundingClientRect();
    menu.style.left =
      (x + rect.width > window.innerWidth ? x - rect.width : x) + "px";
    menu.style.top =
      (y + rect.height > window.innerHeight ? y - rect.height : y) + "px";
  }

  function showContextMenuFor(event, path) {
    const el = document.querySelector(`[data-path="${CSS.escape(path)}"]`);
    const type = el ? el.dataset.type : "file";
    showContextMenu(event.clientX, event.clientY, path, type);
  }

  function bindKeyboardShortcuts() {
    document.addEventListener("keydown", (e) => {
      if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA")
        return;

      if (e.key === "Delete" && selectedFiles.size > 0) {
        e.preventDefault();
        deleteSelected();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "a") {
        e.preventDefault();
        selectAll();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "c" && selectedFiles.size > 0) {
        e.preventDefault();
        copySelected();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "x" && selectedFiles.size > 0) {
        e.preventDefault();
        cutSelected();
      }
      if (
        (e.ctrlKey || e.metaKey) &&
        e.key === "v" &&
        clipboardFiles.length > 0
      ) {
        e.preventDefault();
        pasteFiles();
      }
      if (e.key === "Escape") {
        clearSelection();
        const menu = document.getElementById("context-menu");
        if (menu) menu.style.display = "none";
      }
      if (e.key === "Backspace" && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        navigateUp();
      }
      if (e.key === "F2" && selectedFiles.size === 1) {
        e.preventDefault();
        renameItem(Array.from(selectedFiles)[0]);
      }
    });
  }

  function navigateUp() {
    if (!currentPath) return;
    const parts = currentPath.split("/").filter(Boolean);
    parts.pop();
    loadFiles(parts.join("/"), currentBucket);
  }

  function bindNewFolderButton() {
    const btn = document.getElementById("new-folder-btn");
    if (btn) btn.addEventListener("click", createFolder);
    window.createFolder = createFolder;
  }

  async function createFolder() {
    const name = prompt("Enter folder name:");
    if (!name || !name.trim()) return;

    try {
      await apiRequest("/createFolder", {
        method: "POST",
        body: JSON.stringify({
          bucket: currentBucket,
          path: currentPath,
          name: name.trim(),
          scope: currentScope,
        }),
      });
      showNotification(`Folder "${name}" created`, "success");
      loadFiles(currentPath, currentBucket);
    } catch (err) {
      showNotification(`Failed to create folder: ${err.message}`, "error");
    }
  }

  function bindSearchInput() {
    const searchInput = document.querySelector(
      ".search-box input, #search-input",
    );
    if (!searchInput) return;

    let debounceTimer;
    searchInput.addEventListener("input", (e) => {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        const query = e.target.value.trim();
        if (query) searchFiles(query);
        else loadFiles(currentPath, currentBucket);
      }, 300);
    });
  }

  function bindScopeTabs() {
    document.querySelectorAll(".scope-tab").forEach((tab) => {
      tab.addEventListener("click", function () {
        document.querySelectorAll(".scope-tab").forEach((t) => t.classList.remove("active"));
        this.classList.add("active");
        currentScope = this.dataset.scope;
        sessionStorage.setItem("drive-scope", currentScope);
        currentPath = "";
        selectedFiles.clear();
        updateSelectionBar();
        loadFiles("", currentBucket);
      });
    });

    const savedScope = sessionStorage.getItem("drive-scope") || "user";
    currentScope = savedScope;
    document.querySelectorAll(".scope-tab").forEach((t) => {
      t.classList.toggle("active", t.dataset.scope === savedScope);
    });
  }

  async function searchFiles(query) {
    const content =
      document.getElementById("drive-content") ||
      document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML =
      '<div class="loading-state"><div class="spinner"></div><p>Searching...</p></div>';

    try {
      const params = new URLSearchParams();
      params.set("query", query);
      if (currentBucket) params.set("bucket", currentBucket);
      params.set("scope", currentScope);
      const files = await apiRequest(`/search?${params.toString()}`);
      renderFiles(files);
    } catch (err) {
      content.innerHTML = `<div class="empty-state"><h3>Search failed</h3></div>`;
    }
  }
  async function downloadFile(path) {
    try {
      const fileName = path.split("/").pop() || "download";
      const token =
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token");
      const headers = { "Content-Type": "application/json" };
      if (token) headers["Authorization"] = "Bearer " + token;

      const res = await fetch(API_BASE + "/download-binary", {
        method: "POST",
        headers: headers,
        body: JSON.stringify({ bucket: currentBucket, path: path, scope: currentScope })
      });
      if (!res.ok) {
        throw new Error("HTTP " + res.status);
      }
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      showNotification("Downloaded " + fileName, "success");
    } catch (err) {
      showNotification("Download failed: " + err.message, "error");
    }
  }

  function showPreviewModal(fileName, ext, blob) {
    let modal = document.getElementById("preview-modal");
    if (modal) modal.remove();

    modal = document.createElement("div");
    modal.id = "preview-modal";
    modal.style.cssText = `
      position: fixed; top: 0; left: 0; width: 100%; height: 100%;
      background: rgba(15, 23, 42, 0.85); display: flex; align-items: center;
      justify-content: center; z-index: 9999; backdrop-filter: blur(8px);
    `;

    const container = document.createElement("div");
    container.style.cssText = `
      background: #1e293b; border: 1px solid #334155; border-radius: 12px;
      width: 80%; max-width: 900px; max-height: 90vh; display: flex;
      flex-direction: column; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);
      overflow: hidden;
    `;

    const header = document.createElement("div");
    header.style.cssText = `
      display: flex; align-items: center; justify-content: space-between;
      padding: 16px 24px; border-bottom: 1px solid #334155;
    `;
    header.innerHTML = `<h3 style="margin:0; color:#f8fafc; font-size:1.125rem;">Preview: ${escapeHtml(fileName)}</h3>`;

    const closeBtn = document.createElement("button");
    closeBtn.textContent = "×";
    closeBtn.style.cssText = `
      background: none; border: none; color: #94a3b8; font-size: 28px;
      cursor: pointer; line-height: 1; padding: 0; margin: 0;
    `;
    closeBtn.onclick = () => {
      modal.remove();
      URL.revokeObjectURL(objectUrl);
    };
    header.appendChild(closeBtn);

    const body = document.createElement("div");
    body.style.cssText = `
      padding: 24px; overflow-y: auto; flex-grow: 1; display: flex;
      align-items: center; justify-content: center; background: #0f172a;
    `;

    const objectUrl = URL.createObjectURL(blob);
    let previewEl;

    if (["png", "jpg", "jpeg", "gif", "webp"].includes(ext)) {
      previewEl = document.createElement("img");
      previewEl.src = objectUrl;
      previewEl.style.cssText = "max-width:100%; max-height:70vh; object-fit:contain; border-radius:4px;";
    } else if (ext === "pdf") {
      previewEl = document.createElement("iframe");
      previewEl.src = objectUrl;
      previewEl.style.cssText = "width:100%; height:70vh; border:none; border-radius:4px;";
    } else if (["mp4", "webm"].includes(ext)) {
      previewEl = document.createElement("video");
      previewEl.src = objectUrl;
      previewEl.controls = true;
      previewEl.autoplay = true;
      previewEl.style.cssText = "max-width:100%; max-height:70vh; border-radius:4px;";
    } else if (["mp3", "wav", "ogg"].includes(ext)) {
      previewEl = document.createElement("audio");
      previewEl.src = objectUrl;
      previewEl.controls = true;
      previewEl.autoplay = true;
      previewEl.style.cssText = "width:100%; max-width:500px;";
    } else {
      previewEl = document.createElement("pre");
      previewEl.style.cssText = "color:#f8fafc; font-family:monospace; font-size:14px; white-space:pre-wrap; width:100%; max-height:70vh; overflow-y:auto; margin:0;";
      blob.text().then(t => {
        previewEl.textContent = t.substring(0, 10000);
      }).catch(() => {
        previewEl.textContent = "Preview not available for this file type.";
      });
    }

    body.appendChild(previewEl);
    container.appendChild(header);
    container.appendChild(body);
    modal.appendChild(container);
    document.body.appendChild(modal);
  }

  async function previewFile(path) {
    try {
      showNotification("Loading preview...", "info");
      const response = await apiRequest("/download", {
        method: "POST",
        body: JSON.stringify({ bucket: currentBucket, path: path, scope: currentScope }),
      });

      const content = response.content;
      const fileName = path.split("/").pop() || "preview";
      const ext = fileName.split('.').pop().toLowerCase();
      
      const byteCharacters = atob(content);
      const byteNumbers = new Array(byteCharacters.length);
      for (let i = 0; i < byteCharacters.length; i++) {
        byteNumbers[i] = byteCharacters.charCodeAt(i);
      }
      const byteArray = new Uint8Array(byteNumbers);
      
      let mimeType = "application/octet-stream";
      if (ext === "pdf") mimeType = "application/pdf";
      else if (ext === "png") mimeType = "image/png";
      else if (ext === "jpg" || ext === "jpeg") mimeType = "image/jpeg";
      else if (ext === "gif") mimeType = "image/gif";
      else if (ext === "mp3") mimeType = "audio/mpeg";
      else if (ext === "wav") mimeType = "audio/wav";
      else if (ext === "mp4") mimeType = "video/mp4";
      
      const blob = new Blob([byteArray], { type: mimeType });
      showPreviewModal(fileName, ext, blob);
    } catch (err) {
      showNotification(`Preview failed: ${err.message}`, "error");
    }
  }

  function shareFile(path) {
    const fileName = path.split("/").pop() || "file";
    const shareUrl = `${window.location.origin}/api/files/download?bucket=${encodeURIComponent(currentBucket)}&path=${encodeURIComponent(path)}&scope=${currentScope}`;

    let modal = document.getElementById("share-modal");
    if (modal) modal.remove();

    modal = document.createElement("div");
    modal.id = "share-modal";
    modal.style.cssText = `
      position: fixed; top: 0; left: 0; width: 100%; height: 100%;
      background: rgba(15, 23, 42, 0.85); display: flex; align-items: center;
      justify-content: center; z-index: 9999; backdrop-filter: blur(8px);
    `;

    const container = document.createElement("div");
    container.style.cssText = `
      background: #1e293b; border: 1px solid #334155; border-radius: 12px;
      width: 90%; max-width: 500px; padding: 24px; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);
    `;

    container.innerHTML = `
      <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:20px;">
        <h3 style="margin:0; color:#f8fafc; font-size:1.25rem;">Share "${escapeHtml(fileName)}"</h3>
        <button onclick="document.getElementById('share-modal').remove()" style="background:none; border:none; color:#94a3b8; font-size:24px; cursor:pointer;">×</button>
      </div>
      <p style="color:#94a3b8; font-size:14px; margin-bottom:16px;">Anyone with this link will be able to download the file directly.</p>
      <div style="display:flex; gap:8px; margin-bottom:20px;">
        <input type="text" id="share-link-input" readonly value="${shareUrl}" style="flex-grow:1; background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; padding:10px 14px; font-size:14px;" />
        <button onclick="DriveModule.copyShareLink()" style="background:#3b82f6; border:none; border-radius:6px; color:#fff; padding:0 16px; cursor:pointer; font-weight:500; font-size:14px;">Copy</button>
      </div>
      <div style="display:flex; justify-content:flex-end; gap:12px;">
        <button onclick="document.getElementById('share-modal').remove()" style="background:#334155; border:none; border-radius:6px; color:#f8fafc; padding:10px 20px; cursor:pointer; font-weight:500; font-size:14px;">Close</button>
      </div>
    `;

    modal.appendChild(container);
    document.body.appendChild(modal);
  }

  function copyShareLink() {
    const input = document.getElementById("share-link-input");
    if (input) {
      input.select();
      document.execCommand("copy");
      showNotification("Share link copied to clipboard!", "success");
    }
  }

  async function openFile(path) {
    try {
      const response = await apiRequest("/open", {
        method: "POST",
        body: JSON.stringify({ bucket: currentBucket, path: path, scope: currentScope }),
      });

      const { app, url } = response;

      if (window.htmx) {
        htmx.ajax("GET", url, {
          target: "#main-content",
          swap: "innerHTML",
        });
        window.history.pushState(
          {},
          "",
          `/#${app}?bucket=${encodeURIComponent(currentBucket)}&path=${encodeURIComponent(path)}`,
        );
      } else {
        window.location.href = url;
      }
    } catch (err) {
      console.error("Failed to open file:", err);
      showNotification(`Failed to open file: ${err.message}`, "error");
    }
  }

  function showEditorModal(path, fileName, content) {
    console.log("showEditorModal called:", {
      path,
      fileName,
      contentLength: content?.length,
    });

    let modal = document.getElementById("editor-modal");
    if (modal) {
      console.log("Removing existing modal");
      modal.remove();
    }

    const ext = (fileName.split(".").pop() || "txt").toLowerCase();

    modal = document.createElement("div");
    modal.id = "editor-modal";
    modal.className = "modal-overlay";

    // Build modal HTML
    const headerHtml = `
      <div class="editor-header">
        <div class="editor-title">
          <span class="editor-icon">📝</span>
          <span class="editor-filename">${escapeHtml(fileName)}</span>
          <span class="editor-status" id="editor-status"></span>
        </div>
        <div class="editor-actions">
          <button class="btn-secondary" onclick="DriveModule.closeEditor()">Cancel</button>
          <button class="btn-primary" onclick="DriveModule.saveEditorContent()">
            <span>💾</span> Save
          </button>
        </div>
      </div>
    `;

    const bodyHtml = `
      <div class="editor-body">
        <textarea
          id="editor-textarea"
          class="editor-textarea"
          spellcheck="false"
          data-path="${escapeHtml(path)}"
          data-ext="${ext}"
        ></textarea>
      </div>
    `;

    const footerHtml = `
      <div class="editor-footer">
        <span class="editor-info">Line: <span id="editor-line">1</span>, Col: <span id="editor-col">1</span></span>
        <span class="editor-info">${ext.toUpperCase()}</span>
      </div>
    `;

    modal.innerHTML = `<div class="modal-content editor-modal-content">${headerHtml}${bodyHtml}${footerHtml}</div>`;

    document.body.appendChild(modal);
    console.log("Modal appended to body");

    // Set content via value property to avoid HTML escaping issues
    const textarea = document.getElementById("editor-textarea");
    if (textarea) {
      textarea.value = content || "";
      console.log("Textarea content set, length:", textarea.value.length);
      textarea.focus();
    } else {
      console.error("Textarea not found!");
      return;
    }

    textarea.addEventListener("input", () => {
      document.getElementById("editor-status").textContent = "● Modified";
    });

    textarea.addEventListener("click", updateEditorCursor);
    textarea.addEventListener("keyup", updateEditorCursor);

    textarea.addEventListener("keydown", (e) => {
      if (e.key === "s" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        saveEditorContent();
      }
      if (e.key === "Escape") {
        closeEditor();
      }
      if (e.key === "Tab") {
        e.preventDefault();
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        textarea.value =
          textarea.value.substring(0, start) +
          "  " +
          textarea.value.substring(end);
        textarea.selectionStart = textarea.selectionEnd = start + 2;
      }
    });

    modal.addEventListener("click", (e) => {
      if (e.target === modal) closeEditor();
    });
  }

  function updateEditorCursor() {
    const textarea = document.getElementById("editor-textarea");
    if (!textarea) return;

    const text = textarea.value.substring(0, textarea.selectionStart);
    const lines = text.split("\n");
    const line = lines.length;
    const col = lines[lines.length - 1].length + 1;

    document.getElementById("editor-line").textContent = line;
    document.getElementById("editor-col").textContent = col;
  }

  async function saveEditorContent() {
    const textarea = document.getElementById("editor-textarea");
    if (!textarea) return;

    const path = textarea.dataset.path;
    const content = textarea.value;
    const statusEl = document.getElementById("editor-status");

    statusEl.textContent = "Saving...";

    try {
      await apiRequest("/write", {
        method: "POST",
        body: JSON.stringify({
          bucket: currentBucket,
          path: path,
          content: content,
          scope: currentScope,
        }),
      });

      statusEl.textContent = "✓ Saved";
      showNotification("File saved successfully", "success");

      setTimeout(() => {
        if (statusEl) statusEl.textContent = "";
      }, 2000);
    } catch (err) {
      statusEl.textContent = "✗ Save failed";
      showNotification(`Failed to save: ${err.message}`, "error");
    }
  }

  function closeEditor() {
    const modal = document.getElementById("editor-modal");
    const statusEl = document.getElementById("editor-status");

    if (statusEl && statusEl.textContent.includes("Modified")) {
      if (!confirm("You have unsaved changes. Close anyway?")) {
        return;
      }
    }

    if (modal) modal.remove();
  }

  async function deleteItem(path) {
    const fileName = path.split("/").pop();
    if (!confirm(`Delete "${fileName}"?`)) return;

    try {
      await apiRequest("/delete", {
        method: "POST",
        body: JSON.stringify({ bucket: currentBucket, path: path, scope: currentScope }),
      });
      showNotification("Item deleted", "success");
      selectedFiles.delete(path);
      loadFiles(currentPath, currentBucket);
      loadStorageInfo();
    } catch (err) {
      showNotification(`Delete failed: ${err.message}`, "error");
    }
  }

  async function deleteSelected() {
    if (selectedFiles.size === 0) return;
    const count = selectedFiles.size;
    if (!confirm(`Delete ${count} item(s)?`)) return;

    let deleted = 0;
    for (const path of selectedFiles) {
      try {
        await apiRequest("/delete", {
          method: "POST",
          body: JSON.stringify({ bucket: currentBucket, path: path, scope: currentScope }),
        });
        deleted++;
      } catch (err) {
        console.error(`Failed to delete ${path}:`, err);
      }
    }

    showNotification(
      `Deleted ${deleted} of ${count} item(s)`,
      deleted === count ? "success" : "warning",
    );
    clearSelection();
    loadFiles(currentPath, currentBucket);
    loadStorageInfo();
  }

  async function renameItem(path) {
    const oldName = path.split("/").pop();
    const newName = prompt("Enter new name:", oldName);
    if (!newName || newName === oldName || !newName.trim()) return;

    const parentPath = path.substring(0, path.lastIndexOf("/"));
    const newPath = parentPath
      ? `${parentPath}/${newName.trim()}`
      : newName.trim();

    try {
      await apiRequest("/move", {
        method: "POST",
        body: JSON.stringify({
          source_bucket: currentBucket,
          source_path: path,
          dest_bucket: currentBucket,
          dest_path: newPath,
          scope: currentScope,
        }),
      });
      showNotification(`Renamed to "${newName}"`, "success");
      loadFiles(currentPath, currentBucket);
    } catch (err) {
      showNotification(`Rename failed: ${err.message}`, "error");
    }
  }

  function copyToClipboard(path) {
    clipboardFiles = [path];
    clipboardOperation = "copy";
    showNotification("Copied to clipboard", "info");
  }

  function cutToClipboard(path) {
    clipboardFiles = [path];
    clipboardOperation = "cut";
    showNotification("Cut to clipboard", "info");
  }

  function copySelected() {
    clipboardFiles = Array.from(selectedFiles);
    clipboardOperation = "copy";
    showNotification(`${clipboardFiles.length} item(s) copied`, "info");
  }

  function cutSelected() {
    clipboardFiles = Array.from(selectedFiles);
    clipboardOperation = "cut";
    showNotification(`${clipboardFiles.length} item(s) cut`, "info");
  }

  async function pasteFiles() {
    if (clipboardFiles.length === 0) return;

    const operation = clipboardOperation;
    let processed = 0;

    for (const sourcePath of clipboardFiles) {
      const fileName = sourcePath.split("/").pop();
      const destPath = currentPath ? `${currentPath}/${fileName}` : fileName;

      try {
        const endpoint = operation === "copy" ? "/copy" : "/move";
        await apiRequest(endpoint, {
          method: "POST",
          body: JSON.stringify({
            source_bucket: currentBucket,
            source_path: sourcePath,
            dest_bucket: currentBucket,
            dest_path: destPath,
            scope: currentScope,
          }),
        });
        processed++;
      } catch (err) {
        console.error(`Failed to ${operation} ${sourcePath}:`, err);
      }
    }

    if (operation === "cut") {
      clipboardFiles = [];
      clipboardOperation = null;
    }

    showNotification(
      `${operation === "copy" ? "Copied" : "Moved"} ${processed} item(s)`,
      "success",
    );
    loadFiles(currentPath, currentBucket);
  }

  // =============================================================================
  // MISSING FUNCTIONS FOR HTML ONCLICK HANDLERS
  // =============================================================================

  function toggleView(type) {
    setView(type);
  }

  function setView(type) {
    const gridBtn = document.getElementById("grid-view-btn");
    const listBtn = document.getElementById("list-view-btn");
    const fileGrid = document.getElementById("file-grid");
    const fileList = document.getElementById("file-list");
    const fileView = document.getElementById("file-view");

    if (type === "grid") {
      gridBtn?.classList.add("active");
      listBtn?.classList.remove("active");
      if (fileGrid) fileGrid.style.display = "grid";
      if (fileList) fileList.style.display = "none";
      if (fileView) fileView.className = "file-grid";
    } else {
      gridBtn?.classList.remove("active");
      listBtn?.classList.add("active");
      if (fileGrid) fileGrid.style.display = "none";
      if (fileList) fileList.style.display = "block";
      if (fileView) fileView.className = "file-list";
    }
  }

  function openFolder(el) {
    const path =
      el?.dataset?.path || el?.querySelector(".file-name")?.textContent;
    if (path) {
      currentPath = path.startsWith("/") ? path : currentPath + "/" + path;
      loadFiles(currentPath);
    }
  }

  function selectFile(el) {
    const path = el?.dataset?.path;
    if (path) {
      toggleSelection(path);
      el.classList.toggle("selected", selectedFiles.has(path));
    } else {
      // Toggle visual selection
      document.querySelectorAll(".file-item.selected").forEach((item) => {
        if (item !== el) item.classList.remove("selected");
      });
      el.classList.toggle("selected");
    }
    updateSelectionUI();
  }

  function setActiveNav(el) {
    document.querySelectorAll(".nav-item").forEach((item) => {
      item.classList.remove("active");
    });
    el.classList.add("active");
  }

  function toggleInfoPanel() {
    const panel =
      document.getElementById("info-panel") ||
      document.getElementById("details-panel");
    if (panel) {
      panel.classList.toggle("open");
      panel.classList.toggle("hidden");
    }
  }

  function toggleAIPanel() {
    const panel =
      document.getElementById("ai-panel") ||
      document.querySelector(".ai-panel");
    if (panel) {
      panel.classList.toggle("open");
    }
  }

  async function aiAction(action) {
    const prompts = {
      organize: "Help me organize the files in this drive. Suggest a folder structure based on the current contents.",
      find: "List the most recently modified files in this drive so I can find what I need.",
      analyze: "Analyze the contents of the currently selected file and provide a summary.",
      share: "Show me how to share files in this drive and what permissions are available."
    };
    const userPrompt = prompts[action] || ("Help me with: " + action);

    addAIMessage("user", userPrompt);

    let activeFilePath = null;
    const selectedItem = document.querySelector(".file-item.selected");
    if (selectedItem) {
      activeFilePath = selectedItem.dataset.path || null;
    }

    const container = document.getElementById("ai-messages") || document.querySelector(".ai-messages");
    const typingDiv = document.createElement("div");
    typingDiv.className = "ai-message assistant typing";
    typingDiv.innerHTML = '<div class="ai-message-bubble">Thinking...</div>';
    if (container) {
      container.appendChild(typingDiv);
      container.scrollTop = container.scrollHeight;
    }

    try {
      const response = await apiRequest("/ai/chat", {
        method: "POST",
        body: JSON.stringify({
          message: userPrompt,
          action: action,
          file_path: activeFilePath,
          bucket: currentBucket,
          scope: currentScope
        })
      });
      if (typingDiv.parentNode) typingDiv.parentNode.removeChild(typingDiv);
      addAIMessage("assistant", response.reply || response.message || "Done.");
    } catch (err) {
      if (typingDiv.parentNode) typingDiv.parentNode.removeChild(typingDiv);
      addAIMessage("assistant", "Error: " + err.message);
    }
  }

  async function sendAIMessage() {
    const input = document.getElementById("ai-input");
    if (!input || !input.value.trim()) return;

    const message = input.value.trim();
    input.value = "";

    addAIMessage("user", message);

    let activeFilePath = null;
    const selectedItem = document.querySelector(".file-item.selected");
    if (selectedItem) {
      activeFilePath = selectedItem.dataset.path || null;
    }

    const typingDiv = document.createElement("div");
    typingDiv.className = "ai-message assistant typing";
    typingDiv.innerHTML = '<div class="ai-message-bubble">Thinking...</div>';
    const container = document.getElementById("ai-messages") || document.querySelector(".ai-messages");
    if (container) {
      container.appendChild(typingDiv);
      container.scrollTop = container.scrollHeight;
    }

    try {
      const response = await apiRequest("/ai/chat", {
        method: "POST",
        body: JSON.stringify({
          message: message,
          file_path: activeFilePath,
          bucket: currentBucket,
          scope: currentScope
        })
      });

      if (typingDiv.parentNode) {
        typingDiv.parentNode.removeChild(typingDiv);
      }

      addAIMessage("assistant", response.reply);
    } catch (err) {
      if (typingDiv.parentNode) {
        typingDiv.parentNode.removeChild(typingDiv);
      }
      addAIMessage("assistant", `Error: ${err.message}`);
    }
  }

  function addAIMessage(type, content) {
    const container =
      document.getElementById("ai-messages") ||
      document.querySelector(".ai-messages");
    if (!container) return;

    const div = document.createElement("div");
    div.className = `ai-message ${type}`;
    div.innerHTML = `<div class="ai-message-bubble">${content}</div>`;
    container.appendChild(div);
    container.scrollTop = container.scrollHeight;
  }

  function updateSelectionUI() {
    const count = selectedFiles.size;
    const bulkActions = document.getElementById("bulk-actions");
    if (bulkActions) {
      bulkActions.style.display = count > 0 ? "flex" : "none";
    }
    const countEl = document.getElementById("selection-count");
    if (countEl) {
      countEl.textContent = `${count} selected`;
    }
  }

  function uploadFile() {
    triggerUpload();
  }

  window.DriveModule = {
    init,
    loadFiles,
    loadStorageInfo,
    discoverBuckets,
    retryWithBackoff,
    toggleSelection,
    selectAll,
    clearSelection,
    downloadFile,
    previewFile,
    shareFile,
    copyShareLink,
    openFile,
    deleteItem,
    deleteSelected,
    renameItem,
    createFolder,
    copyToClipboard,
    cutToClipboard,
    copySelected,
    cutSelected,
    pasteFiles,
    showContextMenuFor,
    navigateUp,
  };

  // Export functions for HTML onclick handlers
  window.toggleView = toggleView;
  window.setView = setView;
  window.openFolder = openFolder;
  window.selectFile = selectFile;
  window.setActiveNav = setActiveNav;
  window.toggleInfoPanel = toggleInfoPanel;
  window.toggleAIPanel = toggleAIPanel;
  window.aiAction = aiAction;
  window.sendAIMessage = sendAIMessage;
  window.uploadFile = uploadFile;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
