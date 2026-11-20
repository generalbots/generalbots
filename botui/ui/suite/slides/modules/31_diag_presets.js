"use strict";
/* Slides diagram authoring — deterministic preset auto-layout (#2488).
 * Pure geometry: every reflow is reproducible from nodes + preset id. */

var SD_PRESETS = [
  { id: "process", key: "slides-diagrams-preset-process", fallback: "Process", count: 5 },
  { id: "cycle", key: "slides-diagrams-preset-cycle", fallback: "Cycle", count: 6 },
  { id: "hierarchy", key: "slides-diagrams-preset-hierarchy", fallback: "Hierarchy", count: 7 },
  { id: "matrix", key: "slides-diagrams-preset-matrix", fallback: "Matrix", count: 9 },
  { id: "pyramid", key: "slides-diagrams-preset-pyramid", fallback: "Pyramid", count: 6 },
  { id: "list", key: "slides-diagrams-preset-list", fallback: "List", count: 5 },
  { id: "org", key: "slides-diagrams-preset-org", fallback: "Org chart", count: 6 }
];

var SDPresets = {
  NODE_W: 150,
  NODE_H: 68,
  GAP_X: 34,
  GAP_Y: 46,
  ORIGIN_X: 40,
  ORIGIN_Y: 56,

  def: function (id) {
    for (var i = 0; i < SD_PRESETS.length; i++) {
      if (SD_PRESETS[i].id === id) return SD_PRESETS[i];
    }
    return SD_PRESETS[0];
  },

  name: function (id) {
    var def = this.def(id);
    return sdT(def.key, def.fallback);
  },

  newNode: function (index, level, parentId) {
    return {
      id: sdUid("n"),
      text: sdT("slides-diagrams-node-default", "Node") + " " + (index + 1),
      icon: "",
      color: "",
      level: level || 0,
      parentId: parentId || null,
      x: 0,
      y: 0,
      w: this.NODE_W,
      h: this.NODE_H
    };
  },

  byId: function (diagram, id) {
    var found = null;
    (diagram.nodes || []).forEach(function (node) {
      if (node.id === id) found = node;
    });
    return found;
  },

  indexOf: function (diagram, id) {
    var at = -1;
    (diagram.nodes || []).forEach(function (node, i) {
      if (node.id === id) at = i;
    });
    return at;
  },

  connector: function (from, to, fromPort, toPort) {
    return {
      id: sdUid("c"),
      from: from,
      to: to,
      fromPort: fromPort,
      toPort: toPort,
      path: []
    };
  },

  chain: function (nodes, fromPort, toPort) {
    var out = [];
    for (var i = 0; i + 1 < nodes.length; i++) {
      out.push(this.connector(nodes[i].id, nodes[i + 1].id, fromPort, toPort));
    }
    return out;
  },

  layoutProcess: function (nodes) {
    var self = this;
    nodes.forEach(function (node, i) {
      node.x = self.ORIGIN_X + i * (self.NODE_W + self.GAP_X);
      node.y = self.ORIGIN_Y;
      node.w = self.NODE_W;
      node.h = self.NODE_H;
      node.level = 0;
    });
    return this.chain(nodes, "e", "w");
  },

  layoutCycle: function (nodes) {
    var total = nodes.length || 1;
    var width = (typeof SLIDE_W === "number" ? SLIDE_W : 960) - 2 * this.ORIGIN_X;
    var cx = this.ORIGIN_X + width / 2;
    var cy = this.ORIGIN_Y + 180;
    var rx = Math.max(120, width / 2 - this.NODE_W / 2);
    var ry = 150;
    nodes.forEach(function (node, i) {
      var angle = -Math.PI / 2 + (2 * Math.PI * i) / total;
      node.x = Math.round(cx + rx * Math.cos(angle) - node.w / 2);
      node.y = Math.round(cy + ry * Math.sin(angle) - node.h / 2);
      node.level = 0;
    });
    return this.chain(nodes, "", "");
  },

  layoutTree: function (nodes) {
    var self = this;
    var map = {};
    nodes.forEach(function (node) { map[node.id] = node; });
    var children = {};
    nodes.forEach(function (node) {
      var parent = node.parentId && map[node.parentId] ? node.parentId : "";
      (children[parent] = children[parent] || []).push(node);
    });
    var cursor = 0;
    var step = this.NODE_W + this.GAP_X;
    function place(id, level) {
      var kids = children[id] || [];
      var centre;
      if (!kids.length) {
        var leaf = map[id];
        if (leaf) {
          leaf.x = self.ORIGIN_X + cursor * step;
          leaf.y = self.ORIGIN_Y + level * (self.NODE_H + self.GAP_Y);
          leaf.level = level;
          centre = leaf.x + leaf.w / 2;
        } else {
          centre = self.ORIGIN_X + cursor * step + self.NODE_W / 2;
        }
        cursor++;
        return centre;
      }
      var centres = kids.map(function (kid) { return place(kid.id, level + 1); });
      centre = (centres[0] + centres[centres.length - 1]) / 2;
      var parent = map[id];
      if (parent) {
        parent.x = centre - parent.w / 2;
        parent.y = self.ORIGIN_Y + level * (self.NODE_H + self.GAP_Y);
        parent.level = level;
      }
      return centre;
    }
    (children[""] || []).forEach(function (root) { place(root.id, 0); });
    var out = [];
    nodes.forEach(function (node) {
      if (node.parentId && map[node.parentId]) {
        out.push(self.connector(node.parentId, node.id, "s", "n"));
      }
    });
    return out;
  },

  layoutMatrix: function (nodes) {
    var self = this;
    var cols = Math.max(1, Math.ceil(Math.sqrt(nodes.length || 1)));
    nodes.forEach(function (node, i) {
      node.x = self.ORIGIN_X + (i % cols) * (self.NODE_W + self.GAP_X);
      node.y = self.ORIGIN_Y + Math.floor(i / cols) * (self.NODE_H + self.GAP_Y);
      node.level = 0;
    });
    return [];
  },

  layoutPyramid: function (nodes) {
    var width = (typeof SLIDE_W === "number" ? SLIDE_W : 960) - 2 * this.ORIGIN_X;
    var index = 0;
    var row = 0;
    while (index < nodes.length) {
      var size = Math.min(row + 1, nodes.length - index);
      var rowWidth = size * (this.NODE_W + this.GAP_X) - this.GAP_X;
      var startX = this.ORIGIN_X + Math.max(0, (width - rowWidth) / 2);
      for (var col = 0; col < size; col++) {
        var node = nodes[index++];
        node.x = startX + col * (this.NODE_W + this.GAP_X);
        node.y = this.ORIGIN_Y + row * (this.NODE_H + this.GAP_Y);
        node.level = row;
      }
      row++;
    }
    return [];
  },

  layoutList: function (nodes) {
    var self = this;
    nodes.forEach(function (node, i) {
      node.x = self.ORIGIN_X;
      node.y = self.ORIGIN_Y + i * (self.NODE_H + self.GAP_Y);
      node.level = i;
    });
    return [];
  },

  layout: function (kind, nodes) {
    switch (kind) {
      case "cycle": return this.layoutCycle(nodes);
      case "hierarchy":
      case "org": return this.layoutTree(nodes);
      case "matrix": return this.layoutMatrix(nodes);
      case "pyramid": return this.layoutPyramid(nodes);
      case "list": return this.layoutList(nodes);
      default: return this.layoutProcess(nodes);
    }
  },

  reflow: function (diagram) {
    if (!diagram || !Array.isArray(diagram.nodes)) return diagram;
    diagram.connectors = this.layout(diagram.preset, diagram.nodes);
    if (typeof SDFlow !== "undefined" && SDFlow && typeof SDFlow.recompute === "function") {
      SDFlow.recompute(diagram);
    }
    return diagram;
  },

  create: function (kind, count) {
    var def = this.def(kind);
    var total = Math.max(1, Math.min(30, count || def.count || 5));
    var nodes = [];
    for (var i = 0; i < total; i++) {
      var parentId = null;
      if ((kind === "hierarchy" || kind === "org") && i > 0) {
        parentId = nodes[Math.floor((i - 1) / 2)].id;
      }
      nodes.push(this.newNode(i, 0, parentId));
    }
    var diagram = {
      id: sdUid("dg"),
      slideIndex: GBSlidesDiagrams.slideIndex(),
      preset: def.id,
      flowMode: def.id === "process" || def.id === "cycle",
      grouped: false,
      origin: { x: 0, y: 0 },
      nodes: nodes,
      connectors: []
    };
    return this.reflow(diagram);
  },

  addNode: function (diagram) {
    if (!diagram || !Array.isArray(diagram.nodes)) return null;
    var index = diagram.nodes.length;
    var parentId = null;
    if ((diagram.preset === "hierarchy" || diagram.preset === "org") && index > 0) {
      parentId = diagram.nodes[Math.floor((index - 1) / 2)].id;
    }
    var node = this.newNode(index, 0, parentId);
    diagram.nodes.push(node);
    this.reflow(diagram);
    return node;
  },

  removeNode: function (diagram, id) {
    if (!diagram || !Array.isArray(diagram.nodes)) return false;
    var at = this.indexOf(diagram, id);
    if (at < 0) return false;
    diagram.nodes.splice(at, 1);
    diagram.nodes.forEach(function (node) {
      if (node.parentId === id) node.parentId = null;
    });
    this.reflow(diagram);
    return true;
  },

  promote: function (diagram, id) {
    var at = this.indexOf(diagram, id);
    if (at <= 0) return false;
    var node = diagram.nodes[at];
    if (diagram.preset === "hierarchy" || diagram.preset === "org") {
      var parent = this.byId(diagram, node.parentId);
      if (parent) node.parentId = parent.parentId || null;
      node.level = Math.max(0, (node.level || 0) - 1);
    } else {
      diagram.nodes.splice(at, 1);
      diagram.nodes.splice(at - 1, 0, node);
    }
    this.reflow(diagram);
    return true;
  },

  demote: function (diagram, id) {
    var at = this.indexOf(diagram, id);
    if (at < 0 || at >= diagram.nodes.length - 1) return false;
    var node = diagram.nodes[at];
    if (diagram.preset === "hierarchy" || diagram.preset === "org") {
      var siblings = diagram.nodes.filter(function (item) {
        return item.id !== id && item.parentId === node.parentId;
      });
      if (siblings.length) node.parentId = siblings[siblings.length - 1].id;
      node.level = (node.level || 0) + 1;
    } else {
      diagram.nodes.splice(at, 1);
      diagram.nodes.splice(at + 1, 0, node);
    }
    this.reflow(diagram);
    return true;
  }
};
