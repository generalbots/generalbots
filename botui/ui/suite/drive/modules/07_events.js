/* Drive Module v2.0 — 07 Events: bindings, navigation, shortcuts */
"use strict";

function bindTopTabs() {
    document.querySelectorAll(".top-tab").forEach(function(tab) {
        tab.addEventListener("click", function() {
            document.querySelectorAll(".top-tab").forEach(function(t) { t.classList.remove("active"); });
            this.classList.add("active");
            currentTab = this.dataset.tab;
            sessionStorage.setItem("drive-tab", currentTab);
            selectedFiles.clear();
            currentPath = "";
            // #1801 — leaving the Home tab must restore MinIO discovery; the
            // `local:home` bucket is not a real S3 bucket.
            if (parseLocalBucket(currentBucket)) {
                currentBucket = "";
                currentGborgBucket = null;
                currentGborgBranch = null;
                bucketLayoutResolved = false;
                currentBackend = "minio";
            }
            updateSelectionBar();

            var nav = document.getElementById("drive-nav");

            if (nav) nav.style.display = "block";

            switch (currentTab) {
                case TAB_BRANCHDRIVE:
                    loadBranchDriveTab();
                    break;
                case TAB_SHARED:
                    loadSharedTab();
                    break;
                case TAB_PUBLIC:
                    loadPublicTab();
                    break;
                case TAB_MYFILES:
                    loadMyFilesTab();
                    break;
                case "conversations":
                    loadConversationFilesTab();
                    break;
                case "attachments":
                    loadChatAttachmentsTab();
                    break;
                case TAB_HOME:
                    loadHomeTab();
                    break;
                case TAB_BOTS:
                    loadBotsTab();
                    break;
                case TAB_ROOT:
                    loadRootTab();
                    break;
                case TAB_DESKTOP:
                    loadDesktopTab();
                    break;
                case TAB_EXTERNAL:
                    loadExternalTab();
                    break;
            }
        });
    });
    var savedTab = sessionStorage.getItem("drive-tab") || TAB_MYFILES;
    // Older sessions may remember a tab that no longer exists.
    if (["branchdrive", "bots", "root", "desktop", "external"].indexOf(savedTab) !== -1) {
        savedTab = TAB_MYFILES;
        sessionStorage.setItem("drive-tab", savedTab);
    }
    // Home only exists in the desktop edition.
    if (savedTab === TAB_HOME && !isDesktopLocal()) {
        savedTab = TAB_MYFILES;
        sessionStorage.setItem("drive-tab", savedTab);
    }
    document.querySelectorAll(".top-tab").forEach(function(t) {
        t.classList.toggle("active", t.dataset.tab === savedTab);
    });
}

function switchTab(tabName) {
    var tab = document.querySelector('.top-tab[data-tab="' + tabName + '"]');
    if (tab) tab.click();
}

function bindNavigation() {
    document.querySelectorAll(".drive-nav-item").forEach(function(item) {
        item.addEventListener("click", function() {
            if (this.classList.contains("division-item") || this.classList.contains("bot-item")) return;
            document.querySelectorAll(".drive-nav-item").forEach(function(i) { i.classList.remove("active"); });
            this.classList.add("active");
            var view = this.dataset.view || this.dataset.filter;
            if (view === "my-drive" || !view) loadFiles("", currentBucket);
            else if (view === "recent") loadRecentFiles();
            else if (view === "starred" || view === "favorite") loadStarredFiles();
            else if (view === "shared") loadSharedFiles();
            else if (view === "trash") loadTrashFiles();
        });
    });
}

function bindViewToggle() {
    document.querySelectorAll(".view-toggle, .view-btn").forEach(function(btn) {
        btn.addEventListener("click", function() {
            var view = this.dataset.view;
            if (view) {
                viewMode = view;
                document.querySelectorAll(".view-toggle, .view-btn").forEach(function(b) { b.classList.remove("active"); });
                this.classList.add("active");
                loadFiles(currentPath, currentBucket);
            }
        });
    });
}

function bindUploadButton() {
    const uploadBtn = document.getElementById("upload-btn");
    if (uploadBtn) uploadBtn.addEventListener("click", triggerUpload);
    const toolbarUploadBtn = document.getElementById("toolbar-upload-btn");
    if (toolbarUploadBtn) toolbarUploadBtn.addEventListener("click", triggerUpload);
    window.uploadFile = triggerUpload;
    var fileInput = document.getElementById("file-input");
    if (!fileInput) {
        fileInput = document.createElement("input");
        fileInput.type = "file";
        fileInput.id = "file-input";
        fileInput.multiple = true;
        fileInput.style.display = "none";
        document.body.appendChild(fileInput);
    }
    fileInput.addEventListener("change", handleFileInputChange);

    var cameraBtn = document.getElementById("upload-camera-btn");
    if (cameraBtn) cameraBtn.addEventListener("click", triggerCameraUpload);
    var toolbarCameraBtn = document.getElementById("toolbar-camera-btn");
    if (toolbarCameraBtn) toolbarCameraBtn.addEventListener("click", triggerCameraUpload);
    var cameraInput = document.getElementById("camera-input");
    if (cameraInput) cameraInput.addEventListener("change", handleCameraInputChange);

    // #1751 — folder picker is a separate input (never webkitdirectory on
    // #file-input) so flat file selection still uploads as before.
    var folderBtn = document.getElementById("upload-folder-btn");
    var toolbarFolderBtn = document.getElementById("toolbar-folder-btn");
    var folderInput = document.getElementById("folder-input");
    var folderNote = document.getElementById("folder-unsupported-note");
    var supportsFolder = typeof document !== "undefined" && "webkitdirectory" in document.createElement("input");
    if (!supportsFolder) {
        if (folderBtn) folderBtn.style.display = "none";
        if (toolbarFolderBtn) toolbarFolderBtn.style.display = "none";
        if (folderInput) folderInput.style.display = "none";
        if (folderNote) folderNote.hidden = false;
    } else {
        if (folderBtn) folderBtn.addEventListener("click", triggerFolderUpload);
        if (toolbarFolderBtn) toolbarFolderBtn.addEventListener("click", triggerFolderUpload);
        if (folderInput) folderInput.addEventListener("change", handleFolderInputChange);
    }
}

function triggerUpload() {
    const input = document.getElementById("file-input");
    if (input) input.click();
}

function triggerCameraUpload() {
    const explainedKey = "gb-drive-camera-explained-v1";
    let explained = false;
    try { explained = localStorage.getItem(explainedKey) === "1"; } catch (_) { window.gbLogError ? gbLogError(_, "triggerCameraUpload") : console.error("triggerCameraUpload", _); }
    const proceed = function () {
        try { localStorage.setItem(explainedKey, "1"); } catch (_) { window.gbLogError ? gbLogError(_, "proceed") : console.error("proceed", _); }
        const input = document.getElementById("camera-input");
        if (input) input.click();
    };
    if (explained) { proceed(); return; }
    const msg = window.gbT
        ? window.gbT("drive-camera-permission-explainer", "General Bots will ask for camera access so you can take a photo and upload it to this folder. Your browser controls the permission.")
        : "General Bots will ask for camera access so you can take a photo and upload it to this folder. Your browser controls the permission.";
    const title = window.gbT
        ? window.gbT("drive-camera-permission-title", "Use camera")
        : "Use camera";
    if (window.GBModal && typeof window.GBModal.confirm === "function") {
        window.GBModal.confirm(msg, title, proceed, function () {});
        return;
    }
    proceed();
}

function handleFileInputChange(e) {
    const files = e.target.files;
    if (files && files.length > 0) uploadFiles(Array.from(files));
    e.target.value = "";
}

function handleCameraInputChange(e) {
    const files = e.target.files;
    if (files && files.length > 0) uploadFiles(Array.from(files));
    e.target.value = "";
}

function bindDragAndDrop() {
    const container = document.querySelector(".drive-container, .drive-main");
    if (!container) return;
    ["dragenter", "dragover", "dragleave", "drop"].forEach(function(eventName) {
        container.addEventListener(eventName, function(e) { e.preventDefault(); e.stopPropagation(); });
    });
    container.addEventListener("dragenter", function() {
        container.classList.add("drag-active");
        var overlay = document.getElementById("drop-overlay");
        if (overlay) overlay.classList.add("visible");
    });
    container.addEventListener("dragleave", function(e) {
        if (!container.contains(e.relatedTarget)) {
            container.classList.remove("drag-active");
            var overlay = document.getElementById("drop-overlay");
            if (overlay) overlay.classList.remove("visible");
        }
    });
    container.addEventListener("drop", function(e) {
        container.classList.remove("drag-active");
        var overlay = document.getElementById("drop-overlay");
        if (overlay) overlay.classList.remove("visible");
        // #1751 — walk DataTransferItem entries so dropped directories keep
        // their tree instead of arriving as an empty flat file list.
        handleDroppedTransfer(e.dataTransfer);
    });
}

function bindContextMenu() {
    // #2419 — right-click stays the desktop path; touch gets a long-press
    // shortcut and a visible "..." button (the discoverable path on iOS).
    let lastGestureOpen = 0;

    function openFileMenu(event, path) {
        lastGestureOpen = Date.now();
        showContextMenuFor(event, path);
        const menu = document.getElementById("context-menu");
        if (menu && window.gbGestures) {
            window.gbGestures.clampToViewport(menu, event.clientX, event.clientY);
        }
    }

    document.addEventListener("contextmenu", function(e) {
        const fileEl = e.target.closest(".file-card, .drive-file-item");
        if (fileEl) {
            e.preventDefault();
            openFileMenu(e, fileEl.dataset.path);
        }
    });
    document.addEventListener("click", function(e) {
        const menu = document.getElementById("context-menu");
        if (menu && !menu.contains(e.target)) {
            // A touch long-press is followed by a synthetic click; do not let
            // the outside-dismiss handler close the menu we just opened.
            if (Date.now() - lastGestureOpen < 700) return;
            menu.classList.add("hidden");
            menu.style.display = "none";
        }
    });

    const gestures = window.gbGestures;
    if (!gestures || !gestures.isCoarsePointer()) return;

    gestures.onLongPress(document, function(e) {
        if (!e.target || !e.target.closest) return;
        if (e.target.closest("input, button, a, .btn-icon-sm")) return;
        const fileEl = e.target.closest(".file-card, .drive-file-item");
        if (fileEl) openFileMenu(e, fileEl.dataset.path);
    }, { tolerance: 12 });

    function addOverflowButtons() {
        document.querySelectorAll(".file-card:not([data-gb-overflow])").forEach(function(card) {
            card.setAttribute("data-gb-overflow", "1");
            const btn = document.createElement("button");
            btn.type = "button";
            btn.className = "file-card-more";
            btn.setAttribute("aria-label", window.gbT ? window.gbT("label-more", "More") : "More");
            btn.innerHTML = '<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><circle cx="5" cy="12" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="19" cy="12" r="2"/></svg>';
            // Inline geometry: css/ is owned by another agent in this batch.
            btn.style.cssText = "position:absolute;top:4px;right:4px;width:44px;height:44px;display:flex;align-items:center;justify-content:center;border:0;border-radius:8px;background:var(--surface,rgba(0,0,0,.45));color:inherit;cursor:pointer;z-index:2";
            btn.addEventListener("click", function(e) {
                e.preventDefault();
                e.stopPropagation();
                openFileMenu(e, card.dataset.path);
            });
            card.appendChild(btn);
        });
    }

    addOverflowButtons();
    if (typeof MutationObserver !== "undefined") {
        const root = document.getElementById("drive-app") || document.body;
        let scheduled = false;
        const observer = new MutationObserver(function() {
            if (scheduled) return;
            scheduled = true;
            requestAnimationFrame(function() {
                scheduled = false;
                addOverflowButtons();
            });
        });
        observer.observe(root, { childList: true, subtree: true });
    }
}

