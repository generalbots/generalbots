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
// file's folder once the module is ready (window.__gbAppParams__.path, set by
// the shell). #2143 — links emitted by `drive_save` carry `scope=bot` because
// the saved artifact lives in the bot workspace, not in the caller's personal
// bucket; without it the folder resolved against `users/{id}/` and rendered
// empty.
var __deepParams = window.__gbAppParams__ || {};
var __deepPath = __deepParams.path;
var __deepScope = String(__deepParams.scope || "").toLowerCase();
if (__deepPath) {
    var __origInit = init;
    init = async function () {
        await __origInit();
        if (__deepScope === "bot") currentScope = "bot";
        try { await loadFiles(__deepPath); } catch (e) { /* default listing */ }
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
