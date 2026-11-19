"use strict";
/* Slides animation — motion path endpoint editor with guide snapping (#2431).
 * Loaded before 16_anim_timeline.js. Returns a DOM node; no visible literal
 * strings are authored here except through saT(). */

(function (window) {
  var A = window.SlidesAnimation;
  if (!A) return;
  var NS = "http://www.w3.org/2000/svg";
  var GUIDE = 0.25;

  function node(tag, cls) {
    var n = document.createElement(tag);
    if (cls) n.className = cls;
    return n;
  }
  function svgEl(tag, attrs) {
    var n = document.createElementNS(NS, tag);
    for (var k in attrs) if (Object.prototype.hasOwnProperty.call(attrs, k)) n.setAttribute(k, attrs[k]);
    return n;
  }
  function clamp(v) { return Math.max(0, Math.min(1, v)); }
  function snap(v, on) { return on ? Math.round(v / GUIDE) * GUIDE : Math.round(v * 100) / 100; }

  /* fx.path = { x1, y1, cx, cy, x2, y2 }, all normalised 0..1. */
  function editor(fx, onChange) {
    var wrap = node("div", "sl-anim-motion");
    var svg = svgEl("svg", { viewBox: "0 0 100 100", class: "sl-anim-motion-canvas", role: "img" });
    svg.setAttribute("aria-label", A.saT("slides-anim-cat-motion", "Motion path"));
    var i;
    for (i = 1; i <= 3; i++) {
      svg.appendChild(svgEl("line", {
        x1: i * 25, y1: 0, x2: i * 25, y2: 100, stroke: "#334155", "stroke-width": "0.5"
      }));
      svg.appendChild(svgEl("line", {
        x1: 0, y1: i * 25, x2: 100, y2: i * 25, stroke: "#334155", "stroke-width": "0.5"
      }));
    }
    var pathLine = svgEl("path", { fill: "none", stroke: "#60a5fa", "stroke-width": "1.5" });
    svg.appendChild(pathLine);
    var points = fx.path || { x1: 0, y1: 0, cx: 0.5, cy: 0, x2: 1, y2: 0 };
    var keys = ["start", "control", "end"];
    var dots = [];
    for (i = 0; i < keys.length; i++) {
      (function (key) {
        var c = svgEl("circle", { r: "3.5", fill: "#3b82f6", stroke: "#f8fafc", "stroke-width": "0.8", tabindex: "0" });
        c.setAttribute("aria-label", key === "start" ? A.saT("slides-anim-motion-start", "Path start")
          : key === "end" ? A.saT("slides-anim-motion-end", "Path end")
          : A.saT("slides-anim-cat-motion", "Motion path"));
        c.dataset.point = key;
        svg.appendChild(c);
        dots.push(c);
      })(keys[i]);
    }
    var snapBox = node("label", "sl-anim-check sl-anim-snap-line");
    var snapInput = node("input");
    snapInput.type = "checkbox";
    snapInput.checked = true;
    snapBox.appendChild(snapInput);
    snapBox.appendChild(node("span"));
    snapBox.lastChild.textContent = A.saT("slides-anim-snap", "Snap to guides");
    wrap.appendChild(svg);
    wrap.appendChild(snapBox);

    function coords(key) {
      if (key === "start") return [points.x1, points.y1];
      if (key === "control") return [points.cx, points.cy];
      return [points.x2, points.y2];
    }
    function setCoords(key, x, y) {
      if (key === "start") { points.x1 = x; points.y1 = y; }
      else if (key === "control") { points.cx = x; points.cy = y; }
      else { points.x2 = x; points.y2 = y; }
    }
    function draw() {
      for (var i = 0; i < dots.length; i++) {
        var xy = coords(dots[i].dataset.point);
        dots[i].setAttribute("cx", String(clamp(xy[0]) * 100));
        dots[i].setAttribute("cy", String(clamp(xy[1]) * 100));
      }
      var a = coords("start"), b = coords("control"), c = coords("end");
      pathLine.setAttribute("d", "M " + a[0] * 100 + " " + a[1] * 100 +
        " Q " + b[0] * 100 + " " + b[1] * 100 + " " + c[0] * 100 + " " + c[1] * 100);
    }
    function fromEvent(ev) {
      var rect = svg.getBoundingClientRect();
      if (!rect.width || !rect.height) return null;
      return [snap(clamp((ev.clientX - rect.left) / rect.width), snapInput.checked),
        snap(clamp((ev.clientY - rect.top) / rect.height), snapInput.checked)];
    }
    function bindDot(dot) {
      dot.addEventListener("pointerdown", function (ev) {
        var key = dot.dataset.point;
        dot.setPointerCapture && dot.setPointerCapture(ev.pointerId);
        function move(e) {
          var xy = fromEvent(e);
          if (!xy) return;
          setCoords(key, xy[0], xy[1]);
          draw();
          if (typeof onChange === "function") onChange(points);
        }
        function up() {
          dot.removeEventListener("pointermove", move);
          dot.removeEventListener("pointerup", up);
        }
        dot.addEventListener("pointermove", move);
        dot.addEventListener("pointerup", up);
      });
      dot.addEventListener("keydown", function (ev) {
        var step = ev.shiftKey ? GUIDE : 0.05;
        var xy = coords(dot.dataset.point);
        if (ev.key === "ArrowLeft") xy[0] = clamp(xy[0] - step);
        else if (ev.key === "ArrowRight") xy[0] = clamp(xy[0] + step);
        else if (ev.key === "ArrowUp") xy[1] = clamp(xy[1] - step);
        else if (ev.key === "ArrowDown") xy[1] = clamp(xy[1] + step);
        else return;
        ev.preventDefault();
        setCoords(dot.dataset.point, snap(xy[0], snapInput.checked), snap(xy[1], snapInput.checked));
        draw();
        if (typeof onChange === "function") onChange(points);
      });
    }
    for (i = 0; i < dots.length; i++) bindDot(dots[i]);
    snapInput.addEventListener("change", function () {
      if (!snapInput.checked) return;
      var keys2 = ["start", "control", "end"];
      for (var j = 0; j < keys2.length; j++) {
        var xy = coords(keys2[j]);
        setCoords(keys2[j], snap(xy[0], true), snap(xy[1], true));
      }
      draw();
      if (typeof onChange === "function") onChange(points);
    });
    draw();
    wrap._points = points;
    return wrap;
  }

  window.SlidesAnimationMotion = { editor: editor, GUIDE: GUIDE };
})(window);
