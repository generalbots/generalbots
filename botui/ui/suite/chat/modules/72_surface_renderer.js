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

    function fieldInput(field, index) {
        var id = "surface-field-" + index + "-" + Math.random().toString(36).slice(2, 8);
        var wrapper = element("div", "surface-field");
        var label = document.createElement("label");
        label.setAttribute("for", id);
        label.textContent = field.label || field.id || "Field";
        if (field.required) {
            var required = element("span", "surface-required", " *");
            required.setAttribute("aria-hidden", "true");
            label.appendChild(required);
        }
        var type = field.type || "text";
        var input;
        if (type === "textarea") {
            input = document.createElement("textarea");
            input.rows = 3;
        } else if (type === "select") {
            input = document.createElement("select");
            var placeholder = document.createElement("option");
            placeholder.value = "";
            placeholder.textContent = field.placeholder || "Choose…";
            input.appendChild(placeholder);
            (field.options || []).forEach(function (option) {
                var node = document.createElement("option");
                if (isObject(option)) {
                    node.value = option.value;
                    node.textContent = option.label || option.value;
                } else {
                    node.value = option;
                    node.textContent = option;
                }
                input.appendChild(node);
            });
        } else {
            input = document.createElement("input");
            input.type = type === "number" || type === "date" || type === "email" ? type : "text";
        }
        input.id = id;
        input.name = field.id || "field_" + index;
        if (field.placeholder && type !== "select") input.placeholder = field.placeholder;
        if (field.required) input.required = true;
        if (field.value !== undefined && field.value !== null) input.value = field.value;
        wrapper.appendChild(label);
        wrapper.appendChild(input);
        if (field.help) {
            var help = element("div", "surface-help", field.help);
            help.id = id + "-help";
            input.setAttribute("aria-describedby", help.id);
            wrapper.appendChild(help);
        }
        return wrapper;
    }

    function renderForm(component, context, onSubmit) {
        var form = document.createElement("form");
        form.className = "surface-form";
        form.noValidate = true;
        if (component.title) form.appendChild(element("h4", "", component.title));
        var fields = Array.isArray(component.fields) ? component.fields : [];
        fields.forEach(function (field, index) {
            form.appendChild(fieldInput(field, index));
        });
        var error = element("p", "surface-form-error");
        error.setAttribute("role", "alert");
        error.hidden = true;
        form.appendChild(error);
        var submit = document.createElement("button");
        submit.type = "submit";
        submit.className = "surface-submit";
        submit.textContent = component.submit_label || "Continue";
        form.appendChild(submit);

        form.addEventListener("submit", function (event) {
            event.preventDefault();
            var values = {};
            var missing = [];
            fields.forEach(function (field, index) {
                var input = form.querySelectorAll("input, textarea, select")[index];
                if (!input) return;
                var key = field.id || "field_" + index;
                values[key] = input.value;
                if (field.required && !String(input.value || "").trim()) {
                    missing.push(field.label || key);
                    input.setAttribute("aria-invalid", "true");
                } else {
                    input.removeAttribute("aria-invalid");
                }
            });
            if (missing.length) {
                error.hidden = false;
                error.textContent = "Please fill in: " + missing.join(", ");
                var firstInvalid = form.querySelector('[aria-invalid="true"]');
                if (firstInvalid) firstInvalid.focus();
                return;
            }
            error.hidden = true;
            submit.disabled = true;
            submit.textContent = "Sending…";
            var detail = {
                surface_id: context.surfaceId,
                action: component.submit_action || "submit",
                values: values,
            };
            window.dispatchEvent(new CustomEvent("gb-surface-submit", { detail: detail }));
            if (typeof onSubmit === "function") onSubmit(detail);
        });
        return form;
    }

    function renderPlan(component, context) {
        var wrapper = element("div", "surface-plan");
        wrapper.appendChild(element("h4", "", component.title || "Plan"));
        var steps = (component.steps || []).map(function (step, index) {
            return {
                id: step.id || "step-" + index,
                title: step.title || step.label || "Step " + (index + 1),
                description: step.description || "",
                status: step.status || "pending",
            };
        });
        var list = document.createElement("ol");
        list.className = "surface-plan-steps";
        function draw() {
            list.textContent = "";
            steps.forEach(function (step, index) {
                var item = document.createElement("li");
                item.setAttribute("data-status", step.status);
                var text = element("div", "surface-plan-title", step.title);
                item.appendChild(text);
                if (step.description) item.appendChild(element("div", "surface-help", step.description));
                var controls = element("div", "surface-plan-controls");
                var up = element("button", "", "Move up");
                up.type = "button";
                up.disabled = index === 0;
                up.addEventListener("click", function () {
                    var temp = steps[index - 1];
                    steps[index - 1] = steps[index];
                    steps[index] = temp;
                    draw();
                });
                var down = element("button", "", "Move down");
                down.type = "button";
                down.disabled = index === steps.length - 1;
                down.addEventListener("click", function () {
                    var temp = steps[index + 1];
                    steps[index + 1] = steps[index];
                    steps[index] = temp;
                    draw();
                });
                var remove = element("button", "", "Remove");
                remove.type = "button";
                remove.addEventListener("click", function () {
                    steps.splice(index, 1);
                    draw();
                });
                controls.appendChild(up);
                controls.appendChild(down);
                controls.appendChild(remove);
                item.appendChild(controls);
                list.appendChild(item);
            });
        }
        draw();
        wrapper.appendChild(list);
        var approve = element("button", "surface-submit", component.approve_label || "Approve plan");
        approve.type = "button";
        approve.addEventListener("click", function () {
            approve.disabled = true;
            var detail = { surface_id: context.surfaceId, action: "approve_plan", steps: steps };
            window.dispatchEvent(new CustomEvent("gb-surface-submit", { detail: detail }));
        });
        wrapper.appendChild(approve);
        return wrapper;
    }

    function renderDiff(component) {
        var pre = element("pre", "surface-diff");
        (component.diff || component.text || "").split("\n").forEach(function (line) {
            var row = element("span", "surface-diff-line");
            if (line.indexOf("+") === 0 && line.indexOf("+++") !== 0) row.className += " added";
            else if (line.indexOf("-") === 0 && line.indexOf("---") !== 0) row.className += " removed";
            row.textContent = line;
            pre.appendChild(row);
            pre.appendChild(document.createTextNode("\n"));
        });
        return pre;
    }

    function renderTabs(component, context, onSubmit) {
        var wrapper = element("div", "surface-tabs");
        var tabs = Array.isArray(component.tabs) ? component.tabs : [];
        var nav = element("div", "surface-tab-list");
        nav.setAttribute("role", "tablist");
        var panels = element("div", "surface-tab-panels");
        tabs.forEach(function (tab, index) {
            var button = element("button", "surface-tab", tab.label || "Tab " + (index + 1));
            button.type = "button";
            button.setAttribute("role", "tab");
            button.setAttribute("aria-selected", index === 0 ? "true" : "false");
            button.id = "surface-tab-" + index + "-" + Math.random().toString(36).slice(2, 6);
            var panel = element("div", "surface-tab-panel");
            panel.setAttribute("role", "tabpanel");
            panel.hidden = index !== 0;
            if (Array.isArray(tab.components)) {
                tab.components.forEach(function (child) {
                    panel.appendChild(renderComponent(child, context, onSubmit));
                });
            } else {
                panel.textContent = tab.text || "";
            }
            button.addEventListener("click", function () {
                nav.querySelectorAll('[role="tab"]').forEach(function (node) {
                    node.setAttribute("aria-selected", "false");
                });
                panels.querySelectorAll('[role="tabpanel"]').forEach(function (node) {
                    node.hidden = true;
                });
                button.setAttribute("aria-selected", "true");
                panel.hidden = false;
            });
            nav.appendChild(button);
            panels.appendChild(panel);
        });
        wrapper.appendChild(nav);
        wrapper.appendChild(panels);
        return wrapper;
    }

    function renderUnknown(component) {
        var box = element("div", "surface-callout tone-warning");
        box.appendChild(element("strong", "", "Unsupported component"));
        box.appendChild(element("p", "",
            "The " + escapeText(component.type) + " component is not available in this client."));
        if (component.fallback) box.appendChild(element("p", "", component.fallback));
        return box;
    }

    function renderComponent(component, context, onSubmit) {
        if (!isObject(component)) return element("p", "surface-text", "");
        var type = component.type;
        switch (type) {
            case "text": return renderText(component);
            case "callout": return renderCallout(component);
            case "table": return renderTable(component);
            case "metrics": return window.Viz ? window.Viz.renderMetrics(component) : renderMetrics(component);
            case "bars": return vizFallback(component, "bars");
            case "lines": return vizFallback(component, "lines");
            case "flow": return renderFlow(component);
            case "timeline": return renderTimeline(component);
            case "form": return renderForm(component, context, onSubmit);
            case "plan": return renderPlan(component, context);
            case "diff": return renderDiff(component);
            case "tabs": return renderTabs(component, context, onSubmit);
            default: return renderUnknown(component);
        }
    }

    function componentText(component) {
        if (!isObject(component)) return "";
        switch (component.type) {
            case "text": return component.text || "";
            case "callout": return (component.title ? component.title + ": " : "") + (component.text || "");
            case "table":
                return (component.title || "Table") + " — " +
                    (Array.isArray(component.rows) ? component.rows.length : 0) + " rows";
            case "metrics":
                return (component.items || []).map(function (item) {
                    return (item.label || "") + ": " + escapeText(item.value);
                }).join("; ");
            case "form":
                return (component.title || "Form") + " — " + (component.fields || []).length + " fields";
            case "plan":
                return (component.title || "Plan") + " — " + (component.steps || []).length + " steps";
            default:
                return component.fallback || (component.title || "");
        }
    }

    function generateFallback(surface) {
        return (surface.components || []).map(componentText).filter(Boolean).join("\n");
    }

    function render(surface, fallback, onSubmit) {
        var result = validate(surface);
        var wrapper = element("section", "surface-card");
        wrapper.setAttribute("role", "group");
        wrapper.setAttribute("aria-label", (surface && surface.title) || "Result");
        wrapper.setAttribute("data-surface-id", (surface && surface.surface_id) || "");
        if (!result.ok) {
            var warning = element("div", "surface-callout tone-error");
            warning.setAttribute("role", "alert");
            warning.appendChild(element("strong", "", "This result could not be rendered"));
            warning.appendChild(element("p", "", result.errors.join("; ")));
            warning.appendChild(element("pre", "surface-fallback", fallback || "No fallback available."));
            wrapper.appendChild(warning);
            return wrapper;
        }
        var dataModel = surface.data || {};
        var context = { surfaceId: surface.surface_id || "" };
        var head = element("header", "surface-head");
        head.appendChild(element("h3", "", surface.title || "Result"));
        wrapper.appendChild(head);
        var body = element("div", "surface-body");
        surface.components.forEach(function (raw) {
            var component = resolve(raw, dataModel);
            body.appendChild(renderComponent(component, context, onSubmit));
        });
        // A form whose heading repeats the card title printed it twice
        // ("Before we start" over "Before we start").
        var surfaceTitle = String(surface.title || "").trim();
        body.querySelectorAll(".surface-form > h4").forEach(function (heading) {
            if (heading.textContent.trim() === surfaceTitle) heading.hidden = true;
        });
        wrapper.appendChild(body);
        if (surface.surface_id) {
            mounted[surface.surface_id] = {
                surface: surface,
                body: body,
                context: context,
                onSubmit: onSubmit,
                wrapper: wrapper,
            };
        }
        if (surface.fallback) {
            var details = document.createElement("details");
            details.className = "surface-fallback-details";
            var summary = document.createElement("summary");
            summary.textContent = "Text version";
            details.appendChild(summary);
            details.appendChild(element("pre", "surface-fallback", surface.fallback));
            wrapper.appendChild(details);
        }
        return wrapper;
    }

    function defaultSubmit(detail) {
        if (window.ChatSend && typeof window.ChatSend.sendStructured === "function") {
            window.ChatSend.sendStructured(detail);
            return;
        }
        if (typeof window.sendMessage === "function") {
            var text = detail.action === "approve_plan"
                ? "Approved plan: " + (detail.steps || []).map(function (step) { return step.title; }).join(" → ")
                : "Answers: " + JSON.stringify(detail.values || {});
            window.sendMessage(text);
        }
    }

    function renderFromFrame(frame, rootEl) {
        var payload = frame && frame.data ? frame.data.payload || {} : {};
        var surface = payload.surface;
        if (!surface && Array.isArray(payload.steps)) {
            surface = {
                version: "gb.basic.v1",
                surface_id: payload.plan_id || "",
                title: payload.title || "Plan",
                components: [{
                    type: "plan",
                    id: "plan",
                    title: payload.title || "Plan",
                    steps: payload.steps,
                    approve_label: payload.approve_label || "Approve plan",
                }],
                fallback: payload.fallback || "",
            };
        }
        if (!surface) surface = payload;
        var fallback = payload.fallback || surface.fallback || generateFallback(surface);
        var root = rootEl || document.getElementById("messages");
        if (!root) return null;
        var card = render(surface, fallback, defaultSubmit);
        root.appendChild(card);
        return card;
    }

    function renderQuestion(frame, rootEl) {
        var payload = frame && frame.data ? frame.data.payload || {} : {};
        var fields = Array.isArray(payload.fields) ? payload.fields : [];
        var surface = {
            version: "gb.basic.v1",
            surface_id: payload.question_id || payload.id || "",
            title: payload.title || "A question",
            components: [{
                type: "form",
                id: "question",
                title: payload.title || "",
                fields: fields,
                submit_label: payload.submit_label || "Continue",
            }],
            fallback: payload.fallback || "",
        };
        var root = rootEl || document.getElementById("messages");
        if (!root) return null;
        var card = render(surface, surface.fallback, function (detail) {
            detail.question_id = surface.surface_id;
            detail.action = "answer_question";
            defaultSubmit(detail);
        });
        root.appendChild(card);
        return card;
    }

    function applyPatch(document, patch) {
        if (!Array.isArray(patch)) return false;
        patch.forEach(function (operation) {
            var path = operation && operation.path;
            if (typeof path !== "string") return;
            var parts = path.replace(/^\//, "").split("/").map(function (part) {
                return part.replace(/~1/g, "/").replace(/~0/g, "~");
            });
            if (!parts.length || !parts[0]) return;
            var parent = document;
            for (var i = 0; i < parts.length - 1; i += 1) {
                if (parent === null || typeof parent !== "object") return;
                if (!(parts[i] in parent)) parent[parts[i]] = {};
                parent = parent[parts[i]];
            }
            var key = parts[parts.length - 1];
            if (operation.op === "remove") {
                if (Array.isArray(parent)) parent.splice(Number(key), 1);
                else delete parent[key];
            } else if (operation.op === "add" && Array.isArray(parent) && key === "-") {
                parent.push(operation.value);
            } else {
                parent[key] = operation.value;
            }
        });
        return true;
    }

    function applyUpdate(frame) {
        var payload = frame && frame.data ? frame.data.payload || {} : {};
        var surfaceId = payload.surface_id || (frame.data && frame.data.surface_id);
        var entry = surfaceId ? mounted[surfaceId] : null;
        if (!entry) return false;
        var dataModel = entry.surface.data || {};
        if (payload.state) dataModel = payload.state;
        applyPatch(dataModel, payload.patch || []);
        entry.surface.data = dataModel;
        entry.body.textContent = "";
        (entry.surface.components || []).forEach(function (raw) {
            entry.body.appendChild(renderComponent(resolve(raw, dataModel), entry.context, entry.onSubmit));
        });
        return true;
    }

    window.SurfaceRenderer = {
        render: render,
        renderFromFrame: renderFromFrame,
        renderQuestion: renderQuestion,
        validate: validate,
        applyUpdate: applyUpdate,
        catalog: CATALOG.slice(),
    };
})();
