/* Drive Module v2.0 — 99 Init */
"use strict";

async function init() {
    // Bind the shell first: a slow or failing user lookup must never leave the
    // tabs and buttons unresponsive (seen when Drive is injected over HTMX).
    bindTopTabs();
    bindNavigation();
    bindViewToggle();
    bindDragAndDrop();
    bindContextMenu();
    bindKeyboardShortcuts();
    bindUploadButton();
    bindNewFolderButton();
    bindSearchInput();
    bindRefreshBotsBtn();
    bindBotSearchInput();
    bindNewBotBtn();
    if (window.DriveExternal && DriveExternal.bind) DriveExternal.bind();

    try {
        await fetchUserInfo();
    } catch (e) {
        console.warn("[drive] user lookup failed; continuing with the cached user:", e);
    }

    // Show Root tab only for admins
    var rootTab = document.getElementById("tab-root");
    if (rootTab) rootTab.style.display = isAdmin ? "flex" : "none";

    // #1801 — the Home tab browses the desktop user's real home folder. The
    // server disables the local backend outside the desktop edition, so the
    // tab is never advertised there.
    var homeTab = document.getElementById("tab-home");
    if (homeTab) homeTab.style.display = isDesktopLocal() ? "flex" : "none";

    var savedTab = sessionStorage.getItem("drive-tab") || TAB_MYFILES;
    if (["branchdrive", "bots", "root", "desktop", "external"].indexOf(savedTab) !== -1) {
        savedTab = TAB_MYFILES;
        sessionStorage.setItem("drive-tab", savedTab);
    }
    if (savedTab === TAB_HOME && !isDesktopLocal()) {
        savedTab = TAB_MYFILES;
        sessionStorage.setItem("drive-tab", savedTab);
    }
    switch (savedTab) {
        case TAB_BRANCHDRIVE:
            await loadBranchDriveTab();
            break;
        case TAB_HOME:
            await loadHomeTab();
            break;
        case TAB_SHARED:
            await loadSharedTab();
            break;
        case TAB_PUBLIC:
            await loadPublicTab();
            break;
        case TAB_MYFILES:
            await loadMyFilesTab();
            break;
        case TAB_BOTS:
            await loadBotsTab();
            break;
        case TAB_ROOT:
            if (isAdmin) await loadRootTab();
            else await loadBranchDriveTab();
            break;
        case TAB_EXTERNAL:
            if (window.DriveExternal && DriveExternal.handleCallbackParams) {
                DriveExternal.handleCallbackParams();
            }
            await loadExternalTab();
            break;
        default:
            await loadMyFilesTab();
            break;
    }
    loadStorageInfo();
    // Decorate the rows with their true starred state (one favorite listing
    // per boot; toggleStar keeps the cache current afterwards).
    loadStarredPathSet();
}

window.DriveModule = {
    selectedFiles: selectedFiles,
    init: init,
    getRecentFileNames: getRecentFileNames,
    loadFiles: loadFiles,
    loadBotConfigs: loadBotConfigs,
    loadStorageInfo: loadStorageInfo,
    discoverBuckets: discoverBuckets,
    retryWithBackoff: retryWithBackoff,
    toggleSelection: toggleSelection,
    selectAll: selectAll,
    clearSelection: clearSelection,
    downloadFile: downloadFile,
    downloadSelected: downloadSelected,
    previewFile: previewFile,
    toggleStar: toggleStar,
    toggleStarFromButton: toggleStarFromButton,
    refreshRowStars: refreshRowStars,
    shareFile: shareFile,
    shareFolder: shareFolder,
    createShareLinkNow: createShareLinkNow,
    revokeShareLinkFromModal: revokeShareLinkFromModal,
    shareWithRecipient: shareWithRecipient,
    sendShareEmail: sendShareEmail,
    loadShareEmailSenders: loadShareEmailSenders,
    exportPaths: exportPaths,
    copyShareLink: copyShareLink,
    revokeShareLink: revokeShareLink,
    copyLink: copyLink,
    duplicateItem: duplicateItem,
    copyPathToClipboard: copyPathToClipboard,
    pasteInto: pasteInto,
    askAIOnFile: askAIOnFile,
    openComments: openComments,
    restoreTrash: restoreTrash,
    emptyTrash: emptyTrash,
    showVersionHistory: showVersionHistory,
    restoreVersion: restoreVersion,
    restoreServerVersion: restoreServerVersion,
    saveFileVersion: saveFileVersion,
    openFile: openFile,
    deleteItem: deleteItem,
    deleteSelected: deleteSelected,
    renameItem: renameItem,
    createFolder: createFolder,
    copyToClipboard: copyToClipboard,
    cutToClipboard: cutToClipboard,
    copySelected: copySelected,
    cutSelected: cutSelected,
    pasteFiles: pasteFiles,
    showContextMenuFor: showContextMenuFor,
    navigateUp: navigateUp,
    openSharedFile: openSharedFile,
    loadBranchDriveTab: loadBranchDriveTab,
    loadSharedTab: loadSharedTab,
    loadPublicTab: loadPublicTab,
    loadMyFilesTab: loadMyFilesTab,
    loadHomeTab: loadHomeTab,
    loadBotsTab: loadBotsTab,
    loadRootTab: loadRootTab,
    loadExternalTab: loadExternalTab,
};

// Deep-link from a desktop shortcut or a chat link: jump straight to the
// file's folder once the module is ready (window.__gbAppParams__, set by the
// shell). #2143 — links emitted by `drive_save` carry `scope=bot` because the
// saved artifact lives in the bot workspace. #2283 — saved-artifact links also
// carry `bucket`, so the Drive resolves the path against the bucket the artifact
// was saved into instead of the currently selected one.
var __deepParams = window.__gbAppParams__ || {};
var __deepPath = __deepParams.path;
var __deepBucket = __deepParams.bucket;
var __deepScope = String(__deepParams.scope || "").toLowerCase();
if (__deepPath) {
    var __origInit = init;
    init = async function () {
        await __origInit();
        if (__deepScope === "bot") currentScope = "bot";
        if (__deepBucket) {
            currentBucket = __deepBucket;
            // Mirror search-hit bucket switching (06_render.js) so effective
            // bucket resolution matches the store that holds the deep-linked path.
            if (String(__deepBucket).indexOf(".gborg") > 0) {
                currentGborgBucket = __deepBucket;
                currentGborgBranch = String(__deepBucket).replace(".gborg", "");
            } else {
                currentGborgBucket = __deepBucket.indexOf(".gbai") > 0 ? null : currentGborgBucket;
                if (!currentGborgBucket) currentGborgBranch = null;
            }
            if (typeof updateBucketSelector === "function") updateBucketSelector();
        }
        function showDeepLinkNotFound(reason) {
            var contentEl = document.getElementById("drive-content") || document.getElementById("file-grid");
            if (!contentEl) return;
            var detail = reason ? "<p>" + escapeHtml(humanizeError(reason)) + "</p>" : "";
            contentEl.innerHTML = '<div class="empty-state"><svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="1.5"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg><h3>Not found in the current Drive</h3><p>The linked path could not be resolved.</p>' + detail + '</div>';
        }
        try {
            await loadFiles(__deepPath, __deepBucket || undefined);
            // A deep-linked folder that cannot be resolved must not look like an
            // empty folder. If the load rendered the generic empty state for the
            // requested path, surface an explicit not-found state.
            var content = document.getElementById("drive-content") || document.getElementById("file-grid");
            if (content) {
                var h3 = content.querySelector(".empty-state h3");
                if (h3 && h3.textContent === "This folder is empty") {
                    showDeepLinkNotFound("");
                }
            }
        } catch (e) {
            showDeepLinkNotFound(e && e.message ? e.message : "");
        }
    };
}

// Re-discover buckets and reload when the user signs in (token appears
// in localStorage after the login redirect).
window.addEventListener("storage", function (e) {
    if (e.key === "gb-access-token" && e.newValue && !e.oldValue) {
        bucketLayoutResolved = false;
        currentBucket = "";
        fetchUserInfo().then(function () { init(); });
    }
});
window.addEventListener("gb:auth:login", function () {
    bucketLayoutResolved = false;
    currentBucket = "";
    fetchUserInfo().then(function () { init(); });
});

function __safeInit() { try { init(); } catch (e) { console.error("[drive] init failed:", e); } }
if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", __safeInit);
} else {
    __safeInit();
}
