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
        openMedia(streamUrl, type);
    }

    // #1971 — a viewer must never display the endpoint's JSON error body. The
    // document frame used to be pointed straight at the stream URL, so any
    // non-file answer (404 JSON, an auth error, a metadata object) was painted
    // on screen as raw JSON. Fetch first, inspect the content type, and only
    // then hand a real object URL to the viewer.
    async function openMedia(url, type) {
        try {
            const response = await fetch(url, { headers: playerAuthHeaders() });
            if (!response.ok) {
                throw new Error("HTTP " + response.status);
            }
            const contentType = (response.headers.get("content-type") || "").toLowerCase();
            if (contentType.indexOf("json") !== -1) {
                throw new Error("this file cannot be previewed in the player");
            }
            const blob = await response.blob();
            const blobUrl = URL.createObjectURL(blob);
            if (type === "image") showImageViewer(blobUrl);
            else if (type === "video") showVideoPlayer(blobUrl);
            else if (type === "audio") showAudioPlayer(blobUrl);
            else showDocumentViewer(blobUrl);
        } catch (err) {
            showPlayerError(
                "Could not open " +
                    (currentFile && currentFile.name ? currentFile.name : "this file") +
                    ": " +
                    String(err.message || err),
            );
        }
    }

    function showPlayerError(message) {
        hideAllPlayers();
        const empty = document.getElementById("empty-state");
        if (!empty) return;
        empty.classList.remove("hidden");
        const title = empty.querySelector("h2");
        if (title) title.textContent = "Could not play this file";
        const hint = empty.querySelector("p");
        if (hint) hint.textContent = message;
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
        video.onerror = function () {
            showPlayerError("This video format cannot be played here. Download it to open in a media app.");
        };
        video.src = url;
        player.classList.remove("hidden");
        document.getElementById("playback-controls").classList.remove("hidden");
        currentPlayer = video;
        setupMediaEvents(video);
    }

    function showAudioPlayer(url) {
        const player = document.getElementById("audio-player");
        const audio = document.getElementById("audio-element");
        audio.onerror = function () {
            showPlayerError("This audio format cannot be played here. Download it to open in a media app.");
        };
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
        img.onerror = function () {
            showPlayerError("This image could not be decoded. Download it to open it locally.");
        };
        img.src = url;
        viewer.classList.remove("hidden");
    }

    function showPresentationViewer(url) {
        // #1971 — this used to show an empty deck with a "1 / 1" counter and
        // never load the file. Render the slides through the document viewer.
        showDocumentViewer(url);
        const viewer = document.getElementById("presentation-viewer");
        if (viewer) viewer.classList.add("hidden");
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
        // #1978 — themed SVG instead of the speaker emoji.
        document.getElementById("volume-icon").innerHTML = volumeIcon(currentPlayer.muted);
    }

    function setVolume(value) {
        if (!currentPlayer) return;
        currentPlayer.volume = value / 100;
        document.getElementById("volume-icon").innerHTML = volumeIcon(value == 0);
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

    function togglePlaylistPanel() {
        const panel = document.getElementById("playlist-panel");
        panel.classList.toggle("collapsed");
    }

    // ---------------------------------------------------------------------
