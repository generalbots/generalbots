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

// Top-level twin of the module's visEl: prefer the canvas in the standalone
// app window, else the visible inline surface (both carry the same ids).
function vibeVis(id) {
    var all = document.querySelectorAll("#" + id);
    if (all.length <= 1) return all[0] || null;
    var appWin = document.getElementById("window-vibe-canvas");
    if (appWin) {
        var inApp = appWin.querySelector("#" + id);
        if (inApp && inApp.offsetParent !== null) return inApp;
    }
    for (var i = 0; i < all.length; i++) { if (all[i].offsetParent !== null) return all[i]; }
    return all[all.length - 1];
}

function vibeSetZoom(delta) {
    // Same visible-copy resolution as the other surfaces: getElementById
    // returns the first match, which is the parked hidden copy inside the
    // Vibe window, not the visible canvas — zooming it was a no-op.
    var steps = vibeVis("vibeSteps");
    var design = vibeVis("vibeDesignSurface");
    var generated = vibeVis("vibeGeneratedLayer");
    vibeZoomLevel = Math.min(200, Math.max(40, vibeZoomLevel + (delta * 10)));
    var scale = vibeZoomLevel / 100;
    if (steps) {
        steps.style.transformOrigin = "0 0";
        steps.style.transform = "scale(" + scale + ")";
    }
    // Scale the drawable surface AND the generated architecture behind it
    // from the top-left corner so +/- zoom visibly works (was a no-op).
    if (design) {
        design.style.transformOrigin = "0 0";
        design.style.transform = "scale(" + scale + ")";
    }
    if (generated) {
        generated.style.transformOrigin = "0 0";
        generated.style.transform = "scale(" + scale + ")";
    }
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

    var state = { elements: [], connectors: [], generated: null };

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
        var el = visEl("vibeCanvasSaveState");
        if (el) el.textContent = text;
    }

    function setMode(mode) {
        var windowRoot = document.getElementById("vibeWindow");
        var design = visEl("vibeDesignSurface");
        var steps = visEl("vibeSteps");
        var empty = visEl("vibeCanvasEmpty");
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
        var surface = visEl("vibeDesignSurface");
        if (surface) surface.setAttribute("data-tool", tool);
        setMode("design");
    }

    function setGeneratedDesign(design) {
        state.generated = design || null;
        var host = visEl("vibeGeneratedLayer");
        if (host) host.innerHTML = state.generated && state.generated.svg ? state.generated.svg : "";
    }

    function render() {
        var host = visEl("vibeDesignElements");
        var svg = visEl("vibeDesignConnectors");
        if (!host || !svg) return;
        var generatedHost = visEl("vibeGeneratedLayer");
        if (generatedHost) generatedHost.innerHTML = state.generated && state.generated.svg ? state.generated.svg : "";
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
        var surface = visEl("vibeDesignSurface");
        if (!surface) return { x: 0, y: 0 };
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
            var addLabel = function (label) {
                if (label) {
                    state.elements.push({ id: "el-" + Date.now(), type: "text", x: p.x, y: p.y, w: 180, h: 32, text: label });
                    render(); scheduleSave();
                }
            };
            if (window.WindowManager && window.WindowManager.promptFloating) {
                window.WindowManager.promptFloating("Add text", "Text", "Heading", addLabel);
            } else {
                addLabel(window.prompt("Text", "Heading"));
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

    // project.draw lives at the ROOT of the project workspace (#1191): one
    // portable artifact holding the whole architecture drawing. Preference
    // order when loading: project.draw file → canvases API store → baseline.
    var UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

    function projectId() {
        var project = projectKey();
        return UUID_RE.test(project) ? project : null;
    }

    // The inline canvas in the Vibe main window and the standalone Vibe
    // Canvas app window both carry the same element ids. Prefer the copy in
    // the standalone app window when that window is open and visible (that
    // is where the user draws), otherwise the visible inline one.
    function visEl(id) {
        var all = document.querySelectorAll("#" + id);
        if (all.length <= 1) return all[0] || null;
        var appWin = document.getElementById("window-vibe-canvas");
        if (appWin) {
            var inApp = appWin.querySelector("#" + id);
            if (inApp && inApp.offsetParent !== null) return inApp;
        }
        for (var i = 0; i < all.length; i++) {
            if (all[i].offsetParent !== null) return all[i];
        }
        return all[all.length - 1];
    }

    function applyContent(content) {
        state.elements = Array.isArray(content.elements) ? content.elements : [];
        state.connectors = Array.isArray(content.connectors) ? content.connectors : [];
        setGeneratedDesign(content.generated || null);
    }

    function readProjectDraw(pid) {
        return api("/api/vibe/projects/" + encodeURIComponent(pid) +
            "/files/content?path=" + encodeURIComponent("project.draw"), { method: "GET", headers: {} })
            .then(function (d) {
                var text = d && d.content;
                if (!text) throw new Error("no project.draw yet");
                var parsed = JSON.parse(text);
                if (!parsed || parsed.kind !== "vibe-design") throw new Error("not a vibe design file");
                return parsed;
            });
    }

    function writeProjectDraw(pid, content) {
        return api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files",
            { method: "POST", body: JSON.stringify({ path: "project.draw", content: JSON.stringify(content, null, 2) }) });
    }

    // Draw the architecture FROM the project itself: fetch the real
    // workspace file list and lay out the runtime zones (Web / App / API /
    // Data) using actual module names, then persist it to project.draw.
    function zoneFor(path) {
        var p = String(path || "/").toLowerCase();
        if (p.indexOf("api") !== -1 || p.indexOf("service") !== -1) return "API";
        if (p.indexOf("db") !== -1 || p.indexOf("sql") !== -1 || p.indexOf("data") !== -1 || p.indexOf("model") !== -1) return "Data";
        if (p.indexOf("ui") !== -1 || p.indexOf("web") !== -1 || p.indexOf("html") !== -1 || p.indexOf("public") !== -1) return "Web";
        return "App";
    }

    function svgForWorkspace(files, title) {
        var zones = { Web: [], App: [], API: [], Data: [] };
        (files || []).slice(0, 60).forEach(function (f) {
            var name = String(f).split("/").pop();
            if (!name) return;
            var z = zoneFor(f);
            if (zones[z].length < 4) zones[z].push(name);
        });
        if (!zones.Web.length) zones.Web = ["browser"];
        if (!zones.App.length) zones.App = ["runtime"];
        var cols = ["Web", "App", "API", "Data"];
        var colors = ["#2563eb", "#16a34a", "#d97706", "#7c3aed"];
        var h = "";
        var cx = 40;
        cols.forEach(function (col, i) {
            var x = cx + i * 220;
            var items = zones[col];
            h += '<rect x="' + x + '" y="64" width="180" height="" fill="none" stroke="' + colors[i] + '" stroke-width="2" rx="10"/>';
            h += '<text x="' + (x + 14) + '" y="90" font-family="system-ui" font-size="15" font-weight="700" fill="' + colors[i] + '">' + col + "</text>";
            var y = 118;
            items.forEach(function (item) {
                h += '<text x="' + (x + 14) + '" y="' + y + '" font-family="monospace" font-size="11" fill="#334155">\u2022 ' + item + "</text>";
                y += 20;
            });
        });
        return '<svg viewBox="0 0 960 520" xmlns="http://www.w3.org/2000/svg">' +
            '<rect x="20" y="24" width="920" height="476" rx="14" fill="#f8fafc" stroke="#cbd5e1"/>' +
            '<text x="44" y="52" font-family="system-ui" font-size="16" font-weight="700" fill="#0f172a">' + (title || "Project") + " architecture</text>" +
            h + "</svg>";
    }

    function maybeGenerateBaseline(project) {
        if (project === "unsaved" || state.generated) return;
        var pid = projectId();
        if (pid) {
            api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files", { method: "GET", headers: {} })
                .then(function (d) {
                    var files = (d && (d.files || d.data && d.data.files)) || [];
                    if (!files.length) throw new Error("no files");
                    var design = { kind: "vibe-design", version: 3, generatedAt: new Date().toISOString(),
                        source: "workspace", title: project, svg: svgForWorkspace(files, project) };
                    setGeneratedDesign(design);
                    render();
                    scheduleSave();
                })
                .catch(function () {
                    if (window.VibeCanvasViews && window.VibeCanvasViews.generateProjectDesign) {
                        window.VibeCanvasViews.generateProjectDesign(false);
                    }
                });
            return;
        }
        if (window.VibeCanvasViews && window.VibeCanvasViews.generateProjectDesign) {
            window.VibeCanvasViews.generateProjectDesign(false);
        }
    }

    function loadCanvas() {
        var project = projectKey();
        if (loadedProject === project) return;
        loadedProject = project;
        canvasId = null;
        setStatus("Loading…");
        var pid = projectId();

        var fallbackCanvases = function () {
            api("/api/vibe/canvases", { method: "GET", headers: {} }).then(function (data) {
                var rows = data.canvases || [];
                var match = rows.filter(function (row) {
                    return String(row.project || "") === project && row.content && row.content.kind === "vibe-design";
                }).sort(function (a, b) { return String(b.updated_at).localeCompare(String(a.updated_at)); })[0];
                if (match) {
                    canvasId = match.canvas_id;
                    applyContent(match.content);
                } else {
                    state = { elements: [], connectors: [], generated: null };
                }
                render();
                maybeGenerateBaseline(project);
                setStatus("Saved");
            }).catch(function (error) { setStatus("Local changes only: " + error.message); });
        };

        // Prefer the persistent, exportable file at the workspace root.
        if (!pid) { fallbackCanvases(); return; }
        readProjectDraw(pid).then(function (parsed) {
            applyContent(parsed);
            render();
            setStatus("Saved · project.draw ✓");
        }).catch(fallbackCanvases);
    }

    function scheduleSave() {
        setStatus("Unsaved");
        clearTimeout(saveTimer);
        saveTimer = setTimeout(saveCanvas, 500);
    }

    function saveCanvas() {
        var content = { kind: "vibe-design", version: 2, elements: state.elements, connectors: state.connectors, generated: state.generated };

        var request = canvasId
            ? api("/api/vibe/canvases/" + encodeURIComponent(canvasId), { method: "PUT", body: JSON.stringify({ content: content }) })
            : api("/api/vibe/canvases", { method: "POST", body: JSON.stringify({ title: "Vibe Design", project: projectKey(), content: content }) });
        setStatus("Saving…");
        request.then(function (data) {
            if (data.canvas) canvasId = data.canvas.canvas_id;
            // Mirror the design to project.draw at the root of the workspace
            // so the architecture travels with the project files (#1191).
            var pid = projectId();
            if (!pid) { setStatus("Saved"); return; }
            writeProjectDraw(pid, content)
                .then(function () { setStatus("Saved · project.draw ✓"); })
                .catch(function () { setStatus("Saved"); });
        }).catch(function (error) { setStatus("Save failed: " + error.message); });
    }

    function init() {
        // Bind EVERY design surface (parked panel + standalone app window):
        // the visible one receives the pointer events that matter.
        document.querySelectorAll("#vibeDesignSurface").forEach(function (surface) {
            if (!surface || surface.dataset.ready === "1") return;
            surface.dataset.ready = "1";
            surface.addEventListener("pointerdown", onPointerDown);
            surface.addEventListener("pointermove", onPointerMove);
            surface.addEventListener("pointerup", onPointerUp);
            surface.addEventListener("pointercancel", onPointerUp);
        });
        document.addEventListener("click", function (event) {
            var toolButton = event.target.closest("[data-vibe-tool]");
            if (toolButton) activateTool(toolButton.getAttribute("data-vibe-tool"));
            if (event.target.closest("[data-vibe-mode='flow']")) setMode("flow");
            if (event.target.closest("[data-vibe-zoom-reset]")) {
                vibeZoomLevel = 100; vibeSetZoom(0);
            }
            if (event.target.closest("[data-vibe-canvas-clear]")) {
                const doClear = function () {
                    state = { elements: [], connectors: [], generated: state.generated };

                    selectedId = null;
                    render();
                    scheduleSave();
                };
                if (window.WindowManager && window.WindowManager.confirmFloating) {
                    window.WindowManager.confirmFloating(
                        "Clear canvas",
                        "Clear all design elements?",
                        doClear,
                        null
                    );
                } else if (window.confirm("Clear all design elements?")) {
                    doClear();
                }
            }
        });
        var wheelHost = visEl("vibeCanvas");
        if (wheelHost) wheelHost.addEventListener("wheel", function (event) {
            if (!event.ctrlKey) return;
            event.preventDefault();
            vibeSetZoom(event.deltaY < 0 ? 1 : -1);
        }, { passive: false });
        document.addEventListener("gb:vibe-project", function () {
            loadedProject = null;
            loadCanvas();
        });
    }

    // Draw-by-chat offline parser (#1191). Recognized grammar:
    //   add rect[angle] <label> | rectangle <label> | box <label>
    //   add text <text>          | text <text>
    //   connect|link A -> B      | connect A to B
    //   rename <from> to <to>    | delete|remove <label> | clear
    function chatFeedback(msg) {
        var bar = document.getElementById("vibeDesignChatLog");
        if (bar) {
            bar.textContent = msg;
            clearTimeout(chatFeedback._t);
            chatFeedback._t = setTimeout(function () { bar.textContent = ""; }, 5000);
        }
        setStatus(msg);
    }

    function findLabelled(label) {
        var needle = label.toLowerCase();
        return state.elements.filter(function (e) {
            return e.text && String(e.text).toLowerCase() === needle;
        });
    }

    function labelAnchor(label) {
        var items = findLabelled(label);
        var best = null;
        items.forEach(function (e) {
            if (!best || (e.type === "rectangle" && best.type !== "rectangle")) best = e;
        });
        return best;
    }

    function addRect(label, opts) {
        opts = opts || {};
        var base = Date.now() + Math.floor(Math.random() * 1000);
        var rect = { id: "el-" + base, type: "rectangle", x: opts.x != null ? opts.x : 80 + (state.elements.length % 5) * 40,
            y: opts.y != null ? opts.y : 80 + Math.floor(state.elements.length / 5) * 40 + 60, w: 170, h: 90 };
        state.elements.push(rect);
        if (label) {
            state.elements.push({ id: "el-" + base + "t", type: "text", x: rect.x + 12, y: rect.y + rect.h / 2 - 14,
                w: Math.max(60, rect.w - 24), h: 28, text: label });
        }
        selectedId = rect.id;
        render(); scheduleSave();
        return rect;
    }

    function centerOf(el) { return { x: el.x + el.w / 2, y: el.y + el.h / 2 }; }

    function connect(fromLabel, toLabel) {
        var a = labelAnchor(fromLabel), b = labelAnchor(toLabel);
        if (!a || !b) return false;
        var ca = centerOf(a), cb = centerOf(b);
        state.connectors.push({ id: "line-" + Date.now(), x1: ca.x, y1: ca.y, x2: cb.x, y2: cb.y });
        render(); scheduleSave();
        return true;
    }

    function chatCommand(raw) {
        var text = raw.trim().replace(/;+$/g, "");
        if (!text) return false;
        var m;
        if ((m = text.match(/^(?:add\s+)?(?:rect(?:angle)?|box)\s+(.{1,40})$/i))) {
            addRect(m[1].trim());
            chatFeedback("▭ Added “" + m[1].trim() + "”");
            return true;
        }
        if ((m = text.match(/^(?:add\s+)?text\s+(.{1,120})$/i))) {
            var t = m[1].trim();
            state.elements.push({ id: "el-" + Date.now(), type: "text", x: 110, y: 110 + (state.elements.length % 6) * 30, w: 220, h: 30, text: t });
            render(); scheduleSave();
            chatFeedback("🔤 Added text “" + t + "”");
            return true;
        }
        if ((m = text.match(/^(?:connect|link)\s+['"]?(.+?)['"]?\s*(?:->|→|to)\s*['"]?(.+?)['"]?$/i))) {
            if (connect(m[1].trim(), m[2].trim())) {
                chatFeedback("↔ Connected “" + m[1].trim() + "” → “" + m[2].trim() + "”");
                return true;
            }
            chatFeedback("⚠ Not found. Try: add rect " + m[1].trim() + " first.");
            return true;
        }
        if ((m = text.match(/^rename\s+['"]?(.+?)['"]?\s+to\s+['"]?(.+?)['"]?$/i))) {
            var target = labelAnchor(m[1].trim());
            if (!target) { chatFeedback("⚠ No element named “" + m[1].trim() + "”."); return true; }
            target.text = m[2].trim();
            render(); scheduleSave();
            chatFeedback("✏ Renamed → “" + m[2].trim() + "”");
            return true;
        }
        if ((m = text.match(/^(?:delete|remove)\s+['"]?(.+?)['"]?$/i))) {
            var doomed = findLabelled(m[1].trim());
            if (!doomed.length) { chatFeedback("⚠ Nothing named “" + m[1].trim() + "”."); return true; }
            var ids = {}; doomed.forEach(function (d) { ids[d.id] = 1; });
            state.elements = state.elements.filter(function (e) { return !ids[e.id]; });
            render(); scheduleSave();
            chatFeedback("🗑 Removed “" + m[1].trim() + "”");
            return true;
        }
        if (/^clear(\s+all)?$|^reset$/i.test(text)) {
            state = { elements: [], connectors: [], generated: state.generated };
            selectedId = null;
            render(); scheduleSave();
            chatFeedback("🧽 Canvas cleared (baseline kept)");
            return true;
        }
        chatFeedback("Try: add rect API · add text Login · connect UI to API · rename A to B · delete A · clear");
        return false;
    }

    // Pinned draw-by-chat input docked at the bottom of the drawing surface.
    function installChatBar(hostOverride) {
        var host = hostOverride || visEl("vibeCanvas");
        if (!host || host.dataset.chatReady === "1") return;
        if (host.querySelector("#vibeDesignChatInput")) { host.dataset.chatReady = "1"; return; }
        host.dataset.chatReady = "1";
        var wrap = document.createElement("div");
        wrap.style.cssText = [
            "position:absolute", "left:12px", "bottom:12px", "z-index:20",
            "display:flex", "align-items:center", "gap:8px",
            "background:var(--surface,#14142a)", "border:1px solid var(--border,#333)",
            "border-radius:10px", "padding:7px 10px", "box-shadow:0 10px 26px rgba(0,0,0,.35)",
        ].join(";");
        wrap.innerHTML = '<span title="Draw by chat" style="font-size:13px;">💬</span>' +
            '<input id="vibeDesignChatInput" type="text" autocomplete="off" placeholder=' +
            '"draw by chat: add rect API · connect UI to API" ' +
            'style="width:min(340px,42vw);background:transparent;border:none;color:var(--text,#eee);font-size:12px;outline:none;" />' +
            '<span id="vibeDesignChatLog" style="font-size:11px;color:var(--accent,#84d669);max-width:220px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;"></span>';
        host.appendChild(wrap);
        var input = wrap.querySelector("#vibeDesignChatInput");
        input.addEventListener("keydown", function (ev) {
            if (ev.key === "Enter") {
                ev.preventDefault();
                window.VibeDesign && window.VibeDesign.chat(input.value);
                input.value = "";
            }
        });
    }

    // LLM architecture generation: sends the project context through the
    // Vibe assistant (bot LLM = tokenrouter) asking for a strict JSON zone
    // map, then renders it as the generated diagram and persists project.draw.
    function llmGenerateArchitecture() {
        var pid = projectId();
        if (!pid) { chatFeedback("⚠ Select a project first."); return; }
        chatFeedback("✦ Asking the LLM for the architecture…");
        api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files", { method: "GET", headers: {} })
            .then(function (d) {
                var files = (d && (d.files || d.data && d.data.files)) || [];
                var fileList = (files || []).slice(0, 40).join("\n");
                var prompt = "Design the software architecture for this project. Return ONLY a JSON object, no prose, no markdown, with this exact shape: {" +
                    "\"title\":\"<project name>\",\"zones\":[{\"name\":\"Web\",\"items\":[<short module names>]},{\"name\":\"App\",...},{\"name\":\"API\",...},{\"name\":\"Data\",...}]}. " +
                    "Use the real modules found in this project file list, max 5 items per zone, short names only (no paths). Project files:\n" + fileList;
                window.vibeQuickSubmit(prompt);
                var attempts = 0;
                var timer = setInterval(function () {
                    attempts++;
                    // Runner status/bot messages land in the RUNNER LOG since
                    // the in-runner chat overlay was removed.
                    var messages = document.getElementById("vibeRunnerLogList");
                    if (!messages) { if (attempts > 24) clearInterval(timer); return; }
                    var nodes = messages.querySelectorAll(".vibe-bot-msg, .bot-message, .message.bot");
                    if (!nodes.length) return;
                    var last = nodes[nodes.length - 1];
                    var text = (last.textContent || "").trim();
                    var m = text.match(/\{[\s\S]*\}/);
                    if (!m || attempts > 24) { if (attempts > 24) clearInterval(timer); return; }
                    var parsed = null;
                    try { parsed = JSON.parse(m[0]); } catch (e) { /* not JSON yet */ }
                    if (!parsed || !parsed.zones) return;
                    clearInterval(timer);
                    var design = { kind: "vibe-design", version: 3, generatedAt: new Date().toISOString(),
                        source: "llm", title: parsed.title || projectKey(), svg: svgFromZones(parsed.zones, parsed.title || projectKey()) };
                    setGeneratedDesign(design);
                    render();
                    var content = { kind: "vibe-design", version: 3, elements: state.elements, connectors: state.connectors, generated: state.generated };
                    writeProjectDraw(pid, content).then(function () {
                        chatFeedback("✦ LLM architecture saved to project.draw ✓");
                        setStatus("Saved · project.draw (LLM) ✓");
                    }).catch(function () {
                        chatFeedback("✦ Architecture generated (project.draw save pending)");
                    });
                }, 700);
            }).catch(function () {
                chatFeedback("⚠ Could not read project files.");
            });
    }

    function svgFromZones(zones, title) {
        var cols = ["Web", "App", "API", "Data"];
        var colors = { Web: "#2563eb", App: "#16a34a", API: "#d97706", Data: "#7c3aed" };
        var h = "";
        cols.forEach(function (col, i) {
            var z = (zones || []).find(function (x) { return String(x.name).toLowerCase() === String(col).toLowerCase(); });
            var items = (z && z.items) || [];
            if (!items.length) items = [col.toLowerCase()];
            var x = 40 + i * 225;
            h += '<rect x="' + x + '" y="70" width="185" height="340" rx="12" fill="#ffffff" stroke="' + colors[col] + '" stroke-width="2"/>';
            h += '<text x="' + (x + 16) + '" y="98" font-family="system-ui" font-size="16" font-weight="700" fill="' + colors[col] + '">' + col + "</text>";
            var y = 130;
            items.slice(0, 5).forEach(function (item) {
                h += '<rect x="' + (x + 14) + '" y="' + (y - 14) + '" width="157" height="22" rx="6" fill="#f1f5f9"/>';
                h += '<text x="' + (x + 24) + '" y="' + y + '" font-family="monospace" font-size="11" fill="#334155">' + item + "</text>";
                y += 32;
            });
        });
        return '<svg viewBox="0 0 960 520" xmlns="http://www.w3.org/2000/svg">' +
            '<rect x="16" y="20" width="928" height="480" rx="14" fill="#f8fafc" stroke="#cbd5e1"/>' +
            '<text x="40" y="48" font-family="system-ui" font-size="16" font-weight="700" fill="#0f172a">' + (title || "Architecture") + "</text>" +
            h + "</svg>";
    }

    window.VibeDesign = {
        init: init,
        activate: activateTool,
        render: render,
        getState: function () { return state; },
        setGeneratedDesign: setGeneratedDesign,
        saveSoon: scheduleSave,
        load: loadCanvas,
        chat: function (raw) { return chatCommand(String(raw || "")); },
        installChat: function (hostOverride) { return installChatBar(hostOverride); },
        llmGenerate: llmGenerateArchitecture,
    };
    init();
    installChatBar();
})();
