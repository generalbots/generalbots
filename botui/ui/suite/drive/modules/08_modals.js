/* Drive Module v2.0 — 08 Modals: preview, editor, share, new-bot */
"use strict";

function showPreviewModal(fileName, ext, blob) {
    var modal = document.getElementById("preview-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "preview-modal";
    modal.style.cssText = "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(15, 23, 42, 0.85); display: flex; align-items: center; justify-content: center; z-index: 9999; backdrop-filter: blur(8px);";
    const container = document.createElement("div");
    container.style.cssText = "background: #1e293b; border: 1px solid #334155; border-radius: 12px; width: 80%; max-width: 900px; max-height: 90vh; display: flex; flex-direction: column; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5); overflow: hidden;";
    const header = document.createElement("div");
    header.style.cssText = "display: flex; align-items: center; justify-content: space-between; padding: 16px 24px; border-bottom: 1px solid #334155;";
    header.innerHTML = '<h3 style="margin:0; color:#f8fafc; font-size:1.125rem;">Preview: ' + escapeHtml(fileName) + '</h3>';
    const closeBtn = document.createElement("button");
    closeBtn.textContent = "\u00D7";
    closeBtn.style.cssText = "background: none; border: none; color: #94a3b8; font-size: 28px; cursor: pointer; line-height: 1; padding: 0; margin: 0;";
    var objectUrl;
    closeBtn.onclick = function() { modal.remove(); if (objectUrl) URL.revokeObjectURL(objectUrl); };
    header.appendChild(closeBtn);
    const body = document.createElement("div");
    body.style.cssText = "padding: 24px; overflow-y: auto; flex-grow: 1; display: flex; align-items: center; justify-content: center; background: #0f172a;";
    objectUrl = URL.createObjectURL(blob);
    var previewEl;
    if (["png", "jpg", "jpeg", "gif", "webp"].indexOf(ext) !== -1) {
        previewEl = document.createElement("img");
        previewEl.src = objectUrl;
        previewEl.style.cssText = "max-width:100%; max-height:70vh; object-fit:contain; border-radius:4px;";
    } else if (ext === "pdf") {
        previewEl = document.createElement("iframe");
        previewEl.src = objectUrl;
        previewEl.style.cssText = "width:100%; height:70vh; border:none; border-radius:4px;";
    } else if (["mp4", "webm"].indexOf(ext) !== -1) {
        previewEl = document.createElement("video");
        previewEl.src = objectUrl;
        previewEl.controls = true;
        previewEl.autoplay = true;
        previewEl.style.cssText = "max-width:100%; max-height:70vh; border-radius:4px;";
    } else if (["mp3", "wav", "ogg"].indexOf(ext) !== -1) {
        previewEl = document.createElement("audio");
        previewEl.src = objectUrl;
        previewEl.controls = true;
        previewEl.autoplay = true;
        previewEl.style.cssText = "width:100%; max-width:500px;";
    } else {
        previewEl = document.createElement("pre");
        previewEl.style.cssText = "color:#f8fafc; font-family:monospace; font-size:14px; white-space:pre-wrap; width:100%; max-height:70vh; overflow-y:auto; margin:0;";
        blob.text().then(function(t) { previewEl.textContent = t.substring(0, 10000); }).catch(function() { previewEl.textContent = "Preview not available for this file type."; });
    }
    body.appendChild(previewEl);
    container.appendChild(header);
    container.appendChild(body);
    modal.appendChild(container);
    document.body.appendChild(modal);
}

function shareFile(path) {
    var fileName = path.split("/").pop() || "file";
    var shareUrl = window.location.origin + "/api/files/download?bucket=" + encodeURIComponent(getEffectiveBucket()) + "&path=" + encodeURIComponent(path) + "&scope=" + currentScope;
    var modal = document.getElementById("share-modal");
    if (modal) modal.remove();
    modal = document.createElement("div");
    modal.id = "share-modal";
    modal.style.cssText = "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(15, 23, 42, 0.85); display: flex; align-items: center; justify-content: center; z-index: 9999; backdrop-filter: blur(8px);";
    const container = document.createElement("div");
    container.style.cssText = "background: #1e293b; border: 1px solid #334155; border-radius: 12px; width: 90%; max-width: 500px; padding: 24px; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);";
    container.innerHTML = '<div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:20px;"><h3 style="margin:0; color:#f8fafc; font-size:1.25rem;">Share "' + escapeHtml(fileName) + '"</h3><button onclick="document.getElementById(\'share-modal\').remove()" style="background:none; border:none; color:#94a3b8; font-size:24px; cursor:pointer;">\u00D7</button></div><p style="color:#94a3b8; font-size:14px; margin-bottom:16px;">Anyone with this link will be able to download the file directly.</p><div style="display:flex; gap:8px; margin-bottom:20px;"><input type="text" id="share-link-input" readonly value="' + shareUrl + '" style="flex-grow:1; background:#0f172a; border:1px solid #334155; border-radius:6px; color:#f8fafc; padding:10px 14px; font-size:14px;" /><button onclick="DriveModule.copyShareLink()" style="background:#3b82f6; border:none; border-radius:6px; color:#fff; padding:0 16px; cursor:pointer; font-weight:500; font-size:14px;">Copy</button></div><div style="display:flex; justify-content:flex-end; gap:12px;"><button onclick="document.getElementById(\'share-modal\').remove()" style="background:#334155; border:none; border-radius:6px; color:#f8fafc; padding:10px 20px; cursor:pointer; font-weight:500; font-size:14px;">Close</button></div>';
    modal.appendChild(container);
    document.body.appendChild(modal);
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
