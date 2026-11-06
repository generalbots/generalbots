/* Vibe Run Dock — split from vibe/vibe-run.js for #1731 */

    /* ------------------------------------------------- sessions */

    function loadSessions() {
        var list = q("vibeSessionsList");
        if (!list) {
            retry(loadSessions, "sessions");
            return;
        }
        api("/api/vibe/sessions").then(function (data) {
            var sessions = (data && data.success && Array.isArray(data.sessions)) ? data.sessions : [];
            if (!sessions.length) {
                list.innerHTML = '<div class="vibe-rd-empty">No sessions yet.</div>';
                return;
            }
            list.innerHTML = sessions.slice(0, 6).map(function (s) {
                var runState = s.run ? s.run.state : "idle";
                var st = "Session " + String(s.session_id).substring(0, 4);
                return '<div class="vibe-session-item">' +
                    '<span class="vibe-chip ' + chipState(runState) + '">' + vibeRunEsc(runState) + "</span>" +
                    '<span class="meta" title="' + vibeRunEsc(s.intent) + '">' + vibeRunEsc(s.intent || st) + "</span>" +
                    '<button type="button" data-resume="' + vibeRunEsc(s.session_id) + '">Resume</button>' +
                    '<button type="button" data-fork="' + vibeRunEsc(s.session_id) + '">Fork</button>' +
                    "</div>";
            }).join("");
            list.querySelectorAll("[data-resume]").forEach(function (b) {
                b.addEventListener("click", function () {
                    resumeSession(b.getAttribute("data-resume"));
                });
            });
            list.querySelectorAll("[data-fork]").forEach(function (b) {
                b.addEventListener("click", function () {
                    forkSession(b.getAttribute("data-fork"));
                });
            });
        });
    }

    function resumeSession(sessionId) {
        uiMsg("🔄 Resuming session " + shortRunId(sessionId) + "…");
        api("/api/vibe/sessions/" + sessionId + "/resume", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: "{}",
        }).then(function (data) {
            if (data && data.success && data.run_id) {
                runFocus(data.run_id);
                uiMsg("✅ Session resumed — run " + shortRunId(data.run_id) + " active.");
            } else {
                uiMsg("⚠️ Resume failed: " + ((data && data.error) || "unknown"));
            }
        });
    }

    function forkSession(sessionId) {
        api("/api/vibe/sessions/" + sessionId + "/fork", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: "{}",
        }).then(function (data) {
            if (data && data.success) {
                uiMsg("🍴 Session forked.");
                loadSessions();
            } else {
                uiMsg("⚠️ Fork failed: " + ((data && data.error) || "unknown"));
            }
        });
    }

    /* ------------------------------------------------- runs list */

    // Editing a generated `.bas` belongs to the AutoTask items list (Tasks
    // app), where every created automation is a row — not here. A run is an
    // execution, not an item; its row action pointed at the bot's FIRST .bas,
    // which was ambiguous and duplicated the AutoTask surface.

    function loadRuns() {
        api("/api/vibe/runs?limit=8").then(function (data) {
            var list = q("vibeRunsList");
            if (!list) return;
            var runs = Array.isArray(data) ? data : [];
            if (!runs.length) {
                list.innerHTML = '<div class="vibe-rd-empty">No runs yet.</div>';
                return;
            }
            list.innerHTML = runs.map(function (r) {
                return '<div class="vibe-run-item" style="cursor:pointer" data-run="' + vibeRunEsc(r.run_id) + '">' +
                    '<span class="vibe-chip ' + chipState(r.state) + '">' + vibeRunEsc(r.state) + "</span>" +
                    '<span class="meta" title="' + vibeRunEsc(r.intent) + '">' + vibeRunEsc(r.intent || "run") + "</span>" +
                    '<span class="meta" style="flex:0">' + (r.tool_call_count || 0) + " calls</span>" +
                    "</div>";
            }).join("");
            list.querySelectorAll("[data-run]").forEach(function (el) {
                el.addEventListener("click", function () {
                    runFocus(el.getAttribute("data-run"));
                });
            });
            // #1408 — announce runs that finish during this page session (a
            // chat-originated run is not polled by the dock when a different
            // run is focused, so the runs list is the reliable signal). Only
            // transitions INTO completed count — pre-existing history does not
            // re-target the Browser.
            runs.forEach(function (r) {
                var rid = String(r.run_id);
                var st = String(r.state || "").toLowerCase();
                var prev = seenRunStates[rid];
                seenRunStates[rid] = st;
                if (st === "completed" && prev && prev !== "completed" && prev !== "failed" && prev !== "cancelled") {
                    announceRunFinished(r);
                }
            });
            if (!state.runId) {
                var active = runs.find(function (run) {
                    var runState = String(run.state || "").toLowerCase();
                    return runState === "running";
                });
                if (active && active.run_id) runFocus(active.run_id);
            }
        });
    }

    /* ------------------------------------------------- teams */

    function startTeamPolling() {
        if (state.teamTimer) clearInterval(state.teamTimer);
        loadTeams();
        state.teamTimer = setInterval(loadTeams, 3000);
    }

    function loadTeams() {
        api("/api/vibe/teams").then(function (data) {
            var list = q("vibeTeamsList");
            if (!list) return;
            var teams = (data && data.success && Array.isArray(data.teams)) ? data.teams : [];
            if (!teams.length) {
                list.innerHTML = '<div class="vibe-rd-empty">No teams yet.</div>';
                return;
            }
            var anyRunning = false;
            list.innerHTML = teams.map(function (t) {
                if (t.status === "running") anyRunning = true;
                var chips = (t.members || []).map(function (m) {
                    var cls = m.state === "done" || m.state === "completed" ? "done"
                        : m.state === "failed" ? "failed"
                        : m.state === "pending" ? "" : "working";
                    var runTag = m.run_id ? " · " + shortRunId(m.run_id) : "";
                    return '<span class="vibe-member-chip ' + cls + '">' +
                        vibeRunEsc(m.name) + " · " + vibeRunEsc(m.state || "?") + runTag + "</span>";
                }).join("");
                return '<div class="vibe-team-card">' +
                    '<div class="vibe-team-head"><span>' + vibeRunEsc(t.name) + "</span>" +
                    '<span class="vibe-chip state-running" style="background:transparent;border:1px solid var(--border);color:var(--text-muted)">' + vibeRunEsc(t.status || "?") + "</span></div>" +
                    '<div class="vibe-member-chips">' + chips + "</div>" +
                    "</div>";
            }).join("");
            if (!anyRunning && state.teamTimer) {
                clearInterval(state.teamTimer);
                state.teamTimer = null;
            }
        });
    }

    /* ------------------------------------------------- progress events (WS) */

    function onProgress(eventData, raw) {
        if (!eventData) return;
        var step = eventData.step || eventData.event_type || "";
        var evRunId = eventData.run_id;
        if (evRunId && state.runId && String(evRunId) !== String(state.runId)) return;

        // Report the run's outcome back into the Chat window — the chat only
        // says "Vibe agent started ..." when routing a mention, so without
        // this the user never learns the agent finished or what it did.
        if ((step === "completed" || step === "failed" || step === "cancelled") && evRunId) {
            var detail = eventData.message || "";
            var runCfg = (state.run && (state.run.config || state.run)) || {};
            var wasDeploy = runCfg.pipeline_mode === "deploy" || /publish|deploy/i.test(detail);
            if (step === "completed" && (runCfg.project_id || runCfg.project_name)) {
                // Clickable completion deeplink (shared with the poll path,
                // which is guarded per run — whichever fires first wins).
                showDoneDeeplink(runCfg, wasDeploy, evRunId);
            } else {
                var note;
                if (step === "completed") {
                    note = wasDeploy
                        ? "✅ Deploy finished — your app is published and ready at its public URL."
                        : "✅ Vibe agent finished — changes applied and ready to go.";
                } else if (step === "failed") {
                    note = "❌ Vibe run failed — " + (detail || "check the Runner Log for details.");
                } else {
                    note = "⏹ Vibe run cancelled.";
                }
                if (typeof vibeSafeMsg === "function") {
                    vibeSafeMsg("system", note);
                }
            }
            // #deploy — a finished deploy IS production: open the published
            // app in a NEW tab. window.open from a WS callback is popup-blocked
            // unless we open about:blank first and navigate afterwards.
            if (step === "completed" && wasDeploy) {
                openProdTab(
                    runCfg.project_name || (state.run && state.run.project_name) || "",
                    runCfg.project_id || (state.run && state.run.project_id) || "",
                    evRunId
                );
            }
        }

        if (typeof eventData.progress === "number") {
            state.progress = Math.max(state.progress, eventData.progress);
        } else if (eventData.current_step && eventData.total_steps) {
            state.progress = Math.max(state.progress, Math.round((eventData.current_step / eventData.total_steps) * 100));
        }

        var phase = phaseForStep(step);
        if (phase) state.phase = phase;

        if (String(step).indexOf("pipeline:") === 0) {
            var stageId = String(step).replace("pipeline:", "");
            if (state.pipeline) {
                var st = state.pipeline.stages.find(function (s) { return s.id === stageId; }) ||
                    state.pipeline.stages.find(function (s) { return s.kind && stageKindTool(s.kind) === stageId; });
                if (st) state.stageDone[st.id] = true;
                state.stageActive = null;
                renderPipelineStrip();
            }
        }
        renderRunCard();
        if (String(step).indexOf("team:member:") === 0) {
            startTeamPolling();
        }
    }
