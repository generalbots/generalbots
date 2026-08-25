function vibeAuthFetch(path, options) {
    options = options || {};
    options.headers = Object.assign({}, options.headers || {});
    var token =
        localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        localStorage.getItem("management_token") ||
        "";
    if (token) options.headers.Authorization = "Bearer " + token;
    return fetch(path, options);
}

function esc(text) {
var d = document.createElement("div");
d.textContent = text || "";
return d.innerHTML;
}

function vibeAddMsg(role, text) {
var box = document.getElementById("vibeChatMessages");
if (!box) return;
var div = document.createElement("div");
if (role === "user") {
div.style.cssText =
"align-self:flex-end;background:var(--accent);color:var(--surface);font-weight:500;padding:10px 14px;border-radius:12px 12px 0 12px;max-width:85%;word-wrap:break-word;";
div.textContent = text;
} else if (role === "system") {
div.style.cssText =
"align-self:center;background:rgba(132,214,105,0.12);color: var(--accent);padding:6px 12px;border-radius:8px;font-size:11px;text-align:center;";
div.innerHTML = text;
} else {
div.style.cssText =
"align-self:flex-start;background:var(--surface-hover);color:var(--text);padding:10px 14px;border-radius:12px 12px 12px 0;max-width:85%;word-wrap:break-word;";
div.className = "vibe-bot-msg";
if (typeof marked !== "undefined" && marked.parse) {
div.innerHTML = marked.parse(text);
} else {
div.textContent = text;
}
}
box.appendChild(div);
box.scrollTop = box.scrollHeight;
return div;
}

function vibeAddStreamStart() {
vibeStreamId = "vibe-stream-" + Date.now();
vibeStreamContent = "";
var el = vibeAddMsg("bot", "▍");
if (el) el.id = vibeStreamId;
return el;
}

function vibeUpdateStream(content) {
vibeStreamContent += content || "";
var el = document.getElementById(vibeStreamId);
if (!el) return;
if (typeof marked !== "undefined" && marked.parse) {
el.innerHTML = marked.parse(vibeStreamContent);
} else {
el.textContent = vibeStreamContent;
}
var box = document.getElementById("vibeChatMessages");
if (box) box.scrollTop = box.scrollHeight;
}

function vibeFinalizeStream() {
var el = document.getElementById(vibeStreamId);
if (el) {
if (typeof marked !== "undefined" && marked.parse) {
el.innerHTML = marked.parse(vibeStreamContent);
} else {
el.textContent = vibeStreamContent;
}
el.removeAttribute("id");
}
vibeStreamId = null;
vibeStreamContent = "";
vibeStreaming = false;
}

/* ----------------------------------------------------------------
 * Floating Vibe windows — drag (header) + resize (corner grip).
 * Applies to the chat overlay, run dock, graph panel and metrics
 * panel so every panel is movable/resizable like a desktop window.
 * ---------------------------------------------------------------- */
function vibeMakeWindowDraggable(el, handleSel) {
    if (!el || el.dataset.vibeDrag) return; // already wired
    el.dataset.vibeDrag = "1";
    var handle = handleSel ? el.querySelector(handleSel) : el;
    if (!handle) handle = el;
    handle.style.cursor = "move";
    var startX = 0, startY = 0, origLeft = 0, origTop = 0, dragging = false, moved = false;
    handle.addEventListener("mousedown", function (e) {
        if (e.button !== 0) return;
        // Ignore drags that begin on buttons/inputs/links inside the header.
        if (e.target.closest("button,input,select,a,textarea")) return;
        dragging = true;
        moved = false;
        startX = e.clientX;
        startY = e.clientY;
        // Switch to explicit left/top coordinates (panels start at
        // bottom/right anchored); keep the current position.
        var rect = el.getBoundingClientRect();
        origLeft = rect.left;
        origTop = rect.top;
        el.style.left = rect.left + "px";
        el.style.top = rect.top + "px";
        el.style.right = "auto";
        el.style.bottom = "auto";
        e.preventDefault();
    });
    document.addEventListener("mousemove", function (e) {
        if (!dragging) return;
        var dx = e.clientX - startX;
        var dy = e.clientY - startY;
        if (!moved && Math.hypot(dx, dy) > 4) moved = true;
        if (!moved) return;
        el.style.left = Math.max(0, origLeft + dx) + "px";
        el.style.top = Math.max(0, origTop + dy) + "px";
    });
    document.addEventListener("mouseup", function () {
        dragging = false;
    });
    // Suppress the click (e.g. the run dock's collapse toggle) after an
    // actual drag — a drag must not toggle the panel.
    handle.addEventListener("click", function (e) {
        if (moved) {
            e.preventDefault();
            e.stopPropagation();
            moved = false;
        }
    }, true);
}

function vibeMakeWindowResizable(el, minW, minH) {
    if (!el || el.dataset.vibeResize) return; // already wired
    el.dataset.vibeResize = "1";
    minW = minW || 240;
    minH = minH || 160;
    var grip = document.createElement("div");
    grip.style.cssText =
        "position:absolute;right:0;bottom:0;width:16px;height:16px;cursor:se-resize;" +
        "background:linear-gradient(135deg,transparent 50%,rgba(255,255,255,0.18) 50%);" +
        "border-bottom-right-radius:10px;z-index:5;";
    el.style.position = "absolute";
    el.appendChild(grip);
    var startX = 0, startY = 0, origW = 0, origH = 0, resizing = false;
    grip.addEventListener("mousedown", function (e) {
        if (e.button !== 0) return;
        resizing = true;
        startX = e.clientX;
        startY = e.clientY;
        origW = el.offsetWidth;
        origH = el.offsetHeight;
        e.preventDefault();
        e.stopPropagation();
    });
    document.addEventListener("mousemove", function (e) {
        if (!resizing) return;
        el.style.width = Math.max(minW, origW + (e.clientX - startX)) + "px";
        el.style.height = Math.max(minH, origH + (e.clientY - startY)) + "px";
    });
    document.addEventListener("mouseup", function () {
        resizing = false;
    });
}

// Panels living inside a floating tool window are dragged/resized by the
// WindowManager (VB6-style tool windows) — skip them here to avoid fighting
// the window chrome. Only panels still inside the main vibe window get the
// legacy in-window drag/resize wiring.
function isRelocated(el) {
    if (!el) return true;
    if (el.classList && el.classList.contains("vibe-tool-relocated")) return true;
    var p = el.parentElement;
    while (p) {
        if (/^window-body-/.test(p.id || "")) return true;
        p = p.parentElement;
    }
    return false;
}

function vibeWireWindowPanels() {
    if (!isRelocated(document.getElementById("vibeChatOverlay"))) {
        vibeMakeWindowDraggable(document.getElementById("vibeChatOverlay"), ".vibe-chat-header, #vibeChatOverlay > div:first-child");
        vibeMakeWindowResizable(document.getElementById("vibeChatOverlay"), 300, 240);
    }
    if (!isRelocated(document.getElementById("vibeRunDock"))) {
        vibeMakeWindowDraggable(document.getElementById("vibeRunDock"), ".vibe-rd-handle");
        vibeMakeWindowResizable(document.getElementById("vibeRunDock"), 260, 200);
    }
    if (!isRelocated(document.getElementById("vibeGraphPanel"))) {
        vibeMakeWindowDraggable(document.getElementById("vibeGraphPanel"), "#vibeGraphPanel > div:first-child");
        vibeMakeWindowResizable(document.getElementById("vibeGraphPanel"), 320, 240);
    }
    if (!isRelocated(document.getElementById("vibeMetricsPanel"))) {
        vibeMakeWindowDraggable(document.getElementById("vibeMetricsPanel"), "#vibeMetricsPanel > div:first-child");
        vibeMakeWindowResizable(document.getElementById("vibeMetricsPanel"), 320, 240);
    }
}

function setVibeStatus(status) {
var dot = document.getElementById("vibeChatStatusDot");
var badge = document.getElementById("vibeChatStatusBadge");
if (status === "connected") {
if (dot) {
dot.className = "as-status-dot green";
dot.style.boxShadow = "0 0 8px var(--accent)";
}
if (badge) {
badge.textContent = "ONLINE";
badge.style.background = "var(--accent)";
badge.style.color = "var(--bg)";
}
} else if (status === "connecting") {
if (dot) {
dot.className = "as-status-dot yellow";
dot.style.boxShadow = "0 0 8px #f59e0b";
}
if (badge) {
badge.textContent = "CONNECTING…";
badge.style.background = "var(--surface-hover)";
badge.style.color = "var(--text-muted)";
}
} else {
if (dot) {
dot.className = "as-status-dot red";
dot.style.boxShadow = "0 0 8px #ef4444";
}
if (badge) {
badge.textContent = "OFFLINE";
badge.style.background = "var(--surface-hover)";
badge.style.color = "var(--text-muted)";
}
}
}
