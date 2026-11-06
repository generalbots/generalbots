    // Playlists
    // ---------------------------------------------------------------------

    let currentPlaylist = null;
    let playlistQueue = [];
    let queueIndex = 0;

    function apiFetch(url, options) {
        options = options || {};
        options.headers = Object.assign({ "Content-Type": "application/json" }, options.headers || {});
        const token =
            (typeof window.getAccessToken === "function" && window.getAccessToken()) ||
            sessionStorage.getItem("gb_access_token") ||
            localStorage.getItem("gb_access_token");
        if (token) options.headers.Authorization = "Bearer " + token;
        return fetch(url, options).then((resp) => {
            if (!resp.ok) {
                return resp
                    .json()
                    .catch(() => ({}))
                    .then((body) => {
                        throw new Error((body && body.error) || "HTTP " + resp.status);
                    });
            }
            return resp.json();
        });
    }

    function currentUserId() {
        try {
            const auth = JSON.parse(
                sessionStorage.getItem("gb_auth_user") || localStorage.getItem("gb_auth_user") || "null",
            );
            if (auth && auth.id) return auth.id;
        } catch (e) {
            /* ignore */
        }
        return null;
    }

    async function createPlaylist() {
        const name = window.prompt("Playlist name:");
        if (!name || !name.trim()) return;
        try {
            const playlist = await apiFetch("/api/player/playlists", {
                method: "POST",
                body: JSON.stringify({ name: name.trim(), visibility: "private" }),
            });
            currentPlaylist = playlist;
            await loadPlaylists();
            openPlaylist(playlist.id);
        } catch (err) {
            console.error("Failed to create playlist:", err);
            alert("Failed to create playlist: " + err.message);
        }
    }

    async function loadPlaylists() {
        const container = document.getElementById("playlist-items");
        if (!container) return;
        try {
            const lists = await apiFetch("/api/player/playlists?user_id=" + encodeURIComponent(currentUserId() || ""));
            if (!lists || !lists.length) {
                container.innerHTML = '<p class="empty-message">No playlist created</p>';
                return;
            }
            container.innerHTML =
                lists
                    .map(
                        (p) =>
                            '<div class="playlist-row" data-id="' +
                            p.id +
                            '" onclick="openPlaylist(\'' +
                            p.id +
                            '\')">' +
                            '<span class="playlist-row-name">' +
                            escapeHtml(p.name) +
                            "</span>" +
                            '<span class="playlist-row-count">' +
                            p.item_count +
                            "</span>" +
                            '<button class="playlist-row-delete" title="Delete" onclick="event.stopPropagation();deletePlaylist(\'' +
                            p.id +
                            ')">🗑</button>' +
                            "</div>",
                    )
                    .join("");
        } catch (err) {
            console.error("Failed to load playlists:", err);
            container.innerHTML = '<p class="empty-message">Failed to load playlists</p>';
        }
    }

    async function openPlaylist(playlistId) {
        try {
            const playlist = await apiFetch("/api/player/playlists/" + playlistId);
            currentPlaylist = playlist;
            const container = document.getElementById("playlist-items");
            if (!container) return;
            container.innerHTML =
                '<div class="playlist-header">' +
                '<strong>' +
                escapeHtml(playlist.name) +
                "</strong>" +
                '<button class="btn-secondary btn-sm" onclick="renamePlaylist()">✏️</button>' +
                '<button class="btn-secondary btn-sm" onclick="playPlaylist()">▶ Play</button>' +
                "</div>" +
                (playlist.items && playlist.items.length
                    ? playlist.items
                          .map(
                              (item, index) =>
                                  '<div class="playlist-item" data-id="' +
                                  item.id +
                                  '" draggable="true" ondragstart="dragItem(event)" ondragover="event.preventDefault()" ondrop="dropItem(event)">' +
                                  '<span class="playlist-item-index">' +
                                  (index + 1) +
                                  "</span>" +
                                  '<span class="playlist-item-title" onclick="playPlaylistItem(' +
                                  index +
                                  ')">' +
                                  escapeHtml(item.title) +
                                  "</span>" +
                                  '<button class="playlist-item-remove" onclick="event.stopPropagation();removePlaylistItem(\'' +
                                  item.id +
                                  ')">✕</button>' +
                                  "</div>",
                          )
                          .join("")
                    : '<p class="empty-message">Empty playlist</p>') +
                '<button class="btn-secondary btn-sm" style="margin-top:8px;width:100%" onclick="addCurrentToPlaylist()">+ Add current media</button>';
        } catch (err) {
            console.error("Failed to open playlist:", err);
        }
    }

    async function renamePlaylist() {
        if (!currentPlaylist) return;
        const name = window.prompt("New playlist name:", currentPlaylist.name);
        if (!name || !name.trim()) return;
        try {
            await apiFetch("/api/player/playlists/" + currentPlaylist.id, {
                method: "PATCH",
                body: JSON.stringify({ name: name.trim() }),
            });
            await loadPlaylists();
            await openPlaylist(currentPlaylist.id);
        } catch (err) {
            console.error("Failed to rename playlist:", err);
        }
    }

    async function deletePlaylist(playlistId) {
        if (!window.confirm("Delete this playlist?")) return;
        try {
            await apiFetch("/api/player/playlists/" + playlistId, { method: "DELETE" });
            if (currentPlaylist && currentPlaylist.id === playlistId) {
                currentPlaylist = null;
            }
            await loadPlaylists();
        } catch (err) {
            console.error("Failed to delete playlist:", err);
        }
    }

    async function addCurrentToPlaylist() {
        if (!currentFile) {
            alert("Load a media file first");
            return;
        }
        if (!currentPlaylist) {
            await createPlaylist();
            if (!currentPlaylist) return;
        }
        const playlistId = currentPlaylist.id;
        try {
            await apiFetch("/api/player/playlists/" + playlistId + "/items", {
                method: "POST",
                body: JSON.stringify({
                    media_path: currentFile.path,
                    title: currentFile.name || currentFile.path,
                }),
            });
            await openPlaylist(playlistId);
        } catch (err) {
            console.error("Failed to add item:", err);
            alert("Failed to add to playlist: " + err.message);
        }
    }

    async function removePlaylistItem(itemId) {
        if (!currentPlaylist) return;
        try {
            await apiFetch(
                "/api/player/playlists/" + currentPlaylist.id + "/items/" + itemId,
                { method: "DELETE" },
            );
            await openPlaylist(currentPlaylist.id);
        } catch (err) {
            console.error("Failed to remove item:", err);
        }
    }

    function playPlaylist() {
        if (!currentPlaylist || !currentPlaylist.items || !currentPlaylist.items.length) return;
        playlistQueue = currentPlaylist.items.slice();
        queueIndex = 0;
        playQueueItem(0);
    }

    function playPlaylistItem(index) {
        if (!currentPlaylist || !currentPlaylist.items) return;
        playlistQueue = currentPlaylist.items.slice();
        queueIndex = index;
        playQueueItem(index);
    }

    function playQueueItem(index) {
        const item = playlistQueue[index];
        if (!item) return;
        queueIndex = index;
        const title = item.title || item.media_path;
        const ext = title.split(".").pop().toLowerCase();
        const type = ["mp3", "wav", "ogg", "oga", "flac", "aac", "m4a"].includes(ext)
            ? "audio"
            : "video";
        loadFile(item.media_path, type, title);
        recordPlayback(item);
    }

    function nextQueueItem() {
        if (!playlistQueue.length) return;
        playQueueItem((queueIndex + 1) % playlistQueue.length);
    }

    function recordPlayback(item) {
        apiFetch("/api/player/playbacks", {
            method: "POST",
            body: JSON.stringify({
                item_id: item.id || null,
                media_path: item.media_path,
                event_type: "play",
                position_seconds: 0,
            }),
        }).catch(() => {
            /* analytics are best-effort */
        });
    }

    function dragItem(event) {
        const row = event.target.closest(".playlist-item");
        if (row) event.dataTransfer.setData("text/plain", row.dataset.id);
    }

    async function dropItem(event) {
        event.preventDefault();
        if (!currentPlaylist) return;
        const draggedId = event.dataTransfer.getData("text/plain");
        const target = event.target.closest(".playlist-item");
        if (!target || !draggedId) return;
        const items = currentPlaylist.items.slice();
        const from = items.findIndex((i) => i.id === draggedId);
        const to = items.findIndex((i) => i.id === target.dataset.id);
        if (from === -1 || to === -1 || from === to) return;
        const [moved] = items.splice(from, 1);
        items.splice(to, 0, moved);
        try {
            await apiFetch("/api/player/playlists/" + currentPlaylist.id + "/reorder", {
                method: "PUT",
                body: JSON.stringify({ item_ids: items.map((i) => i.id) }),
            });
            await openPlaylist(currentPlaylist.id);
        } catch (err) {
            console.error("Failed to reorder:", err);
        }
    }

    function escapeHtml(text) {
        return String(text)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");
    }

    // Load playlists on startup and when the panel opens.
    document.addEventListener("DOMContentLoaded", loadPlaylists);

    // Wire the panel toggle to refresh the list.
    const _origTogglePlaylistPanel = togglePlaylistPanel;
    window.togglePlaylistPanel = function () {
        _origTogglePlaylistPanel();
        loadPlaylists();
    };

    // Auto-advance to the next queue item when the current one ends.
    document.addEventListener("ended", function (e) {
        if (e.target && (e.target.tagName === "VIDEO" || e.target.tagName === "AUDIO")) {
            nextQueueItem();
        }
    });

    document.addEventListener("keydown", function (e) {
        if (!currentPlayer) return;

        switch (e.key) {
            case " ":
                e.preventDefault();
                togglePlayPause();
                break;
            case "ArrowLeft":
                skipBackward();
                break;
            case "ArrowRight":
                skipForward();
                break;
            case "ArrowUp":
                e.preventDefault();
                setVolume(Math.min(100, currentPlayer.volume * 100 + 10));
                break;
            case "ArrowDown":
                e.preventDefault();
                setVolume(Math.max(0, currentPlayer.volume * 100 - 10));
                break;
            case "f":
                toggleFullscreen();
                break;
            case "m":
                toggleMute();
                break;
            case "Escape":
                if (document.fullscreenElement) {
                    document.exitFullscreen();
                }
                break;
        }
    });

    document
        .querySelector(".progress-track")
        .addEventListener("click", function (e) {
            if (!currentPlayer) return;
            const rect = this.getBoundingClientRect();
            const percent = (e.clientX - rect.left) / rect.width;
            currentPlayer.currentTime = percent * currentPlayer.duration;
        });
