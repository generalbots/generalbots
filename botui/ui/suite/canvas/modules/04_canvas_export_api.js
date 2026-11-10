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

    C.visibleElements = function visibleElements() {
        return C.elements.filter(function (el) {
            var layer = C.layers.filter(function (l) { return l.id === el.layer; })[0];
            return !layer || !layer.hidden;
        });
    }

    C.exportSvg = function exportSvg(name, bg) {
        const w = C.canvas.width, h = C.canvas.height;
        let svg = '<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + h +
            '" viewBox="0 0 ' + w + " " + h + '">';
        if (bg) svg += '<rect width="100%" height="100%" fill="' + (C.canvasBg || "#ffffff") + '"/>';
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
        svg += "</svg>";
        const blob = new Blob([svg], { type: "image/svg+xml" });
        const link = document.createElement("a");
        link.download = name + ".svg";
        link.href = URL.createObjectURL(blob);
        link.click();
    }

    C.exportPdf = function exportPdf(name, bg) {
        const w = C.canvas.width, h = C.canvas.height;
        const out = document.createElement("canvas");
        out.width = w;
        out.height = h;
        const outCtx = out.getContext("2d");
        if (bg) { outCtx.fillStyle = C.canvasBg; outCtx.fillRect(0, 0, w, h); }
        outCtx.drawImage(C.canvas, 0, 0);
        const jpegUrl = out.toDataURL("image/jpeg", 0.92);
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

    C.shareCanvas = function shareCanvas() {
        const url = window.location.href.split("?")[0] + "?tool=canvas";
        try {
            navigator.clipboard
                .writeText(url)
                .then(function () {
                    (window.GBModal||{alert:alert}).alert((window.gbT ? window.gbT("canvas-share-copied", "Share link copied to clipboard") : "Share link copied to clipboard"));
                })
                .catch(function () {
                    (window.GBModal||{alert:alert}).alert(window.gbT
            ? window.gbT("canvas-share-link", "Share link: {url}", { url: url })
            : "Share link: " + url);
                });
        } catch (ignore) {
            (window.GBModal||{alert:alert}).alert(window.gbT
            ? window.gbT("canvas-share-link", "Share link: {url}", { url: url })
            : "Share link: " + url);
        }
    }

    C.toggleCollaborate = function toggleCollaborate() {
        (window.GBModal||{alert:alert}).alert(
            "Real-time collaboration needs a canvas session; open the Vibe or Docs app to collaborate live."
        );
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
    C.boot();
})();
