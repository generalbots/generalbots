/**
 * Vibe Base (Browser) dialog — real CDP session.
 * Creates a session via /api/browser/session, navigates, and renders
 * live screenshots from /api/browser/session/:id/screenshot.
 */
(function () {
    "use strict";

    var D = window.VibeDialogs;
    var state = { sessionId: null, url: "http://localhost:3000/", busy: false };
    var refreshTimer = null;

    function main() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-main";

        var toolbar = D.el("div", "vibe-dialog-toolbar");
        var url = D.el("input", "vibe-input");
        url.id = "vibeBrowserUrl";
        url.value = state.url;
        url.placeholder = "https://...";
        url.style.flex = "1";
        url.addEventListener("keydown", function (e) {
            if (e.key === "Enter") navigate();
        });
        var go = D.el("button", "vibe-btn primary", "Go");
        go.addEventListener("click", navigate);
        var shot = D.el("button", "vibe-btn", "📸 Screenshot");
        shot.addEventListener("click", screenshot);

        var frame = D.el("div", "vibe-browser-frame");
        frame.id = "vibeBrowserFrame";

        var status = D.el("div", "vibe-browser-status");
        status.innerHTML = '<span class="vibe-status info" id="vibeBrowserState">no session</span>' +
            '<span style="flex:1"></span><span id="vibeBrowserInfo">—</span>';

        toolbar.appendChild(url);
        toolbar.appendChild(go);
        toolbar.appendChild(shot);
        box.appendChild(toolbar);
        box.appendChild(frame);
        box.appendChild(status);
        return box;
    }

    function sidebar() {
        var box = document.createElement("div");
        box.className = "vibe-dialog-sidebar";

        var head = D.el("div", "vibe-dialog-title");
        head.textContent = "SESSION";

        var actions = D.el("div", "vibe-commit-box");
        var sessionBtn = D.el("button", "vibe-btn primary", "▶ New Session");
        sessionBtn.id = "vibeBrowserSessionBtn";
        sessionBtn.addEventListener("click", createSession);
        var closeBtn = D.el("button", "vibe-btn danger", "✕ Close Session");
        closeBtn.id = "vibeBrowserCloseBtn";
        closeBtn.style.display = "none";
        closeBtn.addEventListener("click", closeSession);
        actions.appendChild(sessionBtn);
        actions.appendChild(closeBtn);

        var log = D.el("div", "vibe-list");
        log.id = "vibeBrowserLog";
        log.innerHTML = '<div class="vibe-empty">Create a session to start browsing. ' +
            "The agent can navigate, click and extract content.</div>";

        box.appendChild(head);
        box.appendChild(actions);
        box.appendChild(log);
        return box;
    }

    function logLine(text, cls) {
        var log = document.getElementById("vibeBrowserLog");
        if (!log) return;
        var row = D.el("div", "vibe-list-item");
        row.innerHTML = '<span class="vibe-status ' + (cls || "info") + '">' + text + "</span>";
        log.insertBefore(row, log.firstChild);
        while (log.children.length > 30) log.removeChild(log.lastChild);
    }

    function createSession() {
        var btn = document.getElementById("vibeBrowserSessionBtn");
        var frame = document.getElementById("vibeBrowserFrame");
        var stateEl = document.getElementById("vibeBrowserState");
        if (btn) { btn.disabled = true; btn.textContent = "Creating..."; }
        if (frame) frame.innerHTML = '<div class="vibe-empty">Creating headless session...</div>';
        D.api("/api/browser/session", {
            method: "POST",
            body: { headless: true },
        }).then(function (data) {
            if (data && data.id) {
                state.sessionId = data.id;
                if (btn) { btn.disabled = false; btn.textContent = "Restart Session"; }
                var closeBtn = document.getElementById("vibeBrowserCloseBtn");
                if (closeBtn) closeBtn.style.display = "";
                if (stateEl) {
                    stateEl.textContent = "session active";
                    stateEl.className = "vibe-status ok";
                }
                logLine("session created", "ok");
                navigate();
            } else {
                if (btn) { btn.disabled = false; btn.textContent = "▶ New Session"; }
                if (stateEl) { stateEl.textContent = "create failed"; stateEl.className = "vibe-status err"; }
                logLine("create failed: " + ((data && data.error) || "unknown"), "err");
            }
        }).catch(function (err) {
            if (frame) frame.innerHTML = '<div class="vibe-empty">Session error: ' + D.esc(err) + "</div>";
            if (btn) { btn.disabled = false; btn.textContent = "▶ New Session"; }
            logLine("error: " + err, "err");
        });
    }

    function navigate() {
        if (!state.sessionId) return createSession();
        var url = document.getElementById("vibeBrowserUrl");
        state.url = url ? url.value.trim() : state.url;
        if (!state.url) return;
        state.busy = true;
        var stateEl = document.getElementById("vibeBrowserState");
        if (stateEl) { stateEl.textContent = "navigating..."; stateEl.className = "vibe-status warn"; }
        D.api("/api/browser/session/" + encodeURIComponent(state.sessionId) + "/navigate", {
            method: "POST",
            body: { url: state.url },
        }).then(function (data) {
            logLine("→ " + state.url, "info");
            screenshot();
        }).catch(function (err) {
            logLine("navigate error: " + err, "err");
            state.busy = false;
            if (stateEl) { stateEl.textContent = "navigate failed"; stateEl.className = "vibe-status err"; }
        });
    }

    function screenshot() {
        if (!state.sessionId) return;
        var frame = document.getElementById("vibeBrowserFrame");
        var info = document.getElementById("vibeBrowserInfo");
        var stateEl = document.getElementById("vibeBrowserState");
        D.api("/api/browser/session/" + encodeURIComponent(state.sessionId) + "/screenshot", {
            method: "GET",
        }).then(function (data) {
            state.busy = false;
            if (data && data.image_base64) {
                frame.innerHTML = '<img src="data:image/png;base64,' + data.image_base64 + '" alt="browser screenshot" />';
                if (info) info.textContent = "png " + Math.round((data.size_bytes || 0) / 1024) + " KB";
                if (stateEl) { stateEl.textContent = "live"; stateEl.className = "vibe-status ok"; }
            } else {
                frame.innerHTML = '<div class="vibe-empty">No screenshot yet — ' + D.esc((data && data.error) || "navigate first") + "</div>";
                if (stateEl) { stateEl.textContent = "pending"; stateEl.className = "vibe-status warn"; }
            }
        }).catch(function (err) {
            state.busy = false;
            frame.innerHTML = '<div class="vibe-empty">Screenshot error: ' + D.esc(err) + "</div>";
            if (stateEl) { stateEl.textContent = "error"; stateEl.className = "vibe-status err"; }
        });
    }

    function closeSession() {
        if (!state.sessionId) return;
        D.api("/api/browser/session/" + encodeURIComponent(state.sessionId), {
            method: "DELETE",
        }).then(function () {
            state.sessionId = null;
            var frame = document.getElementById("vibeBrowserFrame");
            if (frame) frame.innerHTML = '<div class="vibe-empty">Session closed.</div>';
            var stateEl = document.getElementById("vibeBrowserState");
            if (stateEl) { stateEl.textContent = "closed"; stateEl.className = "vibe-status info"; }
            var btn = document.getElementById("vibeBrowserSessionBtn");
            if (btn) btn.textContent = "▶ New Session";
            var closeBtn = document.getElementById("vibeBrowserCloseBtn");
            if (closeBtn) closeBtn.style.display = "none";
        }).catch(function () {
            state.sessionId = null;
        });
    }

    D.register("browser", {
        build: function (body) {
            body.appendChild(sidebar());
            body.appendChild(main());
        },
        teardown: function () {
            if (refreshTimer) { clearInterval(refreshTimer); refreshTimer = null; }
        },
    });
})();