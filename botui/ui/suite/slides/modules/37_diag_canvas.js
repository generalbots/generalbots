/* slides diagrams — local-only render layer over the slide canvas (#2488).
 * Uses .sl-diag-node (never .sl-element) so the existing SlideCanvas.persistElement
 * server write is never triggered. Dragging stores coordinates in localStorage. */
"use strict";

(function () {
    "use strict";

    var GB = window.GBSlidesDiagrams;
    var icons = window.SDIcons;
    var flow = window.SDFlow;

    function t(key, fallback) {
        return sdT(key, fallback);
    }

    function canvas() {
        return (window.SlideCanvas && window.SlideCanvas.canvas) || document.getElementById("slide-canvas");
    }

    function scale() {
        var value = window.SlideCanvas && window.SlideCanvas.scale;
        return typeof value === "number" && value > 0 ? value : 1;
      }

    function ensureLayer() {
        var host = canvas();
        if (!host) { return null; }
        var layer = sd$(".sl-diag-layer", host);
        if (layer) { return layer; }
        layer = sdEl("div", "sl-diag-layer");
        layer.setAttribute("role", "group");
        layer.setAttribute("aria-label", t("slides-diagrams-layer-aria", "Diagram layer (local only)"));
        layer.appendChild(GB.localBadge());
        host.appendChild(layer);
        return layer;
    }

    function renderNode(diagram, node, layer) {
        var box = sdEl("div", "sl-diag-node");
        box.dataset.nodeId = node.id;
        box.tabIndex = 0;
        box.setAttribute("role", "button");
        box.setAttribute("aria-label", node.text || t("slides-diagrams-node-default", "Node"));
        box.style.left = (node.x || 0) + "px";
        box.style.top = (node.y || 0) + "px";
        box.style.width = (node.w || 150) + "px";
        box.style.minHeight = (node.h || 68) + "px";
        if (node.color) { box.style.setProperty("--sd-node-color", node.color); }
        if (diagram.grouped) { box.classList.add("is-grouped"); }

        if (node.icon && icons && icons.svg) {
            var glyph = icons.svg(node.icon, icons.label(icons.find(node.icon)));
            if (glyph) { box.appendChild(glyph); }
        }
        var text = sdEl("span", "sl-diag-node-text", node.text || "");
        box.appendChild(text);
        if (node.title) { box.appendChild(sdEl("span", "sl-diag-node-title", node.title)); }
        layer.appendChild(box);
        bindNode(box, diagram, node);
        return box;
    }

    function bindNode(box, diagram, node) {
        box.addEventListener("pointerdown", function (event) {
            if (event.button !== 0) { return; }
            event.preventDefault();
            event.stopPropagation();
            drag(box, diagram.id, node.id, event);
        });
        box.addEventListener("keydown", function (event) {
            var step = event.shiftKey ? 1 : 10;
            var dx = 0;
            var dy = 0;
            if (event.key === "ArrowLeft") { dx = -step; }
            else if (event.key === "ArrowRight") { dx = step; }
            else if (event.key === "ArrowUp") { dy = -step; }
            else if (event.key === "ArrowDown") { dy = step; }
            else { return; }
            event.preventDefault();
            moveBy(diagram.id, node.id, dx, dy);
        });
    }

    function moveBy(id, nodeId, dx, dy) {
        var current = GB.get(id);
        if (!current) { return; }
        var next = sdClone(current);
        var targets = next.grouped ? next.nodes : [sdPresetsNode(next, nodeId)];
        var moved = false;
        targets.forEach(function (node) {
            if (!node) { return; }
            node.x = (node.x || 0) + dx;
            node.y = (node.y || 0) + dy;
            moved = true;
        });
        if (!moved) { return; }
        if (flow && flow.recompute) { flow.recompute(next); }
        GB.upsert(next);
    }

    function sdPresetsNode(diagram, id) {
        return window.SDPresets ? window.SDPresets.byId(diagram, id) : null;
    }

    function drag(box, id, nodeId, event) {
        var current = GB.get(id);
        if (!current) { return; }
        var next = sdClone(current);
        var targets = [];
        next.nodes.forEach(function (node) {
            if (current.grouped || node.id === nodeId) {
                targets.push({ node: node, x: node.x || 0, y: node.y || 0 });
            }
        });
        if (!targets.length) { return; }

        var startX = event.clientX;
        var startY = event.clientY;
        var factor = scale();
        var layer = box.parentNode;
        var boxes = {};
        sd$$(".sl-diag-node", layer).forEach(function (candidate) {
            boxes[candidate.dataset.nodeId] = candidate;
        });
        var svg = sd$(".sl-diag-connectors", layer);
        var moved = false;

        function onMove(moveEvent) {
            var dx = (moveEvent.clientX - startX) / factor;
            var dy = (moveEvent.clientY - startY) / factor;
            if (Math.abs(dx) + Math.abs(dy) < 0.5) { return; }
            moved = true;
            targets.forEach(function (target) {
                target.node.x = Math.round(target.x + dx);
                target.node.y = Math.round(target.y + dy);
                var element = boxes[target.node.id];
                if (element) {
                    element.style.left = target.node.x + "px";
                    element.style.top = target.node.y + "px";
                }
            });
            if (flow && flow.recompute) { flow.recompute(next); }
            if (flow && flow.updateSvg) { flow.updateSvg(next, svg); }
            if (flow && flow.refreshTips) { flow.refreshTips(next, svg); }
        }

        function onUp() {
            window.removeEventListener("pointermove", onMove);
            window.removeEventListener("pointerup", onUp);
            if (!moved) { return; }
            GB.upsert(next);
        }

        window.addEventListener("pointermove", onMove);
        window.addEventListener("pointerup", onUp);
    }

    function refresh() {
        var host = canvas();
        if (!host) { return; }
        var old = sd$(".sl-diag-layer", host);
        if (old && old.parentNode) { old.parentNode.removeChild(old); }
        var diagram = GB.active();
        if (!diagram || !diagram.nodes || !diagram.nodes.length) { return; }
        var layer = ensureLayer();
        if (!layer) { return; }
        if (flow && flow.recompute) { flow.recompute(diagram); }
        if (flow && flow.buildSvg) { layer.appendChild(flow.buildSvg(diagram)); }
        diagram.nodes.forEach(function (node) { renderNode(diagram, node, layer); });
    }

    window.GBSlidesDiagramsCanvas = {
        refresh: refresh,
        canvas: canvas
    };
}());
