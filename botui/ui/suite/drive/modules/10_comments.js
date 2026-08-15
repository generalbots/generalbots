/* Drive Module v2.1 — 10 Comments & presence sidebar
 * Threaded comments with @mentions and emoji reactions on the selected file,
 * plus live presence (who is viewing / typing). Backed by /api/collab/*. */
"use strict";

let commentsPanel = null;
let commentsList = null;
let commentsInput = null;
let commentsFile = null;   // resource_id for the open file
let commentsTimer = null;  // presence heartbeat interval
let commentsTypingSent = false;

function commentsResourceId() {
    return (getEffectiveBucket() || "default") + ":" + (commentsFile || "");
}

// Direct fetch to the cross-app collab API (outside /api/files/*).
async function collabRequest(endpoint, options) {
    const headers = { "Content-Type": "application/json" };
    const token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    if (token) headers["Authorization"] = "Bearer " + token;
    const res = await fetch("/api/collab" + endpoint, {
        headers: headers,
        ...(options || {}),
    });
    if (!res.ok) {
        const errBody = await res.json().catch(function() { return { error: res.statusText }; });
        throw new Error(errBody.error || "Request failed");
    }
    return res.json();
}

function commentsOpen(path) {
    if (!commentsPanel) buildCommentsPanel();
    commentsFile = path || null;
    commentsPanel.classList.add("comments-panel-visible");
    commentsPanel.classList.remove("comments-panel-closed");
    loadComments();
    startCommentsHeartbeat();
}

function commentsClose() {
    if (commentsPanel) {
        commentsPanel.classList.remove("comments-panel-visible");
        commentsPanel.classList.add("comments-panel-closed");
    }
    stopCommentsHeartbeat();
}

function stopCommentsHeartbeat() {
    if (commentsTimer) { clearInterval(commentsTimer); commentsTimer = null; }
    commentsTypingSent = false;
}

function startCommentsHeartbeat() {
    stopCommentsHeartbeat();
    if (!commentsFile) return;
    sendPresence(false);
    commentsTimer = setInterval(function() { sendPresence(commentsTypingSent); }, 30000);
}

async function sendPresence(typing) {
    if (!commentsFile) return;
    try {
        await collabRequest("/presence", {
            method: "POST",
            body: JSON.stringify({
                resource_type: "drive:file",
                resource_id: commentsResourceId(),
                typing: !!typing,
            }),
        });
    } catch (e) { /* presence is best-effort */ }
}

function commentsTime(iso) {
    try {
        const d = new Date(iso);
        return d.toLocaleString();
    } catch (e) { return iso; }
}

function escapeHtml(s) {
    return String(s)
        .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}

// Render body with @mentions highlighted.
function renderBody(body) {
    return escapeHtml(body).replace(/@([\w.@-]+)/g, '<span class="cm-mention">@$1</span>');
}

function reactionsSummary(comment) {
    if (!comment.reactions || comment.reactions.length === 0) return "";
    const counts = {};
    comment.reactions.forEach(function(r) { counts[r.emoji] = (counts[r.emoji] || 0) + 1; });
    return Object.keys(counts).map(function(e) {
        return '<span class="cm-reaction-chip" data-emoji="' + escapeHtml(e) + '" data-comment="' + comment.id + '">' + e + ' ' + counts[e] + '</span>';
    }).join("");
}

function commentNode(comment, isReply) {
    const node = document.createElement("div");
    node.className = "cm-item" + (isReply ? " cm-reply" : "");
    node.innerHTML =
        '<div class="cm-header">' +
            '<span class="cm-author">' + escapeHtml(comment.author_name || comment.author_id) + '</span>' +
            '<span class="cm-time">' + commentsTime(comment.created_at) + '</span>' +
            (comment.author_id === (localStorage.getItem("gb-user-email") || "") ? '' : '') +
        '</div>' +
        '<div class="cm-body">' + renderBody(comment.body) + '</div>' +
        '<div class="cm-actions">' +
            '<span class="cm-react" data-comment="' + comment.id + '" data-emoji="👍">👍</span>' +
            '<span class="cm-react" data-comment="' + comment.id + '" data-emoji="❤️">❤️</span>' +
            '<span class="cm-react" data-comment="' + comment.id + '" data-emoji="😄">😄</span>' +
            '<span class="cm-react" data-comment="' + comment.id + '" data-emoji="🎉">🎉</span>' +
            '<span class="cm-reply-btn" data-comment="' + comment.id + '">Reply</span>' +
            '<span class="cm-del" data-comment="' + comment.id + '">Delete</span>' +
            reactionsSummary(comment) +
        '</div>';
    if (comment.replies && comment.replies.length) {
        const repliesWrap = document.createElement("div");
        repliesWrap.className = "cm-replies";
        comment.replies.forEach(function(r) { repliesWrap.appendChild(commentNode(r, true)); });
        node.appendChild(repliesWrap);
    }
    return node;
}

async function loadComments() {
    if (!commentsFile || !commentsList) return;
    commentsList.innerHTML = '<div class="cm-loading">Loading comments\u2026</div>';
    try {
        const items = await collabRequest("/comments?resource_type=drive:file&resource_id=" + encodeURIComponent(commentsResourceId()));
        commentsList.innerHTML = "";
        if (!items || items.length === 0) {
            commentsList.innerHTML = '<div class="cm-empty">No comments yet. Be the first to leave feedback on this file.</div>';
        } else {
            items.forEach(function(c) { commentsList.appendChild(commentNode(c, false)); });
        }
        loadPresence();
    } catch (err) {
        commentsList.innerHTML = '<div class="cm-error">Failed to load comments: ' + escapeHtml(err.message) + '</div>';
    }
}

async function loadPresence() {
    const chip = document.getElementById("comments-presence");
    if (!chip || !commentsFile) return;
    try {
        const items = await collabRequest("/presence?resource_type=drive:file&resource_id=" + encodeURIComponent(commentsResourceId()));
        if (!items || items.length === 0) {
            chip.textContent = "";
            chip.style.display = "none";
            return;
        }
        const typing = items.filter(function(p) { return p.typing; });
        const names = items.map(function(p) { return p.user_name; });
        const label = typing.length > 0
            ? "typing: " + names.join(", ")
            : "viewing: " + names.join(", ");
        chip.textContent = label;
        chip.style.display = "inline-flex";
    } catch (e) { /* best-effort */ }
}

async function addComment(body, parentId) {
    const payload = {
        resource_type: "drive:file",
        resource_id: commentsResourceId(),
        body: body,
    };
    if (parentId) payload.parent_id = parentId;
    return collabRequest("/comments", {
        method: "POST",
        body: JSON.stringify(payload),
    });
}

async function postComment() {
    if (!commentsInput) return;
    const body = commentsInput.value.trim();
    if (!body) return;
    commentsInput.value = "";
    commentsTypingSent = false;
    try {
        await addComment(body, null);
        await loadComments();
    } catch (err) {
        showNotification("Comment failed: " + err.message, "error");
    }
}

async function postReply(commentId, replyBody) {
    if (!replyBody) return;
    try {
        await addComment(replyBody, commentId);
        await loadComments();
    } catch (err) {
        showNotification("Reply failed: " + err.message, "error");
    }
}

async function toggleReaction(commentId, emoji) {
    try {
        await collabRequest("/comments/" + commentId + "/reactions", {
            method: "POST",
            body: JSON.stringify({ emoji: emoji }),
        });
        await loadComments();
    } catch (err) {
        showNotification("Reaction failed: " + err.message, "error");
    }
}

async function deleteComment(commentId) {
    if (!confirm("Delete this comment?")) return;
    try {
        await collabRequest("/comments/" + commentId, { method: "DELETE" });
        await loadComments();
    } catch (err) {
        showNotification("Delete failed: " + err.message, "error");
    }
}

function buildCommentsPanel() {
    commentsPanel = document.createElement("div");
    commentsPanel.id = "comments-panel";
    commentsPanel.className = "comments-panel";
    commentsPanel.innerHTML =
        '<div class="comments-panel-header">' +
            '<span class="comments-panel-title">Comments</span>' +
            '<span id="comments-presence" class="comments-presence" style="display:none"></span>' +
            '<button class="comments-panel-close" onclick="DriveComments.close()" title="Close">\u00D7</button>' +
        '</div>' +
        '<div id="comments-list" class="comments-list"></div>' +
        '<div class="comments-input-row">' +
            '<input id="comments-input" class="comments-input" type="text" placeholder="Add a comment\u2026 use @name to mention" autocomplete="off" />' +
            '<button id="comments-send-btn" class="comments-send-btn" onclick="DriveComments.post()">Post</button>' +
        '</div>';
    document.body.appendChild(commentsPanel);

    commentsList = document.getElementById("comments-list");
    commentsInput = document.getElementById("comments-input");

    commentsInput.addEventListener("keydown", function(e) {
        if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); postComment(); return; }
        commentsTypingSent = true;
        sendPresence(true);
    });

    // Event delegation for reactions / replies / deletes.
    commentsList.addEventListener("click", function(e) {
        const reactEl = e.target.closest(".cm-react");
        if (reactEl) { toggleReaction(reactEl.dataset.comment, reactEl.dataset.emoji); return; }
        const delEl = e.target.closest(".cm-del");
        if (delEl) { deleteComment(delEl.dataset.comment); return; }
        const replyEl = e.target.closest(".cm-reply-btn");
        if (replyEl) {
            const parentId = replyEl.dataset.comment;
            const replyBody = prompt("Reply to comment:");
            if (replyBody) postReply(parentId, replyBody.trim());
        }
    });
}

window.DriveComments = {
    open: commentsOpen,
    close: commentsClose,
    post: postComment,
};
