/* Player — Drive media list and its filter/search (split from
   modules/01_player_media.js for the 450-line rule). Loaded right after it;
   classic scripts share globals, so the playback helpers above stay usable. */

    // #1953 — the sidebar used to load `/api/files/list` through HTMX, but that
    // endpoint answers JSON, so the player injected raw JSON (or nothing) and
    // looked broken. List the user's Drive media here and render it ourselves.
    let mediaCache = [];
    let mediaQuery = "";

    const KIND_BY_EXT = {
        video: ["mp4", "webm", "ogv", "mov", "mkv", "m4v"],
        audio: ["mp3", "wav", "ogg", "oga", "flac", "aac", "m4a", "opus"],
        image: ["png", "jpg", "jpeg", "gif", "webp", "svg", "avif", "bmp"],
        document: ["pdf", "txt", "md", "csv", "json", "html"],
    };

    function kindFor(name) {
        const ext = String(name || "").split(".").pop().toLowerCase();
        const kinds = Object.keys(KIND_BY_EXT);
        for (let i = 0; i < kinds.length; i++) {
            if (KIND_BY_EXT[kinds[i]].indexOf(ext) !== -1) return kinds[i];
        }
        return null;
    }

    function playerEscape(value) {
        return String(value == null ? "" : value)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;");
    }

    function playerAuthHeaders() {
        const token =
            sessionStorage.getItem("gb_access_token") ||
            localStorage.getItem("gb_access_token");
        return token ? { Authorization: "Bearer " + token } : {};
    }

    function volumeIcon(muted) {
        if (window.GBIcons) return window.GBIcons.html(muted ? "volumeOff" : "volume", 16);
        return muted ? "&#128263;" : "&#128266;";
    }

    function kindIcon(kind) {
        var name = kind === "audio" ? "music" : kind === "video" ? "film" : kind === "image" ? "image" : "file";
        if (window.GBIcons) return window.GBIcons.html(name, 15);
        return "";
    }

    function renderMediaList() {
        const list = document.getElementById("file-list");
        if (!list) return;
        const q = mediaQuery.trim().toLowerCase();
        const active = document.querySelector(".filter-btn.active");
        const type = active ? active.dataset.type : "all";
        const rows = mediaCache.filter(function (item) {
            if (type !== "all" && item.kind !== type) return false;
            return !q || item.name.toLowerCase().indexOf(q) !== -1;
        });
        if (!rows.length) {
            list.innerHTML =
                '<div class="empty-message">No ' +
                (type === "all" ? "media" : type) +
                (q ? ' matching "' + playerEscape(q) + '"' : "") +
                " in your Drive.</div>";
            return;
        }
        list.innerHTML = rows
            .map(function (item) {
                return (
                    '<div class="file-row" data-path="' +
                    playerEscape(item.path) +
                    '" data-kind="' +
                    item.kind +
                    '" title="' +
                    playerEscape(item.name) +
                    '">' +
                    '<span class="file-row-icon">' +
                    kindIcon(item.kind) +
                    "</span>" +
                    '<span class="file-row-name">' +
                    playerEscape(item.name) +
                    "</span></div>"
                );
            })
            .join("");
        list.querySelectorAll(".file-row").forEach(function (row) {
            row.addEventListener("click", function () {
                const path = row.getAttribute("data-path");
                const entry = mediaCache.filter(function (m) { return m.path === path; })[0];
                if (entry) playMediaEntry(entry);
            });
        });
        renderRecentMedia();
    }

    function renderRecentMedia() {
        const recent = document.getElementById("recent-files");
        if (!recent) return;
        if (!mediaCache.length) {
            recent.innerHTML = '<div class="empty-message">Nothing yet</div>';
            return;
        }
        recent.innerHTML = mediaCache
            .slice(0, 5)
            .map(function (item) {
                return (
                    '<div class="file-row" data-path="' +
                    playerEscape(item.path) +
                    '">' +
                    '<span class="file-row-icon">' +
                    kindIcon(item.kind) +
                    "</span>" +
                    '<span class="file-row-name">' +
                    playerEscape(item.name) +
                    "</span></div>"
                );
            })
            .join("");
        recent.querySelectorAll(".file-row").forEach(function (row) {
            row.addEventListener("click", function () {
                const path = row.getAttribute("data-path");
                const entry = mediaCache.filter(function (m) { return m.path === path; })[0];
                if (entry) playMediaEntry(entry);
            });
        });
    }

    async function loadMediaList() {
        const list = document.getElementById("file-list");
        try {
            const response = await fetch("/api/files/list?scope=user&_=" + Date.now(), {
                headers: playerAuthHeaders(),
            });
            if (!response.ok) throw new Error("HTTP " + response.status);
            const payload = await response.json();
            const items = Array.isArray(payload)
                ? payload
                : (payload && (payload.files || payload.items)) || [];
            mediaCache = items
                .filter(function (f) { return !f.is_dir; })
                .map(function (f) {
                    const name = f.name || String(f.path || "").split("/").pop();
                    return { name: name, path: f.path, kind: kindFor(name) };
                })
                .filter(function (f) { return f.kind && f.path; });
            renderMediaList();
        } catch (err) {
            if (list) {
                list.innerHTML =
                    '<div class="empty-message">Could not load your Drive media (' +
                    playerEscape(err.message) +
                    '). Check that Drive is connected and retry.</div>';
            }
        }
    }

    async function playMediaEntry(entry) {
        try {
            const response = await fetch("/api/files/download", {
                method: "POST",
                headers: Object.assign(
                    { "Content-Type": "application/json" },
                    playerAuthHeaders(),
                ),
                body: JSON.stringify({ path: entry.path, scope: "user" }),
            });
            if (!response.ok) throw new Error("HTTP " + response.status);
            const data = await response.json();
            if (!data || !data.content) throw new Error("the file was empty");
            const raw = atob(data.content);
            const bytes = new Uint8Array(raw.length);
            for (let i = 0; i < raw.length; i++) bytes[i] = raw.charCodeAt(i);
            const url = URL.createObjectURL(new Blob([bytes]));
            hideAllPlayers();
            currentFile = { path: entry.path, type: entry.kind, name: entry.name };
            document.getElementById("file-name").textContent = entry.name;
            document.getElementById("file-type").textContent = entry.kind;
            if (entry.kind === "audio") showAudioPlayer(url);
            else if (entry.kind === "image") showImageViewer(url);
            else if (entry.kind === "document") showDocumentViewer(url);
            else showVideoPlayer(url);
        } catch (err) {
            console.error("player: media load failed", err);
            showPlayerError(String(err.message || err));
        }
    }

    function filterFiles(type, query) {
        document.querySelectorAll(".filter-btn").forEach((btn) => {
            btn.classList.toggle("active", btn.dataset.type === type);
        });
        if (typeof query === "string") mediaQuery = query;
        renderMediaList();
    }

    window.filterFiles = filterFiles;
    window.loadMediaList = loadMediaList;
    // Playlists (and deep links) open a file by path; keeping this on `window`
    // is also how the JSON guard in openMedia is exercised.
    window.loadFile = loadFile;
    // The desktop injects app HTML after the page has loaded, so listening for
    // DOMContentLoaded alone would never fire there.
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", loadMediaList);
    } else {
        loadMediaList();
    }

