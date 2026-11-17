/* Drive → "Use in chat" (#2568).
 *
 * The assistant's Drive has a context-menu action that sends a file straight
 * into the conversation; the classic Drive only offered the reverse direction
 * (the chat composer's Drive picker). This module closes that gap: download the
 * file as a Blob, then hand it to the chat composer's existing file input so
 * the normal send path (multi-file payload, staging, preview) is unchanged.
 *
 * The chat app lives in the same desktop document, so when it is already open
 * the composer input is right there. Otherwise the chat deep link is opened and
 * the input is waited for, with a bounded wait so a closed chat never leaves a
 * hanging promise.
 */
"use strict";

var USE_IN_CHAT_WAIT_MS = 12000;
var USE_IN_CHAT_POLL_MS = 400;

function useInChatMessage(key, fallback) {
    if (typeof window.gbT === "function") {
        var text = window.gbT(key, fallback);
        if (text) return text;
    }
    return fallback;
}

function useInChatNotify(text, kind) {
    if (typeof showNotification === "function") {
        showNotification(text, kind || "success");
    }
}

function chatComposerInput() {
    return document.getElementById("messageFile");
}

function addFilesToChatComposer(files) {
    var input = chatComposerInput();
    if (!input || !files.length) return false;
    try {
        var transfer = new DataTransfer();
        Array.prototype.slice.call(input.files || []).forEach(function (existing) {
            transfer.items.add(existing);
        });
        files.forEach(function (file) {
            transfer.items.add(file);
        });
        input.files = transfer.files;
        input.dispatchEvent(new Event("change", { bubbles: true }));
        return true;
    } catch (e) {
        return false;
    }
}

function waitForChatComposer(files, deadline) {
    var input = chatComposerInput();
    if (input) {
        finishChatHandoff(files, addFilesToChatComposer(files));
        return;
    }
    if (Date.now() > deadline) {
        useInChatNotify(useInChatMessage("drive-use-in-chat-timeout",
            "Open the chat to attach the file."), "error");
        return;
    }
    setTimeout(function () { waitForChatComposer(files, deadline); }, USE_IN_CHAT_POLL_MS);
}

// The chat composer only paints its "📎 name" chip from a `change` listener
// bound when the chat app boots. Opened lazily from here, that listener can be
// bound to an earlier instance, so fill the chip when it is still empty — the
// input itself always carries the file for the send path.
function revealChatFileChip(files) {
    var chip = document.getElementById("fileChip");
    if (!chip || chip.style.display !== "none" || chip.textContent.trim()) return;
    var list = Array.prototype.slice.call(files || []);
    if (!list.length) return;
    chip.textContent = "📎 " + list.map(function (file) { return file.name; }).join(", ");
    chip.title = list.map(function (file) { return file.name; }).join("\n");
    chip.style.display = "inline-flex";
}

function finishChatHandoff(files, attached) {
    if (!attached) {
        useInChatNotify(useInChatMessage("drive-use-in-chat-failed",
            "Could not attach the file to the conversation."), "error");
        return;
    }
    revealChatFileChip(files);
    var names = files.map(function (file) { return file.name; }).join(", ");
    useInChatNotify(useInChatMessage("drive-use-in-chat-attached",
        "Attached to the conversation: ") + names, "success");
    var input = chatComposerInput();
    if (input && typeof input.focus === "function") input.focus();
}

async function useInChat(path, bucket) {
    var name = path.split("/").pop() || "file";
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token");
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;

    useInChatNotify(useInChatMessage("drive-use-in-chat-preparing", "Preparing ") + name + "…", "info");

    var composerAlreadyOpen = !!chatComposerInput();
    if (!composerAlreadyOpen && typeof window.openDeepLink === "function") {
        try {
            window.openDeepLink("chat");
        } catch (e) {
            /* the chat may already be the active app */
        }
    }

    try {
        var response = await fetch(API_BASE + "/download-binary", {
            method: "POST",
            headers: headers,
            body: JSON.stringify({
                bucket: bucket || getEffectiveBucket(),
                path: path,
                scope: currentScope
            })
        });
        if (!response.ok) throw new Error("HTTP " + response.status);
        var blob = await response.blob();
        var type = blob.type || "application/octet-stream";
        var file = new File([blob], name, { type: type });
        waitForChatComposer([file], Date.now() + USE_IN_CHAT_WAIT_MS);
    } catch (err) {
        useInChatNotify(useInChatMessage("drive-use-in-chat-failed",
            "Could not attach the file to the conversation."), "error");
        console.error("[drive] useInChat failed:", err);
    }
}
