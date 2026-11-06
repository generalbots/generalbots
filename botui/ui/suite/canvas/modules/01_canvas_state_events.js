(function () {
    "use strict";
    var C = (window._canvasApp = window._canvasApp || {});
    C.currentTool = "select";
    C.isDrawing = false;
    C.startX = 0;
    C.startY = 0;
    C.points = [];
    C.elements = [];
    C.layers = [{ id: "layer_1", name: "Layer 1", hidden: false }];
    C.activeLayerId = "layer_1";
    C.selectedElement = null;
    C.clipboard = null;
    C.history = [];
    C.historyIndex = -1;
    C.zoom = 1;
    C.panX = 0;
    C.panY = 0;
    C.strokeColor = "#000000";
    C.fillColor = "#ffffff";
    C.strokeWidth = 2;
    C.fontSize = 18;
    C.canvas = null;
    C.ctx = null;
    C.gridType = "dots";
    C.snapToGrid = false;
    C.canvasBg = "#ffffff";
    // The app body (including the <canvas> element) arrives through HTMX
    // partial swaps that can finish AFTER DOMContentLoaded, and the desktop
    // window manager can re-inject this script on every open. Boot when the
    // canvas is actually mounted and bail when the app window is gone.
    C.boot = function boot() {
        if (!document.getElementById("canvas-app-root")) return;
        C.canvas = document.getElementById("main-canvas");
        if (!C.canvas) {
            setTimeout(C.boot, 120);
            return;
        }
        C.ctx = C.canvas.getContext("2d");
        C.resizeCanvas();
        window.addEventListener("resize", C.resizeCanvas);
        C.canvas.addEventListener("mousedown", C.handleMouseDown);
        C.canvas.addEventListener("mousemove", C.handleMouseMove);
        C.canvas.addEventListener("mouseup", C.handleMouseUp);
        C.canvas.addEventListener("wheel", C.handleWheel);
        C.canvas.addEventListener("contextmenu", function (e) {
            e.preventDefault();
            C.showContextMenu(e);
        });
        document.addEventListener("keydown", C.handleKeyDown);
        C.renderLayers();
        C.loadCollaborators();
    }

    C.showContextMenu = function showContextMenu(e) {
        var menu = document.getElementById("context-menu");
        if (!menu) return;
        menu.classList.remove("hidden");
        menu.style.position = "fixed";
        menu.style.left = e.clientX + "px";
        menu.style.top = e.clientY + "px";
        var close = function () { menu.classList.add("hidden"); document.removeEventListener("click", close); };
        setTimeout(function () { document.addEventListener("click", close); }, 0);
    }

    C.setZoomLabel = function setZoomLabel() {
        const label = document.getElementById("zoom-level");
        if (label) label.textContent = Math.round(C.zoom * 100) + "%";
    }

    C.resizeCanvas = function resizeCanvas() {
        if (!C.canvas) return;
        const container = document.getElementById("canvas-container");
        if (!container) return;
        C.canvas.width = container.clientWidth || 800;
        C.canvas.height = container.clientHeight || 600;
        C.redraw();
    }

    C.selectTool = function selectTool(tool) {
        C.currentTool = tool;
        document.querySelectorAll(".tool-btn").forEach((btn) => {
            btn.classList.toggle("active", btn.dataset.tool === tool);
        });
        const container = document.getElementById("canvas-container");
        if (container) {
            if (tool === "select") container.style.cursor = "default";
            else if (tool === "pan") container.style.cursor = "grab";
            else if (tool === "text") container.style.cursor = "text";
            else container.style.cursor = "crosshair";
        }
    }

    C.canvasPoint = function canvasPoint(e) {
        const rect = C.canvas.getBoundingClientRect();
        let x = (e.clientX - rect.left - C.panX) / C.zoom;
        let y = (e.clientY - rect.top - C.panY) / C.zoom;
        if (C.snapToGrid) {
            x = Math.round(x / 20) * 20;
            y = Math.round(y / 20) * 20;
        }
        return { x, y };
    }

    C.handleMouseDown = function handleMouseDown(e) {
        if (e.button !== 0) return;
        const p = C.canvasPoint(e);
        C.startX = p.x;
        C.startY = p.y;
        C.points = [{ x: C.startX, y: C.startY }];
        C.isDrawing = true;
        if (C.currentTool === "select") {
            C.selectedElement = C.findElementAt(C.startX, C.startY);
            C.showElementProperties(C.selectedElement);
        } else if (C.currentTool === "pan") {
            document.getElementById("canvas-container").style.cursor =
                "grabbing";
        }
    }

    C.handleMouseMove = function handleMouseMove(e) {
        if (!C.isDrawing || !C.canvas) return;
        const p = C.canvasPoint(e);
        if (C.currentTool === "pan") {
            C.panX += e.movementX;
            C.panY += e.movementY;
            C.redraw();
        } else if (C.currentTool === "pencil" || C.currentTool === "brush" || C.currentTool === "eraser") {
            C.points.push(p);
            C.redraw();
        } else if (C.currentTool === "select" && C.selectedElement) {
            C.selectedElement.x = p.x - (C.startX - C.selectedElement.x);
            C.selectedElement.y = p.y - (C.startY - C.selectedElement.y);
            C.redraw();
        }
    }

    C.handleMouseUp = function handleMouseUp(e) {
        if (!C.isDrawing) return;
        C.isDrawing = false;
        const p = C.canvasPoint(e);
        const endX = p.x;
        const endY = p.y;

        if (C.currentTool === "pan") {
            document.getElementById("canvas-container").style.cursor = "grab";
            return;
        }
        if (C.currentTool === "select") return;

        const base = {
            id: Date.now(),
            stroke: C.strokeColor,
            fill: C.fillColor,
            strokeWidth: C.strokeWidth,
            rotation: 0,
            opacity: 1,
        };
        let element = null;

        if (C.currentTool === "pencil" || C.currentTool === "brush" || C.currentTool === "eraser") {
            C.points.push({ x: endX, y: endY });
            element = Object.assign({}, base, {
                type: C.currentTool,
                points: C.points.slice(),
                x: C.startX,
                y: C.startY,
                width: 0,
                height: 0,
            });
        } else if (C.currentTool === "text") {
            const text = window.prompt("Text content:", "");
            if (text === null || text === "") return;
            element = Object.assign({}, base, {
                type: "text",
                text: text,
                fontSize: C.fontSize,
                x: C.startX,
                y: C.startY,
                width: text.length * C.fontSize * 0.6,
                height: C.fontSize * 1.4,
            });
        } else if (C.currentTool === "image") {
            C.pickImage(function (img) {
                if (!img) return;
                const w = img.width || 160;
                const h = img.height || 120;
                C.elements.push(
                    Object.assign({}, base, {
                        type: "image",
                        img: img,
                        x: C.startX,
                        y: C.startY,
                        width: w,
                        height: h,
                        layer: C.activeLayerId,
                    })
                );
                C.saveHistory();
                C.redraw();
            });
            return;
        } else {
            element = Object.assign({}, base, {
                type: C.currentTool,
                x: Math.min(C.startX, endX),
                y: Math.min(C.startY, endY),
                width: Math.abs(endX - C.startX),
                height: Math.abs(endY - C.startY),
            });
        }

        if (element) {
            element.layer = C.activeLayerId;
            C.elements.push(element);
            C.saveHistory();
            C.redraw();
        }
    }

    C.pickImage = function pickImage(done) {
        const input = document.createElement("input");
        input.type = "file";
        input.accept = "image/*";
        input.onchange = function () {
            const file = input.files && input.files[0];
            if (!file) {
                done(null);
                return;
            }
            const reader = new FileReader();
            reader.onload = function () {
                const img = new Image();
                img.onload = function () {
                    done(img);
                };
                img.onerror = function () {
                    done(null);
                };
                img.src = reader.result;
            };
            reader.readAsDataURL(file);
        };
        input.click();
    }

    C.handleWheel = function handleWheel(e) {
        if (!C.canvas) return;
        e.preventDefault();
        C.zoom = Math.max(0.1, Math.min(5, C.zoom * (e.deltaY > 0 ? 0.9 : 1.1)));
        C.setZoomLabel();
        C.redraw();
    }

    C.handleKeyDown = function handleKeyDown(e) {
        if (e.key === "Delete" && C.selectedElement) {
            C.deleteSelected();
        }
        if (e.ctrlKey && e.key === "z") C.undo();
        if (e.ctrlKey && e.key === "y") C.redo();
        if (e.ctrlKey && e.key === "c" && C.selectedElement) C.copySelected();
        if (e.ctrlKey && e.key === "v") C.pasteClipboard();
        if (e.ctrlKey && e.key === "d" && C.selectedElement) C.duplicateSelected();
        if (e.ctrlKey && e.key.toLowerCase() === "s") {
            e.preventDefault();
            if (typeof window.saveCanvas === "function") window.saveCanvas();
        }
        if (e.ctrlKey && e.key.toLowerCase() === "n") {
            e.preventDefault();
            if (typeof window.newCanvas === "function") window.newCanvas();
        }
    }

})();
