"use strict";
/* Slides canvas object model — connectors, freeform, image and slide-size
 * mutations (#2430). Extends the single object created by 47_om_ops.js so every
 * operation shares the same one-checkpoint undo semantics. Pure logic, no DOM. */

(function () {
  var OM = window.GBSlidesOM;
  var Ops = window.GBSlidesOMOps;
  var H = window.GBSlidesOMOpsHelpers;
  var L = H.L, byId = H.byId, nextZ = H.nextZ, targets = H.targets;

  Ops.smartGuides = function (ids, dx, dy, threshold) {
    var s = OM.slide();
    if (!OM.state.settings.smartGuides) return { dx: dx, dy: dy, guides: [] };
    var limit = threshold == null ? 6 : threshold;
    var moving = OM.bounds(ids, OM.slideIndex());
    if (!moving) return { dx: dx, dy: dy, guides: [] };
    var candidatesX = [], candidatesY = [];
    OM.topLevel(OM.slideIndex()).forEach(function (n) {
      if ((ids || []).indexOf(n.id) >= 0) return;
      var r = OM.rect(s, n.id);
      if (!r) return;
      candidatesX.push(r.x, r.x + r.w / 2, r.x + r.w);
      candidatesY.push(r.y, r.y + r.h / 2, r.y + r.h);
    });
    var size = s.size;
    candidatesX.push(OM.num(size.w, 960) / 2);
    candidatesY.push(OM.num(size.h, 540) / 2);
    var guides = [];
    var bestX = null, bestY = null;
    function pick(edges, list, axis) {
      edges.forEach(function (edge) {
        list.forEach(function (target) {
          var diff = target - (edge + (axis === "x" ? dx : dy));
          if (Math.abs(diff) > limit) return;
          if (axis === "x") {
            if (bestX === null || Math.abs(diff) < Math.abs(bestX.diff)) bestX = { diff: diff, at: target };
          } else if (bestY === null || Math.abs(diff) < Math.abs(bestY.diff)) {
            bestY = { diff: diff, at: target };
          }
        });
      });
    }
    pick([moving.x, moving.x + moving.w / 2, moving.x + moving.w], candidatesX, "x");
    pick([moving.y, moving.y + moving.h / 2, moving.y + moving.h], candidatesY, "y");
    if (bestX) guides.push({ axis: "x", at: bestX.at });
    if (bestY) guides.push({ axis: "y", at: bestY.at });
    return { dx: dx + (bestX ? bestX.diff : 0), dy: dy + (bestY ? bestY.diff : 0), guides: guides };
  };

  Ops.addConnector = function (fromId, toId, fromAnchor, toAnchor, style) {
    if (!fromId || !toId) return null;
    var s = OM.slide();
    var id = OM.uid("con");
    OM.mutate(L("slides-om-add-connector", "attach connector"), function () {
      var connector = {
        id: id, type: "connector", x: 0, y: 0, w: 0, h: 0, z: nextZ(s),
        groupId: null, locked: false, hidden: false,
        stroke: "#64748b", strokeWidth: 2, lineStyle: "solid",
        arrowStart: false, arrowEnd: true, opacity: 1, shadow: "none",
        from: { id: fromId, anchor: fromAnchor || "center" },
        to: { id: toId, anchor: toAnchor || "center" }
      };
      if (style) Object.keys(style).forEach(function (k) { connector[k] = style[k]; });
      s.objects.push(connector);
      OM.updateConnectors(s);
    });
    OM.setSelection([id]);
    return id;
  };

  Ops.setLineStyle = function (props, ids) {
    var list = targets(ids);
    if (!list.length) return false;
    var s = OM.slide();
    OM.mutate(L("slides-om-line-style", "change line style"), function () {
      list.forEach(function (id) {
        var node = byId(s, id);
        if (!node) return;
        Object.keys(props || {}).forEach(function (key) { node[key] = props[key]; });
      });
    });
    return true;
  };

  Ops.addFreeform = function (points, style) {
    if (!Array.isArray(points) || points.length < 2) return null;
    var s = OM.slide();
    var id = OM.uid("free");
    OM.mutate(L("slides-om-add-freeform", "draw freeform"), function () {
      var xs = points.map(function (p) { return p.x; });
      var ys = points.map(function (p) { return p.y; });
      var shape = {
        id: id, type: "freeform", x: Math.min.apply(null, xs), y: Math.min.apply(null, ys),
        w: Math.max.apply(null, xs) - Math.min.apply(null, xs),
        h: Math.max.apply(null, ys) - Math.min.apply(null, ys),
        z: nextZ(s), groupId: null, locked: false, hidden: false,
        stroke: "#3b82f6", strokeWidth: 2, lineStyle: "solid",
        arrowStart: false, arrowEnd: false, opacity: 1, shadow: "none",
        points: OM.clone(points), text: "", src: "", alt: "", decorative: false,
        hyperlink: { kind: "none", target: "" }, crop: null, from: null, to: null
      };
      if (style) Object.keys(style).forEach(function (k) { shape[k] = style[k]; });
      s.objects.push(shape);
    });
    OM.setSelection([id]);
    return id;
  };

  Ops.crop = function (id, patch) {
    var s = OM.slide();
    var node = (s.objects || []).find(function (o) { return o.id === id; });
    if (!node || node.type !== "image") return false;
    OM.mutate(L("slides-om-crop", "crop image"), function () {
      if (!node.crop) {
        node.crop = {
          x: 0, y: 0, w: 1, h: 1, shape: "none",
          original: { x: node.x, y: node.y, w: node.w, h: node.h, src: node.src }
        };
      }
      Object.keys(patch || {}).forEach(function (k) { node.crop[k] = ["x", "y", "w", "h"].indexOf(k) >= 0 ? Math.max(0, Math.min(1, OM.num(patch[k], 0))) : patch[k]; });
    });
    return true;
  };

  Ops.resetCrop = function (id) {
    var s = OM.slide();
    var node = (s.objects || []).find(function (o) { return o.id === id; });
    if (!node || !node.crop) return false;
    OM.mutate(L("slides-om-crop-reset", "reset crop"), function () {
      var original = node.crop.original;
      if (original) {
        node.x = original.x; node.y = original.y;
        node.w = original.w; node.h = original.h; node.src = original.src;
      }
      node.crop = null;
    });
    return true;
  };

  Ops.mask = function (id, shape) {
    return this.crop(id, { shape: shape || "none" });
  };

  Ops.replaceImage = function (id, src) {
    var s = OM.slide();
    var node = (s.objects || []).find(function (o) { return o.id === id; });
    if (!node) return false;
    OM.mutate(L("slides-om-image-replace", "replace image"), function () {
      node.src = src;
      if (node.crop && node.crop.original) node.crop.original.src = src;
    });
    return true;
  };

  Ops.hyperlink = function (id, kind, target) {
    var s = OM.slide();
    var node = byId(s, id);
    if (!node) return false;
    OM.mutate(L("slides-om-link", "set hyperlink"), function () {
      node.hyperlink = { kind: kind || "none", target: target || "" };
    });
    return true;
  };

  Ops.alt = function (id, text, decorative) {
    var s = OM.slide();
    var node = byId(s, id);
    if (!node) return false;
    OM.mutate(L("slides-om-alt", "set alt text"), function () {
      node.alt = text || "";
      node.decorative = !!decorative;
    });
    return true;
  };

  Ops.setSlideSize = function (target, idx) {
    var s = OM.slide(idx);
    var size = typeof target === "string" ? OM.preset(target) : target;
    if (!size || !isFinite(size.w) || !isFinite(size.h)) return null;
    var from = OM.clone(s.size);
    var result = null;
    OM.mutate(L("slides-om-size-change", "change slide size"), function () {
      result = OM.rescale(s, from, size);
    });
    return { from: from, to: OM.clone(s.size), scale: result };
  };
}());
