/* Vibe Canvas — split from vibe/vibe-canvas.js for #1731 */
(function () {
    "use strict";
    var V = window.VibeDesign = window.VibeDesign || {};

    V.state = { elements: [], connectors: [], generated: null };

    var tool = "select";
    V.selectedId = null;
    var gesture = null;

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
        if (isDesign) V.load();
    }

    function activateTool(nextTool) {
        tool = nextTool || "select";
        var surface = visEl("vibeDesignSurface");
        if (surface) surface.setAttribute("data-tool", tool);
        setMode("design");
    }

    function setGeneratedDesign(design) {
        V.state.generated = design || null;
        var host = visEl("vibeGeneratedLayer");
        if (host) host.innerHTML = V.state.generated && V.state.generated.svg ? V.state.generated.svg : "";
    }

    function render() {
        var host = visEl("vibeDesignElements");
        var svg = visEl("vibeDesignConnectors");
        if (!host || !svg) return;
        var generatedHost = visEl("vibeGeneratedLayer");
        if (generatedHost) generatedHost.innerHTML = V.state.generated && V.state.generated.svg ? V.state.generated.svg : "";
        host.innerHTML = "";
        V.state.elements.forEach(function (item) {
            var el = document.createElement("div");
            el.className = "vibe-design-element " + item.type + (item.id === V.selectedId ? " selected" : "");
            el.dataset.designId = item.id;
            el.style.left = item.x + "px";
            el.style.top = item.y + "px";
            el.style.width = item.w + "px";
            el.style.height = item.h + "px";
            if (item.type === "text") el.textContent = item.text || "Text";
            host.appendChild(el);
        });
        svg.innerHTML = '<defs><marker id="vibeArrowhead" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="#2563eb"></path></marker></defs>' +
            V.state.connectors.map(function (line) {
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
            V.selectedId = hit ? hit.dataset.designId : null;
            var item = V.state.elements.find(function (entry) { return entry.id === V.selectedId; });
            if (item) gesture = { kind: "move", id: item.id, x: p.x, y: p.y, ox: item.x, oy: item.y };
            render();
            return;
        }
        if (tool === "text") {
            var addLabel = function (label) {
                if (label) {
                    V.state.elements.push({ id: "el-" + Date.now(), type: "text", x: p.x, y: p.y, w: 180, h: 32, text: label });
                    render(); V.scheduleSave();
                }
            };
            if (window.WindowManager && window.WindowManager.promptFloating) {
                window.WindowManager.promptFloating("Add text", "Text", "Heading", addLabel);
            } else {
                addLabel((window.GBModal||{prompt:prompt}).prompt("Text", "Heading"));
            }
            return;
        }
        if (tool === "rectangle") {
            var id = "el-" + Date.now();
            V.state.elements.push({ id: id, type: "rectangle", x: p.x, y: p.y, w: 1, h: 1 });
            V.selectedId = id;
            gesture = { kind: "rectangle", id: id, x: p.x, y: p.y };
        } else if (tool === "connector") {
            var connector = { id: "line-" + Date.now(), x1: p.x, y1: p.y, x2: p.x, y2: p.y };
            V.state.connectors.push(connector);
            gesture = { kind: "connector", id: connector.id };
        }
        event.currentTarget.setPointerCapture(event.pointerId);
        render();
    }

    function onPointerMove(event) {
        if (!gesture) return;
        var p = point(event);
        if (gesture.kind === "move") {
            var moving = V.state.elements.find(function (entry) { return entry.id === gesture.id; });
            if (moving) { moving.x = gesture.ox + p.x - gesture.x; moving.y = gesture.oy + p.y - gesture.y; }
        } else if (gesture.kind === "rectangle") {
            var rect = V.state.elements.find(function (entry) { return entry.id === gesture.id; });
            if (rect) {
                rect.x = Math.min(gesture.x, p.x); rect.y = Math.min(gesture.y, p.y);
                rect.w = Math.abs(p.x - gesture.x); rect.h = Math.abs(p.y - gesture.y);
            }
        } else if (gesture.kind === "connector") {
            var line = V.state.connectors.find(function (entry) { return entry.id === gesture.id; });
            if (line) { line.x2 = p.x; line.y2 = p.y; }
        }
        render();
    }

    function onPointerUp() {
        if (!gesture) return;
        if (gesture.kind === "rectangle") {
            var rect = V.state.elements.find(function (entry) { return entry.id === gesture.id; });
            if (rect && (rect.w < 12 || rect.h < 12)) { rect.w = 160; rect.h = 96; }
        }
        gesture = null;
        render();
        V.scheduleSave();
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
        V.state.elements = Array.isArray(content.elements) ? content.elements : [];
        V.state.connectors = Array.isArray(content.connectors) ? content.connectors : [];
        setGeneratedDesign(content.generated || null);
    }

    V.render = render;
    V.setStatus = setStatus;
    V.setGeneratedDesign = setGeneratedDesign;
    V.api = api;
    V.projectKey = projectKey;
    V.projectId = projectId;
    V.onPointerDown = onPointerDown;
    V.onPointerMove = onPointerMove;
    V.onPointerUp = onPointerUp;
    V.visEl = visEl;
    V.applyContent = applyContent;
    V.setMode = setMode;
    V.activate = activateTool;
})();
