/* Drive Module v2.0 — 03 Utils */
"use strict";

function escapeHtml(str) {
    if (!str) return "";
    return String(str)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}

function escapeJs(str) {
    if (!str) return "";
    return String(str)
        .replace(/\\\\/g, "\\\\\\\\")
        .replace(/'/g, "\\\\'")
        .replace(/"/g, '\\\\"');
}

function formatFileSize(bytes) {
    if (!bytes || bytes === 0) return "0 B";
    const units = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + " " + units[i];
}

function formatDate(dateStr) {
    if (!dateStr) return "";
    const d = new Date(dateStr);
    return d.toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
    });
}

function getFileTypeClass(filename) {
    const ext = (filename || "").split(".").pop().toLowerCase();
    const types = {
        document: ["doc", "docx", "pdf", "txt", "rtf", "odt"],
        image: ["jpg", "jpeg", "png", "gif", "svg", "webp", "bmp"],
        video: ["mp4", "avi", "mov", "mkv", "webm"],
        audio: ["mp3", "wav", "ogg", "flac", "aac"],
        archive: ["zip", "rar", "7z", "tar", "gz"],
        code: ["js", "ts", "py", "rs", "go", "java", "c", "cpp", "h", "html", "css", "json", "xml"],
    };
    for (const [type, exts] of Object.entries(types)) {
        if (exts.includes(ext)) return type;
    }
    return "file";
}

function getFolderIcon() {
    return '<svg width="20" height="20" viewBox="0 0 24 24" fill="#5f6368" stroke="none"><path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg>';
}

function getFileIcon(filename) {
    const ext = (filename || "").split(".").pop().toLowerCase();
    const colors = {
        pdf: "#ea4335", doc: "#4285f4", docx: "#4285f4",
        xls: "#0f9d58", xlsx: "#0f9d58",
        ppt: "#fbbc04", pptx: "#fbbc04",
    };
    const color = colors[ext] || "#5f6368";
    return '<svg width="20" height="20" viewBox="0 0 24 24" fill="' + color + '" stroke="none"><path d="M14,2H6A2,2 0 0,0 4,4V20A2,2 0 0,0 6,22H18A2,2 0 0,0 20,20V8L14,2M18,20H6V4H13V9H18V20Z"/></svg>';
}

function getGborgIcon() {
    return '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>';
}

function showNotification(message, type) {
    const existing = document.querySelector(".drive-notification");
    if (existing) existing.remove();
    const notification = document.createElement("div");
    notification.className = "drive-notification notification-" + (type || "info");
    notification.textContent = message;
    notification.style.cssText = "position:fixed;bottom:20px;right:20px;padding:12px 20px;border-radius:8px;background:#333;color:#fff;z-index:9999;animation:slideIn 0.3s ease;";
    if (type === "error") notification.style.background = "#ef4444";
    if (type === "success") notification.style.background = "#22c55e";
    if (type === "warning") notification.style.background = "#f59e0b";
    document.body.appendChild(notification);
    setTimeout(() => notification.remove(), 4000);
}

function readFileAsBase64(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => {
            const base64 = reader.result.split(",")[1] || reader.result;
            resolve(base64);
        };
        reader.onerror = reject;
        reader.readAsDataURL(file);
    });
}

function getEffectiveBucket() {
    if (currentGborgBucket) return currentGborgBucket;
    if (currentBucket) return currentBucket;
    return undefined;
}

function getGbaiDirName() {
    if (currentGborgBranch) return currentGborgBranch + ".gbai/";
    return "";
}
