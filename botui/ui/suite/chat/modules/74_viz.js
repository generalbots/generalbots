/**
 * Declarative visuals (#1673): flow, timeline, metrics, bars and lines.
 *
 * Every visual is data rendered by trusted code. SVG is built with
 * createElementNS (never innerHTML), each figure has an accessible name and a
 * textual summary, and motion follows prefers-reduced-motion.
 */
(function () {
    "use strict";

    var NS = "http://www.w3.org/2000/svg";
    var reducedMotion = window.matchMedia
        ? window.matchMedia("(prefers-reduced-motion: reduce)")
        : { matches: false };

    function svg(tag, attrs) {
        var node = document.createElementNS(NS, tag);
        Object.keys(attrs || {}).forEach(function (key) {
            node.setAttribute(key, String(attrs[key]));
        });
        return node;
    }

    function el(tag, className, text) {
        var node = document.createElement(tag);
        if (className) node.className = className;
        if (text !== undefined) node.textContent = String(text);
        return node;
    }

    function figure(component, summary) {
        var wrap = el("figure", "gb-viz");
        if (component.title) {
            var caption = el("figcaption", "gb-viz-title", component.title);
            wrap.appendChild(caption);
        }
        wrap.setAttribute("role", "group");
        wrap.setAttribute("aria-label", summary);
        wrap.appendChild(el("p", "gb-viz-summary", summary));
        return wrap;
    }

    function normalizeSeries(component) {
        var labels = Array.isArray(component.labels) ? component.labels : [];
        var series = Array.isArray(component.series) ? component.series : [];
        if (!series.length && Array.isArray(component.values)) {
            series = [{ name: component.title || "value", values: component.values }];
        }
        return { labels: labels, series: series };
    }

    function renderBars(component) {
        var data = normalizeSeries(component);
        var summary = (component.title || "Bar chart") + ": " + data.series.map(function (series) {
            return (series.name || "series") + " " + (series.values || []).join(", ");
        }).join("; ");
        var wrap = figure(component, summary);
        var width = 520;
        var height = 220;
        var margin = { top: 16, right: 10, bottom: 34, left: 36 };
        var chart = svg("svg", {
            viewBox: "0 0 " + width + " " + height,
            width: "100%",
            height: height,
            role: "img",
            "aria-label": summary,
        });
        var values = [];
        data.series.forEach(function (series) {
            (series.values || []).forEach(function (value) {
                values.push(Number(value) || 0);
            });
        });
        var max = Math.max.apply(null, values.concat([1]));
        var plotW = width - margin.left - margin.right;
        var plotH = height - margin.top - margin.bottom;
        var groups = Math.max(1, data.labels.length);
        var groupW = plotW / groups;
        var barW = Math.max(2, (groupW * 0.7) / Math.max(1, data.series.length));
        data.series.forEach(function (series, seriesIndex) {
            (series.values || []).forEach(function (value, index) {
                var v = Number(value) || 0;
                var barH = (v / max) * plotH;
                var x = margin.left + index * groupW + groupW * 0.15 + seriesIndex * barW;
                var rect = svg("rect", {
                    x: x,
                    y: margin.top + plotH - barH,
                    width: barW,
                    height: Math.max(0, barH),
                    class: "gb-viz-bar series-" + seriesIndex,
                });
                rect.appendChild(svg("title")).textContent =
                    (data.labels[index] || index) + ": " + v;
                chart.appendChild(rect);
            });
        });
        data.labels.forEach(function (label, index) {
            var text = svg("text", {
                x: margin.left + index * groupW + groupW / 2,
                y: height - 12,
                "text-anchor": "middle",
                class: "gb-viz-axis-label",
            });
            text.textContent = label;
            chart.appendChild(text);
        });
        wrap.appendChild(chart);
        return wrap;
    }

    function renderLines(component) {
        var data = normalizeSeries(component);
        var summary = (component.title || "Line chart") + ": " + data.series.map(function (series) {
            return (series.name || "series") + " " + (series.values || []).join(", ");
        }).join("; ");
        var wrap = figure(component, summary);
        var width = 520;
        var height = 220;
        var margin = { top: 16, right: 12, bottom: 34, left: 36 };
        var plotW = width - margin.left - margin.right;
        var plotH = height - margin.top - margin.bottom;
        var all = [];
        data.series.forEach(function (series) {
            (series.values || []).forEach(function (value) { all.push(Number(value) || 0); });
        });
        var max = Math.max.apply(null, all.concat([1]));
        var chart = svg("svg", {
            viewBox: "0 0 " + width + " " + height,
            width: "100%",
            height: height,
            role: "img",
            "aria-label": summary,
        });
        data.series.forEach(function (series, seriesIndex) {
            var values = series.values || [];
            var step = values.length > 1 ? plotW / (values.length - 1) : plotW;
            var points = values.map(function (value, index) {
                var x = margin.left + index * step;
                var y = margin.top + plotH - ((Number(value) || 0) / max) * plotH;
                return x.toFixed(1) + "," + y.toFixed(1);
            }).join(" ");
            chart.appendChild(svg("polyline", {
                points: points,
                fill: "none",
                class: "gb-viz-line series-" + seriesIndex,
            }));
        });
        data.labels.forEach(function (label, index) {
            var step = data.labels.length > 1 ? plotW / (data.labels.length - 1) : 0;
            var text = svg("text", {
                x: margin.left + index * step,
                y: height - 12,
                "text-anchor": "middle",
                class: "gb-viz-axis-label",
            });
            text.textContent = label;
            chart.appendChild(text);
        });
        wrap.appendChild(chart);
        return wrap;
    }

    function renderFlow(component) {
        var nodes = Array.isArray(component.nodes) ? component.nodes : [];
        var summary = (component.title || "Flow") + ": " + nodes.map(function (node) {
            return node.label || node.id || "";
        }).join(" → ");
        var wrap = figure(component, summary);
        var list = el("ol", "gb-viz-flow");
        nodes.forEach(function (node) {
            var item = el("li", "gb-viz-flow-node");
            if (node.status) item.setAttribute("data-status", node.status);
            item.appendChild(el("span", "gb-viz-flow-label", node.label || node.id || ""));
            if (node.description) item.appendChild(el("span", "gb-viz-flow-desc", node.description));
            list.appendChild(item);
        });
        wrap.appendChild(list);
        return wrap;
    }

    function renderTimeline(component) {
        var items = Array.isArray(component.items) ? component.items : [];
        var summary = (component.title || "Timeline") + ": " + items.map(function (item) {
            return (item.time ? item.time + " " : "") + (item.title || "");
        }).join("; ");
        var wrap = figure(component, summary);
        var list = el("ol", "gb-viz-timeline");
        items.forEach(function (item) {
            var row = el("li", "gb-viz-timeline-item");
            if (item.status) row.setAttribute("data-status", item.status);
            row.appendChild(el("span", "gb-viz-timeline-time", item.time || ""));
            row.appendChild(el("span", "gb-viz-timeline-title", item.title || ""));
            if (item.description) row.appendChild(el("span", "gb-viz-timeline-desc", item.description));
            list.appendChild(row);
        });
        wrap.appendChild(list);
        return wrap;
    }

    function renderMetrics(component) {
        var items = Array.isArray(component.items) ? component.items : [];
        var summary = (component.title || "Metrics") + ": " + items.map(function (item) {
            return (item.label || "") + " " + item.value + (item.unit ? " " + item.unit : "");
        }).join("; ");
        var wrap = figure(component, summary);
        var grid = el("div", "gb-viz-metrics");
        items.forEach(function (item) {
            var card = el("div", "gb-viz-metric");
            card.appendChild(el("span", "gb-viz-metric-label", item.label || ""));
            var value = el("span", "gb-viz-metric-value", item.value);
            if (item.unit) value.textContent += " " + item.unit;
            card.appendChild(value);
            if (item.delta !== undefined && item.delta !== null) {
                var delta = el("span", "gb-viz-metric-delta", item.delta);
                delta.setAttribute("data-trend", item.trend || (Number(item.delta) >= 0 ? "up" : "down"));
                card.appendChild(delta);
            }
            grid.appendChild(card);
        });
        wrap.appendChild(grid);
        return wrap;
    }

    /** Live data updates: re-render a mounted visual from its new spec. */
    function update(container, component) {
        if (!container || !component) return;
        var next;
        switch (component.type) {
            case "bars": next = renderBars(component); break;
            case "lines": next = renderLines(component); break;
            case "flow": next = renderFlow(component); break;
            case "timeline": next = renderTimeline(component); break;
            case "metrics": next = renderMetrics(component); break;
            default: return;
        }
        if (!reducedMotion.matches) {
            next.classList.add("gb-viz-enter");
        }
        container.replaceWith(next);
    }

    window.Viz = {
        renderBars: renderBars,
        renderLines: renderLines,
        renderFlow: renderFlow,
        renderTimeline: renderTimeline,
        renderMetrics: renderMetrics,
        update: update,
        reducedMotion: function () { return !!reducedMotion.matches; },
    };
})();
