(function () {
    "use strict";
    var C = window._canvasApp;
    C.drawGrid = function drawGrid() {
        if (!C.ctx) return;
        C.ctx.save();
        C.ctx.strokeStyle = "var(--border-color, #d1d5db)";
        C.ctx.lineWidth = 0.5;
        const gridSize = 20;
        if (C.gridType === "dots") {
            C.ctx.fillStyle = "var(--border-color, #d1d5db)";
            for (let x = 0; x < C.canvas.width; x += gridSize) {
                for (let y = 0; y < C.canvas.height; y += gridSize) {
                    C.ctx.fillRect(x, y, 1, 1);
                }
            }
        } else if (C.gridType === "lines" || C.gridType === "squares") {
            for (let x = 0; x < C.canvas.width; x += gridSize) {
                C.ctx.beginPath();
                C.ctx.moveTo(x, 0);
                C.ctx.lineTo(x, C.canvas.height);
                C.ctx.stroke();
            }
            for (let y = 0; y < C.canvas.height; y += gridSize) {
                C.ctx.beginPath();
                C.ctx.moveTo(0, y);
                C.ctx.lineTo(C.canvas.width, y);
                C.ctx.stroke();
            }
        }
        C.ctx.restore();
    }

    C.redraw = function redraw() {
        if (!C.ctx) return;
        C.ctx.clearRect(0, 0, C.canvas.width, C.canvas.height);
        C.ctx.fillStyle = C.canvasBg;
        C.ctx.fillRect(0, 0, C.canvas.width, C.canvas.height);
        C.drawGrid();
        C.ctx.save();
        C.ctx.translate(C.panX, C.panY);
        C.ctx.scale(C.zoom, C.zoom);
        C.elements.forEach(function (el) {
            var layer = C.layers.filter(function (l) { return l.id === el.layer; })[0];
            if (layer && layer.hidden) return;
            C.drawElement(el);
        });
        if (C.currentTool === "pencil" || C.currentTool === "brush" || C.currentTool === "eraser") {
            C.drawInProgress();
        }
        if (C.selectedElement) C.drawSelection(C.selectedElement);
        C.ctx.restore();
        C.drawMiniMap();
    }

    C.drawMiniMap = function drawMiniMap() {
        var mm = document.getElementById("mini-map-canvas");
        if (!mm) return;
        var mctx = mm.getContext("2d");
        var W = mm.width || 160, H = mm.height || 100;
        mctx.clearRect(0, 0, W, H);
        mctx.fillStyle = "rgba(255,255,255,0.6)";
        mctx.fillRect(0, 0, W, H);
        if (!C.elements.length) return;
        var minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
        C.elements.forEach(function (el) {
            if (el.x != null) { minX = Math.min(minX, el.x); minY = Math.min(minY, el.y); }
            if (el.x != null) { maxX = Math.max(maxX, el.x + (el.width || 50)); maxY = Math.max(maxY, el.y + (el.height || 30)); }
        });
        var cw = C.canvas.width || 1200, ch = C.canvas.height || 800;
        var pad = 8;
        var scale = Math.min((W - pad * 2) / cw, (H - pad * 2) / ch);
        mctx.setTransform(scale, 0, 0, scale, pad, pad);
        mctx.fillStyle = "#3b82f6";
        C.elements.forEach(function (el) {
            if (el.x == null) return;
            mctx.fillRect(el.x, el.y, Math.max(2, el.width || 10), Math.max(2, el.height || 8));
        });
        mctx.setTransform(1, 0, 0, 1, 0, 0);
    }

    C.loadCollaborators = function loadCollaborators() {
        var list = document.getElementById("collaborators-list");
        if (!list) return;
        if (typeof getGBAccessToken !== "function") {
            list.innerHTML = '<div class="collaborator-empty">No collaborators</div>';
            return;
        }
        fetch("/api/canvas/collaborators", {
            headers: { Authorization: "Bearer " + (getGBAccessToken() || "") },
        })
            .then(function (r) {
                if (!r.ok) throw new Error("fetch failed");
                return r.json();
            })
            .then(function (data) {
                var people = (data && (data.collaborators || data.users || [])) || [];
                if (!people.length) {
                    list.innerHTML = '<div class="collaborator-empty">No collaborators</div>';
                    return;
                }
                list.innerHTML = people
                    .map(function (p) {
                        var name = p.name || p.email || p.username || "User";
                        return '<div class="collaborator" title="' + name + '">' + name.charAt(0).toUpperCase() + "</div>";
                    })
                    .join("");
            })
            .catch(function () {
                list.innerHTML = '<div class="collaborator-empty">No collaborators</div>';
            });
    }

    C.drawInProgress = function drawInProgress() {
        if (!C.ctx || C.points.length < 2) return;
        C.ctx.save();
        C.ctx.strokeStyle = C.strokeColor;
        C.ctx.lineWidth = C.currentTool === "eraser" ? C.strokeWidth * 4 : C.strokeWidth;
        C.ctx.lineCap = "round";
        C.ctx.lineJoin = "round";
        if (C.currentTool === "eraser") C.ctx.globalCompositeOperation = "destination-out";
        C.ctx.beginPath();
        C.ctx.moveTo(C.points[0].x, C.points[0].y);
        for (let i = 1; i < C.points.length; i++) {
            C.ctx.lineTo(C.points[i].x, C.points[i].y);
        }
        C.ctx.stroke();
        C.ctx.restore();
    }

    C.drawElement = function drawElement(el) {
        C.ctx.save();
        C.ctx.globalAlpha = el.opacity == null ? 1 : el.opacity;
        C.ctx.translate(el.x + el.width / 2, el.y + el.height / 2);
        C.ctx.rotate(((el.rotation || 0) * Math.PI) / 180);
        C.ctx.translate(-(el.x + el.width / 2), -(el.y + el.height / 2));
        C.ctx.strokeStyle = el.stroke;
        C.ctx.fillStyle = el.fill;
        C.ctx.lineWidth = el.strokeWidth || 2;
        C.ctx.lineCap = "round";
        C.ctx.lineJoin = "round";

        switch (el.type) {
            case "rectangle":
                C.ctx.strokeRect(el.x, el.y, el.width, el.height);
                if (el.fill && el.fill !== "transparent") {
                    C.ctx.fillRect(el.x, el.y, el.width, el.height);
                }
                break;
            case "ellipse":
                C.ctx.beginPath();
                C.ctx.ellipse(
                    el.x + el.width / 2,
                    el.y + el.height / 2,
                    el.width / 2,
                    el.height / 2,
                    0,
                    0,
                    Math.PI * 2
                );
                C.ctx.stroke();
                if (el.fill && el.fill !== "transparent") C.ctx.fill();
                break;
            case "line":
            case "arrow":
            case "connector":
                C.ctx.beginPath();
                C.ctx.moveTo(el.x, el.y);
                if (el.type === "connector") {
                    C.ctx.bezierCurveTo(
                        el.x + el.width / 2,
                        el.y,
                        el.x + el.width / 2,
                        el.y + el.height,
                        el.x + el.width,
                        el.y + el.height
                    );
                } else {
                    C.ctx.lineTo(el.x + el.width, el.y + el.height);
                }
                C.ctx.stroke();
                if (el.type === "arrow") {
                    const angle = Math.atan2(el.height, el.width);
                    const head = Math.max(8, el.strokeWidth * 3);
                    C.ctx.beginPath();
                    C.ctx.moveTo(el.x + el.width, el.y + el.height);
                    C.ctx.lineTo(
                        el.x + el.width - head * Math.cos(angle - Math.PI / 6),
                        el.y + el.height - head * Math.sin(angle - Math.PI / 6)
                    );
                    C.ctx.moveTo(el.x + el.width, el.y + el.height);
                    C.ctx.lineTo(
                        el.x + el.width - head * Math.cos(angle + Math.PI / 6),
                        el.y + el.height - head * Math.sin(angle + Math.PI / 6)
                    );
                    C.ctx.stroke();
                }
                break;
            case "pencil":
            case "brush":
            case "eraser":
                if (!el.points || el.points.length < 2) break;
                if (el.type === "eraser") C.ctx.globalCompositeOperation = "destination-out";
                C.ctx.beginPath();
                C.ctx.moveTo(el.points[0].x, el.points[0].y);
                for (let i = 1; i < el.points.length; i++) {
                    C.ctx.lineTo(el.points[i].x, el.points[i].y);
                }
                C.ctx.stroke();
                break;
            case "text":
                C.ctx.fillStyle = el.stroke;
                C.ctx.font = (el.fontSize || 18) + "px Inter, sans-serif";
                C.ctx.fillText(el.text || "", el.x, el.y + (el.fontSize || 18));
                break;
            case "sticky":
                C.ctx.fillStyle = "#fff9c4";
                C.ctx.fillRect(el.x, el.y, el.width, el.height);
                C.ctx.strokeRect(el.x, el.y, el.width, el.height);
                C.ctx.fillStyle = "#5b5700";
                C.ctx.font = "14px Inter, sans-serif";
                C.ctx.fillText(el.text || "", el.x + 6, el.y + 20);
                break;
            case "frame":
                C.ctx.setLineDash([6, 4]);
                C.ctx.strokeRect(el.x, el.y, el.width, el.height);
                C.ctx.setLineDash([]);
                C.ctx.fillStyle = el.stroke;
                C.ctx.font = "12px Inter, sans-serif";
                C.ctx.fillText(el.text || "Frame", el.x + 4, el.y - 4);
                break;
            case "image":
                if (el.img && el.img.complete) {
                    C.ctx.drawImage(el.img, el.x, el.y, el.width, el.height);
                } else {
                    C.ctx.strokeRect(el.x, el.y, el.width, el.height);
                }
                break;
        }
        C.ctx.restore();
    }

    C.drawSelection = function drawSelection(el) {
        C.ctx.strokeStyle = "#007bff";
        C.ctx.lineWidth = 1;
        C.ctx.setLineDash([5, 5]);
        C.ctx.strokeRect(el.x - 5, el.y - 5, el.width + 10, el.height + 10);
        C.ctx.setLineDash([]);
    }

    C.findElementAt = function findElementAt(x, y) {
        for (let i = C.elements.length - 1; i >= 0; i--) {
            const el = C.elements[i];
            if (
                x >= el.x - 5 &&
                x <= el.x + el.width + 5 &&
                y >= el.y - 5 &&
                y <= el.y + el.height + 5
            ) {
                return el;
            }
        }
        return null;
    }

    C.showElementProperties = function showElementProperties(el) {
        const panel = document.getElementById("element-properties");
        if (!panel) return;
        panel.classList.toggle("hidden", !el);
        if (!el) return;
        const set = function (id, val) {
            const input = document.getElementById(id);
            if (input) input.value = val;
        };
        set("prop-x", Math.round(el.x));
        set("prop-y", Math.round(el.y));
        set("prop-width", Math.round(el.width));
        set("prop-height", Math.round(el.height));
        set("prop-rotation", el.rotation || 0);
        set("prop-opacity", Math.round((el.opacity == null ? 1 : el.opacity) * 100));
        set("prop-stroke", el.stroke || "#000000");
        set("prop-fill", el.fill || "#ffffff");
    }

    C.hideElementProperties = function hideElementProperties() {
        const panel = document.getElementById("element-properties");
        if (panel) panel.classList.add("hidden");
    }

})();
