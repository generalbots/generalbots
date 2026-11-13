"use strict";
/* #2172/#2175 — apply gb-doc-delta/v1 shape ops to canvas.
   The app paints `elements` onto a 2D <canvas>. The old handler appended SVG
   nodes to #canvas-bg-svg (a plain div), so nothing appeared (#2191). The
   state object is also replaced by the export API, so read/write elements
   through the public getCanvasElements/setCanvasElements accessors. */
(function () {
  var SHAPES = {
    rect: "rectangle",
    rectangle: "rectangle",
    box: "rectangle",
    ellipse: "ellipse",
    circle: "ellipse",
    line: "line",
    arrow: "arrow",
    connector: "connector",
    text: "text",
    image: "image",
  };

  function readElements() {
    if (typeof window.getCanvasElements === "function") {
      var viaApi = window.getCanvasElements();
      if (Array.isArray(viaApi)) return viaApi.slice();
    }
    var C = window._canvasApp;
    return C && Array.isArray(C.elements) ? C.elements.slice() : null;
  }

  function commitElements(elements) {
    if (typeof window.setCanvasElements === "function") {
      window.setCanvasElements(elements);
      return;
    }
    var C = window._canvasApp;
    if (!C) return;
    C.elements = elements;
    if (typeof C.saveHistory === "function") C.saveHistory();
    if (typeof C.redraw === "function") C.redraw();
  }

  function num(value, fallback) {
    var n = parseFloat(value);
    return isNaN(n) ? fallback : n;
  }

  function findElement(elements, id) {
    for (var i = 0; i < elements.length; i++) {
      if (String(elements[i].id) === String(id)) return elements[i];
    }
    return null;
  }

  function elementFromOp(C, op, id) {
    var attrs = op.attrs || {};
    var type = SHAPES[String(op.shape || op.type || "").toLowerCase()] ||
      (op.op === "add_text" ? "text" : "rectangle");
    var el = {
      id: id,
      type: type,
      x: num(attrs.x, 40),
      y: num(attrs.y, 40),
      width: num(attrs.width, 160),
      height: num(attrs.height, 90),
      stroke: attrs.stroke || "#0284c7",
      fill: attrs.fill || (type === "text" ? "transparent" : "#e0f2fe"),
      strokeWidth: num(attrs["stroke-width"] || attrs.strokeWidth, 2),
      rotation: num(attrs.rotation, 0),
      opacity: num(attrs.opacity, 1),
      layer: op.layer || (C && C.activeLayerId) || undefined,
    };
    if (op.text != null) el.text = String(op.text);
    if (type === "text") {
      el.fontSize = num(attrs["font-size"] || attrs.fontSize, 18);
      el.width = num(attrs.width, String(el.text || "").length * el.fontSize * 0.6);
      el.height = num(attrs.height, el.fontSize * 1.4);
    }
    return el;
  }

  function apply(payload) {
    var elements = readElements();
    if (!elements) return;
    var C = window._canvasApp;
    var changed = false;
    (payload.ops || []).forEach(function (op) {
      if (!op) return;
      if (op.op === "clear") {
        elements = [];
        var bg = document.getElementById("canvas-bg-svg");
        if (bg) bg.innerHTML = "";
        changed = true;
        return;
      }
      if (op.op === "add_shape" || op.op === "add_text" || op.op === "add_image") {
        var id = op.id != null ? op.id : "delta-" + Date.now() + "-" +
          Math.random().toString(36).slice(2, 7);
        var el = elementFromOp(C, op, id);
        var existing = findElement(elements, id);
        if (existing) Object.assign(existing, el);
        else elements.push(el);
        changed = true;
        return;
      }
      if (op.op === "update_shape" || op.op === "set_style" || op.op === "set_layer") {
        var target = op.id != null ? findElement(elements, op.id) : null;
        if (!target) return;
        var attrs = op.attrs || {};
        ["x", "y", "width", "height", "rotation", "opacity"].forEach(function (key) {
          if (attrs[key] != null) target[key] = num(attrs[key], target[key]);
        });
        if (attrs.fill != null) target.fill = attrs.fill;
        if (attrs.stroke != null) target.stroke = attrs.stroke;
        if (attrs["stroke-width"] != null) target.strokeWidth = num(attrs["stroke-width"], target.strokeWidth);
        if (attrs["font-size"] != null) target.fontSize = num(attrs["font-size"], target.fontSize);
        if (op.style && op.style.fill) target.fill = op.style.fill;
        if (op.text != null) target.text = String(op.text);
        if (op.layer != null) target.layer = op.layer;
        changed = true;
        return;
      }
      if (op.op === "delete_shape" && op.id != null) {
        elements = elements.filter(function (item) {
          return String(item.id) !== String(op.id);
        });
        changed = true;
        return;
      }
      if (op.op === "bring_to_front" && op.id != null) {
        var front = findElement(elements, op.id);
        if (!front) return;
        elements = elements.filter(function (item) { return item !== front; });
        elements.push(front);
        changed = true;
        return;
      }
      if (op.op === "append_block" || op.op === "add_block") {
        var textId = op.id != null ? op.id : "delta-text-" + Date.now();
        var textEl = findElement(elements, textId);
        if (textEl) {
          textEl.text = String(op.text || op.value || "");
        } else {
          elements.push(elementFromOp(C, { op: "add_text", text: op.text || op.value || "" }, textId));
        }
        changed = true;
      }
    });
    if (changed) commitElements(elements);
  }

  function registerWhenReady(attempt) {
    if (window.GBDocDelta && typeof window.GBDocDelta.register === "function") {
      window.GBDocDelta.register("canvas", apply);
      return;
    }
    if ((attempt || 0) >= 20) return;
    setTimeout(function () { registerWhenReady((attempt || 0) + 1); }, 250);
  }
  registerWhenReady(0);
})();
