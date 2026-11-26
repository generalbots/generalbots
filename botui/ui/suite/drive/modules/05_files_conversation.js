/* Drive parity with the gbgov assistant shell: conversation-aware files.
 *
 * Two surfaces the classic Drive did not have:
 *   1. "Conversations" — files the assistant saved under
 *      <bot>.gbdrive/conversations/<folder>/, grouped per conversation.
 *   2. "Attachments" — files attached to chat messages, gathered from the
 *      generic chat-history API and deduplicated by their Drive path.
 *
 * Only endpoints already present in main are used (/api/files/list and
 * /api/chat/history/sessions). No assistant/ or gbgov-only API is required.
 */
"use strict";

var DRIVE_CONV_FOLDER_LIMIT = 80;
var DRIVE_HISTORY_SESSION_LIMIT = 20;
var DRIVE_HISTORY_MESSAGE_LIMIT = 100;
var driveConversationRequest = 0;
var driveAttachmentRequest = 0;

function driveParityT(key, fallback) {
    if (typeof window.gbT === "function") {
        var text = window.gbT(key, fallback);
        if (text) return text;
    }
    if (typeof gbT === "function") return gbT(key, fallback);
    return fallback;
}

function driveParityBot() {
    return window.__INITIAL_BOT_NAME__ || "default";
}

// A local:home bucket is a filesystem alias, not an S3 bucket; never send it
// to the object-store listing the conversation folders live in.
function driveParityBucket() {
    var bucket = getEffectiveBucket();
    return bucket && !parseLocalBucket(bucket) ? bucket : "";
}

function driveListParams(path) {
    var params = new URLSearchParams();
    params.set("scope", "bot");
    params.set("path", path);
    var bucket = driveParityBucket();
    if (bucket) params.set("bucket", bucket);
    return params.toString();
}

function driveChatRequest(path) {
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    var headers = {};
    if (token) headers.Authorization = "Bearer " + token;
    return fetch(path, { headers: headers }).then(function (response) {
        if (!response.ok) {
            var error = new Error("HTTP " + response.status);
            error.status = response.status;
            throw error;
        }
        return response.json();
    });
}

// Older folder names are a title slug, sometimes suffixed with an 8-hex
// conversation id. Strip the id and turn the slug back into readable words.
function driveConversationTitle(folder) {
    var name = String(folder || "")
        .replace(/-[0-9a-f]{8}$/i, "")
        .replace(/^conversation$/i, "");
    return name ? name.replace(/-/g, " ") : driveParityT("sidebar-conversations", "Conversations");
}

function driveGroupedRows(groups, attachmentMode) {
    var html = "";
    for (var i = 0; i < groups.length; i++) {
        var group = groups[i];
        html += '<div class="drive-conv-group">' +
            '<div class="drive-conv-group-head" role="heading" aria-level="3">' +
            '<span class="drive-conv-title">' + escapeHtml(group.title) + '</span>' +
            '<span class="drive-conv-count">' + group.files.length + '</span></div>';
        for (var j = 0; j < group.files.length; j++) {
            var f = group.files[j];
            var path = f.path || f.name;
            var bucket = f.bucket || "";
            var badge = typeof driveFileBadge === "function" ? driveFileBadge(f.name, false) : "";
            var sub = attachmentMode && f.conversation
                ? '<span class="drive-conv-row-sub">' + escapeHtml(f.conversation) + "</span>" : "";
            var downloadTarget = attachmentMode ? "downloadDriveAttachment" : "downloadFile";
            var downloadArg = attachmentMode ? f.name : path;
            html += '<div class="drive-file-item drive-conv-row" data-path="' + escapeHtml(path) +
                '" data-name="' + escapeHtml(f.name) + '" data-bucket="' + escapeHtml(bucket) +
                '" data-type="file" data-size="' + (f.size || 0) + '" data-drive-badge="1"' +
                (attachmentMode ? ' data-drive-attachment="1"' : "") + ">" +
                '<div class="file-col file-name-col">' + badge + "<span>" + escapeHtml(f.name) + "</span>" + sub + "</div>" +
                '<div class="file-col file-modified-col">' + (f.modified ? escapeHtml(String(f.modified).slice(0, 10)) : "\u2014") + "</div>" +
                '<div class="file-col file-size-col">' + formatFileSize(f.size) + "</div>" +
                '<div class="file-col file-actions-col"><button class="btn-icon-sm" title="' +
                escapeHtml(driveParityT("action-download", "Download")) +
                '" onclick="event.stopPropagation(); ' + downloadTarget + "('" + escapeJs(downloadArg) +
                "', '" + escapeJs(bucket) + "')\">\u2193</button></div></div>";
        }
        html += "</div>";
    }
    return html;
}

function driveParityEmpty(title, content) {
    content.innerHTML = '<div class="empty-state drive-conv-empty" role="status" aria-live="polite">' +
        '<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">' +
        '<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>' +
        "<h3>" + escapeHtml(title) + "</h3></div>";
}

function driveParityLoading(content) {
    content.innerHTML = '<div class="loading-state" role="status" aria-live="polite" aria-busy="true">' +
        '<div class="spinner" aria-hidden="true"></div><p>' +
        escapeHtml(driveParityT("drive-loading-files", "Loading files...")) + "</p></div>";
}

function driveParityCrumbs(label) {
    var crumb = document.querySelector(".breadcrumb, .drive-breadcrumb");
    if (crumb) crumb.innerHTML = '<span class="breadcrumb-current">' + escapeHtml(label) + "</span>";
}

async function loadConversationFilesTab() {
    var mine = ++driveConversationRequest;
    var content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    currentTab = "conversations";
    currentPath = "";
    currentScope = "bot";
    driveParityCrumbs(driveParityT("sidebar-conversations", "Conversations"));
    driveParityLoading(content);
    if (!currentBucket) await discoverBuckets();
    try {
        var root = driveParityBot() + ".gbdrive/conversations";
        var folders = await apiRequest("/list?" + driveListParams(root));
        var dirs = (Array.isArray(folders) ? folders : []).filter(function (f) { return f.is_dir; })
            .slice(0, DRIVE_CONV_FOLDER_LIMIT);
        var groups = await Promise.all(dirs.map(function (dir) {
            return apiRequest("/list?" + driveListParams(dir.path)).then(function (files) {
                return {
                    title: driveConversationTitle(dir.name),
                    files: (Array.isArray(files) ? files : []).filter(function (f) { return !f.is_dir; })
                };
            }).catch(function () { return { title: driveConversationTitle(dir.name), files: [] }; });
        }));
        if (mine !== driveConversationRequest) return;
        groups = groups.filter(function (g) { return g.files.length; });
        if (!groups.length) {
            driveParityEmpty(driveParityT("chat-no-conversations-yet", "No conversations yet"), content);
            return;
        }
        content.innerHTML = '<div class="drive-conv-list">' + driveGroupedRows(groups, false) + "</div>";
        bindFileEvents();
        updateSelectionBar();
    } catch (err) {
        if (mine !== driveConversationRequest) return;
        driveParityEmpty(driveParityT("chat-no-conversations-yet", "No conversations yet"), content);
        showNotification(driveParityT("drive-list-error-title", "This list could not be loaded") +
            ": " + (err && err.message ? err.message : "unavailable"), "warning");
    }
}

// Collect attachments from the authenticated user's recent chat history. One
// unreadable conversation is skipped; it must never blank out the whole list.
async function fetchDriveChatAttachments() {
    var bot = driveParityBot();
    var sessionsData = await driveChatRequest("/api/chat/history/sessions?limit=50&bot_name=" + encodeURIComponent(bot));
    var sessions = (sessionsData && sessionsData.sessions) || [];
    var seen = {};
    var files = [];
    var slice = sessions.slice(0, DRIVE_HISTORY_SESSION_LIMIT);
    for (var i = 0; i < slice.length; i++) {
        var session = slice[i];
        var sid = session.session_id || session.id;
        if (!sid) continue;
        try {
            var data = await driveChatRequest("/api/chat/history/sessions/" + encodeURIComponent(sid) +
                "/messages?limit=" + DRIVE_HISTORY_MESSAGE_LIMIT + "&offset=0");
            var messages = (data && data.messages) || [];
            for (var m = 0; m < messages.length; m++) {
                var message = messages[m];
                if (message.role !== "user" || !message.attachments) continue;
                for (var a = 0; a < message.attachments.length; a++) {
                    var att = message.attachments[a];
                    var key = att.drive_path || att.name;
                    if (!key || seen[key]) continue;
                    seen[key] = true;
                    files.push({
                        name: att.name || key.split("/").pop(),
                        path: att.drive_path || ("inbox/" + att.name),
                        size: att.size || 0,
                        modified: session.updated_at,
                        conversation: session.title || "",
                        is_dir: false
                    });
                }
            }
        } catch (err) {
            if (window.console) console.warn("[drive] attachments for one conversation unavailable", err);
        }
    }
    return files;
}

async function loadChatAttachmentsTab() {
    var mine = ++driveAttachmentRequest;
    var content = document.getElementById("drive-content") || document.getElementById("file-grid");
    if (!content) return;
    currentTab = "attachments";
    currentPath = "";
    currentScope = "bot";
    driveParityCrumbs(driveParityT("email-attachments", "Attachments"));
    driveParityLoading(content);
    try {
        var files = await fetchDriveChatAttachments();
        if (mine !== driveAttachmentRequest) return;
        if (!files.length) {
            driveParityEmpty(driveParityT("drive-state-no-recent-files", "No recent files"), content);
            return;
        }
        var groups = [{ title: driveParityT("email-attachments", "Attachments"), files: files }];
        content.innerHTML = '<div class="drive-conv-list">' + driveGroupedRows(groups, true) + "</div>";
        bindFileEvents();
        updateSelectionBar();
    } catch (err) {
        if (mine !== driveAttachmentRequest) return;
        driveParityEmpty(driveParityT("drive-state-no-recent-files", "No recent files"), content);
        showNotification(driveParityT("drive-list-error-title", "This list could not be loaded") +
            ": " + (err && err.message ? err.message : "unavailable"), "warning");
    }
}

// Inbox attachments are resolved by the chat preview endpoint, not by the
// generic files download-binary used for regular Drive rows.
async function downloadDriveAttachment(name) {
    try {
        var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
        var headers = {};
        if (token) headers.Authorization = "Bearer " + token;
        var response = await fetch("/api/chat/attachments/preview?download=1&bot=" +
            encodeURIComponent(driveParityBot()) + "&name=" + encodeURIComponent(name), { headers: headers });
        if (!response.ok) throw new Error("HTTP " + response.status);
        var blob = await response.blob();
        if (window.GBShared && typeof window.GBShared.saveBlob === "function") {
            await window.GBShared.saveBlob(blob, name);
        } else {
            var url = URL.createObjectURL(blob);
            var a = document.createElement("a");
            a.href = url;
            a.download = name;
            document.body.appendChild(a);
            a.click();
            a.remove();
            URL.revokeObjectURL(url);
        }
    } catch (err) {
        showNotification(driveParityT("chat-parity-attach-download-failed", "Could not download the attachment.") +
            " " + (err && err.message ? err.message : ""), "error");
    }
}

// Inbox rows must not be opened through /api/files/download-inline, which
// cannot resolve the bot's inbox layout; download them instead.
document.addEventListener("dblclick", function (event) {
    var row = event.target && event.target.closest ? event.target.closest('[data-drive-attachment="1"]') : null;
    if (!row) return;
    event.stopPropagation();
    downloadDriveAttachment(row.getAttribute("data-name"));
}, true);

window.DriveConversationFiles = {
    loadConversations: loadConversationFilesTab,
    loadAttachments: loadChatAttachmentsTab,
    title: driveConversationTitle,
    listParams: driveListParams,
    downloadAttachment: downloadDriveAttachment,
    fetchAttachments: fetchDriveChatAttachments
};
