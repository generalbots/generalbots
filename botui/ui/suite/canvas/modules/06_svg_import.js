(function () {
    "use strict";

    // #2155 — convert mappable SVG into editable canvas elements for flowcharts.
    // Falls back to the original SVG as background when fidelity would be lost.
    function importAsElements(svgText, opts) {
        opts = opts || {};
        var app = opts.app || window._canvasApp;
        var bg = opts.bg || document.getElementById("canvas-bg-svg");
        var setStatus = opts.setStatus || function () {};
        var doc;
        try { doc = new DOMParser().parseFromString(svgText, "image/svg+xml"); } catch (e) { doc = null; }
        var root = doc && doc.documentElement;
        if (!root || root.nodeName.toLowerCase() !== "svg" || doc.querySelector("parsererror")) {
            setStatus("Could not read SVG");
            return;
        }
        function keepBackground(reason) {
            if (bg) bg.innerHTML = svgText;
            if (app && typeof app.setCanvasBackgroundHidden === "function") app.setCanvasBackgroundHidden();
            setStatus("SVG kept as background — " + reason);
        }
        var sansMarkers = svgText.replace(/marker-(?:start|mid|end)\s*=\s*"url\([^"]*\)"/gi, "");
        sansMarkers = sansMarkers.replace(/marker-(?:start|mid|end)\s*=\s*'url\([^']*\)'/gi, "");
        if (/url\s*\(/i.test(sansMarkers)) { keepBackground("uses gradients/filters"); return; }
        if (root.querySelector("[transform]")) { keepBackground("uses transforms"); return; }

        var MAPPABLE = { rect: 1, circle: 1, ellipse: 1, line: 1, text: 1, polyline: 1, polygon: 1, path: 1 };
        var STRUCTURAL = { svg: 1, g: 1, title: 1, desc: 1, defs: 1, style: 1, metadata: 1, switch: 1, marker: 1 };
        var num = function (v, d) { var n = parseFloat(v); return isNaN(n) ? (d || 0) : n; };
        var strokeOf = function (n) { var v = n.getAttribute("stroke"); return v && v !== "none" ? v : "#000000"; };
        var fillOf = function (n) { var v = n.getAttribute("fill"); return v && v !== "none" ? v : "transparent"; };
        var swOf = function (n) { return num(n.getAttribute("stroke-width"), 2); };
        var parsePoints = function (raw) {
            return (raw || "").trim().split(/[\s,]+/).reduce(function (acc, tok, i, arr) {
                if (i % 2 === 0 && arr[i + 1] !== undefined) acc.push({ x: num(tok), y: num(arr[i + 1]) });
                return acc;
            }, []);
        };
        var bboxOfPoints = function (ps) {
            if (!ps.length) return { x: 0, y: 0, width: 0, height: 0 };
            var xs = ps.map(function (p) { return p.x; }), ys = ps.map(function (p) { return p.y; });
            var minX = Math.min.apply(Math, xs), maxX = Math.max.apply(Math, xs);
            var minY = Math.min.apply(Math, ys), maxY = Math.max.apply(Math, ys);
            return { x: minX, y: minY, width: maxX - minX, height: maxY - minY };
        };
        function parseSimplePath(d) {
            var t = String(d || "").replace(/,/g, " ").trim();
            var re = /([MLHVZmlhvz])|([-+]?(?:\d*\.\d+|\d+)(?:e[-+]?\d+)?)/g;
            var tokens = [], m;
            while ((m = re.exec(t)) !== null) tokens.push(m[1] || parseFloat(m[2]));
            var i = 0, cmd = "", x = 0, y = 0, sx = 0, sy = 0, segs = [], seg = null;
            function flush() { if (seg && seg.length > 1) segs.push(seg); seg = null; }
            while (i < tokens.length) {
                var tok = tokens[i++];
                if (typeof tok === "string") { cmd = tok; if (cmd === "Z" || cmd === "z") { if (seg) seg.push({ x: sx, y: sy }); flush(); cmd = ""; } continue; }
                i--;
                if (cmd === "M" || cmd === "m") {
                    flush();
                    x = (cmd === "m" ? x : 0) + tokens[i++]; y = (cmd === "m" ? y : 0) + tokens[i++];
                    sx = x; sy = y; seg = [{ x: x, y: y }]; cmd = (cmd === "M" ? "L" : "l");
                } else if (cmd === "L" || cmd === "l") {
                    x = (cmd === "l" ? x : 0) + tokens[i++]; y = (cmd === "l" ? y : 0) + tokens[i++];
                    if (!seg) seg = [{ x: x, y: y }]; else seg.push({ x: x, y: y });
                } else if (cmd === "H" || cmd === "h") {
                    x = (cmd === "h" ? x : 0) + tokens[i++]; if (!seg) seg = [{ x: x, y: y }]; else seg.push({ x: x, y: y });
                } else if (cmd === "V" || cmd === "v") {
                    y = (cmd === "v" ? y : 0) + tokens[i++]; if (!seg) seg = [{ x: x, y: y }]; else seg.push({ x: x, y: y });
                } else { return null; }
            }
            flush();
            return segs.length ? segs : null;
        }

        var unsupported = 0;
        root.querySelectorAll("*").forEach(function (n) {
            var name = n.nodeName.toLowerCase();
            if (STRUCTURAL[name]) return;
            if (!MAPPABLE[name]) { unsupported++; return; }
            if (name === "path" && !parseSimplePath(n.getAttribute("d"))) unsupported++;
        });
        if (unsupported > 0) { keepBackground(unsupported + " unsupported element(s)"); return; }

        var seed = 0, out = [];
        var add = function (type, base) { base.id = "svg" + (++seed) + "_" + Date.now(); base.type = type; out.push(base); };
        root.querySelectorAll("rect").forEach(function (n) {
            add("rectangle", { x: num(n.getAttribute("x")), y: num(n.getAttribute("y")),
                width: num(n.getAttribute("width")), height: num(n.getAttribute("height")),
                stroke: strokeOf(n), fill: fillOf(n), strokeWidth: swOf(n) });
        });
        root.querySelectorAll("circle").forEach(function (n) {
            var r = num(n.getAttribute("r"));
            add("ellipse", { x: num(n.getAttribute("cx")) - r, y: num(n.getAttribute("cy")) - r,
                width: 2 * r, height: 2 * r, stroke: strokeOf(n), fill: fillOf(n), strokeWidth: swOf(n) });
        });
        root.querySelectorAll("ellipse").forEach(function (n) {
            var rx = num(n.getAttribute("rx")), ry = num(n.getAttribute("ry"));
            add("ellipse", { x: num(n.getAttribute("cx")) - rx, y: num(n.getAttribute("cy")) - ry,
                width: 2 * rx, height: 2 * ry, stroke: strokeOf(n), fill: fillOf(n), strokeWidth: swOf(n) });
        });
        root.querySelectorAll("line").forEach(function (n) {
            var x1 = num(n.getAttribute("x1")), y1 = num(n.getAttribute("y1"));
            var x2 = num(n.getAttribute("x2")), y2 = num(n.getAttribute("y2"));
            add("line", { x1: x1, y1: y1, x2: x2, y2: y2, x: Math.min(x1, x2), y: Math.min(y1, y2),
                width: Math.abs(x2 - x1), height: Math.abs(y2 - y1), stroke: strokeOf(n), strokeWidth: swOf(n) });
        });
        root.querySelectorAll("text").forEach(function (n) {
            var fs = num(n.getAttribute("font-size"), 18), fill = n.getAttribute("fill");
            var content = (n.textContent || "").trim();
            add("text", { x: num(n.getAttribute("x")), y: num(n.getAttribute("y")) - fs, fontSize: fs, text: content,
                width: content.length * fs * 0.6, height: fs * 1.4, stroke: fill && fill !== "none" ? fill : "#000000" });
        });
        root.querySelectorAll("polyline, polygon").forEach(function (n) {
            var ps = parsePoints(n.getAttribute("points"));
            if (n.nodeName.toLowerCase() === "polygon" && ps.length > 2) {
                var a = ps[0], b = ps[ps.length - 1];
                if (a.x !== b.x || a.y !== b.y) ps = ps.concat([{ x: a.x, y: a.y }]);
            }
            add("pencil", Object.assign({ points: ps, stroke: strokeOf(n), strokeWidth: swOf(n) }, bboxOfPoints(ps)));
        });
        root.querySelectorAll("path").forEach(function (n) {
            (parseSimplePath(n.getAttribute("d")) || []).forEach(function (ps) {
                add("pencil", Object.assign({ points: ps, stroke: strokeOf(n), strokeWidth: swOf(n) }, bboxOfPoints(ps)));
            });
        });
        if (!out.length) { keepBackground("no mappable shapes"); return; }
        if (app && typeof app.setCanvasElements === "function") app.setCanvasElements(out);
        if (bg) bg.innerHTML = "";
        setStatus("Imported " + out.length + " SVG shape(s) — editable");
    }

    window.GBCanvasSvgImport = { importAsElements: importAsElements };
})();