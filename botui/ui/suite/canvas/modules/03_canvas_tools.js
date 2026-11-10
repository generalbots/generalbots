(function () {
    "use strict";
    var C = window._canvasApp;
    C.saveHistory = function saveHistory() {
        C.history = C.history.slice(0, C.historyIndex + 1);
        C.history.push(JSON.stringify(C.elements));
        C.historyIndex = C.history.length - 1;
    }

    C.undo = function undo() {
        if (C.historyIndex > 0) {
            C.historyIndex--;
            C.elements = JSON.parse(C.history[C.historyIndex]);
            C.selectedElement = null;
            C.redraw();
        }
    }

    C.redo = function redo() {
        if (C.historyIndex < C.history.length - 1) {
            C.historyIndex++;
            C.elements = JSON.parse(C.history[C.historyIndex]);
            C.redraw();
        }
    }

    C.setStrokeColor = function setStrokeColor(color) {
        C.strokeColor = color;
        if (C.selectedElement) {
            C.selectedElement.stroke = color;
            C.saveHistory();
            C.redraw();
        }
    }

    C.setFillColor = function setFillColor(color) {
        C.fillColor = color;
        if (C.selectedElement) {
            C.selectedElement.fill = color;
            C.saveHistory();
            C.redraw();
        }
    }

    C.setStrokeWidth = function setStrokeWidth(width) {
        C.strokeWidth = parseInt(width) || 2;
        if (C.selectedElement) {
            C.selectedElement.strokeWidth = C.strokeWidth;
            C.saveHistory();
            C.redraw();
        }
    }

    C.zoomIn = function zoomIn() {
        C.zoom = Math.min(5, C.zoom * 1.2);
        C.setZoomLabel();
        C.redraw();
    }

    C.zoomOut = function zoomOut() {
        C.zoom = Math.max(0.1, C.zoom / 1.2);
        C.setZoomLabel();
        C.redraw();
    }

    C.resetZoom = function resetZoom() {
        C.zoom = 1;
        C.panX = 0;
        C.panY = 0;
        C.setZoomLabel();
        C.redraw();
    }

    C.fitToScreen = function fitToScreen() {
        C.resetZoom();
    }

    C.clearCanvas = function clearCanvas() {
        if (C.elements.length && (window.GBModal||{confirm:confirm}).confirm((window.gbT ? window.gbT("canvas-clear-confirm", "Clear all elements?") : "Clear all elements?"))) {
            C.elements = [];
            C.selectedElement = null;
            C.saveHistory();
            C.redraw();
        }
    }

    C.copySelected = function copySelected() {
        if (!C.selectedElement) return;
        C.clipboard = JSON.parse(JSON.stringify(C.selectedElement));
    }

    C.duplicateSelected = function duplicateSelected() {
        if (!C.selectedElement) return;
        const copy = JSON.parse(JSON.stringify(C.selectedElement));
        copy.id = Date.now();
        copy.x += 20;
        copy.y += 20;
        C.elements.push(copy);
        C.selectedElement = copy;
        C.saveHistory();
        C.redraw();
    }

    C.pasteClipboard = function pasteClipboard() {
        if (!C.clipboard) return;
        const copy = JSON.parse(JSON.stringify(C.clipboard));
        copy.id = Date.now();
        copy.x += 20;
        copy.y += 20;
        C.elements.push(copy);
        C.selectedElement = copy;
        C.saveHistory();
        C.redraw();
    }

    C.deleteSelected = function deleteSelected() {
        if (!C.selectedElement) return;
        C.elements = C.elements.filter(function (el) {
            return el !== C.selectedElement;
        });
        C.selectedElement = null;
        C.hideElementProperties();
        C.saveHistory();
        C.redraw();
    }

    C.addLayer = function addLayer() {
        const newId = "layer_" + (C.layers.length + 1);
        C.layers.push({ id: newId, name: "Layer " + C.layers.length, hidden: false });
        C.activeLayerId = newId;
        C.renderLayers();
    }

    C.renderLayers = function renderLayers() {
        const list = document.getElementById("layers-list");
        if (!list) return;
        list.innerHTML = C.layers
            .map(function (l) {
                return (
                    '<div class="layer-item' + (l.id === C.activeLayerId ? " active" : "") + '" data-layer-id="' + l.id + '">' +
                    '<span class="layer-visibility" data-act="vis">' + (l.hidden ? (window.GBIcons ? window.GBIcons.html("eyeOff", 14) : "hidden") : (window.GBIcons ? window.GBIcons.html("eye", 14) : "visible")) + "</span>" +
                    '<span class="layer-name">' + l.name + "</span>" +
                    '<span class="layer-lock" data-act="lock">' + (l.locked ? (window.GBIcons ? window.GBIcons.html("lock", 14) : "locked") : (window.GBIcons ? window.GBIcons.html("unlock", 14) : "unlocked")) + "</span>" +
                    "</div>"
                );
            })
            .join("");
        Array.prototype.forEach.call(list.querySelectorAll(".layer-item"), function (item) {
            item.addEventListener("click", function (e) {
                const id = item.getAttribute("data-layer-id");
                if (e.target && e.target.getAttribute("data-act") === "vis") {
                    const layer = C.layers.filter(function (l) { return l.id === id; })[0];
                    if (layer) { layer.hidden = !layer.hidden; C.renderLayers(); C.redraw(); }
                    return;
                }
                if (e.target && e.target.getAttribute("data-act") === "lock") {
                    const layer = C.layers.filter(function (l) { return l.id === id; })[0];
                    if (layer) { layer.locked = !layer.locked; C.renderLayers(); }
                    return;
                }
                C.activeLayerId = id;
                C.renderLayers();
            });
        });
    }

    C.bringToFront = function bringToFront() {
        if (!C.selectedElement) return;
        C.elements = C.elements.filter(function (el) {
            return el !== C.selectedElement;
        });
        C.elements.push(C.selectedElement);
        C.saveHistory();
        C.redraw();
    }

    C.sendToBack = function sendToBack() {
        if (!C.selectedElement) return;
        C.elements = C.elements.filter(function (el) {
            return el !== C.selectedElement;
        });
        C.elements.unshift(C.selectedElement);
        C.saveHistory();
        C.redraw();
    }

    C.togglePropertiesPanel = function togglePropertiesPanel() {
        const panel = document.getElementById("properties-panel");
        if (panel) panel.classList.toggle("collapsed");
    }

    C.toggleSnapToGrid = function toggleSnapToGrid(enabled) {
        C.snapToGrid = !!enabled;
        const checkbox = document.getElementById("snap-grid");
        if (checkbox) checkbox.checked = C.snapToGrid;
    }

    C.setGridType = function setGridType(type) {
        C.gridType = type || "none";
        C.redraw();
    }

    C.setCanvasBackground = function setCanvasBackground(color) {
        C.canvasBg = color || "#ffffff";
        const picker = document.getElementById("canvas-bg");
        if (picker) picker.value = C.canvasBg;
        C.redraw();
    }

    C.updateElementProperty = function updateElementProperty(prop, value) {
        if (!C.selectedElement) return;
        const el = C.selectedElement;
        if (prop === "strokeColor") el.stroke = value;
        else if (prop === "fillColor") el.fill = value;
        else if (prop === "opacity") el.opacity = parseInt(value) / 100;
        else if (prop === "rotation") el.rotation = parseInt(value) || 0;
        else if (prop === "width" || prop === "height" || prop === "x" || prop === "y") {
            el[prop] = parseInt(value) || 0;
        }
        C.saveHistory();
        C.redraw();
    }

    C.renameCanvas = function renameCanvas(name) {
        const input = document.getElementById("canvas-name");
        if (input && name) input.value = name;
        const status = document.getElementById("save-status");
        if (status) status.textContent = (window.gbT ? window.gbT("canvas-saved", "Saved") : "Saved");
    }

})();
