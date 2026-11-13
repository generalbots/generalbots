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

    // #1708 — open a generated .svg as editable canvas elements. Only the
    // shape vocabulary the canvas models maps over (rect/circle/ellipse/
    // line/text/polyline/polygon, no gradients/transforms/use/image); when
    // anything outside that vocabulary appears the original SVG stays as the
    // non-interactive background so nothing is ever lost or double-drawn.
    function importSvgAsElements(svgText) {
        var app = window._canvasApp;
        var bg = document.getElementById("canvas-bg-svg");
        var doc;
        try { doc = new DOMParser().parseFromString(svgText, "image/svg+xml"); } catch (e) { doc = null; }
        var root = doc && doc.documentElement;
        if (!root || root.nodeName.toLowerCase() !== "svg" || doc.querySelector("parsererror")) {
            setStatus("Could not read SVG");
            return;
        }
        var keepBackground = function (reason) {
            if (bg) bg.innerHTML = svgText;
            if (app && typeof app.setCanvasBackgroundHidden === "function") app.setCanvasBackgroundHidden();
            setStatus("SVG kept as background — " + reason);
        };
        // Gradients/filters/patterns referenced by url(#) cannot be modelled
        // by the canvas fill/stroke values: keep the original for fidelity.
        if (/url\s*\(/.test(svgText)) {
            keepBackground("uses gradients/filters");
            return;
        }
        var MAPPABLE = { rect: 1, circle: 1, ellipse: 1, line: 1, text: 1, polyline: 1, polygon: 1 };
        var STRUCTURAL = { svg: 1, g: 1, title: 1, desc: 1, defs: 1, style: 1, metadata: 1, switch: 1 };
        var unsupported = 0;
        root.querySelectorAll("*").forEach(function (n) {
            var name = n.nodeName.toLowerCase();
            if (STRUCTURAL[name]) return;
            if (!MAPPABLE[name]) unsupported++;
            if (name === "polygon" && n.getAttribute("fill") && n.getAttribute("fill") !== "none") unsupported++;
        });
        if (unsupported > 0) {
            keepBackground(unsupported + " unsupported element(s)");
            return;
        }
        var num = function (v, d) { var n = parseFloat(v); return isNaN(n) ? (d || 0) : n; };
        var strokeOf = function (n) { var v = n.getAttribute("stroke"); return v && v !== "none" ? v : "#000000"; };
        var fillOf = function (n) { var v = n.getAttribute("fill"); return v && v !== "none" ? v : "transparent"; };
        var swOf = function (n) { return num(n.getAttribute("stroke-width"), 2); };
        var seed = 0;
        var out = [];
        var add = function (type, base) { base.id = "svg" + (++seed) + "_" + Date.now(); base.type = type; out.push(base); };
        root.querySelectorAll("rect").forEach(function (n) {
            add("rectangle", { x: num(n.getAttribute("x")), y: num(n.getAttribute("y")),
                width: num(n.getAttribute("width")), height: num(n.getAttribute("height")),
                stroke: strokeOf(n), fill: fillOf(n), strokeWidth: swOf(n) });
        });
        root.querySelectorAll("circle").forEach(function (n) {
            var r = num(n.getAttribute("r"));
            add("ellipse", { x: num(n.getAttribute("cx")) - r, y: num(n.getAttribute("cy")) - r,
                width: 2 * r, height: 2 * r, stroke: strokeOf(n), fill: fillOf(n), strokeWidth: swOf(n) });
        });
        root.querySelectorAll("ellipse").forEach(function (n) {
            var rx = num(n.getAttribute("rx")), ry = num(n.getAttribute("ry"));
            add("ellipse", { x: num(n.getAttribute("cx")) - rx, y: num(n.getAttribute("cy")) - ry,
                width: 2 * rx, height: 2 * ry, stroke: strokeOf(n), fill: fillOf(n), strokeWidth: swOf(n) });
        });
        root.querySelectorAll("line").forEach(function (n) {
            var x1 = num(n.getAttribute("x1")), y1 = num(n.getAttribute("y1")),
                x2 = num(n.getAttribute("x2")), y2 = num(n.getAttribute("y2"));
            // drawElement and its rotation prelude read x/y/width/height;
            // x1..y2 stay for exportSvg (#1708).
            add("line", { x1: x1, y1: y1, x2: x2, y2: y2,
                x: Math.min(x1, x2), y: Math.min(y1, y2),
                width: Math.abs(x2 - x1), height: Math.abs(y2 - y1),
                stroke: strokeOf(n), strokeWidth: swOf(n) });
        });
        root.querySelectorAll("text").forEach(function (n) {
            var fs = num(n.getAttribute("font-size"), 18);
            var fill = n.getAttribute("fill");
            var content = (n.textContent || "").trim();
            // drawElement's rotation prelude reads x/width: text needs a bbox (#1708).
            add("text", { x: num(n.getAttribute("x")), y: num(n.getAttribute("y")) - fs,
                fontSize: fs, text: content,
                width: content.length * fs * 0.6, height: fs * 1.4,
                stroke: fill && fill !== "none" ? fill : "#000000" });
        });
        var parsePoints = function (raw) {
            return (raw || "").trim().split(/[\s,]+/).reduce(function (acc, tok, i, arr) {
                if (i % 2 === 0) acc.push({ x: num(tok), y: num(arr[i + 1]) });
                return acc;
            }, []);
        };
        var bboxOfPoints = function (ps) {
            var xs = ps.map(function (p) { return p.x; });
            var ys = ps.map(function (p) { return p.y; });
            var minX = Math.min.apply(Math, xs), maxX = Math.max.apply(Math, xs);
            var minY = Math.min.apply(Math, ys), maxY = Math.max.apply(Math, ys);
            return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
        };
        root.querySelectorAll("polyline").forEach(function (n) {
            var ps = parsePoints(n.getAttribute("points"));
            // drawElement has no "polyline" case — pencil draws from points,
            // but its prelude still needs the bounding box (#1708).
            add("pencil", Object.assign({ points: ps, stroke: strokeOf(n), strokeWidth: swOf(n) }, bboxOfPoints(ps)));
        });
        root.querySelectorAll("polygon").forEach(function (n) {
            var ps = parsePoints(n.getAttribute("points"));
            add("pencil", Object.assign({ points: ps, stroke: strokeOf(n), strokeWidth: swOf(n) }, bboxOfPoints(ps)));
        });
        if (!out.length) {
            keepBackground("no mappable shapes");
            return;
        }
        if (app && typeof app.setCanvasElements === "function") app.setCanvasElements(out);
        if (bg) bg.innerHTML = "";
        setStatus("Imported " + out.length + " SVG shape(s) — editable");
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
