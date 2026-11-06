/**
 * Tier 1 surface renderer (#1669).
 *
 * One renderer for questions, forms, plans, tables, metrics and visuals. The
 * model sends data against the trusted catalog `gb.basic.v1`; this module is
 * the only code that turns it into DOM. Unknown components degrade to their
 * fallback, never to raw HTML.
 */
(function () {
    "use strict";

    var CATALOG = [
        "text", "callout", "table", "metrics", "bars", "lines",
        "flow", "timeline", "form", "plan", "diff", "tabs",
    ];

    // Mounted surfaces by id, so a STATE_DELTA / gb.surface_update can patch
    // the data model and re-render only the affected card.
    var mounted = Object.create(null);

    function isObject(value) {
        return value !== null && typeof value === "object" && !Array.isArray(value);
    }

    function escapeText(value) {
        if (value === null || value === undefined) return "";
        return String(value);
    }

    function pointerGet(root, pointer) {
        if (!pointer || pointer === "/") return root;
        var path = String(pointer).replace(/^\//, "").split("/");
        var current = root;
        for (var i = 0; i < path.length; i += 1) {
            if (current === null || current === undefined) return undefined;
            var key = path[i].replace(/~1/g, "/").replace(/~0/g, "~");
            current = current[key];
        }
        return current;
    }

    function resolve(value, dataModel) {
        if (isObject(value) && typeof value.$bind === "string") {
            return pointerGet(dataModel, value.$bind);
        }
        if (Array.isArray(value)) {
            return value.map(function (item) { return resolve(item, dataModel); });
        }
        if (isObject(value)) {
            var mapped = {};
            Object.keys(value).forEach(function (key) {
                mapped[key] = resolve(value[key], dataModel);
            });
            return mapped;
        }
        return value;
    }

    function validate(surface) {
        var errors = [];
        if (!isObject(surface)) {
            return { ok: false, errors: ["surface is not an object"] };
        }
        if (surface.version && surface.version !== "gb.basic.v1") {
            errors.push("unsupported surface version " + surface.version);
        }
        if (!Array.isArray(surface.components)) {
            errors.push("components must be an array");
            return { ok: false, errors: errors };
        }
        surface.components.forEach(function (component, index) {
            if (!isObject(component)) {
                errors.push("components[" + index + "] is not an object");
                return;
            }
            if (!component.id) errors.push("components[" + index + "].id is required");
            if (!component.type) errors.push("components[" + index + "].type is required");
        });
        return { ok: errors.length === 0, errors: errors };
    }

    function element(tag, className, text) {
        var node = document.createElement(tag);
        if (className) node.className = className;
        if (text !== undefined) node.textContent = escapeText(text);
        return node;
    }

    function renderText(component) {
        return element("p", "surface-text", component.text || component.value || "");
    }

    function renderCallout(component) {
        var tone = ["info", "success", "warning", "error"].indexOf(component.tone) >= 0
            ? component.tone : "info";
        var box = element("div", "surface-callout tone-" + tone);
        box.setAttribute("role", tone === "error" ? "alert" : "status");
        box.appendChild(element("strong", "", component.title || ""));
        box.appendChild(element("p", "", component.text || ""));
        return box;
    }

    function renderTable(component) {
        var wrapper = element("div", "surface-table-wrap");
        var table = element("table", "surface-table");
        var caption = component.title || component.caption;
        if (caption) {
            var cap = element("caption", "", caption);
            table.appendChild(cap);
        }
        var columns = Array.isArray(component.columns) ? component.columns : [];
        var rows = Array.isArray(component.rows) ? component.rows : [];
        if (columns.length) {
            var thead = document.createElement("thead");
            var headRow = document.createElement("tr");
            columns.forEach(function (column) {
                var th = document.createElement("th");
                th.scope = "col";
                th.textContent = escapeText(isObject(column) ? column.label : column);
                headRow.appendChild(th);
            });
            thead.appendChild(headRow);
            table.appendChild(thead);
        }
        var tbody = document.createElement("tbody");
        rows.forEach(function (row) {
            var tr = document.createElement("tr");
            var cells = Array.isArray(row) ? row : columns.map(function (column) {
                var key = isObject(column) ? column.key : column;
                return row ? row[key] : "";
            });
            cells.forEach(function (cell) {
                tr.appendChild(element("td", "", cell));
            });
            tbody.appendChild(tr);
        });
        table.appendChild(tbody);
        wrapper.appendChild(table);
        return wrapper;
    }

    function renderMetrics(component) {
        var grid = element("div", "surface-metrics");
        var items = Array.isArray(component.items) ? component.items : [];
        items.forEach(function (item) {
            var card = element("div", "surface-metric");
            card.appendChild(element("div", "surface-metric-label", item.label || ""));
            var value = element("div", "surface-metric-value");
            value.textContent = escapeText(item.value) + (item.unit ? " " + item.unit : "");
            card.appendChild(value);
            if (item.delta !== undefined && item.delta !== null) {
                var delta = element("div", "surface-metric-delta");
                var trend = item.trend || (Number(item.delta) >= 0 ? "up" : "down");
                delta.textContent = (trend === "down" ? "▼ " : trend === "up" ? "▲ " : "") +
                    escapeText(item.delta);
                delta.setAttribute("data-trend", trend);
                card.appendChild(delta);
            }
            grid.appendChild(card);
        });
        return grid;
    }

    function vizFallback(component, kind) {
        if (window.Viz) {
            try {
                return kind === "bars" ? window.Viz.renderBars(component) : window.Viz.renderLines(component);
            } catch (e) { /* fall through to the table */ }
        }
        var title = component.title || kind;
        var wrapper = element("div", "surface-viz");
        wrapper.appendChild(element("h4", "", title));
        if (Array.isArray(component.labels) && Array.isArray(component.series)) {
            wrapper.appendChild(renderTable({
                columns: ["series"].concat(component.labels),
                rows: component.series.map(function (series) {
                    var row = [series.name || "series"];
                    (series.values || []).forEach(function (value) { row.push(value); });
                    return row;
                }),
            }));
        } else {
            wrapper.appendChild(element("p", "surface-muted",
                "This visualization is not available in this client."));
        }
        return wrapper;
    }

    function renderFlow(component) {
        if (window.Viz && typeof window.Viz.renderFlow === "function") {
            try { return window.Viz.renderFlow(component); } catch (e) { /* fall through */ }
        }
        var wrapper = element("div", "surface-flow");
        wrapper.appendChild(element("h4", "", component.title || "Flow"));
        var list = document.createElement("ol");
        (component.nodes || []).forEach(function (node) {
            var item = element("li", "", node.label || node.id || "");
            if (node.status) item.setAttribute("data-status", node.status);
            list.appendChild(item);
        });
        wrapper.appendChild(list);
        return wrapper;
    }

    function renderTimeline(component) {
        if (window.Viz && typeof window.Viz.renderTimeline === "function") {
            try { return window.Viz.renderTimeline(component); } catch (e) { /* fall through */ }
        }
        var wrapper = element("div", "surface-timeline");
        wrapper.appendChild(element("h4", "", component.title || "Timeline"));
        var list = document.createElement("ol");
        (component.items || []).forEach(function (item) {
            var row = element("li", "", (item.time ? item.time + " — " : "") + (item.title || ""));
            if (item.status) row.setAttribute("data-status", item.status);
            list.appendChild(row);
        });
        wrapper.appendChild(list);
        return wrapper;
    }

    // Internals shared with the sibling files (same split for #1731).
    var SR = (window.SurfaceRenderer = window.SurfaceRenderer || {});
    SR._isObject = isObject;
    SR._escapeText = escapeText;
    SR._pointerGet = pointerGet;
    SR._resolve = resolve;
    SR._validate = validate;
    SR._element = element;
    SR._mounted = mounted;
    SR._CATALOG = CATALOG;
    SR._renderText = renderText;
    SR._renderCallout = renderCallout;
    SR._renderTable = renderTable;
    SR._renderMetrics = renderMetrics;
    SR._vizFallback = vizFallback;
    SR._renderFlow = renderFlow;
    SR._renderTimeline = renderTimeline;
})();
