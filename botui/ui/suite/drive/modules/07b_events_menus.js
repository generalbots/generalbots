/* Drive Module v2.0 — 07b Events: context menu, keyboard shortcuts and
   bucket/selector bindings. Split out of 07_events.js so every module
   stays under the 450-line limit; behaviour is unchanged. */
"use strict";

function showContextMenu(x, y, path, type, bucket) {
    var menu = document.getElementById("context-menu");
    if (!menu) {
        menu = document.createElement("div");
        menu.id = "context-menu";
        menu.className = "context-menu";
        document.body.appendChild(menu);
    }
    var isFolder = type === "folder";
    var ep = escapeJs(path);
    var eb = escapeJs(bucket || "");
    var icons = {
        open: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>',
        download: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>',
        copy: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>',
        cut: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="6" cy="6" r="3"></circle><circle cx="6" cy="18" r="3"></circle><line x1="20" y1="4" x2="8.12" y2="15.88"></line><line x1="14.47" y1="14.48" x2="20" y2="20"></line><line x1="8.12" y1="8.12" x2="12" y2="12"></line></svg>',
        rename: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path></svg>',
        delete: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>',
        link: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path></svg>',
        copyPath: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>',
        duplicate: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>',
        paste: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path><rect x="8" y="2" width="8" height="4" rx="1" ry="1"></rect></svg>',
        ai: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2a4 4 0 0 1 4 4 4 4 0 0 1 4 4 4 4 0 0 1-4 4 4 4 0 0 1-4 4 4 4 0 0 1-4-4 4 4 0 0 1-4-4 4 4 0 0 1 4-4 4 4 0 0 1 4-4z"></path><path d="M12 6v12M6 12h12"></path></svg>',
        chat: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>',
    };
    var useInChatLabel = window.gbT
        ? window.gbT("drive-use-in-chat", "Use in chat")
        : "Use in chat";
    var hideMenu = "document.getElementById('context-menu').style.display='none';";
    var hasClipboard = clipboardFiles.length > 0;
    var isStarred = (window._starredPaths && window._starredPaths[path]) ||
        sessionStorage.getItem("drive-tab") === "starred";
    var html = '';
    // #1866 — folders get an "Open" + "Share folder" (drop box stub); files get
    // the full action set including the existing per-file "Share link" and
    // "Copy link" actions. The shareFolder function in 98_actions_extra.js
    // shows a "coming soon" dialog until the botpublic relay ships.
    html += isFolder
        ? '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.loadFiles(\'' + ep + '\', \'' + (eb || currentBucket) + '\')">' + icons.open + '<span>Open</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.shareFolder(\'' + ep + '\', \'' + (eb || currentBucket) + '\')">' + icons.link + '<span>Share folder</span></div>'
        : '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.openFile(\'' + ep + '\', \'' + eb + '\')">' + icons.open + '<span>Open</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.previewFile(\'' + ep + '\', \'' + eb + '\')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="margin-right:8px;"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg><span>Preview</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.downloadFile(\'' + ep + '\', \'' + eb + '\')">' + icons.download + '<span>Download</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.useInChat(\'' + ep + '\', \'' + eb + '\')">' + icons.chat + '<span>' + useInChatLabel + '</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.shareFile(\'' + ep + '\', \'' + (eb || currentBucket) + '\')">' + icons.link + '<span>Share link</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.copyLink(\'' + ep + '\', \'' + (eb || currentBucket) + '\')">' + icons.link + '<span>Copy link</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.toggleStar(\'' + ep + '\', \'' + (eb || currentBucket) + '\', ' + (isStarred ? "false" : "true") + ')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="margin-right:8px;"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon></svg><span>' + (isStarred ? "Remove from starred" : "Add to starred") + '</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.openComments(\'' + ep + '\')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="margin-right:8px;"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg><span>Comments</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.askAIOnFile(\'' + ep + '\')">' + icons.ai + '<span>Ask AI</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.duplicateItem(\'' + ep + '\', \'' + (eb || currentBucket) + '\')">' + icons.duplicate + '<span>Duplicate</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.showVersionHistory(\'' + ep + '\')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg><span>Version history</span></div>';
    html += '<div class="context-menu-divider"></div>';
    html += '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.copyToClipboard(\'' + ep + '\', \'' + (eb || currentBucket) + '\')">' + icons.copy + '<span>Copy</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.cutToClipboard(\'' + ep + '\', \'' + (eb || currentBucket) + '\')">' + icons.cut + '<span>Cut</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.copyPathToClipboard(\'' + ep + '\')">' + icons.copyPath + '<span>Copy path</span></div>'
        + '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.renameItem(\'' + ep + '\', \'' + (eb || currentBucket) + '\')">' + icons.rename + '<span>Rename</span></div>';
    if (hasClipboard) {
        html += isFolder
            ? '<div class="context-menu-item" onclick="' + hideMenu + 'DriveModule.pasteInto(\'' + ep + '\')">' + icons.paste + '<span>Paste into folder</span></div>'
            : '';
    }
    html += '<div class="context-menu-divider"></div>'
        + '<div class="context-menu-item danger" onclick="' + hideMenu + 'DriveModule.deleteItem(\'' + ep + '\', \'' + (eb || currentBucket) + '\')">' + icons.delete + '<span>Delete</span></div>';
    menu.innerHTML = html;
    // The simple shell menu (Cut/Copy/Paste) may have registered after this
    // module in some load orders; remove any of its leftover menus before
    // showing ours.
    document.querySelectorAll(".simple-context-menu").forEach(function (el) {
      el.remove();
    });
    menu.style.display = "block";
    menu.classList.remove("hidden");
    const rect = menu.getBoundingClientRect();
    // Anchor the menu at the cursor and clamp it into the viewport. The old
    // logic flipped a menu that did not fit below the cursor fully above it,
    // which put long menus (14 items) far above the pointer.
    const margin = 8;
    menu.style.left = Math.max(margin, Math.min(x, window.innerWidth - rect.width - margin)) + "px";
    menu.style.top = Math.max(margin, Math.min(y, window.innerHeight - rect.height - margin)) + "px";
}

function showContextMenuFor(event, path) {
    var el = document.querySelector('[data-path="' + CSS.escape(path) + '"]');
    var type = el ? el.dataset.type : "file";
    var bucket = el ? (el.dataset.bucket || "") : "";
    showContextMenu(event.clientX, event.clientY, path, type, bucket);
}

function bindKeyboardShortcuts() {
    document.addEventListener("keydown", function(e) {
        if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") return;
        if (e.key === "Delete" && selectedFiles.size > 0) { e.preventDefault(); deleteSelected(); }
        if ((e.ctrlKey || e.metaKey) && e.key === "a") { e.preventDefault(); selectAll(); }
        if ((e.ctrlKey || e.metaKey) && e.key === "c" && selectedFiles.size > 0) { e.preventDefault(); copySelected(); }
        if ((e.ctrlKey || e.metaKey) && e.key === "x" && selectedFiles.size > 0) { e.preventDefault(); cutSelected(); }
        if ((e.ctrlKey || e.metaKey) && e.key === "v" && clipboardFiles.length > 0) { e.preventDefault(); pasteFiles(); }
        if (e.key === "Escape") { clearSelection(); var menu = document.getElementById("context-menu"); if (menu) menu.style.display = "none"; }
        if (e.key === "Backspace" && !e.ctrlKey && !e.metaKey) { e.preventDefault(); navigateUp(); }
        if (e.key === "F2" && selectedFiles.size === 1) { e.preventDefault(); renameItem(Array.from(selectedFiles)[0]); }
    });
    // OS-clipboard paste: when the in-app clipboard is empty but the OS
    // clipboard carries files (e.g. screenshots), upload them into the
    // current folder instead of silently doing nothing.
    document.addEventListener("paste", function(e) {
        if (e.target && e.target.tagName && (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA")) return;
        const items = e.clipboardData && e.clipboardData.items;
        if (!items) return;
        var files = [];
        for (var i = 0; i < items.length; i++) {
            if (items[i].kind === "file") {
                const f = items[i].getAsFile();
                if (f) files.push(f);
            }
        }
        if (files.length > 0) {
            e.preventDefault();
            uploadClipboardFiles(files);
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

function bindSearchInput() {
    const searchInput = document.querySelector(".search-box input, #search-input");
    if (!searchInput) return;
    var debounceTimer;
    searchInput.addEventListener("input", function(e) {
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(function() {
            const query = e.target.value.trim();
            // #1638 — one character is not a query; restore the listing.
            if (query.length >= 2) searchFiles(query, true);
            else if (!query) loadFiles(currentPath, currentBucket);
        }, 300);
    });
    // Enter searches immediately; Escape clears and returns to the folder.
    searchInput.addEventListener("keydown", function(e) {
        if (e.key === "Enter") {
            e.preventDefault();
            clearTimeout(debounceTimer);
            const query = searchInput.value.trim();
            if (query) searchFiles(query, true);
        } else if (e.key === "Escape") {
            searchInput.value = "";
            loadFiles(currentPath, currentBucket);
        }
    });
    // #1973 — Ctrl/Cmd+Shift+K opens the recent-files quick list
    // (14_quick_open.js). Plain Ctrl/Cmd+K belongs to the global command
    // palette (command-palette.js), which stops immediate propagation; the
    // Shift modifier keeps the Drive recents reachable inside the app.
    document.addEventListener("keydown", function(e) {
        if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "k" || e.key === "K")) {
            e.preventDefault();
            if (window.DriveQuickOpen) window.DriveQuickOpen.toggle();
        }
    });
}



function updateBucketSelector() {
    const selector = document.getElementById("bucket-selector");
    if (!selector) return;
    if (availableBuckets.length <= 1) {
        selector.style.display = "none";
        return;
    }
    selector.style.display = "block";
    selector.innerHTML = availableBuckets.map(function(b) {
        var label = b.is_gborg ? b.name.replace(".gborg", "") + " (Org)"
            : b.is_gbai ? b.name.replace(".gbai", "")
            : b.name;
        return '<option value="' + escapeHtml(b.name) + '" ' + (b.name === currentBucket ? "selected" : "") + '>' + escapeHtml(label) + '</option>';
    }).join("");
    selector.removeEventListener("change", handleBucketChange);
    selector.addEventListener("change", handleBucketChange);
}

function handleBucketChange(e) {
    const newBucket = e.target.value;
    const bucketInfo = availableBuckets.find(function(b) { return b.name === newBucket; });
    if (bucketInfo && bucketInfo.is_gborg) {
        currentGborgBucket = newBucket;
        currentGborgBranch = newBucket.replace(".gborg", "");
    } else {
        currentGborgBucket = null;
        currentGborgBranch = null;
    }
    currentBucket = newBucket;
    currentPath = "";
    if (currentTab === TAB_BRANCHDRIVE) loadBranchDriveTab();
    else if (currentTab === TAB_SHARED) loadSharedTab();
    else if (currentTab === TAB_PUBLIC) loadPublicTab();
    else if (currentTab === TAB_MYFILES) loadMyFilesTab();
    else if (currentTab === TAB_BOTS) loadBotsTab();
    else if (currentTab === TAB_ROOT) loadRootTab();
    else loadFiles();
}

function bindRefreshBotsBtn() {}
function bindBotSearchInput() {}
function bindNewBotBtn() {}

// ── Drag source for desktop shortcuts (#1188) ────────────────────
// Delegated on the app container so re-renders keep working: any file
// card/list row becomes draggable and hands its coordinates to the
// desktop via the private drag type consumed by js/desktop-shortcuts.js.
(function () {
    var root = document.getElementById("drive-app");
    if (!root || root.dataset.gbDragSource === "1") return;
    root.dataset.gbDragSource = "1";

    root.addEventListener("dragstart", function (e) {
        var item = e.target.closest(".file-card, .drive-file-item");
        if (!item || item.getAttribute("data-type") === "folder") return;
        var payload = {
            name: item.getAttribute("data-name") || "File",
            path: item.getAttribute("data-path") || "",
            bucket: "",
            type: item.getAttribute("data-type") || "file",
        };
        e.dataTransfer.setData("application/x-gb-drive-file", JSON.stringify(payload));
        e.dataTransfer.effectAllowed = "copy";
    });
})();
