function vibeSafeStatus(status) {
    if (typeof setVibeStatus === "function") {
        setVibeStatus(status);
        return;
    }
    var dot = document.getElementById("vibeChatStatusDot");
    var badge = document.getElementById("vibeChatStatusBadge");
    if (dot) {
        dot.className =
            "as-status-dot " +
            (status === "connected" ? "green" : status === "connecting" ? "yellow" : "red");
    }
    if (badge) {
        badge.textContent =
            status === "connected"
                ? "ONLINE"
                : status === "connecting"
                ? "CONNECTING…"
                : "OFFLINE";
    }
}

function vibeSafeMsg(role, text) {
    if (typeof vibeAddMsg === "function") {
        return vibeAddMsg(role, text);
    }
    var box = document.getElementById("vibeRunnerLogList");
    if (!box) return null;
    var div = document.createElement("div");
    div.textContent = text;
    div.style.cssText =
        "align-self:center;background:rgba(132,214,105,0.12);color:var(--accent);padding:6px 12px;border-radius:8px;font-size:11px;text-align:center;";
    box.appendChild(div);
    box.scrollTop = box.scrollHeight;
    return div;
}

function vibeSafeEsc(text) {
    if (typeof esc === "function") return esc(text);
    var d = document.createElement("div");
    d.textContent = text || "";
    return d.innerHTML;
}

function vibeSafeTaskNode(title, description, meta) {
    if (typeof addTaskNode === "function") addTaskNode(title, description, meta);
}

function vibeRouteEvent(eventData, raw) {
    if (
        typeof window.VibeRun !== "undefined" &&
        window.VibeRun &&
        typeof window.VibeRun.onProgress === "function"
    ) {
        window.VibeRun.onProgress(eventData, raw);
    }
}

function connectVibeWs() {
    vibeSafeStatus("connecting");

    var botName = window.__INITIAL_BOT_NAME__ || "default";
    vibeAuthFetch("/api/auth?bot_name=" + encodeURIComponent(botName))
        .then(function (r) {
            return r.json();
        })
        .then(function (auth) {
            vibeUserId = auth.user_id;
            vibeSessionId = auth.session_id;
            vibeBotId = auth.bot_id || "default";
            vibeBotName = botName;

            var proto =
                location.protocol === "https:" ? "wss://" : "ws://";
            var url =
                proto +
                location.host +
                "/ws?session_id=" +
                vibeSessionId +
                "&user_id=" +
                vibeUserId +
                "&bot_name=" +
                vibeBotName;
            vibeWs = new WebSocket(url);

            vibeWs.onopen = function () {
                vibeSafeStatus("connected");
            };

            vibeWs.onmessage = function (event) {
                try {
                    var data = JSON.parse(event.data);
                    if (data.type === "connected") return;
                    if (data.event) {
                        vibeRouteEvent(data.event, data);
                        return;
                    }

                    if (data.type === "thought_process") {
                        vibeSafeMsg("system", "💭 " + vibeSafeEsc(data.content));
                        return;
                    }
                    if (data.type === "terminal_output") {
                        vibeSafeMsg("system", "🖥️ " + vibeSafeEsc(data.line));
                        return;
                    }
                    if (data.type === "step_progress") {
                        // Progress now lives in the Run Dock; the removed
                        // fake agent card has no progress bar to update.
                        return;
                    }

                    if (data.message_type === 2) {
                        if (data.is_complete) {
                            if (vibeStreaming) {
                                if (typeof vibeFinalizeStream === "function") {
                                    vibeFinalizeStream();
                                } else {
                                    vibeStreaming = false;
                                }
                            } else if (
                                data.content &&
                                data.content.trim()
                            ) {
                                vibeSafeMsg("bot", data.content);
                            }
                            vibeStreaming = false;
                        } else {
                            if (!vibeStreaming) {
                                vibeStreaming = true;
                                if (typeof vibeAddStreamStart === "function") {
                                    vibeAddStreamStart();
                                }
                                if (typeof vibeUpdateStream === "function") {
                                    vibeUpdateStream(data.content || "");
                                }
                            } else if (typeof vibeUpdateStream === "function") {
                                vibeUpdateStream(data.content || "");
                            }
                        }
                    }
                } catch (e) {
                    console.error("Vibe WS parse error:", e);
                }
            };

            vibeWs.onclose = function () {
                vibeSafeStatus("disconnected");
            };
            vibeWs.onerror = function () {
                vibeSafeStatus("disconnected");
            };
        })
        .catch(function () {
            /* Silent: toolbar mode has no chat client portion to show the
               "could not connect" banner in — status dot only. */
            vibeSafeStatus("disconnected");
        });
}

function vibeSendWs(content) {
    if (vibeWs && vibeWs.readyState === WebSocket.OPEN) {
        vibeWs.send(
            JSON.stringify({
                bot_id: vibeBotId,
                user_id: vibeUserId,
                session_id: vibeSessionId,
                channel: "web",
                content: content,
                message_type: 1,
                timestamp: new Date().toISOString(),
            }),
        );
    }
}

function connectTaskProgressWs(taskId) {
    var proto = location.protocol === "https:" ? "wss://" : "ws://";
    var url =
        proto +
        location.host +
        "/ws/task-progress" +
        (taskId ? "/" + taskId : "");
    if (taskProgressWs) {
        try {
            taskProgressWs.close();
        } catch (ignore) { }
    }
    taskProgressWs = new WebSocket(url);

    taskProgressWs.onmessage = function (event) {
        try {
            var data = JSON.parse(event.data);
            if (data.type === "connected") return;
            if (data.event) {
                vibeRouteEvent(data.event, data);
                return;
            }

            if (
                data.event_type === "agent_thought" ||
                data.step === "agent_thought"
            ) {
                var agentLabel = (data.details || "vibe_1").replace(
                    "vibe_",
                    "Vibe #",
                );
                vibeSafeMsg(
                    "system",
                    "💭 " +
                        agentLabel +
                        ": " +
                        vibeSafeEsc(data.text || data.message || ""),
                );
                return;
            }

            if (
                data.event_type === "task_node" ||
                data.step === "task_node"
            ) {
                try {
                    var nodeInfo =
                        typeof data.details === "string"
                            ? JSON.parse(data.details)
                            : data.details;
                    if (nodeInfo) {
                        vibeSafeTaskNode(
                            nodeInfo.title || data.message || "Task",
                            nodeInfo.description || "",
                            {
                                status: nodeInfo.status || "Planning",
                                estimated_files:
                                    nodeInfo.estimated_files,
                                estimated_time:
                                    nodeInfo.estimated_time,
                                estimated_tokens:
                                    nodeInfo.estimated_tokens,
                                fileList: nodeInfo.files || [],
                            },
                        );
                    }
                } catch (ignore) {
                    vibeSafeTaskNode(data.message || "Task", "", {
                        status: "Planning",
                    });
                }
                return;
            }

            if (
                data.event_type === "step_progress" ||
                data.step === "step_progress"
            ) {
                // Progress now lives in the Run Dock; the removed fake
                // agent card has no progress bar to update.
                var stageMap = {
                    Planning: "plan",
                    Building: "build",
                    Reviewing: "review",
                    Deploying: "deploy",
                    Monitoring: "monitor",
                };
                var stageLabel = data.message || "";
                var tabStage = stageMap[stageLabel];
                if (tabStage && window.VibePipeline) {
                    window.VibePipeline.activate(tabStage);
                }
                return;
            }

            if (
                data.event_type === "pipeline_complete" ||
                data.step === "pipeline_complete"
            ) {
                vibeSafeMsg(
                    "system",
                    "✅ Pipeline complete — all stages finished",
                );
                return;
            }

            if (data.event_type === "manifest_update") {
                return;
            }
        } catch (e) {
            console.error("Task progress parse error:", e);
        }
    };

    taskProgressWs.onerror = function () { };
    taskProgressWs.onclose = function () { };
}