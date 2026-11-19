"use strict";
/* Slides canvas object model — core mutations (#2430). Every public operation
 * wraps a single OM.mutate() checkpoint so multi-object, group and deck-level
 * changes undo in one step. Pure logic, no DOM: safe for the offline harness. */

(function () {
  var OM = window.GBSlidesOM;
  function L(key, fallback) { return OM.t(key, fallback); }
  function byId(s, id) {
    return (s.objects || []).find(function (o) { return o.id === id; }) ||
      (s.groups || []).find(function (g) { return g.id === id; }) || null;
  }
  function nextZ(s) {
    var max = 0;
    s.objects.concat(s.groups).forEach(function (n) { max = Math.max(max, OM.num(n.z, 0)); });
    return max + 1;
  }
  function targets(ids) { return (ids && ids.length ? ids : OM.selection).slice(); }

  var Ops = {
    addShape: function (type, x, y) {
      var s = OM.slide();
      var id = OM.uid("om");
      var shape = {
        id: id, type: type, x: x == null ? 120 : x, y: y == null ? 90 : y,
        w: type === "text" ? 220 : 160, h: type === "text" ? 60 : 110,
        z: nextZ(s), groupId: null, locked: false, hidden: false,
        fill: type === "image" ? "transparent" : "#3b82f6",
        stroke: "#1e3a8a", strokeWidth: 1, lineStyle: "solid",
        arrowStart: false, arrowEnd: false, opacity: 1, shadow: "none",
        text: "", src: "", alt: "", decorative: false,
        hyperlink: { kind: "none", target: "" }, crop: null, points: [],
        from: null, to: null
      };
      OM.mutate(L("slides-om-add", "add object"), function () {
        s.objects.push(shape);
      });
      OM.setSelection([id]);
      return id;
    },

    deleteSelected: function () {
      var ids = targets();
      if (!ids.length) return 0;
      var s = OM.slide();
      var removed = 0;
      OM.mutate(L("slides-om-delete", "delete objects"), function () {
        ids.forEach(function (id) {
          var before = s.objects.length + s.groups.length;
          OM.descendants(s, id).forEach(function (node) { node._deleted = true; });
          s.objects = s.objects.filter(function (o) {
            var detached = (o.from && o.from.id === id) || (o.to && o.to.id === id);
            return o.id !== id && !o._deleted && !detached;
          });
          s.groups = s.groups.filter(function (g) { return g.id !== id && !g._deleted; });
          s.objects.forEach(function (o) { delete o._deleted; });
          s.groups.forEach(function (g) { delete g._deleted; });
          removed += before - (s.objects.length + s.groups.length);
        });
        OM.updateConnectors(s);
      });
      OM.clearSelection();
      return removed;
    },

    duplicateSelected: function () {
      var ids = targets();
      if (!ids.length) return [];
      var s = OM.slide();
      var copies = [];
      OM.mutate(L("slides-om-duplicate", "duplicate objects"), function () {
        ids.forEach(function (id) {
          var node = byId(s, id);
          if (!node) return;
          var copy = OM.clone(node);
          copy.id = OM.uid("om");
          copy.x += 16; copy.y += 16; copy.z = nextZ(s) + copies.length;
          if (copy.groupId) copy.groupId = null;
          if (copy.parentId) copy.parentId = null;
          copies.push(copy.id);
          if (copy.from || copy.to) { copy.from = null; copy.to = null; }
          if (node.type === undefined) s.groups.push(copy);
          else s.objects.push(copy);
        });
      });
      OM.setSelection(copies);
      return copies;
    },

    nudge: function (dx, dy) {
      var ids = targets();
      if (!ids.length) return false;
      var s = OM.slide();
      OM.mutate(L("slides-om-nudge", "nudge objects"), function () {
        OM.move(s, ids, dx, dy);
      });
      return true;
    },

    group: function (ids) {
      var list = targets(ids);
      if (list.length < 2) return null;
      var s = OM.slide();
      var id = OM.uid("grp");
      OM.mutate(L("slides-om-group", "group objects"), function () {
        var group = { id: id, parentId: null, children: list.slice(), z: nextZ(s),
          locked: false, hidden: false };
        list.forEach(function (nodeId) {
          var node = byId(s, nodeId);
          if (!node) return;
          if (node.type === undefined) node.parentId = id;
          else node.groupId = id;
        });
        s.groups.push(group);
      });
      OM.setSelection([id]);
      return id;
    },

    ungroup: function (ids) {
      var list = targets(ids);
      var s = OM.slide();
      var released = [];
      OM.mutate(L("slides-om-ungroup", "ungroup objects"), function () {
        list.forEach(function (gid) {
          var group = (s.groups || []).find(function (g) { return g.id === gid; });
          if (!group) return;
          released = released.concat(group.children || []);
          (s.objects || []).forEach(function (o) {
            if (o.groupId === gid) o.groupId = group.parentId || null;
          });
          (s.groups || []).forEach(function (g) {
            if (g.parentId === gid) g.parentId = group.parentId || null;
          });
          s.groups = s.groups.filter(function (g) { return g.id !== gid; });
        });
      });
      if (released.length) OM.setSelection(released);
      return released.length;
    },

    zOrder: function (mode, ids) {
      var list = targets(ids);
      if (!list.length) return false;
      var s = OM.slide();
      OM.mutate(L("slides-om-zorder", "change layer order"), function () {
        var nodes = OM.topLevel(OM.slideIndex()).slice();
        var picked = nodes.filter(function (n) { return list.indexOf(n.id) >= 0; });
        var rest = nodes.filter(function (n) { return list.indexOf(n.id) < 0; });
        var ordered;
        if (mode === "front") ordered = rest.concat(picked);
        else if (mode === "back") ordered = picked.concat(rest);
        else if (mode === "forward") {
          ordered = rest.slice();
          picked.forEach(function (n) {
            var last = -1;
            for (var i = 0; i < ordered.length; i++) {
              if (ordered[i].z <= n.z) last = i;
            }
            ordered.splice(last + 1, 0, n);
          });
        } else {
          ordered = rest.slice();
          picked.slice().reverse().forEach(function (n) {
            var at = ordered.findIndex(function (o) { return o.z > n.z; });
            ordered.splice(at < 0 ? 0 : at, 0, n);
          });
        }
        ordered.forEach(function (n, index) { n.z = index + 1; });
      });
      return true;
    },

    lockHide: function (property, value, ids) {
      var list = targets(ids);
      if (!list.length) return false;
      var s = OM.slide();
      OM.mutate(L("slides-om-lock", "change object visibility"), function () {
        list.forEach(function (id) {
          var node = byId(s, id);
          if (!node) return;
          node[property] = !!value;
          if ((s.groups || []).some(function (g) { return g.id === id; })) {
            OM.descendants(s, id).forEach(function (child) { child[property] = !!value; });
          }
        });
      });
      return true;
    },

    align: function (mode, ids) {
      var list = targets(ids);
      if (!list.length) return false;
      var s = OM.slide();
      var size = s.size;
      var reference = list.length === 1
        ? { x: 0, y: 0, w: OM.num(size.w, 960), h: OM.num(size.h, 540) }
        : OM.bounds(list, OM.slideIndex());
      if (!reference) return false;
      OM.mutate(L("slides-om-align", "align objects"), function () {
        list.forEach(function (id) {
          var r = OM.rect(s, id);
          if (!r) return;
          var dx = 0, dy = 0;
          if (mode === "left") dx = reference.x - r.x;
          else if (mode === "hcenter") dx = reference.x + (reference.w - r.w) / 2 - r.x;
          else if (mode === "right") dx = reference.x + reference.w - (r.x + r.w);
          else if (mode === "top") dy = reference.y - r.y;
          else if (mode === "vcenter") dy = reference.y + (reference.h - r.h) / 2 - r.y;
          else if (mode === "bottom") dy = reference.y + reference.h - (r.y + r.h);
          if (dx || dy) OM.move(s, [id], dx, dy);
        });
      });
      return true;
    },

    distribute: function (axis, ids) {
      var list = targets(ids);
      if (list.length < 3) return false;
      return this._spacing(axis, list, false);
    },

    spaceEvenly: function (axis, ids) {
      var list = targets(ids);
      if (list.length < 2) return false;
      return this._spacing(axis, list, true);
    },

    _spacing: function (axis, list, includeEdges) {
      var s = OM.slide();
      var size = s.size;
      var horizontal = axis === "h";
      var limit = horizontal ? OM.num(size.w, 960) : OM.num(size.h, 540);
      var sorted = list.map(function (id) { return { id: id, r: OM.rect(s, id) }; })
        .filter(function (n) { return n.r; })
        .sort(function (a, b) {
          return (horizontal ? a.r.x : a.r.y) - (horizontal ? b.r.x : b.r.y);
        });
      if (sorted.length < 2) return false;
      OM.mutate(L("slides-om-distribute", "distribute objects"), function () {
        var span = horizontal ? "w" : "h";
        var lead = horizontal ? "x" : "y";
        var total = sorted.reduce(function (acc, n) { return acc + n.r[span]; }, 0);
        var first = sorted[0].r[lead];
        var last = sorted[sorted.length - 1].r[lead] + sorted[sorted.length - 1].r[span];
        var available = includeEdges ? limit : (last - first);
        var gap = (available - total) / (sorted.length - 1);
        var cursor = includeEdges ? Math.max(0, (limit - total - gap * (sorted.length - 1)) / 2) : first;
        sorted.forEach(function (n) {
          var delta = cursor - n.r[lead];
          OM.move(s, [n.id], horizontal ? delta : 0, horizontal ? 0 : delta);
          cursor += n.r[span] + gap;
        });
      });
      return true;
    },

    equalSize: function (dimension, ids) {
      var list = targets(ids);
      if (!list.length) return false;
      var s = OM.slide();
      var maxW = 0, maxH = 0;
      list.forEach(function (id) {
        var r = OM.rect(s, id);
        if (r) { maxW = Math.max(maxW, r.w); maxH = Math.max(maxH, r.h); }
      });
      OM.mutate(L("slides-om-equal-size", "make objects same size"), function () {
        list.forEach(function (id) {
          var r = OM.rect(s, id);
          if (!r) return;
          var sx = dimension !== "h" && r.w > 0 ? maxW / r.w : 1;
          var sy = dimension !== "w" && r.h > 0 ? maxH / r.h : 1;
          scaleNode(s, id, sx, sy);
        });
        OM.updateConnectors(s);
      });
      return true;
    },

    addGuide: function (axis, at) {
      var s = OM.slide();
      var id = OM.uid("gd");
      OM.mutate(L("slides-om-guide-add", "add guide"), function () {
        s.guides.push({ id: id, axis: axis === "x" ? "x" : "y", at: OM.num(at, 0) });
      });
      return id;
    },

    removeGuide: function (id) {
      var s = OM.slide();
      OM.mutate(L("slides-om-guide-remove", "remove guide"), function () {
        s.guides = s.guides.filter(function (g) { return g.id !== id; });
      });
    },

    setting: function (key, value) {
      OM.mutate(L("slides-om-setting", "change snapping setting"), function () {
        OM.state.settings[key] = value;
      });
    }
  };

  function scaleNode(s, id, sx, sy) {
    var node = byId(s, id);
    if (!node) return;
    if (node.x === undefined) {
      OM.descendants(s, id).forEach(function (child) {
        if (child.x === undefined) return;
        var r = { x: child.x, y: child.y, w: child.w, h: child.h };
        child.x = r.x + (r.w - r.w * sx) / 2;
        child.y = r.y + (r.h - r.h * sy) / 2;
        child.w = r.w * sx; child.h = r.h * sy;
      });
      return;
    }
    node.w = Math.max(1, OM.num(node.w, 0) * sx);
    node.h = Math.max(1, OM.num(node.h, 0) * sy);
    if (Array.isArray(node.points)) {
      node.points.forEach(function (p) { p.x *= sx; p.y *= sy; });
    }
  }

  window.GBSlidesOMOps = Ops;
  window.GBSlidesOMOpsHelpers = {
    L: L, byId: byId, nextZ: nextZ, targets: targets, scaleNode: scaleNode
  };
}());
