/* Drive Module v2.0 — 08 Modals (editor): revoke/copy link, editor, new bot. */
"use strict";

// Split from 08_modals.js for #1731. Loaded right after the modals file.

async function revokeShareLink(token, path) {
    if (!token) return;
    if (!confirm("Revoke this link? Anyone holding it will lose access immediately.")) return;
    try {
        await apiRequest("/revoke-link", {
            method: "POST",
            body: JSON.stringify({
                token: token,
                bucket: getEffectiveBucket(),
                path: path,
                scope: currentScope,
            }),
        });
        var row = document.getElementById("share-link-row");
        if (row) {
            row.style.display = "none";
            row.dataset.token = "";
        }
        var createBtn = document.getElementById("share-create-btn");
        if (createBtn) createBtn.style.display = "inline-block";
        var status = document.getElementById("share-status");
        if (status) status.textContent = "Link revoked. Create a new one if you still need to share.";
        showNotification("Link revoked", "success");
    } catch (error) {
        showNotification("Revoke failed: " + (error.message || "unknown error"), "error");
    }
}

async function revokeShareLinkFromModal(path) {
    var row = document.getElementById("share-link-row");
    var token = row ? row.dataset.token : "";
    if (!token) return;
    await revokeShareLink(token, path);
}

function closeModal(modalId) {
    var el = document.getElementById(modalId);
    if (el) el.remove();
}

function copyShareLink() {
    const input = document.getElementById("share-link-input");
    if (input) { input.select(); document.execCommand("copy"); showNotification("Share link copied to clipboard!", "success"); }
}

function showEditorModal(path, fileName, content) {
    var modal = document.getElementById("editor-modal");
    if (modal) modal.remove();
    const ext = (fileName.split(".").pop() || "txt").toLowerCase();
    modal = document.createElement("div");
    modal.id = "editor-modal";
    modal.className = "modal-overlay";
    var headerHtml = '<div class="editor-header"><div class="editor-title"><span class="editor-icon">\uD83D\uDCDD</span><span class="editor-filename">' + escapeHtml(fileName) + '</span><span class="editor-status" id="editor-status"></span></div><div class="editor-actions"><button class="btn-secondary" onclick="DriveModule.closeEditor()">Cancel</button><button class="btn-primary" onclick="DriveModule.saveEditorContent()"><span>\uD83D\uDCBE</span> Save</button></div></div>';
    var bodyHtml = '<div class="editor-body"><textarea id="editor-textarea" class="editor-textarea" spellcheck="false" data-path="' + escapeHtml(path) + '" data-ext="' + ext + '"></textarea></div>';
    var footerHtml = '<div class="editor-footer"><span class="editor-info">Line: <span id="editor-line">1</span>, Col: <span id="editor-col">1</span></span><span class="editor-info">' + ext.toUpperCase() + '</span></div>';
    modal.innerHTML = '<div class="modal-content editor-modal-content">' + headerHtml + bodyHtml + footerHtml + '</div>';
    document.body.appendChild(modal);
    const textarea = document.getElementById("editor-textarea");
    if (textarea) { textarea.value = content || ""; textarea.focus(); }
    textarea.addEventListener("input", function() { document.getElementById("editor-status").textContent = "\u25CF Modified"; });
    textarea.addEventListener("click", updateEditorCursor);
    textarea.addEventListener("keyup", updateEditorCursor);
    textarea.addEventListener("keydown", function(e) {
        if (e.key === "s" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); saveEditorContent(); }
        if (e.key === "Escape") { closeEditor(); }
        if (e.key === "Tab") {
            e.preventDefault();
            var start = textarea.selectionStart;
            var end = textarea.selectionEnd;
            textarea.value = textarea.value.substring(0, start) + "  " + textarea.value.substring(end);
            textarea.selectionStart = textarea.selectionEnd = start + 2;
        }
    });
    modal.addEventListener("click", function(e) { if (e.target === modal) closeEditor(); });
}

function updateEditorCursor() {
    const textarea = document.getElementById("editor-textarea");
    if (!textarea) return;
    const text = textarea.value.substring(0, textarea.selectionStart);
    const lines = text.split("\n");
    document.getElementById("editor-line").textContent = lines.length;
    document.getElementById("editor-col").textContent = lines[lines.length - 1].length + 1;
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
            body: JSON.stringify({ bucket: getEffectiveBucket(), path: path, content: content, scope: currentScope }),
        });
        statusEl.textContent = "\u2713 Saved";
        if (typeof saveFileVersion === "function") saveFileVersion(path, content);
        showNotification("File saved successfully", "success");
        setTimeout(function() { if (statusEl) statusEl.textContent = ""; }, 2000);
    } catch (err) {
        statusEl.textContent = "\u2717 Save failed";
        showNotification("Failed to save: " + err.message, "error");
    }
}

function closeEditor() {
    const modal = document.getElementById("editor-modal");
    const statusEl = document.getElementById("editor-status");
    if (statusEl && statusEl.textContent.indexOf("Modified") !== -1) {
        if (!confirm("You have unsaved changes. Close anyway?")) return;
    }
    if (modal) modal.remove();
}

function openNewBotModal() {
    var modal = document.getElementById("new-bot-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "new-bot-modal";
    modal.className = "modal-overlay";
    modal.innerHTML = '<div class="modal-content" style="max-width:480px">'
        + '<div class="editor-header"><div class="editor-title"><span class="editor-icon">\uD83E\uDD16</span><span class="editor-filename">Create New Bot</span></div><button class="btn-icon" onclick="closeNewBotModal()" style="background:none;border:none;color:#94a3b8;font-size:24px;cursor:pointer;">\u00D7</button></div>'
        + '<div style="padding:24px">'
        + '<label style="display:block;margin-bottom:8px;color:var(--text-secondary);font-size:14px">Bot Name</label>'
        + '<input type="text" id="new-bot-name" placeholder="my-new-bot" style="width:100%;padding:10px 14px;background:#0f172a;border:1px solid var(--border);border-radius:6px;color:#f8fafc;font-size:14px;box-sizing:border-box;margin-bottom:12px" oninput="document.getElementById(\'new-bot-preview\').textContent=this.value + \'.gbai\'" />'
        + '<p style="color:var(--text-secondary);font-size:12px;margin-bottom:16px">Lowercase letters, numbers, and hyphens only (3-50 chars). Creates bucket: <code id="new-bot-preview" style="color:var(--primary)">my-new-bot.gbai</code></p>'
        + '<div style="display:flex;gap:8px;justify-content:flex-end">'
        + '<button class="btn-secondary" onclick="closeNewBotModal()">Cancel</button>'
        + '<button class="btn-primary" id="confirm-new-bot-btn" onclick="confirmNewBot()">Create Bot</button>'
        + '</div></div></div>';
    document.body.appendChild(modal);
    document.getElementById("new-bot-name").focus();
}

function closeNewBotModal() {
    var modal = document.getElementById("new-bot-modal");
    if (modal) modal.remove();
}

async function confirmNewBot() {
    var name = document.getElementById("new-bot-name").value.trim();
    if (!name) { showNotification("Enter a bot name", "error"); return; }
    var btn = document.getElementById("confirm-new-bot-btn");
    btn.disabled = true;
    btn.textContent = "Creating...";
    await createNewBot(name);
    btn.disabled = false;
    btn.textContent = "Create Bot";
}

async function sendShareEmail(path) {
    var to = document.getElementById("share-email-to");
    var sender = document.getElementById("share-email-sender");
    var message = document.getElementById("share-email-message");
    var status = document.getElementById("share-email-status");
    var button = document.getElementById("share-email-send");
    if (!to || !to.value.trim()) {
        if (status) status.textContent = "Type at least one recipient email.";
        return;
    }
    if (button) button.disabled = true;
    if (status) status.textContent = "Sending…";
    try {
        var payload = await apiRequest("/share/email", {
            method: "POST",
            body: JSON.stringify({
                bucket: getEffectiveBucket(),
                path: path,
                scope: currentScope,
                origin: window.location.origin,
                recipients: [to.value],
                message: message ? message.value : "",
                expires_at: shareExpiryIso(),
                account_id: sender && !sender.disabled ? sender.value : "",
            }),
        });
        if (status) {
            status.textContent = "Sent to " + (payload.recipients || []).join(", ") +
                (payload.failed ? " (" + payload.failed + " failed)" : "") + ".";
        }
        if (window.GBToasts) GBToasts.show("Email sent", "The share link was emailed.", "success");
    } catch (error) {
        if (status) status.textContent = error.message || "The email could not be sent.";
    } finally {
        if (button) button.disabled = false;
    }
}
