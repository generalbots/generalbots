/* Drive Module v2.0 — 99 Init */
"use strict";

async function init() {
    await fetchUserInfo();

    // Show Root tab only for admins
    var rootTab = document.getElementById("tab-root");
    if (rootTab) rootTab.style.display = isAdmin ? "flex" : "none";

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

    var savedTab = sessionStorage.getItem("drive-tab") || TAB_BRANCHDRIVE;
    switch (savedTab) {
        case TAB_BRANCHDRIVE:
            await loadBranchDriveTab();
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
            await loadBranchDriveTab();
            break;
    }
    loadStorageInfo();
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
    shareFile: shareFile,
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
    loadBotsTab: loadBotsTab,
    loadRootTab: loadRootTab,
    loadExternalTab: loadExternalTab,
};

// Deep-link from a desktop shortcut: jump straight to the file's folder
// once the module is ready (window.__gbAppParams__.path, set by the shell).
var __deepPath = (window.__gbAppParams__ || {}).path;
if (__deepPath) {
    var __origInit = init;
    init = async function () {
        await __origInit();
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
