/* Drive Module v2.1 — 09 AI Assistant panel
 * Floating chat with the Drive AI (POST /api/files/ai/chat), optionally
 * scoped to a selected file. Exposed as window.DriveAI. */
"use strict";

let aiPanel = null;
let aiInput = null;
let aiMessages = null;
let aiFileContext = null;
let aiBusy = false;

function aiToggle() {
    if (!aiPanel) buildAIPanel();
    const visible = aiPanel.classList.toggle("ai-panel-visible");
    aiPanel.classList.remove("ai-panel-closed");
    if (visible && aiInput) aiInput.focus();
}

function aiClose() {
    if (aiPanel) {
        aiPanel.classList.remove("ai-panel-visible");
        aiPanel.classList.add("ai-panel-closed");
    }
}

function aiAddMessage(role, text) {
    if (!aiMessages) return;
    const row = document.createElement("div");
    row.className = "ai-msg ai-msg-" + role;
    const label = document.createElement("div");
    label.className = "ai-msg-label";
    label.textContent = role === "user" ? "You" : "Drive AI";
    const body = document.createElement("div");
    body.className = "ai-msg-body";
    body.textContent = text;
    row.appendChild(label);
    row.appendChild(body);
    aiMessages.appendChild(row);
    aiMessages.scrollTop = aiMessages.scrollHeight;
}

function aiSetContextLabel(path) {
    const label = document.getElementById("ai-file-context");
    if (!label) return;
    if (path) {
        label.textContent = "Context: " + path;
        label.style.display = "inline-flex";
    } else {
        label.style.display = "none";
    }
}

async function aiSend() {
    if (aiBusy || !aiInput) return;
    const message = aiInput.value.trim();
    if (!message) return;
    aiInput.value = "";
    aiAddMessage("user", message);
    aiBusy = true;
    const sendBtn = document.getElementById("ai-send-btn");
    if (sendBtn) { sendBtn.disabled = true; sendBtn.textContent = "…"; }
    try {
        const payload = {
            message: message,
            bucket: getEffectiveBucket(),
            scope: currentScope,
        };
        if (aiFileContext) payload.file_path = aiFileContext;
        const resp = await apiRequest("/ai/chat", {
            method: "POST",
            body: JSON.stringify(payload),
        });
        aiAddMessage("bot", resp.reply || "(no response)");
    } catch (err) {
        aiAddMessage("bot", "Error: " + err.message);
    } finally {
        aiBusy = false;
        if (sendBtn) { sendBtn.disabled = false; sendBtn.textContent = "Send"; }
        if (aiInput) aiInput.focus();
    }
}

function buildAIPanel() {
    aiPanel = document.createElement("div");
    aiPanel.id = "ai-panel";
    aiPanel.className = "ai-panel";
    aiPanel.innerHTML =
        '<div class="ai-panel-header">' +
            '<span class="ai-panel-title">Drive AI Assistant</span>' +
            '<span id="ai-file-context" class="ai-file-context" style="display:none"></span>' +
            '<button class="ai-panel-close" onclick="DriveAI.close()" title="Close">\u00D7</button>' +
        '</div>' +
        '<div id="ai-messages" class="ai-messages"></div>' +
        '<div class="ai-input-row">' +
            '<input id="ai-input" class="ai-input" type="text" placeholder="Ask about your files\u2026 (Enter to send)" autocomplete="off" />' +
            '<button id="ai-send-btn" class="ai-send-btn" onclick="DriveAI.send()">Send</button>' +
        '</div>';
    document.body.appendChild(aiPanel);

    aiMessages = document.getElementById("ai-messages");
    aiInput = document.getElementById("ai-input");
    aiInput.addEventListener("keydown", function(e) {
        if (e.key === "Enter") { e.preventDefault(); aiSend(); }
    });

    const toggle = document.createElement("button");
    toggle.id = "ai-toggle-btn";
    toggle.className = "ai-toggle-btn";
    toggle.title = "Drive AI Assistant";
    toggle.innerHTML =
        '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">' +
        '<path d="M12 2a4 4 0 0 1 4 4 4 4 0 0 1 4 4 4 4 0 0 1-4 4 4 4 0 0 1-4 4 4 4 0 0 1-4-4 4 4 0 0 1-4-4 4 4 0 0 1 4-4 4 4 0 0 1 4-4z"></path>' +
        '<path d="M12 6v12M6 12h12"></path></svg>';
    toggle.addEventListener("click", aiToggle);
    document.body.appendChild(toggle);

    aiAddMessage("bot", "Hi! I can help you understand, summarise, or manage the files in this drive. Right-click a file and choose \u201CAsk AI\u201D for file-specific help.");
}

function openWithFile(path) {
    if (!aiPanel) buildAIPanel();
    aiFileContext = path || null;
    aiSetContextLabel(aiFileContext);
    aiPanel.classList.add("ai-panel-visible");
    aiPanel.classList.remove("ai-panel-closed");
    if (aiInput) aiInput.focus();
}

window.DriveAI = {
    toggle: aiToggle,
    close: aiClose,
    send: aiSend,
    openWithFile: openWithFile,
};
