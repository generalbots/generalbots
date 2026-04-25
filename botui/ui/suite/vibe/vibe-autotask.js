function callAutotask(intent) {
updateMantis1("working");
vibeAddMsg("system", "🔄 Mantis #1 is analyzing your request…");

connectTaskProgressWs(null);

var breadcrumb = document.querySelector(
".vibe-canvas div:first-child",
);
if (breadcrumb) {
currentProject = intent
.substring(0, 40)
.replace(/[^a-zA-Z0-9 ]/g, "");
breadcrumb.innerHTML =
'// DASHBOARD <span style="color: var(--text-secondary);margin:0 6px;">&gt;</span> // ' +
esc(currentProject.toUpperCase()) +
' <div style="float:right;"><button style="border: 1px solid var(--border);background: var(--bg);border-radius:4px;padding:2px 8px;cursor:pointer;">-</button><span style="font-size:11px;margin:0 8px;color: var(--text);">100%</span><button style="border: 1px solid var(--border);background: var(--bg);border-radius:4px;padding:2px 8px;cursor:pointer;">+</button></div>';
}

var token =
localStorage.getItem("gb-access-token") ||
sessionStorage.getItem("gb-access-token");
fetch("/api/autotask/classify", {
method: "POST",
headers: {
"Content-Type": "application/json",
Authorization: "Bearer " + token,
},
body: JSON.stringify({ intent: intent, auto_process: true }),
})
.then(function (r) {
return r.json();
})
.then(function (data) {
updateMantis1("done");

if (data.success && data.result) {
var r = data.result;

if (r.task_id) {
connectTaskProgressWs(r.task_id);
}

if (
r.created_resources &&
r.created_resources.length > 0
) {
r.created_resources.forEach(function (res, i) {
setTimeout(function () {
addTaskNode(
res.name || res.resource_type,
res.resource_type +
(res.path ? " → " + res.path : ""),
{ status: "Done" },
);
}, i * 400);
});
} else {
addTaskNode(
"Project Setup",
"Setting up: " + intent,
{ status: "Planning" },
);
}

vibeAddMsg(
"bot",
r.message || "Done! Your project is ready.",
);

if (r.app_url) {
vibeAddMsg(
"system",
'✅ App available at <a href="' +
r.app_url +
'" target="_blank" style="color: var(--accent);text-decoration:underline;">' +
esc(r.app_url) +
"</a>",
);

var preview =
document.getElementById("vibePreview");
var urlBar =
document.getElementById("vibePreviewUrl");
var content =
document.getElementById("vibePreviewContent");
if (preview) preview.style.display = "";
if (urlBar) urlBar.value = r.app_url;
if (content)
content.innerHTML =
'<iframe src="' +
r.app_url +
'" style="width:100%;height:100%;border:none;"></iframe>';
}

if (r.next_steps && r.next_steps.length > 0) {
vibeAddMsg(
"bot",
"**Next steps:**\n" +
r.next_steps
.map(function (s) { return "• " + s; })
.join("\n"),
);
}
} else {
vibeAddMsg(
"bot",
"I classified your intent as **" +
(data.intent_type || "UNKNOWN") +
"**. " +
(data.error || "Processing complete."),
);
addTaskNode("Analysis", intent, { status: "Planning" });
}
})
.catch(function (err) {
updateMantis1("done");
vibeAddMsg(
"system",
"⚠️ Backend unavailable — showing plan preview.",
);
var words = intent.split(/[.,;]/);
addTaskNode(
"Project Setup",
"Create project structure and install dependencies",
{ status: "Planning" },
);
if (words.length > 1) {
setTimeout(function () {
addTaskNode(
"Database Schema",
"Define tables for: " + words.slice(0, 3).join(", "),
{ status: "Pending" },
);
}, 500);
}
vibeAddMsg(
"bot",
"I've created a preliminary plan with " +
Math.min(words.length + 1, 5) +
" nodes. Once the backend is available, I'll process the full build.",
);
});
}
