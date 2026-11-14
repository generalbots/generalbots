(function () {
    "use strict";

    // #2282 — per-window file context (page globals clobbered by next open).
    var __canvasRoot = (function () { var s = document.currentScript; return s && s.closest ? s.closest('[id^="window-body-canvas"]') : null; })();
    var __canvasWinId = __canvasRoot ? (__canvasRoot.dataset.windowId || __canvasRoot.id.replace("window-body-", "")) : "";
    var __canvasFileCtx = (function () {
        var ds = (__canvasRoot && __canvasRoot.dataset) || {};
        var m = (window.__CANVAS_FILE_CTX && __canvasWinId && window.__CANVAS_FILE_CTX[__canvasWinId]) || (window.__GB_WINDOW_FILE_CTX && __canvasWinId && window.__GB_WINDOW_FILE_CTX[__canvasWinId]) || {};
        var gp = window.__gbAppParams__ || {}, qs = new URLSearchParams(window.location.search);
        return { bucket: ds.gbFileBucket || m.bucket || window.__CANVAS_FILE_BUCKET || gp.bucket || qs.get("bucket") || "",
            path: ds.gbFilePath || m.path || window.__CANVAS_FILE_PATH || gp.file || qs.get("path") || "",
            scope: ds.gbFileScope || m.scope || window.__CANVAS_FILE_SCOPE || gp.scope || qs.get("scope") || "bot" };
    })();
    function __setCanvasPath(p) {
        __canvasFileCtx.path = p || "";
        if (__canvasRoot && __canvasRoot.dataset) __canvasRoot.dataset.gbFilePath = p || "";
        if (__canvasWinId) { window.__CANVAS_FILE_CTX = window.__CANVAS_FILE_CTX || {}; window.__CANVAS_FILE_CTX[__canvasWinId] = window.__CANVAS_FILE_CTX[__canvasWinId] || {}; window.__CANVAS_FILE_CTX[__canvasWinId].path = p || ""; }
    }
    function currentCanvasFile() {
        var p = __canvasFileCtx.path || "", b = __canvasFileCtx.bucket || "", s = __canvasFileCtx.scope || "bot";
        var folder = p ? (p.indexOf("/") === -1 ? "" : p.substring(0, p.lastIndexOf("/"))) : "";
        return { path: p, bucket: b, scope: s, folder: folder, name: p ? p.split("/").pop() : "" };
    }
    if (window._canvasApp) window._canvasApp._fileCtx = __canvasFileCtx;

    function driveFileOpen() {
        var f = currentCanvasFile();
        return !!f.bucket && !!f.path;
    }

    function api(url, opts) {
        opts = opts || {};
        var headers = Object.assign({ "Content-Type": "application/json" }, opts.headers || {});
        var token = "";
        if (window.getGBAccessToken) token = window.getGBAccessToken() || "";
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

    function setStatus(text) {
        var status = document.getElementById("save-status");
        if (status) status.textContent = text;
    }

    // Normalize any element shape (full canvas element, or legacy
    // vibe-design {type,x,y,w,h,text}) into a drawable canvas element.
    function toCanvasElement(el) {
        if (!el) return null;
        var copy = Object.assign({}, el);
        delete copy.img; // HTMLImageElement is not JSON-serializable
        // Legacy vibe-design elements store w/h; the canvas draw shape is
        // width/height. Normalize first so saved designs take the
        // full-element branch below and keep stroke/fill/strokeWidth (#1713).
        if (copy.width === undefined && copy.w !== undefined) {
            copy.width = copy.w;
            copy.height = copy.h;
        }
        if (copy.imgSrc && !copy.img) {
            copy.img = new Image();
            copy.img.src = copy.imgSrc;
        }
        if (copy.width !== undefined || copy.type === "text" || copy.points) {
            if (copy.type === "text") copy.fontSize = copy.fontSize || 18;
            if (!copy.stroke) copy.stroke = "#000000";
            if (!copy.strokeWidth) copy.strokeWidth = 2;
            if (copy.id === undefined) copy.id = Date.now() + Math.floor(Math.random() * 10000);
            return copy;
        }
        // Legacy design element
        var w = el.w || el.width || 160;
        var h = el.h || el.height || 90;
        var elem = {
            id: el.id || Date.now() + Math.floor(Math.random() * 10000),
            type: el.type || "rectangle",
            x: el.x || 0,
            y: el.y || 0,
            width: w,
            height: h,
            stroke: "#0f172a",
            fill: el.text ? "#e0f2fe" : "transparent",
            strokeWidth: 2,
        };
        if (el.text) { elem.text = el.text; elem.type = "text"; }
        return elem;
    }

    function serializeElements(els) {
        var keys = ["id", "type", "x", "y", "x1", "y1", "x2", "y2", "width", "height", "stroke", "fill", "strokeWidth", "rotation", "opacity", "fontSize", "text", "points"];
        return (els || []).map(function (el) {
            var copy = {};
            keys.forEach(function (k) {
                if (el[k] !== undefined) copy[k] = el[k];
            });
            // HTMLImageElement is not JSON-serializable; keep its source so
            // an image element survives save/reload (#1708 no-loss).
            if (el.img && el.img.src) copy.imgSrc = el.img.src;
            return copy;
        });
    }

    // #1708 — the generated architecture SVG loaded as background must
    // survive a save: both save paths used to write `generated: null`,
    // destroying it on the first Save/undo/redo.
    function currentGenerated() {
        var bg = document.getElementById("canvas-bg-svg");
        var html = bg && bg.innerHTML ? bg.innerHTML.trim() : "";
        return html.indexOf("<svg") === 0 ? { svg: html } : null;
    }

    function canvasTitle() {
        var el = document.getElementById("canvas-name");
        var t = ((el && el.value) || "Untitled Canvas").trim();
        return t.replace(/\.draw$/i, "");
    }

    // #2155 — delegate to svg import helper so flowcharts (paths/markers) can
    // become editable; falls back to background when fidelity would be lost.
    function importSvgAsElements(svgText) {
        if (window.GBCanvasSvgImport && typeof window.GBCanvasSvgImport.importAsElements === "function") {
            window.GBCanvasSvgImport.importAsElements(svgText, {
                app: window._canvasApp,
                bg: document.getElementById("canvas-bg-svg"),
                setStatus: setStatus,
            });
            return;
        }
        // Fallback: keep drawing visible even if helper script is missing.
        var bg = document.getElementById("canvas-bg-svg");
        if (bg) bg.innerHTML = svgText;
        if (window._canvasApp && typeof window._canvasApp.setCanvasBackgroundHidden === "function") {
            window._canvasApp.setCanvasBackgroundHidden();
        }
        setStatus("SVG kept as background");
    }

    function loadDriveDraw() {
        var f = currentCanvasFile();
        // A deep link can carry only the path: the API resolves the bucket from
        // the scope (default bucket) when it is omitted (#1708).
        if (!f.path) return;
        var body = { path: f.path, scope: f.scope };
        if (f.bucket) body.bucket = f.bucket;
        api("/api/files/download", {
            method: "POST",
            body: JSON.stringify(body),
        })
            .then(function (d) {
                if (!d || !d.content) return;
                var text;
                try { text = atob(d.content); } catch (e) { return; }
                if (/\.svg$/i.test(f.path || "")) {
                    // Keep the source SVG untouched: subsequent saves go to a
                    // sibling .draw file (which also round-trips the original
                    // SVG through `generated` when it stays a background).
                    __setCanvasPath(f.path.replace(/\.svg$/i, ".draw"));
                    importSvgAsElements(text);
                    return;
                }
                var parsed;
                try { parsed = JSON.parse(text); } catch (e) { return; }
                var items = Array.isArray(parsed.elements) ? parsed.elements : [];
                var app = window._canvasApp;
                if (app && typeof app.setCanvasElements === "function") {
                    app.setCanvasElements(items.map(toCanvasElement).filter(Boolean));
                }
                // LLM-generated architecture background (vibe-style .draw)
                if (parsed.generated && parsed.generated.svg) {
                    var bg = document.getElementById("canvas-bg-svg");
                    if (bg) bg.innerHTML = parsed.generated.svg;
                    if (app && typeof app.setCanvasBackgroundHidden === "function") {
                        app.setCanvasBackgroundHidden();
                    }
                }
                var nameEl = document.getElementById("canvas-name");
                if (nameEl) nameEl.value = parsed.title || f.name.replace(/\.draw$/i, "");
                setStatus("Loaded · " + f.name);
            })
            .catch(function () { /* file missing — blank canvas */ });
    }

    // Write the current elements back to a .draw file on the drive (or
    // download a local .draw when there is no drive context).
    function saveDriveDraw() {
        var f = currentCanvasFile();
        var app = window._canvasApp;
        var els = app && typeof app.getCanvasElements === "function" ? app.getCanvasElements() : [];
        var content = JSON.stringify({
            kind: "canvas-draw",
            version: 2,
            title: canvasTitle(),
            elements: serializeElements(els),
            connectors: [],
            generated: currentGenerated(),
        }, null, 2);

        var name = canvasTitle();

        // #1975 — the saved file must carry the name the user typed, otherwise
        // renaming a canvas left the old filename on the drive and searching
        // for the new name found nothing. Same-folder rename, sanitized once.
        function desiredName(folder) {
            var safe = String(name || "Untitled Canvas")
                .replace(/[\/\\?%*:|"<>]/g, "-")
                .replace(/\.draw$/i, "");
            if (!safe) safe = "Untitled Canvas";
            var file = safe + ".draw";
            return folder ? folder + "/" + file : file;
        }

        function writeDraw(path, scope, bucket) {
            var writeBody = { path: path, content: btoa(content), scope: scope };
            if (bucket) writeBody.bucket = bucket;
            return api("/api/files/write", {
                method: "POST",
                body: JSON.stringify(writeBody),
            }).then(function () {
                __setCanvasPath(path);
                setStatus("Saved · " + path.split("/").pop() + " ✓");
            });
        }

        if (!f.bucket && !f.path) {
            // Standalone canvas: keep a copy in the caller's Drive so the name
            // is searchable; fall back to a local download when Drive answers
            // nothing (offline or desktop-local without an account).
            writeDraw(desiredName(""), "user", null).catch(function () {
                var blob = new Blob([content], { type: "application/json" });
                var link = document.createElement("a");
                link.download = name + ".draw";
                link.href = URL.createObjectURL(blob);
                link.click();
                setTimeout(function () { URL.revokeObjectURL(link.href); }, 4000);
                setStatus("Saved (download)");
            });
            return;
        }

        var folder = f.folder ||
            (f.path && f.path.indexOf("/") !== -1 ? f.path.substring(0, f.path.lastIndexOf("/")) : "");
        var target = desiredName(folder);
        var previous = f.path || "";

        var rename = previous && previous !== target
            ? api("/api/files/move", {
                method: "POST",
                body: JSON.stringify({
                    source_path: previous,
                    dest_path: target,
                    scope: f.scope,
                    source_bucket: f.bucket || null,
                    dest_bucket: f.bucket || null,
                }),
            }).catch(function () { /* no old file to rename: write anyway */ })
            : Promise.resolve();

        rename
            .then(function () { return writeDraw(target, f.scope, f.bucket); })
            .catch(function () {
                setStatus("Save failed");
            });
    }

    // Reset the board to a blank canvas; the next Save creates a new .draw
    // file (same folder when launched from Drive, local download otherwise).
    function newCanvas() {
        var app = window._canvasApp;
        if (app && typeof app.setCanvasElements === "function") app.setCanvasElements([]);
        var nameEl = document.getElementById("canvas-name");
        if (nameEl) nameEl.value = "Untitled Canvas";
        __setCanvasPath("");
        setStatus("New canvas — press Save");
    }

    // Drive-aware save dispatch: Drive file → write .draw to Drive; Vibe
    // project → project.draw; otherwise → download a local .draw.
    var prevClear = window._canvasApp && window._canvasApp.clearCanvas;
    var prevUndo = window._canvasApp && window._canvasApp.undo;
    var prevRedo = window._canvasApp && window._canvasApp.redo;

    function driveSync(fn) {
        return function () {
            var r = fn ? fn.apply(this, arguments) : undefined;
            if (driveFileOpen()) saveDriveDraw();
            return r;
        };
    }

    function hasVibeProject() {
        var p = (window.__gbAppParams__ || {}).project;
        if (!p) {
            try { p = new URLSearchParams(window.location.search).get("project"); } catch (e) { p = null; }
        }
        return !!p;
    }

    function saveCanvasDispatch() {
        if (driveFileOpen()) { saveDriveDraw(); return; }
        if (hasVibeProject() && window.saveProjectDraw) { window.saveProjectDraw(); return; }
        saveDriveDraw(); // standalone — downloads .draw
    }

    if (window._canvasApp) {
        window._canvasApp.saveCanvas = saveCanvasDispatch;
        window._canvasApp.clearCanvas = driveSync(prevClear);
        window._canvasApp.undo = driveSync(prevUndo);
        window._canvasApp.redo = driveSync(prevRedo);
        window._canvasApp.newCanvas = newCanvas;
    }
    window.saveCanvas = saveCanvasDispatch;
    window.newCanvas = newCanvas;

    // Boot once the canvas is mounted.
    (function waitBoot() {
        if (document.getElementById("main-canvas")) {
            // #1975 — a drawing handed off from chat renders immediately,
            // without a Drive round-trip (the Drive write may be unavailable
            // on builds whose suite session token is not a JWT).
            var pending = null;
            var pendingName = "";
            try {
                pending = localStorage.getItem("gb-canvas-pending-svg");
                pendingName = localStorage.getItem("gb-canvas-pending-name") || "";
            } catch (e) { /* storage unavailable */ }
            if (pending) {
                try {
                    localStorage.removeItem("gb-canvas-pending-svg");
                    localStorage.removeItem("gb-canvas-pending-name");
                } catch (e) { /* ignore */ }
                var nameEl = document.getElementById("canvas-name");
                if (nameEl && pendingName) nameEl.value = pendingName;
                importSvgAsElements(pending);
            } else {
                loadDriveDraw();
            }
        } else {
            setTimeout(waitBoot, 150);
        }
    })();
})();
