(function () {
    "use strict";

    if (window.GBAppLifecycle) GBAppLifecycle.begin("jukebox");
    var root = document.getElementById("jukeboxApp");
    if (!root) return;

    var STORAGE_KEY = "gb_jukebox_station_v1";
    var GENRES = {
        ambient: { label: "Ambient", prompt: "spacious ambient soundscape, evolving analog pads, soft piano fragments, subtle field textures, slow meditative movement" },
        lofi: { label: "Lo-fi Hip Hop", prompt: "dusty lo-fi hip hop beat, warm vinyl texture, mellow Rhodes chords, relaxed drums, gentle bass groove" },
        jazz: { label: "Jazz", prompt: "intimate modern jazz quartet, expressive piano, upright bass, brushed drums, lyrical saxophone improvisation" },
        classical: { label: "Classical", prompt: "contemporary classical chamber ensemble, expressive strings and piano, elegant development, natural concert hall acoustics" },
        electronic: { label: "Electronic", prompt: "modern electronic groove, crisp programmed drums, layered synthesizers, deep bass, evolving melodic hooks" },
        synthwave: { label: "Synthwave", prompt: "retro synthwave drive, analog arpeggios, gated drums, neon bass line, cinematic 1980s atmosphere" },
        bossa: { label: "Brazilian Bossa", prompt: "Brazilian bossa nova ensemble, nylon guitar, subtle percussion, acoustic bass, warm Rhodes, relaxed sophisticated harmony" },
        cinematic: { label: "Cinematic", prompt: "cinematic orchestral score, patient strings, restrained percussion, atmospheric synth layers, broad emotional final lift" },
        acoustic: { label: "Acoustic", prompt: "warm acoustic guitar composition, fingerpicked patterns, gentle piano, light hand percussion, organic intimate recording" },
        chillhouse: { label: "Chill House", prompt: "sunset chill house, soft four-on-the-floor groove, rounded bass, airy synth chords, understated melodic lead" }
    };
    var NO_VOICE = "Instrumental music only. No vocals, no singing, no spoken word, no voice, no choir, and no vocal samples.";
    var BUFFER_TARGET = 2;
    var savedGenre = loadGenre();
    var state = {
        genre: GENRES[savedGenre] ? savedGenre : "ambient",
        stationActive: false,
        engineReady: false,
        queue: [],
        current: null,
        pending: null,
        objectUrl: null,
        sequence: 0,
        primed: false
    };

    var audio = document.getElementById("jukeboxAudio");
    var playButton = document.getElementById("jukeboxPlay");
    var stopButton = document.getElementById("jukeboxStop");
    var genreTrigger = document.getElementById("jukeboxGenreTrigger");
    var genreMenu = document.getElementById("jukeboxGenreMenu");
    var progress = document.getElementById("jukeboxProgress");

    function resizeHostWindow() {
        var host = root.closest(".window-element-glass");
        if (!host) return;
        var width = Math.min(430, Math.max(340, window.innerWidth - 28));
        var height = Math.min(590, Math.max(480, window.innerHeight - 72));
        host.style.width = width + "px";
        host.style.height = height + "px";
        host.style.maxWidth = "calc(100vw - 16px)";
        host.style.maxHeight = "calc(100vh - 54px)";
    }

    function loadGenre() {
        try { return localStorage.getItem(STORAGE_KEY) || "ambient"; }
        catch (_error) { return "ambient"; }
    }

    function saveGenre() {
        try { localStorage.setItem(STORAGE_KEY, state.genre); }
        catch (_error) { /* Genre persistence is optional. */ }
    }

    function getToken() {
        return (typeof window.getAccessToken === "function" && window.getAccessToken()) ||
            sessionStorage.getItem("gb_access_token") || localStorage.getItem("gb_access_token");
    }

    async function apiFetch(url, options) {
        options = options || {};
        options.headers = Object.assign({ "Content-Type": "application/json" }, options.headers || {});
        var token = getToken();
        if (token) options.headers.Authorization = "Bearer " + token;
        var response = await fetch(url, options);
        var body = await response.json().catch(function () { return {}; });
        if (!response.ok) throw new Error(body.detail || body.error || "Request failed (" + response.status + ")");
        return body;
    }

    async function audioBlob(path) {
        var headers = {};
        var token = getToken();
        if (token) headers.Authorization = "Bearer " + token;
        var url = "/api/jukebox/audio?path=" + encodeURIComponent(path);
        var lastError = null;
        for (var attempt = 0; attempt < 3; attempt += 1) {
            try {
                var response = await fetch(url, { headers: headers, cache: "no-store" });
                if (!response.ok) {
                    var body = await response.json().catch(function () { return {}; });
                    var responseError = new Error(body.detail || body.error || "Generated audio is unavailable");
                    if (response.status < 500 || attempt === 2) throw responseError;
                    lastError = responseError;
                } else {
                    return await response.blob();
                }
            } catch (error) {
                lastError = error;
                if (attempt === 2) throw error;
            }
            await new Promise(function (resolve) { setTimeout(resolve, 400 * (attempt + 1)); });
        }
        throw lastError || new Error("Generated audio is unavailable");
    }

    function escapeHtml(value) {
        return String(value == null ? "" : value).replace(/[&<>"']/g, function (character) {
            return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[character];
        });
    }

    function formatTime(seconds) {
        var value = Number.isFinite(seconds) ? Math.max(0, Math.floor(seconds)) : 0;
        return String(Math.floor(value / 60)).padStart(2, "0") + ":" + String(value % 60).padStart(2, "0");
    }

    function setStatus(title, detail) {
        document.getElementById("jukeboxTrackTitle").textContent = title;
        document.getElementById("jukeboxTrackStatus").textContent = detail;
    }

    function showMessage(message) {
        var box = document.getElementById("jukeboxMessage");
        box.textContent = message || "";
        box.hidden = !message;
    }

    function setControls() {
        playButton.disabled = state.stationActive && !audio.paused;
        stopButton.disabled = !state.stationActive && !state.current;
        root.classList.toggle("is-playing", !audio.paused && !audio.ended);
    }

    function renderQueue() {
        var list = document.getElementById("jukeboxQueue");
        var entries = [];
        if (state.current) entries.push({ title: state.current.title, status: audio.paused ? "READY" : "PLAY", current: true });
        state.queue.forEach(function (track) { entries.push({ title: track.title, status: "READY" }); });
        if (state.pending) entries.push({ title: state.pending.title, status: state.pending.status === "queued" ? "QUEUE" : "RENDER", pending: true });
        document.getElementById("jukeboxQueueCount").textContent = state.queue.length + " READY";
        if (!entries.length) {
            list.innerHTML = '<li class="jukebox-queue-empty">The station will keep one instrumental track ready.</li>';
            return;
        }
        list.innerHTML = entries.slice(0, 7).map(function (entry, index) {
            return '<li data-number="' + String(index + 1).padStart(2, "0") + '" class="' +
                (entry.current ? "current " : "") + (entry.pending ? "pending" : "") + '"><i class="jukebox-vinyl" aria-hidden="true"><b>' +
                String(index + 1).padStart(2, "0") + "</b></i><strong>" +
                escapeHtml(entry.title) + '</strong><span>' + entry.status + "</span></li>";
        }).join("");
    }

    function updateGenreDisplay() {
        var genre = GENRES[state.genre];
        document.getElementById("jukeboxGenreValue").textContent = genre.label;
        document.getElementById("jukeboxStationLabel").textContent = genre.label.toUpperCase();
        genreMenu.querySelectorAll("[data-genre]").forEach(function (button) {
            button.setAttribute("aria-selected", String(button.dataset.genre === state.genre));
        });
    }

    function buildPayload() {
        var genre = GENRES[state.genre];
        state.sequence += 1;
        return {
            title: genre.label + " Radio " + String(state.sequence).padStart(2, "0"),
            simple_mode: false,
            prompt: genre.prompt + ". " + NO_VOICE,
            description: "",
            lyrics: "",
            instrumental: true,
            // A longer playback window plus the fast background profile keeps
            // one full instrumental ready before the current track ends.
            duration: 60,
            batch_size: 1,
            inference_steps: state.sequence === 1 ? 4 : 2,
            thinking: false,
            enhance: false,
            audio_format: "wav"
        };
    }

    async function generateNext() {
        if (state.pending || state.queue.length >= BUFFER_TARGET) return;
        var payload = buildPayload();
        var requestedGenre = state.genre;
        state.pending = { title: payload.title, genre: requestedGenre, status: "queued", job_id: null };
        setStatus("COMPOSING " + GENRES[requestedGenre].label.toUpperCase(), "INSTRUMENTAL ENGINE IS RENDERING THE NEXT TRACK");
        renderQueue();
        try {
            var created = await apiFetch("/api/jukebox/generate", { method: "POST", body: JSON.stringify(payload) });
            if (!state.pending || state.pending.genre !== requestedGenre) return;
            state.pending.job_id = created.job_id;
            state.pending.status = created.status || "queued";
            renderQueue();
        } catch (error) {
            if (state.pending && state.pending.genre === requestedGenre) state.pending = null;
            showMessage(error.message || "ACE-Step could not start this track.");
            setStatus("ENGINE UNAVAILABLE", "CHECK ACE-STEP AND PRESS PLAY TO RETRY");
            renderQueue();
        }
    }

    async function pollPending() {
        var pending = state.pending;
        if (!pending || !pending.job_id) return;
        try {
            var result = await apiFetch("/api/jukebox/jobs/" + encodeURIComponent(pending.job_id));
            if (state.pending !== pending) return;
            pending.status = result.status;
            if (result.status === "failed") throw new Error(result.error || "Music generation failed");
            if (result.status !== "succeeded") { renderQueue(); return; }
            state.pending = null;
            if (pending.genre !== state.genre) return;
            (result.tracks || []).forEach(function (track) {
                state.queue.push({ path: track.audio_path, title: pending.title, genre: pending.genre });
            });
            renderQueue();
            if (state.stationActive && !state.current) await playNext();
            else if (state.stationActive) generateNext();
        } catch (error) {
            if (state.pending === pending) state.pending = null;
            showMessage(error.message || "ACE-Step could not finish this track.");
            setStatus("TRACK FAILED", "PRESS PLAY TO TRY ANOTHER INSTRUMENTAL");
            renderQueue();
        }
    }

    async function loadTrack(track) {
        state.current = track;
        setStatus(track.title.toUpperCase(), "LOADING GENERATED INSTRUMENTAL");
        renderQueue();
        var blob = await audioBlob(track.path);
        if (state.objectUrl) URL.revokeObjectURL(state.objectUrl);
        state.objectUrl = URL.createObjectURL(blob);
        audio.src = state.objectUrl;
        audio.load();
        setStatus(track.title.toUpperCase(), "ACE-STEP INSTRUMENTAL • " + GENRES[track.genre].label.toUpperCase());
    }

    async function playNext() {
        if (!state.stationActive) return;
        if (!state.primed && state.queue.length < BUFFER_TARGET) {
            setStatus("BUILDING PLAYLIST RESERVE", state.queue.length + " OF " + BUFFER_TARGET + " INSTRUMENTALS READY");
            await generateNext();
            return;
        }
        if (!state.primed) state.primed = true;
        if (!state.queue.length) {
            setStatus("BUFFERING STATION", "COMPOSING THE NEXT INSTRUMENTAL TRACK");
            await generateNext();
            return;
        }
        var track = state.queue.shift();
        try {
            await loadTrack(track);
            generateNext();
            await audio.play();
            showMessage("");
        } catch (error) {
            if (error && error.name === "NotAllowedError") {
                setStatus(track.title.toUpperCase(), "TRACK READY • PRESS PLAY TO BEGIN");
            } else {
                showMessage(error.message || "Generated audio could not be played.");
                state.current = null;
                setStatus("TRACK UNAVAILABLE", "PRESS PLAY TO RETRY THE INSTRUMENTAL STATION");
                renderQueue();
            }
        }
        setControls();
    }

    async function startStation() {
        showMessage("");
        state.stationActive = true;
        stopButton.disabled = false;
        if (state.current && audio.src && audio.paused && !audio.ended) {
            try { await audio.play(); }
            catch (error) { showMessage(error.message || "Playback could not start."); }
            generateNext();
        } else {
            if (audio.ended) state.current = null;
            await playNext();
        }
        setControls();
    }

    function stopStation() {
        state.stationActive = false;
        audio.pause();
        if (audio.currentTime) audio.currentTime = 0;
        if (state.current) setStatus(state.current.title.toUpperCase(), "STOPPED • PRESS PLAY TO RESUME");
        else setStatus("STATION STOPPED", "SELECT A GENRE OR PRESS PLAY");
        setControls();
        renderQueue();
    }

    function clearCurrentAudio() {
        audio.pause();
        audio.removeAttribute("src");
        audio.load();
        if (state.objectUrl) URL.revokeObjectURL(state.objectUrl);
        state.objectUrl = null;
        state.current = null;
    }

    function chooseGenre(value) {
        if (!GENRES[value] || value === state.genre) { closeGenreMenu(); return; }
        var wasActive = state.stationActive;
        state.genre = value;
        saveGenre();
        state.queue = [];
        state.pending = null;
        state.primed = false;
        clearCurrentAudio();
        updateGenreDisplay();
        renderQueue();
        closeGenreMenu();
        setStatus(GENRES[value].label.toUpperCase() + " STATION", wasActive ? "TUNING A NEW INSTRUMENTAL PLAYLIST" : "PRESS PLAY TO START THE STATION");
        if (wasActive) generateNext();
    }

    function toggleGenreMenu() {
        var opening = genreMenu.hidden;
        genreMenu.hidden = !opening;
        genreTrigger.setAttribute("aria-expanded", String(opening));
    }

    function closeGenreMenu() {
        genreMenu.hidden = true;
        genreTrigger.setAttribute("aria-expanded", "false");
    }

    async function checkEngine() {
        var badge = document.getElementById("jukeboxEngine");
        var label = document.getElementById("jukeboxEngineLabel");
        try {
            var result = await apiFetch("/api/jukebox/health");
            state.engineReady = Boolean(result.healthy);
            badge.dataset.state = state.engineReady ? "online" : "offline";
            label.textContent = state.engineReady ? "ACE READY" : "OFFLINE";
        } catch (_error) {
            state.engineReady = false;
            badge.dataset.state = "offline";
            label.textContent = "OFFLINE";
        }
    }

    genreTrigger.addEventListener("click", toggleGenreMenu);
    genreMenu.addEventListener("click", function (event) {
        var option = event.target.closest("[data-genre]");
        if (option) chooseGenre(option.dataset.genre);
    });
    root.addEventListener("click", function (event) {
        if (!event.target.closest("#jukeboxGenre")) closeGenreMenu();
    });
    playButton.addEventListener("click", startStation);
    stopButton.addEventListener("click", stopStation);
    audio.addEventListener("play", function () { setControls(); renderQueue(); });
    audio.addEventListener("pause", function () { setControls(); renderQueue(); });
    audio.addEventListener("timeupdate", function () {
        var ratio = audio.duration ? audio.currentTime / audio.duration : 0;
        progress.style.width = (ratio * 100) + "%";
        document.getElementById("jukeboxClock").textContent = formatTime(audio.currentTime);
    });
    audio.addEventListener("ended", function () {
        state.current = null;
        progress.style.width = "0";
        document.getElementById("jukeboxClock").textContent = "00:00";
        renderQueue();
        if (state.stationActive) playNext(); else setControls();
    });

    if (window.GBAppLifecycle) {
        GBAppLifecycle.interval("jukebox", function () {
            if (!document.getElementById("jukeboxApp")) return;
            pollPending();
        }, 3500);
    } else {
        clearInterval(window.__jukeboxPollTimer);
        window.__jukeboxPollTimer = setInterval(function () {
            if (!document.getElementById("jukeboxApp")) { clearInterval(window.__jukeboxPollTimer); return; }
            pollPending();
        }, 3500);
    }

    resizeHostWindow();
    updateGenreDisplay();
    renderQueue();
    checkEngine();
})();
