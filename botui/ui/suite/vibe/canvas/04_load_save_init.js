/* Vibe Canvas — split from vibe/vibe-canvas.js for #1731 */
(function () {
    "use strict";
    var V = window.VibeDesign;

    var canvasId = null;
    var loadedProject = null;
    var saveTimer = null;
    function readProjectDraw(pid) {
        return V.api("/api/vibe/projects/" + encodeURIComponent(pid) +
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
        return V.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files",
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
        if (project === "unsaved" || V.state.generated) return;
        var pid = V.projectId();
        if (pid) {
            V.api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files", { method: "GET", headers: {} })
                .then(function (d) {
                    var files = (d && (d.files || d.data && d.data.files)) || [];
                    if (!files.length) throw new Error("no files");
                    var design = { kind: "vibe-design", version: 3, generatedAt: new Date().toISOString(),
                        source: "workspace", title: project, svg: svgForWorkspace(files, project) };
                    V.setGeneratedDesign(design);
                    V.render();
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
        var project = V.projectKey();
        if (loadedProject === project) return;
        loadedProject = project;
        canvasId = null;
        V.setStatus("Loading…");
        var pid = V.projectId();

        var fallbackCanvases = function () {
            V.api("/api/vibe/canvases", { method: "GET", headers: {} }).then(function (data) {
                var rows = data.canvases || [];
                var match = rows.filter(function (row) {
                    return String(row.project || "") === project && row.content && row.content.kind === "vibe-design";
                }).sort(function (a, b) { return String(b.updated_at).localeCompare(String(a.updated_at)); })[0];
                if (match) {
                    canvasId = match.canvas_id;
                    V.applyContent(match.content);
                } else {
                    V.state = { elements: [], connectors: [], generated: null };
                }
                V.render();
                maybeGenerateBaseline(project);
                V.setStatus("Saved");
            }).catch(function (error) { V.setStatus("Local changes only: " + error.message); });
        };

        // Prefer the persistent, exportable file at the workspace root.
        if (!pid) { fallbackCanvases(); return; }
        readProjectDraw(pid).then(function (parsed) {
            V.applyContent(parsed);
            V.render();
            V.setStatus("Saved · project.draw ✓");
        }).catch(fallbackCanvases);
    }

    function scheduleSave() {
        V.setStatus("Unsaved");
        clearTimeout(saveTimer);
        saveTimer = setTimeout(saveCanvas, 500);
    }

    function saveCanvas() {
        var content = { kind: "vibe-design", version: 2, elements: V.state.elements, connectors: V.state.connectors, generated: V.state.generated };

        var request = canvasId
            ? V.api("/api/vibe/canvases/" + encodeURIComponent(canvasId), { method: "PUT", body: JSON.stringify({ content: content }) })
            : V.api("/api/vibe/canvases", { method: "POST", body: JSON.stringify({ title: "Vibe Design", project: V.projectKey(), content: content }) });
        V.setStatus("Saving…");
        request.then(function (data) {
            if (data.canvas) canvasId = data.canvas.canvas_id;
            // Mirror the design to project.draw at the root of the workspace
            // so the architecture travels with the project files (#1191).
            var pid = V.projectId();
            if (!pid) { V.setStatus("Saved"); return; }
            writeProjectDraw(pid, content)
                .then(function () { V.setStatus("Saved · project.draw ✓"); })
                .catch(function () { V.setStatus("Saved"); });
        }).catch(function (error) { V.setStatus("Save failed: " + error.message); });
    }

    function init() {
        // Bind EVERY design surface (parked panel + standalone app window):
        // the visible one receives the pointer events that matter.
        document.querySelectorAll("#vibeDesignSurface").forEach(function (surface) {
            if (!surface || surface.dataset.ready === "1") return;
            surface.dataset.ready = "1";
            surface.addEventListener("pointerdown", V.onPointerDown);
            surface.addEventListener("pointermove", V.onPointerMove);
            surface.addEventListener("pointerup", V.onPointerUp);
            surface.addEventListener("pointercancel", V.onPointerUp);
        });
        document.addEventListener("click", function (event) {
            var toolButton = event.target.closest("[data-vibe-tool]");
            if (toolButton) V.activate(toolButton.getAttribute("data-vibe-tool"));
            if (event.target.closest("[data-vibe-mode='flow']")) V.setMode("flow");
            if (event.target.closest("[data-vibe-zoom-reset]")) {
                vibeZoomLevel = 100; vibeSetZoom(0);
            }
            if (event.target.closest("[data-vibe-canvas-clear]")) {
                const doClear = function () {
                    V.state = { elements: [], connectors: [], generated: V.state.generated };

                    V.selectedId = null;
                    V.render();
                    scheduleSave();
                };
                if (window.WindowManager && window.WindowManager.confirmFloating) {
                    window.WindowManager.confirmFloating(
                        "Clear canvas",
                        "Clear all design elements?",
                        doClear,
                        null
                    );
                } else if (window.GBModal && typeof window.GBModal.confirm === "function") {
                    window.GBModal.confirm("Clear all design elements?", "Clear canvas", doClear, function () {});
                }
            }
        });
        var wheelHost = V.visEl("vibeCanvas");
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

    V.load = loadCanvas;
    V.scheduleSave = scheduleSave;
    V.saveSoon = scheduleSave;
    V.writeProjectDraw = writeProjectDraw;
    V.init = init;
})();
