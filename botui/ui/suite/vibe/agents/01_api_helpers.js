/* Vibe Agents sidebar — split from vibe/vibe-agents.js for #1731 */

/**
 * Vibe sidebar — real data (2026-08-14 cleanup).
 *
 * The previous implementation simulated a "vibe farm": hardcoded Vibe #1-#4
 * agent cards (EVOLVED/BRED/WILD badges), DOM-only workspace items and
 * drag-and-drop assignment. None of it had a backend, so it was removed.
 *
 * The sidebar now lists real projects from /api/vibe/projects. Selecting a
 * project dispatches `gb:vibe-project` (Members + dialogs react), and
 * "+ Create a New Project" opens the real New Project modal.
 */
"use strict";

function authHeaders(extra) {
    if (window.GBShared && window.GBShared.auth) return window.GBShared.auth.headers(extra, { legacy: true });
    var headers = Object.assign({}, extra || {});
    var token =
        (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
        localStorage.getItem("token") ||
        localStorage.getItem("id_token") ||
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        "";
    if (token) headers["Authorization"] = "Bearer " + token;
    return headers;
}

function vibeApi(path, options) {
    options = options || {};
    options.headers = authHeaders(options.headers || {});
    if (options.body && typeof options.body !== "string") {
        options.headers["Content-Type"] = "application/json";
        options.body = JSON.stringify(options.body);
    }
    return fetch(path, options).then(function (r) {
        return r.json().catch(function () {
            return { success: false, error: "HTTP " + r.status };
        });
    });
}

function esc(s) {
    var d = document.createElement("div");
    d.textContent = s == null ? "" : String(s);
    return d.innerHTML;
}
