/* Vibe Run Dock — split from vibe/vibe-run.js for #1731 */

    /* ------------------------------------------------- polling */

    function startPolling() {
        if (window.GBAppLifecycle) GBAppLifecycle.begin("vibe");
        stopPolling();
        state.pollTimer = (window.GBAppLifecycle ? GBAppLifecycle.interval("vibe", loadAll, 1500) : setInterval(loadAll, 1500));
    }

    function stopPolling() {
        if (state.pollTimer) {
            clearInterval(state.pollTimer);
            state.pollTimer = null;
        }
    }

    function loadAll() {
        if (!state.runId) return;
        pollRun();
    }

    // #1190 — animated execution overlay over the Vibe window while a run is
    // executing; hidden on terminal states.
    function syncExecOverlay() {
        var overlay = q("vibeExecOverlay");
        if (!overlay) return;
        var active = state.run && ["running", "pending"].indexOf(String(state.run.state)) !== -1;
        overlay.style.display = active ? "flex" : "none";
        overlay.classList.toggle("hidden", !active);
    }

    function pollRun() {
        api("/api/vibe/run/" + state.runId).then(function (data) {
            if (!data || !data.run_id) return;
            var changed = !state.run || String(data.state) !== String(state.run.state);
            state.run = data;
            renderRunCard();
            syncExecOverlay();
            if (changed) {
                // Keep the RUNS list truthful: the dock polls the focused run,
                // but the list only refreshed on runFocus/start — so a run that
                // just completed would stay as a stale "RUNNING 0 calls" entry.
                loadRuns();
                if (data.state === "completed") {
                    stopPolling();
                    // #run-done — a finished run must end with the result
                    // visible: (1) drop a clickable deeplink in the Runner
                    // Log, (2) refresh the Browser window with the app (dev
                    // VM first) or open the production tab for deploys.
                    var runProjectId = data.project_id;
                    var isDeploy = String(data.pipeline_mode || "").indexOf("deploy") !== -1;
                    if (runProjectId || data.project_name) {
                        showDoneDeeplink(data, isDeploy, state.runId);
                    }
                    if (isDeploy) {
                        // Deploy IS production: open the published app in a
                        // new tab. openProdTab is sessionStorage-guarded per
                        // run, so a duplicate call (the WS path may already
                        // have opened it) is a no-op.
                        if (runProjectId) {
                            openProdTab(data.project_name || "", runProjectId, state.runId);
                        }
                    } else {
                        // #1408 — a finished run announces so the shell
                        // refreshes every open integrated app: the Browser
                        // preview (re-targeted to the dev VM / static
                        // preview), the Editor tree, the Canvas project.draw
                        // and the Database schema.
                        announceRunFinished(data);
                    }
                } else if (data.state === "failed" || data.state === "cancelled") {
                    uiMsg("⛔ Run " + data.state + ".");
                    stopPolling();
                }
            }
            api("/api/vibe/metrics/" + state.runId).then(function (m) {
                if (m && m.success && m.metrics) {
                    state.metrics = m.metrics;
                }
            });
            if (!state.loadedPipeline) loadPipeline();
            if (data.state === "running") {
                api("/api/vibe/events/" + state.runId).then(function (ev) {
                    var list = null;
                    if (ev && Array.isArray(ev.events)) list = ev.events;
                    else if (Array.isArray(ev)) list = ev;
                    if (!list) return;
                    var changed = list.length !== state.events.length;
                    state.events = list;
                    renderSources();
                    // #vibe-verbose — narrate every tool event into the
                    // Runner Log as it happens; previously the fetched events
                    // were only counted, leaving the user staring at an idle
                    // dock between "run started" and the final verdict.
                    if (changed) renderEventLog();
                });
            }
            // #1396 — suppress a server-side milestone the user already saw
            // before a botserver restart (server poller state resets on boot).
            if (state.runId && String(data.state) === "running") {
                var seen = q("vibeLastMilestone");
                if (seen && seen.textContent && isStaleMilestone(state.runId, seen.textContent)) {
                    seen.dataset.suppressed = "1";
                }
            }
        });
    }

    // #vibe-verbose — live event narration in the Runner Log. Renders the
    // last N telemetry events (tool calls, verifications, approvals) with
    // timestamps; new events since the previous poll are appended only.
    // #1396 — autoscroll pauses while the user scrolls up to read and
    // resumes when they return to the bottom.
    function renderEventLog() {
        var box = q("vibeRunnerLogList");
        if (!box) return;
        var events = state.events.slice(-40);
        if (!events.length) return;
        var existing = box.querySelectorAll("[data-ev-id]").length;
        if (existing === events.length) return; // nothing new
        var atBottom = box.scrollHeight - box.scrollTop - box.clientHeight < 24;
        box.innerHTML = "";
        events.forEach(function (e) {
            var line = document.createElement("div");
            line.className = "vibe-runner-log-line";
            line.setAttribute("data-ev-id", e.event_id || "");
            var when = "";
            try { when = new Date(e.timestamp).toLocaleTimeString(); } catch (ignore) { when = ""; }
            var icon = "•";
            var label = String(e.event_type || "");
            if (label === "tool_call_started") { icon = "⚙️"; label = "running " + (e.tool_name || "tool"); }
            else if (label === "tool_call_completed") { icon = e.success === false ? "⚠️" : "✅"; label = (e.success === false ? "failed " : "done ") + (e.tool_name || "tool"); }
            else if (label === "tool_call_failed") { icon = "⚠️"; label = "failed " + (e.tool_name || "tool"); }
            else if (label === "run_started") { icon = "▶️"; label = "run started"; }
            else if (label === "run_completed") { icon = "🏁"; label = "run completed"; }
            else if (label === "run_failed") { icon = "❌"; label = "run failed"; }
            else if (label === "approval_requested") { icon = "⏸"; label = "approval requested"; }
            else if (label === "approval_granted") { icon = "🔓"; label = "approved"; }
            else if (label === "approval_denied") { icon = "🚫"; label = "denied"; }
            if (e.error) label += " — " + String(e.error).substring(0, 120);
            line.innerHTML = '<span class="vibe-runner-log-ts">' + vibeRunEsc(when) + '</span> ' +
                '<span>' + icon + '</span> <span>' + vibeRunEsc(label) + '</span>';
            box.appendChild(line);
        });
        if (atBottom) box.scrollTop = box.scrollHeight;
    }

    function startTicker() {
        if (state.tickTimer) clearInterval(state.tickTimer);
        var tick = function () {
            if (!state.run || !state.run.created_at) return;
            // A finished run must freeze the clock, not keep counting (#run-timer).
            var runState = String(state.run.state || "").toLowerCase();
            if (["completed", "failed", "cancelled"].indexOf(runState) !== -1) {
                freezeElapsed();
                return;
            }
            var start = new Date(state.run.created_at).getTime();
            if (isNaN(start)) return;
            var s = Math.max(0, Math.floor((Date.now() - start) / 1000));
            setText("vibeRunElapsed", s + "s");
        };
        state.tickTimer = window.GBAppLifecycle
            ? GBAppLifecycle.interval("vibe", tick, 1000)
            : setInterval(tick, 1000);
    }

    function stopTicker() {
        if (state.tickTimer) {
            clearInterval(state.tickTimer);
            state.tickTimer = null;
        }
    }

    // Final elapsed for a terminal run: prefer the backend's completed_at so
    // the frozen value is the true duration, not "now minus start".
    function freezeElapsed() {
        stopTicker();
        if (!state.run || !state.run.created_at) return;
        var start = new Date(state.run.created_at).getTime();
        var end = state.run.completed_at ? new Date(state.run.completed_at).getTime() : NaN;
        if (isNaN(start)) return;
        if (isNaN(end)) end = Date.now();
        setText("vibeRunElapsed", Math.max(0, Math.floor((end - start) / 1000)) + "s");
    }
