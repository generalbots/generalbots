function addTaskNode(title, description, meta) {
var stepsContainer = document.getElementById("vibeSteps");
if (!stepsContainer) return;
stepsContainer.style.display = "flex";
var emptyState = document.getElementById("vibeCanvasEmpty");
if (emptyState) emptyState.style.display = "none";

nodeIdCounter++;
meta = meta || {};
var fileCount =
meta.estimated_files ||
meta.files ||
Math.floor(Math.random() * 15 + 3);
var time =
meta.estimated_time ||
meta.time ||
Math.floor(Math.random() * 20 + 5) + "m";
var tokens =
meta.estimated_tokens ||
meta.tokens ||
"~" + Math.floor(Math.random() * 30 + 10) + "k tokens";
var status = meta.status || "Planning";
var fileList = meta.fileList || [];
var isFirst = stepsContainer.children.length === 0;
var nodeId = "vibe-node-" + nodeIdCounter;

var statusBg =
status === "Done"
? "var(--accent)"
: status === "Planning"
? "var(--success-light, #eef8eb)"
: "var(--warning-light, var(--bg)3cd)";
var statusColor =
status === "Done"
? "var(--bg)"
: status === "Planning"
? "var(--accent)"
: "var(--warning, #856404)";

var subTasksHtml = "";
if (fileList.length > 0) {
subTasksHtml =
'<div id="' +
nodeId +
'-files" style="display:none;padding:8px 16px;border-top:1px solid var(--border);font-size:10px;color:var(--text-muted, #555);">';
for (var fi = 0; fi < fileList.length; fi++) {
subTasksHtml +=
'<div style="padding:2px 0;display:flex;align-items:center;gap:4px;"><span style="color: var(--accent);">📄</span> ' +
esc(fileList[fi]) +
"</div>";
}
subTasksHtml += "</div>";
}

var node = document.createElement("div");
node.className = "vibe-task-node";
node.style.cssText =
"background: var(--bg);border:" +
(isFirst
? "2px solid var(--accent)"
: "1px solid var(--border)") +
";border-radius:8px;width:280px;box-shadow:0 " +
(isFirst ? "4" : "2") +
"px 12px rgba(" +
(isFirst ? "132,214,105,0.15" : "0,0,0,0.05") +
");position:relative;flex-shrink:0;animation:nodeIn 0.4s ease;";

node.innerHTML =
'<div style="padding:12px 16px;border-bottom: 1px solid var(--border);">' +
'<div style="display:flex;justify-content:space-between;margin-bottom:8px;font-size:10px;color: var(--text-muted);">' +
"<span>" +
fileCount +
" files</span><span>" +
time +
"</span><span>" +
tokens +
"</span>" +
"</div>" +
'<h4 style="margin:0 0 8px 0;font-size:14px;color: var(--text);font-weight:700;">' +
esc(title) +
"</h4>" +
'<p style="margin:0;font-size:11px;color: var(--text-muted);line-height:1.4;">' +
esc(description) +
"</p>" +
"</div>" +
'<div style="padding:10px 16px;background: var(--surface);border-bottom: 1px solid var(--border);font-size:11px;">' +
'<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;">' +
'<span style="color: var(--text-muted);">Status</span>' +
'<span style="background:' +
statusBg +
";color:" +
statusColor +
';padding:2px 8px;border-radius:12px;font-weight:600;">' +
esc(status) +
"</span>" +
"</div>" +
'<div style="display:flex;justify-content:space-between;align-items:center;">' +
'<span style="color: var(--text-muted);">Mantis Manager</span>' +
'<span style="display:flex;align-items:center;gap:4px;"><span class="as-status-dot green"></span> Mantis #1</span>' +
"</div>" +
"</div>" +
'<div style="padding:8px 16px;font-size:10px;font-weight:700;color: var(--text-muted);">' +
'<div data-toggle="' +
nodeId +
"-files\" style=\"padding:4px 0;cursor:pointer;user-select:none;\" onclick=\"(function(el){var t=document.getElementById(el.getAttribute('data-toggle'));if(t){t.style.display=t.style.display==='none'?'':'none';var a=el.querySelector('span');if(a)a.textContent=t.style.display==='none'?'▶':'▼';}})(this)\">// SUB-TASKS <span style=\"float:right;\">▶</span></div>" +
'<div style="padding:4px 0;cursor:pointer;">// LOGS <span style="float:right;">▶</span></div>' +
"</div>" +
subTasksHtml;

if (isFirst || stepsContainer.children.length > 0) {
var line = document.createElement("div");
line.style.cssText =
"position:absolute;right:-60px;top:50%;width:60px;height:2px;background:var(--accent);z-index:10;";
node.appendChild(line);
if (!isFirst) {
var dot = document.createElement("div");
dot.style.cssText =
"position:absolute;left:-5px;top:50%;transform:translateY(-50%);width:10px;height:10px;border-radius:50%;background:var(--accent);z-index:20;";
node.appendChild(dot);
}
}

stepsContainer.appendChild(node);
stepsContainer.scrollLeft = stepsContainer.scrollWidth;
taskNodes.push({
title: title,
description: description,
meta: meta,
});
return node;
}
