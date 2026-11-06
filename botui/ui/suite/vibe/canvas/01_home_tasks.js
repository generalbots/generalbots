/* Vibe Canvas — split from vibe/vibe-canvas.js for #1731 */

function vibeGoHome() {
var stepsContainer = document.getElementById("vibeSteps");
if (stepsContainer) {
stepsContainer.innerHTML = "";
stepsContainer.style.display = "none";
}
var emptyState = document.getElementById("vibeCanvasEmpty");
if (emptyState) emptyState.style.display = "flex";
nodeIdCounter = 0;
try { if (window.VibeDialogs) window.VibeDialogs.close(); } catch (e) {}
try { if (window.VibeNewProject) window.VibeNewProject.close(); } catch (e) {}
try { if (window.VibeMembers) window.VibeMembers.close(); } catch (e) {}
try { if (window.VibeGraph) window.VibeGraph.togglePanel(false); } catch (e) {}
}

function addTaskNode(title, description, meta) {
var stepsContainer = document.getElementById("vibeSteps");
if (!stepsContainer) return;
stepsContainer.style.display = "flex";
var emptyState = document.getElementById("vibeCanvasEmpty");
if (emptyState) emptyState.style.display = "none";

nodeIdCounter++;
meta = meta || {};
// Only real values are shown — the previous Math.random() estimates
// fabricated file/time/token counts (removed 2026-08-14).
var fileCount = meta.estimated_files || meta.files || null;
var time = meta.estimated_time || meta.time || null;
var tokens = meta.estimated_tokens || meta.tokens || null;
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
'<div style="padding:2px 0;display:flex;align-items:center;gap:4px;"><span style="color: var(--accent); display:inline-flex" aria-hidden="true"><svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg></span> ' +
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

var metaRow = [];
if (fileCount) metaRow.push(fileCount + " files");
if (time) metaRow.push(time);
if (tokens) metaRow.push(tokens);
var metaHtml = metaRow.length
? '<div style="display:flex;justify-content:space-between;margin-bottom:8px;font-size:10px;color: var(--text-muted);">' +
  metaRow.map(function (m) { return "<span>" + esc(m) + "</span>"; }).join("") +
  "</div>"
: "";

node.innerHTML =
'<div style="padding:12px 16px;border-bottom: 1px solid var(--border);">' +
metaHtml +
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
'<span style="color: var(--text-muted);">Vibe Manager</span>' +
'<span style="display:flex;align-items:center;gap:4px;"><span class="as-status-dot green"></span> Vibe Assistant</span>' +
"</div>" +
"</div>" +
'<div style="padding:8px 16px;font-size:10px;font-weight:700;color: var(--text-muted);">' +
'<div data-toggle="' +
nodeId +
"-files\" style=\"padding:4px 0;cursor:pointer;user-select:none;\" onclick=\"(function(el){var t=document.getElementById(el.getAttribute('data-toggle'));if(t){t.style.display=t.style.display==='none'?'':'none';var a=el.querySelector('span');if(a)a.textContent=t.style.display==='none'?'▶':'▼';}})(this)\">// SUB-TASKS <span style=\"float:right;\">▶</span></div>" +
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
