/**
 * Tier 1 surface renderer (#1669) — component dispatch, card assembly,
 * frame entry points and state patches. Split from 72_surface_renderer.js
 * for #1731; publishes the public window.SurfaceRenderer API.
 */
(function () {
    "use strict";

    var SR = window.SurfaceRenderer;
    var element = SR._element;
    var escapeText = SR._escapeText;
    var isObject = SR._isObject;
    var resolve = SR._resolve;
    var validate = SR._validate;
    var mounted = SR._mounted;
    var CATALOG = SR._CATALOG;
    var renderText = SR._renderText;
    var renderCallout = SR._renderCallout;
    var renderTable = SR._renderTable;
    var renderMetrics = SR._renderMetrics;
    var vizFallback = SR._vizFallback;
    var renderFlow = SR._renderFlow;
    var renderTimeline = SR._renderTimeline;
    var renderForm = SR._renderForm;
    var renderPlan = SR._renderPlan;
    var renderDiff = SR._renderDiff;
    var renderTabs = SR._renderTabs;

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

    // #2027 — a surface is a single card per `surface_id`. Live re-delivery and
    // replayed frames (`gb_event: true` from `run_resume`) used to append a
    // second, identical form next to the first, inviting a duplicate answer.
    function mountSurface(root, card, surfaceId) {
        if (!surfaceId) {
            root.appendChild(card);
            return card;
        }
        // Look the card up by attribute, not by `mounted[]`: `render()` records
        // the new wrapper in `mounted` before this runs, so the registry entry
        // is already the detached card we are about to insert.
        card.setAttribute("data-surface-id", surfaceId);
        var scopes = [root, document];
        for (var s = 0; s < scopes.length; s++) {
            var nodes = scopes[s].querySelectorAll("[data-surface-id]");
            for (var i = 0; i < nodes.length; i++) {
                if (nodes[i] === card) continue;
                if (nodes[i].getAttribute("data-surface-id") !== surfaceId) continue;
                if (nodes[i].parentNode) nodes[i].parentNode.replaceChild(card, nodes[i]);
                return card;
            }
        }
        root.appendChild(card);
        return card;
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
        mountSurface(root, card, surface.surface_id);
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
        mountSurface(root, card, surface.surface_id);
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

    SR._renderComponent = renderComponent;
    SR.render = render;
    SR.renderFromFrame = renderFromFrame;
    SR.renderQuestion = renderQuestion;
    SR.validate = validate;
    SR.applyUpdate = applyUpdate;
    SR.catalog = CATALOG.slice();
})();
