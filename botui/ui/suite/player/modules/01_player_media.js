    let currentFile = null;
    let currentPlayer = null;
    let currentSlide = 0;
    let totalSlides = 1;
    let driveSource = null;

    document.addEventListener("DOMContentLoaded", loadFromUrlParams);

    async function loadFromUrlParams() {
        const urlParams = new URLSearchParams(window.location.search);
        const hash = window.location.hash;
        let bucket = urlParams.get("bucket");
        let path = urlParams.get("path");

        if (hash) {
            const hashQueryIndex = hash.indexOf("?");
            if (hashQueryIndex !== -1) {
                const hashParams = new URLSearchParams(
                    hash.substring(hashQueryIndex + 1),
                );
                bucket = bucket || hashParams.get("bucket");
                path = path || hashParams.get("path");
            }
        }

        if (bucket && path) {
            await loadFromDrive(bucket, path);
        }
    }

    async function loadFromDrive(bucket, path) {
        const fileName = path.split("/").pop() || "media";
        const ext = fileName.split(".").pop().toLowerCase();

        driveSource = { bucket, path };

        const audioExts = [
            "mp3",
            "wav",
            "ogg",
            "oga",
            "flac",
            "aac",
            "m4a",
            "wma",
            "aiff",
            "aif",
        ];
        const type = audioExts.includes(ext) ? "audio" : "video";

        hideAllPlayers();
        currentFile = { path, type, name: fileName };

        document.getElementById("file-name").textContent = fileName;
        document.getElementById("btn-download").disabled = false;
        document.getElementById("btn-share").disabled = false;
        document.getElementById("btn-fullscreen").disabled = false;
        document.getElementById("btn-add-playlist").disabled = false;

        try {
            const response = await fetch("/api/files/download", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ bucket, path }),
            });

            if (!response.ok) {
                throw new Error(`Failed to load file: ${response.status}`);
            }

            const blob = await response.blob();
            const url = URL.createObjectURL(blob);

            if (type === "audio") {
                showAudioPlayer(url);
            } else {
                showVideoPlayer(url);
            }
        } catch (err) {
            console.error("Failed to load file from drive:", err);
            document.getElementById("empty-state").classList.remove("hidden");
        }
    }

    function loadFile(path, type, name) {
        hideAllPlayers();
        currentFile = { path, type, name };

        document.getElementById("file-name").textContent = name;
        document.getElementById("btn-download").disabled = false;
        document.getElementById("btn-share").disabled = false;
        document.getElementById("btn-fullscreen").disabled = false;
        document.getElementById("btn-add-playlist").disabled = false;

        const streamUrl = `/api/player/default/stream/${path}`;

        switch (type) {
            case "video":
                showVideoPlayer(streamUrl);
                break;
            case "audio":
                showAudioPlayer(streamUrl);
                break;
            case "document":
            case "pdf":
                showDocumentViewer(streamUrl);
                break;
            case "image":
                showImageViewer(streamUrl);
                break;
            case "presentation":
                showPresentationViewer(streamUrl);
                break;
            default:
                showDocumentViewer(streamUrl);
        }
    }

    function hideAllPlayers() {
        document.getElementById("video-player").classList.add("hidden");
        document.getElementById("audio-player").classList.add("hidden");
        document.getElementById("document-viewer").classList.add("hidden");
        document.getElementById("image-viewer").classList.add("hidden");
        document.getElementById("presentation-viewer").classList.add("hidden");
        document.getElementById("empty-state").classList.add("hidden");
        document.getElementById("playback-controls").classList.add("hidden");
    }

    function showVideoPlayer(url) {
        const player = document.getElementById("video-player");
        const video = document.getElementById("video-element");
        video.src = url;
        player.classList.remove("hidden");
        document.getElementById("playback-controls").classList.remove("hidden");
        currentPlayer = video;
        setupMediaEvents(video);
    }

    function showAudioPlayer(url) {
        const player = document.getElementById("audio-player");
        const audio = document.getElementById("audio-element");
        audio.src = url;
        player.classList.remove("hidden");
        document.getElementById("playback-controls").classList.remove("hidden");
        currentPlayer = audio;
        setupMediaEvents(audio);
    }

    function showDocumentViewer(url) {
        const viewer = document.getElementById("document-viewer");
        const frame = document.getElementById("document-frame");
        frame.src = url;
        viewer.classList.remove("hidden");
    }

    function showImageViewer(url) {
        const viewer = document.getElementById("image-viewer");
        const img = document.getElementById("image-element");
        img.src = url;
        viewer.classList.remove("hidden");
    }

    function showPresentationViewer(url) {
        const viewer = document.getElementById("presentation-viewer");
        viewer.classList.remove("hidden");
        currentSlide = 0;
        updateSlideCounter();
    }

    function setupMediaEvents(media) {
        media.addEventListener("timeupdate", updateProgress);
        media.addEventListener("loadedmetadata", function () {
            document.getElementById("total-time").textContent = formatTime(
                media.duration,
            );
            document.getElementById("file-duration").textContent = formatTime(
                media.duration,
            );
        });
        media.addEventListener("play", function () {
            document.getElementById("play-icon").textContent = "⏸";
        });
        media.addEventListener("pause", function () {
            document.getElementById("play-icon").textContent = "▶";
        });
    }

    function updateProgress() {
        if (!currentPlayer) return;
        const progress =
            (currentPlayer.currentTime / currentPlayer.duration) * 100;
        document.getElementById("progress-fill").style.width = progress + "%";
        document.getElementById("progress-handle").style.left = progress + "%";
        document.getElementById("current-time").textContent = formatTime(
            currentPlayer.currentTime,
        );
    }

    function formatTime(seconds) {
        if (isNaN(seconds)) return "0:00";
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, "0")}`;
    }

    function togglePlayPause() {
        if (!currentPlayer) return;
        if (currentPlayer.paused) {
            currentPlayer.play();
        } else {
            currentPlayer.pause();
        }
    }

    function skipBackward() {
        if (!currentPlayer) return;
        currentPlayer.currentTime = Math.max(0, currentPlayer.currentTime - 10);
    }

    function skipForward() {
        if (!currentPlayer) return;
        currentPlayer.currentTime = Math.min(
            currentPlayer.duration,
            currentPlayer.currentTime + 10,
        );
    }

    function toggleMute() {
        if (!currentPlayer) return;
        currentPlayer.muted = !currentPlayer.muted;
        document.getElementById("volume-icon").textContent = currentPlayer.muted
            ? "🔇"
            : "🔊";
    }

    function setVolume(value) {
        if (!currentPlayer) return;
        currentPlayer.volume = value / 100;
        document.getElementById("volume-icon").textContent =
            value == 0 ? "🔇" : "🔊";
    }

    function setPlaybackSpeed(speed) {
        if (!currentPlayer) return;
        currentPlayer.playbackRate = parseFloat(speed);
    }

    function prevSlide() {
        if (currentSlide > 0) {
            currentSlide--;
            updateSlideCounter();
        }
    }

    function nextSlide() {
        if (currentSlide < totalSlides - 1) {
            currentSlide++;
            updateSlideCounter();
        }
    }

    function updateSlideCounter() {
        document.getElementById("slide-counter").textContent =
            `${currentSlide + 1} / ${totalSlides}`;
    }

    function toggleFullscreen() {
        const container = document.getElementById("player-container");
        if (document.fullscreenElement) {
            document.exitFullscreen();
        } else {
            container.requestFullscreen();
        }
    }

    function downloadFile() {
        if (!currentFile) return;
        const link = document.createElement("a");
        link.href = `/api/player/default/stream/${currentFile.path}`;
        link.download = currentFile.name;
        link.click();
    }

    function shareFile() {
        if (!currentFile) return;
        const url = `${window.location.origin}/player?file=${encodeURIComponent(currentFile.path)}`;
        navigator.clipboard.writeText(url);
        alert("Link copied to clipboard!");
    }

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

    function kindIcon(kind) {
        return kind === "audio" ? "🎵" : kind === "video" ? "🎬" : kind === "image" ? "🖼️" : "📄";
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
            const empty = document.getElementById("empty-state");
            if (empty) {
                empty.classList.remove("hidden");
                const title = empty.querySelector("h2");
                if (title) title.textContent = "Could not play this file";
                const hint = empty.querySelector("p");
                if (hint) hint.textContent = String(err.message || err);
            }
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
    // The desktop injects app HTML after the page has loaded, so listening for
    // DOMContentLoaded alone would never fire there.
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", loadMediaList);
    } else {
        loadMediaList();
    }

    function togglePlaylistPanel() {
        const panel = document.getElementById("playlist-panel");
        panel.classList.toggle("collapsed");
    }

    // ---------------------------------------------------------------------
