/* Drive Module v2.0 — 05 Files (tabs): path builders and tab loaders. */
"use strict";

// Split from 05_files.js for #1731 (450-line limit). Mechanical split:
// same functions, globals and load order.

// ── Tab Path Builders ────────────────────────────────────────────
function buildPathBranchDrive() {
    if (currentGborgBranch) {
        // If bucket is .gborg, files are nested inside the branch's .gbai
        // workspace under the BOT's own directory. That directory equals the
        // branch name only when the bot is its branch's namesake; a twin such
        // as `cristo-test`, or `oppbot` in branch `opportunity-oppbot`, keeps
        // its own `{bot}.gbdrive` inside the branch workspace.
        if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
            var botName = window.__INITIAL_BOT_NAME__ || currentGborgBranch;
            return currentGborgBranch + ".gbai/" + botName + ".gbdrive";
        }
        return currentGborgBranch + ".gbdrive";
    }
    // Standalone layout: the bucket IS the bot's {bot}.gbai and its Drive
    // sits at {bot}.gbdrive. Without this the Drive tab fell back to
    // scope=user with an empty path and rendered "This folder is empty"
    // for every standalone bot (beiner in production).
    if (currentBucket && currentBucket.indexOf('.gbai') > 0) {
        return currentBucket.replace('.gbai', '.gbdrive');
    }
    return "";
}

function buildPathShared() {
    if (!currentGborgBranch) return "";
    if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
        return currentGborgBranch + ".gbai/shared.gbdrive";
    }
    return "shared.gbdrive";
}

function buildPathPublic() {
    if (!currentGborgBranch) return "";
    if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
        return currentGborgBranch + ".gbai/public.gbdrive";
    }
    return "public.gbdrive";
}

function buildPathMyFiles() {
    if (!currentGborgBranch) return "";
    var login = (userInfo ? (userInfo.username || "unknown") : "unknown").toLowerCase();
    if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
        return currentGborgBranch + ".gbai/users.gbdrive/" + login;
    }
    return "users.gbdrive/" + login;
}

function buildPathRoot() {
    if (!currentGborgBranch) return "";
    if (currentBucket && currentBucket.indexOf('.gborg') > 0) {
        return currentGborgBranch + ".gbai";
    }
    return "";
}

// ── Tab Loaders ───────────────────────────────────────────────────
async function loadBranchDriveTab() {
    if (!currentBucket) await discoverBuckets();
    var path = buildPathBranchDrive();
    if (path) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket || currentBucket);
    } else {
        currentPath = "";
        currentScope = "user";
        await loadFiles("", currentBucket);
    }
}

async function loadSharedTab() {
    const content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    content.innerHTML = '<div class="loading-state" role="status" aria-live="polite" aria-busy="true"><div class="spinner" aria-hidden="true"></div><p>Loading shared items...</p></div>';
    try {
        const params = new URLSearchParams();
        params.set("scope", currentScope);
        const items = await apiRequest("/shared?" + params.toString());
        sharedCache = Array.isArray(items) ? items : [];
        renderSharedList(sharedCache);
    } catch (err) {
        content.innerHTML = '<div class="empty-state" role="alert" aria-live="assertive"><h3>Failed to load shared items</h3></div>';
    }
}

async function loadPublicTab() {
    var path = buildPathPublic();
    if (!currentBucket) await discoverBuckets();
    if (path) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket || currentBucket);
    } else if (currentBucket) {
        currentScope = "user";
        await loadFiles("", currentBucket);
    }
}

async function loadMyFilesTab() {
    var path = buildPathMyFiles();
    if (!currentBucket) await discoverBuckets();
    if (path) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket || currentBucket);
    } else if (currentBucket) {
        currentScope = "user";
        await loadFiles("", currentBucket);
    }
}

// ── Home tab (#1801): the desktop user's real home folder. The server keeps
// every path inside an allowlisted root, hides dotfiles and skips symlinks
// that escape it. Read-only in this slice: browse, preview and download.
async function loadHomeTab() {
    currentBackend = "local";
    currentLocalRoot = "home";
    currentScope = "user";
    currentPath = "";
    // Local rows use a synthetic bucket; clear the org pointers so the
    // breadcrumb and effective-bucket helpers do not mix the two backends.
    currentGborgBucket = null;
    currentGborgBranch = null;
    currentBucket = localBucket(currentLocalRoot);
    await loadFiles("", currentBucket);
}

async function loadRootTab() {
    var path = buildPathRoot();
    if (!currentBucket) await discoverBuckets();
    if (path && currentGborgBucket) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket);
    } else {
        showNotification("Root tab requires an org (.gborg) bucket", "warning");
    }
}

// ── Desktop tab (#1154): the user's Desktop folder on Drive, where
// per-user shortcuts (.gbdesktop.json) and dropped files live.
async function loadDesktopTab() {
    var path = buildPathDesktop();
    if (!currentBucket) await discoverBuckets();
    if (path) {
        currentPath = path;
        currentScope = "bot";
        await loadFiles(path, currentGborgBucket || currentBucket);
    } else if (currentBucket) {
        currentScope = "user";
        await loadFiles("", currentBucket);
    }
}

function buildPathDesktop() {
    var base = buildPathMyFiles();
    if (!base) return "";
    return base + "/Desktop";
}
