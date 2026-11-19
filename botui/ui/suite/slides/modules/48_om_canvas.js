"use strict";
/* Slides object model — local-only render + pointer layer (#2430). Objects use
 * .sl-om-* classes, never .sl-element, so the existing server-persisting
 * SlideCanvas bindings are never triggered. All writes go through OM/Ops. */

(function () {
  var OM = window.GBSlidesOM;
  var Ops = window.GBSlidesOMOps;
  var smart = [];
  var drag = null;
  var svgNS = "http://www.w3.org/2000/svg";

  function canvas() {
    return (window.SlideCanvas && window.SlideCanvas.canvas) ||
      (typeof document !== "undefined" && document.getElementById("slide-canvas")) || null;
  }
  function el(tag, cls) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    return node;
  }
  function svg(tag) { return document.createElementNS(svgNS, tag); }

  function ensureLayer() {
    var host = canvas();
    if (!host) return null;
    var layer = host.querySelector(".sl-om-layer");
    if (layer) return layer;
    layer = el("div", "sl-om-layer");
    layer.setAttribute("role", "group");
    layer.setAttribute("aria-label", OM.t("slides-om-layer-aria", "Object layer (local only)"));
    var rulers = el("div", "sl-om-rulers");
    rulers.appendChild(el("div", "sl-om-ruler sl-om-ruler-x"));
    rulers.appendChild(el("div", "sl-om-ruler sl-om-ruler-y"));
    layer.appendChild(rulers);
    layer.appendChild(el("div", "sl-om-grid"));
    var lines = svg("svg");
    lines.setAttribute("class", "sl-om-connectors");
    lines.setAttribute("aria-hidden", "true");
    layer.appendChild(lines);
    layer.appendChild(el("div", "sl-om-guides"));
    layer.appendChild(el("div", "sl-om-objects"));
    layer.appendChild(el("div", "sl-om-rubber"));
    var badge = el("span", "sl-om-local-badge", OM.t("slides-om-local-only", "Local only — not saved to the server"));
    badge.setAttribute("data-i18n", "slides-om-local-only");
    layer.appendChild(badge);
    host.appendChild(layer);
    return layer;
  }

  function drawNode(node, host) {
    var r = { x: OM.num(node.x, 0), y: OM.num(node.y, 0), w: OM.num(node.w, 0), h: OM.num(node.h, 0) };
    var box = el("div", "sl-om-node sl-om-" + (node.type || "shape"));
    box.setAttribute("data-om-id", node.id);
    box.tabIndex = 0;
    box.setAttribute("role", "button");
    box.setAttribute("aria-label", node.alt || node.text ||
      OM.t("slides-om-node-default", "Object"));
    if (node.decorative) box.setAttribute("aria-hidden", "true");
    box.style.left = r.x + "px"; box.style.top = r.y + "px";
    box.style.width = r.w + "px"; box.style.height = r.h + "px";
    box.style.zIndex = String(node.z || 1);
    if (node.fill) box.style.background = node.fill;
    if (node.stroke) box.style.borderColor = node.stroke;
    if (node.opacity != null) box.style.opacity = String(node.opacity);
    if (node.locked) box.classList.add("is-locked");
    if (node.hidden) box.classList.add("is-hidden");
    if (OM.isSelected(node.id)) box.classList.add("is-selected");
    if (node.type === "text" && node.text) box.appendChild(el("span", "sl-om-text", node.text));
    if (node.type === "image" && node.src) {
      var img = el("img", "sl-om-image");
      img.setAttribute("src", node.src);
      img.setAttribute("alt", node.decorative ? "" : (node.alt || ""));
      box.appendChild(img);
    }
    if (node.crop && node.crop.shape && node.crop.shape !== "none") {
      box.classList.add("sl-om-mask-" + node.crop.shape);
    }
    if (node.type === "freeform" && Array.isArray(node.points) && node.points.length > 1) {
      var path = svg("svg");
      path.setAttribute("class", "sl-om-freeform");
      path.setAttribute("viewBox", "0 0 " + Math.max(1, r.w) + " " + Math.max(1, r.h));
      var line = svg("polyline");
      line.setAttribute("points", node.points.map(function (p) {
        return (p.x - r.x) + "," + (p.y - r.y);
      }).join(" "));
      line.setAttribute("fill", "none");
      line.setAttribute("stroke", node.stroke || "#3b82f6");
      line.setAttribute("stroke-width", String(node.strokeWidth || 2));
      path.appendChild(line);
      box.appendChild(path);
    }
    host.appendChild(box);
    return box;
  }

  function drawGroup(group, s, host) {
    var r = OM.rect(s, group.id);
    if (!r || !OM.descendants(s, group.id).length) return null;
    var box = el("div", "sl-om-node sl-om-group");
    box.setAttribute("data-om-id", group.id);
    box.setAttribute("role", "button");
    box.tabIndex = 0;
    box.setAttribute("aria-label", OM.t("slides-om-group-label", "Group"));
    box.style.left = r.x + "px"; box.style.top = r.y + "px";
    box.style.width = r.w + "px"; box.style.height = r.h + "px";
    box.style.zIndex = String(group.z || 1);
    if (group.locked) box.classList.add("is-locked");
    if (group.hidden) box.classList.add("is-hidden");
    if (OM.isSelected(group.id)) box.classList.add("is-selected");
    host.appendChild(box);
    return box;
  }

  function drawConnectors(s, svgRoot) {
    while (svgRoot.firstChild) svgRoot.removeChild(svgRoot.firstChild);
    OM.updateConnectors(s);
    (s.objects || []).forEach(function (o) {
      if (o.type !== "connector" || o.hidden || !o._a || !o._b) return;
      var line = svg("line");
      line.setAttribute("x1", String(o._a.x)); line.setAttribute("y1", String(o._a.y));
      line.setAttribute("x2", String(o._b.x)); line.setAttribute("y2", String(o._b.y));
      line.setAttribute("stroke", o.stroke || "#64748b");
      line.setAttribute("stroke-width", String(o.strokeWidth || 2));
      if (o.lineStyle === "dashed") line.setAttribute("stroke-dasharray", "8 5");
      if (o.lineStyle === "dotted") line.setAttribute("stroke-dasharray", "2 4");
      if (o.arrowEnd) line.setAttribute("marker-end", "url(#sl-om-arrow)");
      svgRoot.appendChild(line);
    });
    if (!svgRoot.querySelector("defs")) {
      var defs = svg("defs");
      var marker = svg("marker");
      marker.setAttribute("id", "sl-om-arrow");
      marker.setAttribute("markerWidth", "8"); marker.setAttribute("markerHeight", "8");
      marker.setAttribute("refX", "7"); marker.setAttribute("refY", "4");
      marker.setAttribute("orient", "auto");
      var triangle = svg("path");
      triangle.setAttribute("d", "M0,0 L8,4 L0,8 z");
      marker.appendChild(triangle); defs.appendChild(marker);
      svgRoot.insertBefore(defs, svgRoot.firstChild);
    }
  }

  function drawGuides(s, host) {
    while (host.firstChild) host.removeChild(host.firstChild);
    (s.guides || []).concat(smart).forEach(function (g) {
      var line = el("div", "sl-om-guide sl-om-guide-" + g.axis);
      if (g.axis === "x") line.style.left = g.at + "px";
      else line.style.top = g.at + "px";
      host.appendChild(line);
    });
  }

  function drawGrid(layer, s) {
    var grid = layer.querySelector(".sl-om-grid");
    var settings = OM.settings();
    layer.classList.toggle("show-grid", !!settings.grid);
    layer.classList.toggle("show-rulers", !!settings.rulers);
    layer.classList.toggle("snap-grid", !!settings.snap);
    if (grid) {
      grid.style.backgroundSize = (settings.gridSize || 10) + "px " + (settings.gridSize || 10) + "px";
    }
  }

  function render() {
    var layer = ensureLayer();
    if (!layer) return;
    var s = OM.slide();
    var objects = layer.querySelector(".sl-om-objects");
    var lines = layer.querySelector(".sl-om-connectors");
    var guides = layer.querySelector(".sl-om-guides");
    if (!objects || !lines || !guides) return;
    while (objects.firstChild) objects.removeChild(objects.firstChild);
    OM.topLevel(OM.slideIndex()).forEach(function (node) {
      if (node.type === undefined) drawGroup(node, s, objects);
      else drawNode(node, objects);
    });
    drawConnectors(s, lines);
    drawGuides(s, guides);
    drawGrid(layer, s);
  }

  function nodeFromEvent(event) {
    var target = event.target;
    return target && target.closest ? target.closest(".sl-om-node") : null;
  }
  function localPoint(event) {
    var host = canvas();
    if (!host) return { x: 0, y: 0 };
    var rect = host.getBoundingClientRect ? host.getBoundingClientRect() : { left: 0, top: 0 };
    var factor = (window.SlideCanvas && window.SlideCanvas.scale) || 1;
    return { x: (event.clientX - rect.left) / factor, y: (event.clientY - rect.top) / factor };
  }

  function beginDrag(event, box) {
    var id = box.getAttribute("data-om-id");
    if (event.shiftKey) OM.toggle(id);
    else if (!OM.isSelected(id)) OM.setSelection([id]);
    if (OM.penMode || OM.connectorMode) return;
    var working = OM.clone(OM.state);
    var s = working.slides[OM.slideKey()];
    if (!s) return;
    var selected = OM.selection.slice();
    var start = localPoint(event);
    var moved = false;
    smart = [];
    function onMove(moveEvent) {
      var point = localPoint(moveEvent);
      var dx = point.x - start.x, dy = point.y - start.y;
      if (Math.abs(dx) + Math.abs(dy) < 1 && !moved) return;
      moved = true;
      var snapped = Ops.smartGuides(selected, dx, dy);
      dx = snapped.dx; dy = snapped.dy;
      smart = snapped.guides;
      OM.move(s, selected, dx - (drag.dx || 0), dy - (drag.dy || 0));
      drag.dx = dx; drag.dy = dy;
      renderWorking(working, smart);
    }
    function onUp() {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      drag = null; smart = [];
      if (moved) OM.replaceState(working, OM.t("slides-om-move", "move objects"));
      else render();
    }
    drag = { dx: 0, dy: 0 };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }

  function renderWorking(working, guides) {
    var previous = OM.state;
    OM.state = working;
    smart = guides;
    render();
    OM.state = previous;
  }

  function beginRubber(event) {
    var layer = ensureLayer();
    if (!layer) return;
    var rubber = layer.querySelector(".sl-om-rubber");
    if (!rubber) return;
    var start = localPoint(event);
    if (!event.shiftKey) OM.clearSelection();
    function onMove(moveEvent) {
      var point = localPoint(moveEvent);
      var x = Math.min(start.x, point.x), y = Math.min(start.y, point.y);
      rubber.style.left = x + "px"; rubber.style.top = y + "px";
      rubber.style.width = Math.abs(point.x - start.x) + "px";
      rubber.style.height = Math.abs(point.y - start.y) + "px";
      rubber.classList.add("is-active");
    }
    function onUp(moveEvent) {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      rubber.classList.remove("is-active");
      rubber.style.width = "0px"; rubber.style.height = "0px";
      var end = localPoint(moveEvent);
      var box = { x: Math.min(start.x, end.x), y: Math.min(start.y, end.y),
        w: Math.abs(end.x - start.x), h: Math.abs(end.y - start.y) };
      if (box.w < 3 || box.h < 3) return;
      var s = OM.slide();
      OM.topLevel(OM.slideIndex()).forEach(function (node) {
        var r = OM.rect(s, node.id);
        if (!r) return;
        if (r.x < box.x + box.w && r.x + r.w > box.x && r.y < box.y + box.h && r.y + r.h > box.y) {
          OM.add(node.id);
        }
      });
    }
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }

  function beginPen(event) {
    var points = [localPoint(event)];
    function onMove(moveEvent) { points.push(localPoint(moveEvent)); }
    function onUp() {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      Ops.addFreeform(points);
      OM.penMode = false;
      render();
    }
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }

  function onPointerDown(event) {
    if (event.button !== 0) return;
    var box = nodeFromEvent(event);
    if (box) {
      event.preventDefault();
      if (OM.connectorMode && OM.connectorMode.from && OM.connectorMode.from !== box.getAttribute("data-om-id")) {
        Ops.addConnector(OM.connectorMode.from, box.getAttribute("data-om-id"),
          OM.connectorMode.anchor || "center", "center");
        OM.connectorMode = null;
        render();
        return;
      }
      beginDrag(event, box);
      return;
    }
    if (OM.penMode) { event.preventDefault(); beginPen(event); return; }
    if (event.target && event.target.closest && event.target.closest(".sl-om-layer")) beginRubber(event);
  }

  function onKeyDown(event) {
    var target = event.target;
    if (target && target.closest && target.closest("input,textarea,select,[contenteditable='true']")) return;
    var key = event.key;
    if (!OM.selection.length) return;
    var step = event.shiftKey ? 1 : 10;
    if (key === "ArrowLeft") Ops.nudge(-step, 0);
    else if (key === "ArrowRight") Ops.nudge(step, 0);
    else if (key === "ArrowUp") Ops.nudge(0, -step);
    else if (key === "ArrowDown") Ops.nudge(0, step);
    else if (key === "Delete" || key === "Backspace") Ops.deleteSelected();
    else return;
    event.preventDefault();
  }

  function bind() {
    var host = canvas();
    if (host && !host.__omBound) {
      host.__omBound = true;
      host.addEventListener("pointerdown", onPointerDown);
    }
    if (!window.__omKeysBound) {
      window.__omKeysBound = true;
      document.addEventListener("keydown", onKeyDown);
    }
    OM.on("change", render);
    OM.on("undo", render);
    OM.on("redo", render);
    OM.on("selection", render);
    if (document.body) {
      document.body.addEventListener("htmx:afterSwap", function () {
        setTimeout(function () { render(); }, 0);
      });
    }
  }

  window.GBSlidesOMCanvas = {
    render: render,
    bind: bind,
    canvas: canvas,
    clearSmartGuides: function () { smart = []; }
  };

  if (typeof document !== "undefined") {
    if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bind);
    else bind();
  }
}());
