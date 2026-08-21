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
if (window.VibePipeline) window.VibePipeline.activate("plan");
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

var vibeZoomLevel = 100;

function vibeSetZoom(delta) {
    var steps = document.getElementById("vibeSteps");
    var design = document.getElementById("vibeDesignSurface");
    vibeZoomLevel = Math.min(200, Math.max(40, vibeZoomLevel + (delta * 10)));
    var scale = vibeZoomLevel / 100;
    if (steps) {
        steps.style.transformOrigin = "0 0";
        steps.style.transform = "scale(" + scale + ")";
    }
    if (design) design.style.transform = "scale(" + scale + ")";
    var labels = document.querySelectorAll("[data-vibe-zoom-label]");
    labels.forEach(function (label) {
        label.textContent = vibeZoomLevel + "%";
    });
}

document.addEventListener("click", function (e) {
    var btn = e.target.closest("[data-vibe-zoom]");
    if (!btn) return;
    vibeSetZoom(parseInt(btn.getAttribute("data-vibe-zoom"), 10) || 0);
});

(function () {
    "use strict";

    var state = { elements: [], connectors: [] };
    var tool = "select";
    var selectedId = null;
    var gesture = null;
    var canvasId = null;
    var loadedProject = null;
    var saveTimer = null;

    function projectKey() {
        return typeof window.currentProjectId !== "undefined" && window.currentProjectId
            ? String(window.currentProjectId)
            : (typeof window.currentProject !== "undefined" && window.currentProject ? String(window.currentProject) : "unsaved");
    }

    function setStatus(text) {
        var el = document.getElementById("vibeCanvasSaveState");
        if (el) el.textContent = text;
    }

    function setMode(mode) {
        var windowRoot = document.getElementById("vibeWindow");
        var design = document.getElementById("vibeDesignSurface");
        var steps = document.getElementById("vibeSteps");
        var empty = document.getElementById("vibeCanvasEmpty");
        var isDesign = mode === "design";
        if (windowRoot) windowRoot.classList.toggle("vibe-design-mode", isDesign);
        if (design) design.classList.toggle("active", isDesign);
        if (steps) steps.style.display = !isDesign && steps.children.length ? "flex" : "none";
        if (empty) empty.style.display = !isDesign && (!steps || !steps.children.length) ? "flex" : "none";
        document.querySelectorAll("[data-vibe-mode], [data-vibe-tool]").forEach(function (button) {
            button.classList.toggle("active", isDesign ? button.getAttribute("data-vibe-tool") === tool : button.hasAttribute("data-vibe-mode"));
        });
        if (isDesign) loadCanvas();
    }

    function activateTool(nextTool) {
        tool = nextTool || "select";
        var surface = document.getElementById("vibeDesignSurface");
        if (surface) surface.setAttribute("data-tool", tool);
        setMode("design");
    }

    function render() {
        var host = document.getElementById("vibeDesignElements");
        var svg = document.getElementById("vibeDesignConnectors");
        if (!host || !svg) return;
        host.innerHTML = "";
        state.elements.forEach(function (item) {
            var el = document.createElement("div");
            el.className = "vibe-design-element " + item.type + (item.id === selectedId ? " selected" : "");
            el.dataset.designId = item.id;
            el.style.left = item.x + "px";
            el.style.top = item.y + "px";
            el.style.width = item.w + "px";
            el.style.height = item.h + "px";
            if (item.type === "text") el.textContent = item.text || "Text";
            host.appendChild(el);
        });
        svg.innerHTML = '<defs><marker id="vibeArrowhead" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#2563eb"></path></marker></defs>' +
            state.connectors.map(function (line) {
                return '<line class="vibe-design-connector" x1="' + line.x1 + '" y1="' + line.y1 + '" x2="' + line.x2 + '" y2="' + line.y2 + '"></line>';
            }).join("");
    }

    function point(event) {
        var surface = document.getElementById("vibeDesignSurface");
        var rect = surface.getBoundingClientRect();
        var scale = vibeZoomLevel / 100;
        return { x: (event.clientX - rect.left) / scale, y: (event.clientY - rect.top) / scale };
    }

    function onPointerDown(event) {
        if (event.button !== 0) return;
        var p = point(event);
        var hit = event.target.closest("[data-design-id]");
        if (tool === "select") {
            selectedId = hit ? hit.dataset.designId : null;
            var item = state.elements.find(function (entry) { return entry.id === selectedId; });
            if (item) gesture = { kind: "move", id: item.id, x: p.x, y: p.y, ox: item.x, oy: item.y };
            render();
            return;
        }
        if (tool === "text") {
            var label = window.prompt("Text", "Heading");
            if (label) {
                state.elements.push({ id: "el-" + Date.now(), type: "text", x: p.x, y: p.y, w: 180, h: 32, text: label });
                render(); scheduleSave();
            }
            return;
        }
        if (tool === "rectangle") {
            var id = "el-" + Date.now();
            state.elements.push({ id: id, type: "rectangle", x: p.x, y: p.y, w: 1, h: 1 });
            selectedId = id;
            gesture = { kind: "rectangle", id: id, x: p.x, y: p.y };
        } else if (tool === "connector") {
            var connector = { id: "line-" + Date.now(), x1: p.x, y1: p.y, x2: p.x, y2: p.y };
            state.connectors.push(connector);
            gesture = { kind: "connector", id: connector.id };
        }
        event.currentTarget.setPointerCapture(event.pointerId);
        render();
    }

    function onPointerMove(event) {
        if (!gesture) return;
        var p = point(event);
        if (gesture.kind === "move") {
            var moving = state.elements.find(function (entry) { return entry.id === gesture.id; });
            if (moving) { moving.x = gesture.ox + p.x - gesture.x; moving.y = gesture.oy + p.y - gesture.y; }
        } else if (gesture.kind === "rectangle") {
            var rect = state.elements.find(function (entry) { return entry.id === gesture.id; });
            if (rect) {
                rect.x = Math.min(gesture.x, p.x); rect.y = Math.min(gesture.y, p.y);
                rect.w = Math.abs(p.x - gesture.x); rect.h = Math.abs(p.y - gesture.y);
            }
        } else if (gesture.kind === "connector") {
            var line = state.connectors.find(function (entry) { return entry.id === gesture.id; });
            if (line) { line.x2 = p.x; line.y2 = p.y; }
        }
        render();
    }

    function onPointerUp() {
        if (!gesture) return;
        if (gesture.kind === "rectangle") {
            var rect = state.elements.find(function (entry) { return entry.id === gesture.id; });
            if (rect && (rect.w < 12 || rect.h < 12)) { rect.w = 160; rect.h = 96; }
        }
        gesture = null;
        render();
        scheduleSave();
    }

    function api(path, options) {
        options = options || {};
        options.headers = Object.assign({ "Content-Type": "application/json" }, options.headers || {});
        return vibeAuthFetch(path, options).then(function (response) {
            return response.json().then(function (data) {
                if (!response.ok || data.success === false) throw new Error(data.error || ("HTTP " + response.status));
                return data;
            });
        });
    }

    function loadCanvas() {
        var project = projectKey();
        if (loadedProject === project) return;
        loadedProject = project;
        canvasId = null;
        setStatus("Loading…");
        api("/api/vibe/canvases", { method: "GET", headers: {} }).then(function (data) {
            var rows = data.canvases || [];
            var match = rows.filter(function (row) {
                return String(row.project || "") === project && row.content && row.content.kind === "vibe-design";
            }).sort(function (a, b) { return String(b.updated_at).localeCompare(String(a.updated_at)); })[0];
            if (match) {
                canvasId = match.canvas_id;
                state.elements = Array.isArray(match.content.elements) ? match.content.elements : [];
                state.connectors = Array.isArray(match.content.connectors) ? match.content.connectors : [];
            } else {
                state = { elements: [], connectors: [] };
            }
            render(); setStatus("Saved");
        }).catch(function (error) { setStatus("Local changes only: " + error.message); });
    }

    function scheduleSave() {
        setStatus("Unsaved");
        clearTimeout(saveTimer);
        saveTimer = setTimeout(saveCanvas, 500);
    }

    function saveCanvas() {
        var content = { kind: "vibe-design", version: 1, elements: state.elements, connectors: state.connectors };
        var request = canvasId
            ? api("/api/vibe/canvases/" + encodeURIComponent(canvasId), { method: "PUT", body: JSON.stringify({ content: content }) })
            : api("/api/vibe/canvases", { method: "POST", body: JSON.stringify({ title: "Vibe Design", project: projectKey(), content: content }) });
        setStatus("Saving…");
        request.then(function (data) {
            if (data.canvas) canvasId = data.canvas.canvas_id;
            setStatus("Saved");
        }).catch(function (error) { setStatus("Save failed: " + error.message); });
    }

    function init() {
        var surface = document.getElementById("vibeDesignSurface");
        if (!surface || surface.dataset.ready === "1") return;
        surface.dataset.ready = "1";
        surface.addEventListener("pointerdown", onPointerDown);
        surface.addEventListener("pointermove", onPointerMove);
        surface.addEventListener("pointerup", onPointerUp);
        surface.addEventListener("pointercancel", onPointerUp);
        document.addEventListener("click", function (event) {
            var toolButton = event.target.closest("[data-vibe-tool]");
            if (toolButton) activateTool(toolButton.getAttribute("data-vibe-tool"));
            if (event.target.closest("[data-vibe-mode='flow']")) setMode("flow");
            if (event.target.closest("[data-vibe-zoom-reset]")) {
                vibeZoomLevel = 100; vibeSetZoom(0);
            }
            if (event.target.closest("[data-vibe-canvas-clear]") && window.confirm("Clear all design elements?")) {
                state = { elements: [], connectors: [] }; selectedId = null; render(); scheduleSave();
            }
        });
        document.getElementById("vibeCanvas").addEventListener("wheel", function (event) {
            if (!event.ctrlKey) return;
            event.preventDefault();
            vibeSetZoom(event.deltaY < 0 ? 1 : -1);
        }, { passive: false });
        document.addEventListener("gb:vibe-project", function () {
            loadedProject = null;
            loadCanvas();
        });
    }

    window.VibeDesign = { init: init, activate: activateTool, render: render };
    init();
})();
