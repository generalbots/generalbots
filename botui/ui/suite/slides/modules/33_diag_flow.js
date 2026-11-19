"use strict";
/* Slides diagram authoring — shape ports and orthogonal connectors (#2488).
 * A connector stores endpoint node ids + port names, never absolute points:
 * recompute() re-routes every path from the current node rectangles, so boxes
 * can move without arrows detaching. */

var SDFlow = {
  PORTS: ["n", "e", "s", "w"],

  portPoint: function (node, port) {
    var x = node.x || 0;
    var y = node.y || 0;
    var w = node.w || 150;
    var h = node.h || 68;
    if (port === "n") return { x: x + w / 2, y: y };
    if (port === "e") return { x: x + w, y: y + h / 2 };
    if (port === "s") return { x: x + w / 2, y: y + h };
    return { x: x, y: y + h / 2 };
  },

  orthogonal: function (from, to, fromPort, toPort) {
    var a = this.portPoint(from, fromPort);
    var b = this.portPoint(to, toPort);
    var horizontal = (fromPort === "e" || fromPort === "w") && (toPort === "e" || toPort === "w");
    var vertical = (fromPort === "n" || fromPort === "s") && (toPort === "n" || toPort === "s");
    if (horizontal) {
      var mx = (a.x + b.x) / 2;
      return [[a.x, a.y], [mx, a.y], [mx, b.y], [b.x, b.y]];
    }
    if (vertical) {
      var my = (a.y + b.y) / 2;
      return [[a.x, a.y], [a.x, my], [b.x, my], [b.x, b.y]];
    }
    return [[a.x, a.y], [b.x, a.y], [b.x, b.y]];
  },

  autoPorts: function (from, to) {
    var ax = (from.x || 0) + (from.w || 150) / 2;
    var ay = (from.y || 0) + (from.h || 68) / 2;
    var bx = (to.x || 0) + (to.w || 150) / 2;
    var by = (to.y || 0) + (to.h || 68) / 2;
    if (Math.abs(bx - ax) >= Math.abs(by - ay)) {
      return bx >= ax ? ["e", "w"] : ["w", "e"];
    }
    return by >= ay ? ["s", "n"] : ["n", "s"];
  },

  recompute: function (diagram) {
    if (!diagram || !Array.isArray(diagram.nodes) || !Array.isArray(diagram.connectors)) return;
    var map = {};
    diagram.nodes.forEach(function (node) { map[node.id] = node; });
    diagram.connectors.forEach(function (connector) {
      var from = map[connector.from];
      var to = map[connector.to];
      if (!from || !to) {
        connector.path = [];
        return;
      }
      var ports = SDFlow.autoPorts(from, to);
      var fromPort = connector.fromPort || ports[0];
      var toPort = connector.toPort || ports[1];
      connector.path = SDFlow.orthogonal(from, to, fromPort, toPort);
    });
  },

  pathD: function (connector) {
    var points = (connector && connector.path) || [];
    if (!points.length) return "";
    return "M " + points.map(function (point) {
      return point[0] + " " + point[1];
    }).join(" L ");
  },

  updateSvg: function (diagram, svg) {
    if (!svg || !diagram || !Array.isArray(diagram.connectors)) return;
    var paths = sd$$("path[data-connector-id]", svg);
    paths.forEach(function (element) {
      var id = element.getAttribute("data-connector-id");
      var connector = null;
      diagram.connectors.forEach(function (candidate) {
        if (candidate.id === id) connector = candidate;
      });
      if (connector) element.setAttribute("d", SDFlow.pathD(connector));
    });
  },

  buildSvg: function (diagram) {
    var ns = "http://www.w3.org/2000/svg";
    var svg = document.createElementNS(ns, "svg");
    svg.setAttribute("class", "sl-diag-connectors");
    svg.setAttribute("viewBox", "0 0 " + (typeof SLIDE_W === "number" ? SLIDE_W : 960) + " " +
      (typeof SLIDE_H === "number" ? SLIDE_H : 540));
    svg.setAttribute("preserveAspectRatio", "none");
    svg.setAttribute("aria-hidden", "true");
    (diagram.connectors || []).forEach(function (connector) {
      var path = document.createElementNS(ns, "path");
      path.setAttribute("class", "sl-diag-connector");
      path.setAttribute("data-connector-id", connector.id);
      path.setAttribute("d", SDFlow.pathD(connector));
      path.setAttribute("fill", "none");
      path.setAttribute("stroke", "currentColor");
      path.setAttribute("stroke-width", "2");
      svg.appendChild(path);
      var arrow = document.createElementNS(ns, "circle");
      arrow.setAttribute("class", "sl-diag-connector-tip");
      arrow.setAttribute("data-tip-id", connector.id);
      var points = connector.path || [];
      if (points.length) {
        arrow.setAttribute("cx", points[points.length - 1][0]);
        arrow.setAttribute("cy", points[points.length - 1][1]);
      }
      arrow.setAttribute("r", "3");
      svg.appendChild(arrow);
    });
    return svg;
  },

  refreshTips: function (diagram, svg) {
    if (!svg) return;
    (diagram.connectors || []).forEach(function (connector) {
      var points = connector.path || [];
      if (!points.length) return;
      var tip = sd$('circle[data-tip-id="' + connector.id + '"]', svg);
      if (!tip) return;
      tip.setAttribute("cx", points[points.length - 1][0]);
      tip.setAttribute("cy", points[points.length - 1][1]);
    });
  }
};
