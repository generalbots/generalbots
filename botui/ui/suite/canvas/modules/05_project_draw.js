(function () {
    "use strict";

    function currentProjectId() {
        var p = window.__gbAppParams__ || {};
        var qs = new URLSearchParams(window.location.search);
        return p.project || qs.get("project") || "";
    }

    // JSON POST helper consistent with the rest of the suite.
    function api(url, opts) {
        opts = opts || {};
        var headers = Object.assign({ "Content-Type": "application/json" }, opts.headers || {});
        // Bearer token so the project-files endpoints accept the read/write.
        var token = "";
        if (window.getGBAccessToken) { token = window.getGBAccessToken() || ""; }
        if (!token) token = window.localStorage.getItem("gb-access-token") || "";
        if (!token) token = window.sessionStorage.getItem("gb-access-token") || "";
        if (window.GB_TOKEN) token = window.GB_TOKEN;
        if (token) headers["Authorization"] = "Bearer " + token;
        return fetch(url, {
            method: opts.method || "GET",
            headers: headers,
            body: opts.body || undefined,
        }).then(function (r) { return r.json(); });
    }

    // Map a project.draw design element to the canvas-app element shape.
    function loadProjectDraw() {
        var pid = currentProjectId();
        if (!pid || !/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(pid)) return;
        api("/api/vibe/projects/" + encodeURIComponent(pid) +
            "/files/content?path=" + encodeURIComponent("project.draw"))
            .then(function (d) {
                if (!d || !d.content) return;
                var parsed;
                try { parsed = JSON.parse(d.content); } catch (e) { return; }
                // Render the LLM-generated architecture SVG as a non-interactive
                // background so the user sees the diagram and can draw on top of
                // it; the interactive canvas sits transparent above it.
                var bg = document.getElementById("canvas-bg-svg");
                if (parsed.generated && parsed.generated.svg) {
                    if (bg) bg.innerHTML = parsed.generated.svg;
                    var app = window._canvasApp;
                    if (app && typeof app.setCanvasBackgroundHidden === "function") {
                        app.setCanvasBackgroundHidden();
                    }
                }
                var name = document.getElementById("canvas-name");
                if (name) name.value = (parsed.title || "project") + " · project.draw";

                var items = Array.isArray(parsed.elements) ? parsed.elements : [];
                if (!items.length) return;
                // Push saved design elements into the live canvas via the app API.
                var app = window._canvasApp;
                if (app && typeof app.appendCanvasElements === "function") {
                    app.appendCanvasElements(items.map(toCanvasElement).filter(Boolean));
                }
            })
            .catch(function () { /* no project.draw yet — blank canvas */ });
    }

    // #1408 — a finished Vibe agent run may have rewritten project.draw;
    // reset the canvas and reload it so the open Canvas reflects the change.
    document.addEventListener('gb:vibe-app-refresh', function () {
        var app = window._canvasApp;
        if (app && typeof app.clearCanvas === 'function') app.clearCanvas();
        loadProjectDraw();
    });

    // Persist current elements back to project.draw at the project root.
    function saveProjectDraw() {
        var pid = currentProjectId();
        if (!pid || !/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(pid)) return;
        var app = window._canvasApp;
        var els = app && typeof app.getCanvasElements === "function" ? app.getCanvasElements() : [];
        // Keep the full canvas geometry (#1713): the old mapping collapsed
        // every element to text|rectangle, so ellipses/lines/polylines/images
        // came back as boxes on the next open. `w`/`h` stay for the Vibe
        // design tools that read the legacy shape.
        var designElements = els.map(function (el) {
            var design = {
                id: el.id,
                type: el.type,
                x: Math.round(el.x),
                y: Math.round(el.y),
                w: Math.round(el.width),
                h: Math.round(el.height),
            };
            ["x1", "y1", "x2", "y2", "stroke", "fill", "strokeWidth", "fontSize", "points"].forEach(function (k) {
                if (el[k] !== undefined) design[k] = el[k];
            });
            if (el.text) design.text = el.text;
            if (el.img && el.img.src) design.imgSrc = el.img.src;
            return design;
        });
        var content = {
            kind: "vibe-design",
            version: 2,
            title: "project",
            elements: designElements,
            connectors: [],
            generated: currentGenerated(),
        };
        api("/api/vibe/projects/" + encodeURIComponent(pid) + "/files", {
            method: "POST",
            body: JSON.stringify({ path: "project.draw", content: JSON.stringify(content, null, 2) }),
        })
            .then(function () {
                var status = document.getElementById("save-status");
                if (status) status.textContent = "Saved · project.draw ✓";
            })
            .catch(function () {
                var status = document.getElementById("save-status");
                if (status) status.textContent = (window.gbT ? window.gbT("canvas-saved-local", "Saved (local)") : "Saved (local)");
            });
    }

    // Bridge into the native canvas app: after boot + load, sync the drawn
    // elements to project.draw on every save/undo/redo/clear.
    var origSave = window._canvasApp && window._canvasApp.saveCanvas;
    var origClear = window._canvasApp && window._canvasApp.clearCanvas;
    var origUndo = window._canvasApp && window._canvasApp.undo;
    var origRedo = window._canvasApp && window._canvasApp.redo;

    function syncSave() {
        if (origSave) origSave();
        saveProjectDraw();
    }
    function syncClear() {
        if (origClear) origClear();
        saveProjectDraw();
    }
    function syncUndo() {
        if (origUndo) origUndo();
        saveProjectDraw();
    }
    function syncRedo() {
        if (origRedo) origRedo();
        saveProjectDraw();
    }
    if (window._canvasApp) {
        window._canvasApp.saveCanvas = syncSave;
        window._canvasApp.clearCanvas = syncClear;
        window._canvasApp.undo = syncUndo;
        window._canvasApp.redo = syncRedo;
        Object.assign(window, {
            saveCanvas: syncSave,
            clearCanvas: syncClear,
            undo: syncUndo,
            redo: syncRedo,
        });
        window.saveProjectDraw = saveProjectDraw;
    }

    // Boot once the canvas is mounted.
    (function waitBoot() {
        if (document.getElementById("main-canvas")) {
            loadProjectDraw();
        } else {
            setTimeout(waitBoot, 150);
        }
    })();
})();
