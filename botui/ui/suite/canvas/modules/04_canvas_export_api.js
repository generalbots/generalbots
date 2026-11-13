(function () {
    "use strict";
    var C = window._canvasApp;
    C.exportCanvas = function exportCanvas() {
        const modal = document.getElementById("export-modal");
        if (modal) modal.classList.remove("hidden");
    }

    C.closeExportModal = function closeExportModal() {
        const modal = document.getElementById("export-modal");
        if (modal) modal.classList.add("hidden");
    }

    C.doExport = function doExport() {
        const format = (document.getElementById("export-format") || {}).value || "png";
        const scale = parseInt((document.getElementById("export-scale") || {}).value || "2");
        const bg = (document.getElementById("export-bg") || {}).checked;
        const name = (document.getElementById("canvas-name") || {}).value || "canvas";

        if (format === "png") {
            // #2144 — the PNG is rasterised from the composed SVG (background
            // layer + elements) instead of the bitmap, which never contained
            // the layer. It also drops the grid and honours "Include
            // Background" literally (unchecked = transparent), matching the
            // SVG export. The bitmap path stays as the fallback.
            const exporter = window.GBArtifactExport;
            if (exporter && typeof exporter.rasterise === "function") {
                exporter.rasterise(C.composeSvg(bg), { scale: scale })
                    .then(function (out) { return exporter.toBlob(out, "image/png"); })
                    .then(function (blob) { return exporter.saveBlob(blob, name + ".png"); })
                    .catch(function () { C.pngFromBitmap(name, scale, bg); });
            } else {
                C.pngFromBitmap(name, scale, bg);
            }
        } else if (format === "json") {
            const blob = new Blob([JSON.stringify({ elements: C.elements, zoom: C.zoom, panX: C.panX, panY: C.panY })], {
                type: "application/json",
            });
            const link = document.createElement("a");
            link.download = name + ".json";
            link.href = URL.createObjectURL(blob);
            link.click();
        } else if (format === "svg") {
            C.exportSvg(name, bg);
        } else if (format === "pdf") {
            C.exportPdf(name, bg);
        }
        C.closeExportModal();
    }

    // The bitmap export: what C.canvas holds (fill + grid + elements). It does
    // not contain the DOM background layer, so it is only the fallback path.
    C.pngFromBitmap = function pngFromBitmap(name, scale, bg) {
        const out = document.createElement("canvas");
        out.width = C.canvas.width * scale;
        out.height = C.canvas.height * scale;
        const outCtx = out.getContext("2d");
        if (bg) {
            outCtx.fillStyle = C.canvasBg;
            outCtx.fillRect(0, 0, out.width, out.height);
        }
        outCtx.save();
        outCtx.scale(scale, scale);
        outCtx.drawImage(C.canvas, 0, 0);
        outCtx.restore();
        const link = document.createElement("a");
        link.download = name + ".png";
        link.href = out.toDataURL("image/png");
        link.click();
    }

    C.visibleElements = function visibleElements() {
        return C.elements.filter(function (el) {
            var layer = C.layers.filter(function (l) { return l.id === el.layer; })[0];
            return !layer || !layer.hidden;
        });
    }

    // #2144 — the background layer (#canvas-bg-svg) is a DOM layer *under* the
    // bitmap, not part of C.canvas: an export built from C.elements alone
    // silently dropped a drawing imported from chat ("Open in Canvas"). The
    // layer is serialised into the composed document, stretched to the canvas
    // box exactly like the CSS layer (width/height 100%).
    C.backgroundLayerSvg = function backgroundLayerSvg() {
        const host = document.getElementById("canvas-bg-svg");
        if (!host) return "";
        const layer = host.querySelector("svg");
        const markup = layer ? layer.outerHTML : String(host.innerHTML || "").trim();
        if (!markup) return "";
        const exporter = window.GBArtifactExport;
        if (exporter && typeof exporter.serialize === "function") {
            return exporter.serialize(markup, {
                width: "100%",
                height: "100%",
                background: null,
            });
        }
        return markup;
    }

    C.composeSvg = function composeSvg(bg) {
        const w = C.canvas.width, h = C.canvas.height;
        let svg = '<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + h +
            '" viewBox="0 0 ' + w + " " + h + '">';
        if (bg) svg += '<rect width="100%" height="100%" fill="' + (C.canvasBg || "#ffffff") + '"/>';
        const layer = C.backgroundLayerSvg();
        if (layer) svg += layer;
        C.visibleElements().forEach(function (el) {
            const stroke = el.stroke || C.strokeColor;
            const fill = el.fill && el.fill !== "transparent" ? el.fill : "none";
            const sw = el.strokeWidth || C.strokeWidth;
            const esc = function (s) { return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;"); };
            if (el.type === "rectangle" || el.type === "frame") {
                svg += '<rect x="' + el.x + '" y="' + el.y + '" width="' + el.width + '" height="' + el.height +
                    '" stroke="' + stroke + '" fill="' + fill + '" stroke-width="' + sw + '"/>';
            } else if (el.type === "ellipse") {
                svg += '<ellipse cx="' + (el.x + el.width / 2) + '" cy="' + (el.y + el.height / 2) +
                    '" rx="' + (el.width / 2) + '" ry="' + (el.height / 2) +
                    '" stroke="' + stroke + '" fill="' + fill + '" stroke-width="' + sw + '"/>';
            } else if (el.type === "line" || el.type === "arrow") {
                // Tool-drawn lines carry a bounding box (x/y/width/height),
                // SVG-imported ones carry x1..y2 — support both (#1708).
                const lx1 = el.x1 !== undefined ? el.x1 : el.x;
                const ly1 = el.y1 !== undefined ? el.y1 : el.y;
                const lx2 = el.x2 !== undefined ? el.x2 : el.x + el.width;
                const ly2 = el.y2 !== undefined ? el.y2 : el.y + el.height;
                svg += '<line x1="' + lx1 + '" y1="' + ly1 + '" x2="' + lx2 + '" y2="' + ly2 +
                    '" stroke="' + stroke + '" stroke-width="' + sw + '"/>';
            } else if (el.type === "text") {
                svg += '<text x="' + el.x + '" y="' + el.y + '" font-size="' + (el.fontSize || 18) +
                    '" fill="' + stroke + '">' + esc(el.text) + "</text>";
            } else if (el.type === "image" && el.img && el.img.src) {
                svg += '<image x="' + el.x + '" y="' + el.y + '" width="' + el.width + '" height="' + el.height +
                    '" href="' + el.img.src + '"/>';
            } else if (el.points && el.points.length) {
                const pts = el.points.map(function (p) { return p.x + "," + p.y; }).join(" ");
                svg += '<polyline points="' + pts + '" stroke="' + stroke + '" fill="none" stroke-width="' + sw + '"/>';
            }
        });
        return svg + "</svg>";
    }

    C.exportSvg = function exportSvg(name, bg) {
        const svg = C.composeSvg(bg);
        const blob = new Blob([svg], { type: "image/svg+xml" });
        const link = document.createElement("a");
        link.download = name + ".svg";
        link.href = URL.createObjectURL(blob);
        link.click();
    }

    // #2144 — the PDF embeds the same composed picture as SVG/PNG. Before this
    // it embedded the bitmap, which never held the background layer, so a
    // drawing imported from chat produced a blank PDF page.
    C.exportPdf = function exportPdf(name, bg) {
        const exporter = window.GBArtifactExport;
        if (exporter && typeof exporter.rasterise === "function") {
            exporter.rasterise(C.composeSvg(bg), { scale: 1 })
                .then(function (out) { C.writePdf(name, out); })
                .catch(function () { C.writePdf(name, C.bitmapCanvas(bg)); });
            return;
        }
        C.writePdf(name, C.bitmapCanvas(bg));
    }

    C.bitmapCanvas = function bitmapCanvas(bg) {
        const w = C.canvas.width, h = C.canvas.height;
        const out = document.createElement("canvas");
        out.width = w;
        out.height = h;
        const outCtx = out.getContext("2d");
        if (bg) { outCtx.fillStyle = C.canvasBg; outCtx.fillRect(0, 0, w, h); }
        outCtx.drawImage(C.canvas, 0, 0);
        return out;
    }

    C.writePdf = function writePdf(name, source) {
        const w = source.width, h = source.height;
        const jpegUrl = source.toDataURL("image/jpeg", 0.92);
        const bin = atob(jpegUrl.split(",")[1]);
        const jpeg = new Uint8Array(bin.length);
        for (let i = 0; i < bin.length; i++) jpeg[i] = bin.charCodeAt(i);

        const parts = [];
        let offset = 0;
        const objOffsets = [];
        const push = function (b) {
            parts.push(b);
            offset += (typeof b === "string" ? b.length : b.length);
        };
        const pad10 = function (n) { return ("0000000000" + n).slice(-10); };
        push("%PDF-1.3\n");
        const obj = function (body) { objOffsets.push(offset); push((objOffsets.length) + " 0 obj\n" + body + "\nendobj\n"); };
        obj("<< /Type /Catalog /Pages 2 0 R >>");
        obj("<< /Type /Pages /Kids [3 0 R] /Count 1 >>");
        obj("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 " + w + " " + h +
            "] /Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>");
        objOffsets.push(offset);
        push("4 0 obj\n<< /Type /XObject /Subtype /Image /Width " + w + " /Height " + h +
            " /ColorSpace /DeviceRGB /Filter /DCTDecode /BitsPerComponent 8 /Length " + jpeg.length +
            " >>\nstream\n");
        push(jpeg);
        push("\nendstream\nendobj\n");
        const content = "q\n" + w + " 0 0 " + h + " 0 0 cm /Im0 Do\nQ\n";
        obj("<< /Length " + content.length + " >>\nstream\n" + content + "\nendstream");
        const xrefStart = offset;
        let xref = "xref\n0 " + (objOffsets.length + 1) + "\n0000000000 65535 f \n";
        for (let k = 0; k < objOffsets.length; k++) xref += pad10(objOffsets[k]) + " 00000 n \n";
        push(xref);
        push("trailer\n<< /Size " + (objOffsets.length + 1) + " /Root 1 0 R >>\nstartxref\n" + xrefStart + "\n%%EOF");
        const blob = new Blob(parts, { type: "application/pdf" });
        const link = document.createElement("a");
        link.download = name + ".pdf";
        link.href = URL.createObjectURL(blob);
        link.click();
    }

    C.saveCanvas = function saveCanvas() {
        const status = document.getElementById("save-status");
        if (status) status.textContent = (window.gbT ? window.gbT("canvas-saved", "Saved ✓") : "Saved ✓");
        C.doExport();
    }

    // The drawing a teammate has to open. Sharing used to copy
    // `?tool=canvas`, which the canvas app ignores, so the second browser got
    // an empty board — "Share" reported a link but never the same drawing
    // (UAT CANVAS-05). The identity the app already understands is the Drive
    // path (or the project it was opened from); without one there is nothing
    // to share yet.
    function canvasShareLink() {
        var gp = window.__gbAppParams__ || {};
        var qs = new URLSearchParams(window.location.search);
        var path = window.__CANVAS_FILE_PATH || gp.file || qs.get("path") || "";
        var bucket = window.__CANVAS_FILE_BUCKET || gp.bucket || qs.get("bucket") || "";
        var project = gp.project || qs.get("project") || "";
        if (!path && !project) return "";
        var params = new URLSearchParams({ app: "canvas" });
        if (path) params.set("path", path);
        if (bucket) params.set("bucket", bucket);
        if (project) params.set("project", project);
        return window.location.origin + window.location.pathname + "?" + params.toString();
    }

    // People who already have access to a canvas in this workspace. The route
    // answers a bare JSON array; reading `data.collaborators` always reported
    // zero, which made every share look like a fresh, empty canvas.
    function shareAudience(callback) {
        var headers = { Accept: "application/json" };
        var token = (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
            window.localStorage.getItem("gb-access-token") || "";
        if (token) headers.Authorization = "Bearer " + token;
        fetch("/api/canvas/collaborators", { headers: headers })
            .then(function (r) { return r.ok ? r.json() : []; })
            .then(function (data) {
                var people = Array.isArray(data) ? data
                    : (data && (data.collaborators || data.users)) || [];
                callback(people.length);
            })
            .catch(function () { callback(0); });
    }

    function reportShare(url, people) {
        var audience = people > 0
            ? people + " collaborator(s) have access to this canvas"
            : "No collaborators yet — anyone with the link can open the drawing";
        (window.GBModal || { alert: alert }).alert(
            (window.gbT ? window.gbT("canvas-share-copied", "Share link copied to clipboard") : "Share link copied to clipboard") +
            "\n" + url + "\n" + audience
        );
    }

    C.shareCanvas = function shareCanvas() {
        var url = canvasShareLink();
        if (!url) {
            (window.GBModal || { alert: alert }).alert(
                "Save the canvas first (Save → .draw): an unsaved drawing has no file to share."
            );
            return;
        }
        var copy = function () {
            shareAudience(function (people) { reportShare(url, people); });
        };
        try {
            navigator.clipboard.writeText(url).then(copy).catch(copy);
        } catch (ignore) {
            copy();
        }
    }

    C.toggleCollaborate = function toggleCollaborate() {
        // Report the real state of this canvas instead of a static hint: who
        // can reach it, and whether the drawing is stored somewhere another
        // browser can load it from.
        var url = canvasShareLink();
        shareAudience(function (people) {
            var lines = [];
            lines.push(url ? "Share link ready for this canvas." : "This canvas is not saved yet — save it to share.");
            lines.push(people > 0
                ? people + " collaborator(s) have access."
                : "No collaborators have access yet.");
            var connected = !!(window.GBCollab && window.GBCollab.connected);
            lines.push(connected
                ? "Real-time collaboration session active."
                : "Collaboration session not connected yet.");
            (window.GBModal || { alert: alert }).alert(lines.join("\n"));
        });
    }

    // Expose the API to the toolbar/properties/export partials, which the
    // desktop window manager injects as plain HTML with inline on* handlers.
    window._canvasApp = {
        selectTool: C.selectTool,
        setStrokeColor: C.setStrokeColor,
        setFillColor: C.setFillColor,
        setStrokeWidth: C.setStrokeWidth,
        setGridType: C.setGridType,
        setCanvasBackground: C.setCanvasBackground,
        setCanvasBackgroundHidden: function () {
            C.canvasBg = "transparent";
            C.redraw();
        },
        toggleSnapToGrid: C.toggleSnapToGrid,
        updateElementProperty: C.updateElementProperty,
        clearCanvas: C.clearCanvas,
        undo: C.undo,
        redo: C.redo,
        zoomIn: C.zoomIn,
        zoomOut: C.zoomOut,
        resetZoom: C.resetZoom,
        fitToScreen: C.fitToScreen,
        exportCanvas: C.exportCanvas,
        closeExportModal: C.closeExportModal,
        doExport: C.doExport,
        saveCanvas: C.saveCanvas,
        shareCanvas: C.shareCanvas,
        toggleCollaborate: C.toggleCollaborate,
        togglePropertiesPanel: C.togglePropertiesPanel,
        copySelected: C.copySelected,
        pasteClipboard: C.pasteClipboard,
        duplicateSelected: C.duplicateSelected,
        deleteSelected: C.deleteSelected,
        addLayer: C.addLayer,
        bringToFront: C.bringToFront,
        sendToBack: C.sendToBack,
        renameCanvas: C.renameCanvas,
        getCanvasElements: function () { return C.elements; },
        setCanvasElements: function (arr) {
            C.elements = Array.isArray(arr) ? arr : [];
            C.saveHistory();
            C.redraw();
        },
        appendCanvasElements: function (arr) {
            if (Array.isArray(arr)) arr.forEach(function (el) { C.elements.push(el); });
            C.saveHistory();
            C.redraw();
        },
        canvasSaveMethods: { saveCanvas: C.saveCanvas, clearCanvas: C.clearCanvas, undo: C.undo, redo: C.redo },
    };
    Object.assign(window, window._canvasApp);
    C.initCollab = function initCollab() {
        if (!window.GBCollab) return;
        var gp = window.__gbAppParams__ || {};
        var qs = new URLSearchParams(window.location.search);
        var path = window.__CANVAS_FILE_PATH || gp.file || qs.get("path") || "";
        var bucket = window.__CANVAS_FILE_BUCKET || gp.bucket || qs.get("bucket") || "";
        var docId = path || (bucket ? bucket + "/canvas" : "");
        if (!docId) return;
        var applyingRemote = false;
        window.GBCollab.connect({
            app: "canvas",
            docId: docId,
            collaboratorsEl: document.getElementById("collaborators") || null,
            onConnect: function () {},
            onDisconnect: function () {},
            onEdit: function (msg) {
                if (!msg || !msg.content) return;
                try {
                    var els = JSON.parse(msg.content);
                    if (!Array.isArray(els)) return;
                    applyingRemote = true;
                    C.elements = els;
                    if (typeof C.redraw === "function") C.redraw();
                    applyingRemote = false;
                } catch (e) { applyingRemote = false; }
            }
        });
        if (!C._collabHooked) {
            C._collabHooked = true;
            var origSaveHistory = C.saveHistory;
            C.saveHistory = function () {
                if (origSaveHistory) origSaveHistory();
                if (applyingRemote) return;
                if (window.GBCollab && window.GBCollab.connected) {
                    try { window.GBCollab.sendEdit({ content: JSON.stringify(C.elements || []) }); } catch (e) {}
                }
            };
        }
    };
    C.boot();
    setTimeout(function () { if (typeof C.initCollab === "function") C.initCollab(); }, 800);
})();
