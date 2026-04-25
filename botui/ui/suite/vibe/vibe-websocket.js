function connectVibeWs() {
setVibeStatus("connecting");

var botName = window.__INITIAL_BOT_NAME__ || "default";
fetch("/api/auth?bot_name=" + encodeURIComponent(botName))
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
setVibeStatus("connected");
};

vibeWs.onmessage = function (event) {
try {
var data = JSON.parse(event.data);
if (data.type === "connected") return;
if (data.event) return;

if (data.type === "thought_process") {
vibeAddMsg("system", "💭 " + esc(data.content));
return;
}
if (data.type === "terminal_output") {
vibeAddMsg("system", "🖥️ " + esc(data.line));
return;
}
if (data.type === "step_progress") {
var pct = Math.round(
(data.current / data.total) * 100,
);
updateMantis1("working");
var bar = document.querySelector(
'.as-agent-card[data-agent-id="1"] .as-bar-fill',
);
if (bar) bar.style.width = pct + "%";
return;
}

if (data.message_type === 2) {
if (data.is_complete) {
if (vibeStreaming) {
vibeFinalizeStream();
} else if (
data.content &&
data.content.trim()
) {
vibeAddMsg("bot", data.content);
}
vibeStreaming = false;
} else {
if (!vibeStreaming) {
vibeStreaming = true;
vibeAddStreamStart();
vibeUpdateStream(data.content || "");
} else {
vibeUpdateStream(data.content || "");
}
}
}
} catch (e) {
console.error("Vibe WS parse error:", e);
}
};

vibeWs.onclose = function () {
setVibeStatus("disconnected");
};
vibeWs.onerror = function () {
setVibeStatus("disconnected");
};
})
.catch(function () {
setVibeStatus("disconnected");
vibeAddMsg(
"system",
"⚠️ Could not connect to backend. You can still plan offline.",
);
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

if (
data.event_type === "agent_thought" ||
data.step === "agent_thought"
) {
var agentLabel = (data.details || "mantis_1").replace(
"mantis_",
"Mantis #",
);
vibeAddMsg(
"system",
"💭 " +
agentLabel +
": " +
esc(data.text || data.message || ""),
);
return;
}

if (
data.event_type === "agent_update" ||
data.step === "agent_update"
) {
try {
var info =
typeof data.details === "string"
? JSON.parse(data.details)
: data.details;
if (info) {
updateAgentCard(
info.agent_id,
info.status,
info.detail,
);
}
} catch (ignore) { }
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
addTaskNode(
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
addTaskNode(data.message || "Task", "", {
status: "Planning",
});
}
return;
}

if (
data.event_type === "step_progress" ||
data.step === "step_progress"
) {
var pct = 0;
if (data.current_step && data.total_steps) {
pct = Math.round(
(data.current_step / data.total_steps) * 100,
);
} else if (data.current && data.total) {
pct = Math.round((data.current / data.total) * 100);
}
updateMantis1("working");
var bar = document.querySelector(
'.as-agent-card[data-agent-id="1"] .as-bar-fill',
);
if (bar) bar.style.width = pct + "%";

var stageMap = {
Planning: "plan",
Building: "build",
Reviewing: "review",
Deploying: "deploy",
Monitoring: "monitor",
};
var stageLabel = data.message || "";
var tabStage = stageMap[stageLabel];
if (tabStage) {
var allTabs =
document.querySelectorAll(".vibe-pipeline-tab");
allTabs.forEach(function (t) {
t.classList.remove("active");
});
var activeTab = document.querySelector(
'.vibe-pipeline-tab[data-stage="' +
tabStage +
'"]',
);
if (activeTab) activeTab.classList.add("active");
}
return;
}

if (
data.event_type === "pipeline_complete" ||
data.step === "pipeline_complete"
) {
updateMantis1("done");
vibeAddMsg(
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
