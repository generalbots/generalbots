/* slides diagrams — accessible local editor dialog (deterministic, no server writes). */
"use strict";

(function () {
    "use strict";

    var GB = window.GBSlidesDiagrams;
    var presets = window.SDPresets;
    var org = window.SDOrg;
    var flow = window.SDFlow;

    function sdT(key, fallback, params) {
        var out = typeof window.gbT === "function" ? window.gbT(key, fallback, params) : fallback;
        if (params) {
            Object.keys(params).forEach(function (name) {
                out = out.split("{" + name + "}").join(params[name])
                    .split("{ $" + name + " }").join(params[name]);
            });
        }
        return out;
    }

    var PRESET_KEYS = ["process", "cycle", "hierarchy", "matrix", "pyramid", "list"];

    function t(key) {
        var fallbacks = window.SD_DIALOG_FALLBACKS || {};
        return sdT(key, fallbacks[key] || key);
    }

    var dialog = null;
    var lastFocus = null;
    var active = null;
    var selectedNode = null;
    var status = null;

    function el(tag, cls, text) {
        var node = document.createElement(tag);
        if (cls) { node.className = cls; }
        if (text !== undefined && text !== null) { node.textContent = String(text); }
        return node;
    }

    function button(label, cls, handler) {
        var b = el("button", cls || "sd-btn", label);
        b.type = "button";
        if (handler) { b.addEventListener("click", handler); }
        return b;
    }

    function announce(message) {
        if (status) { status.textContent = message; }
    }

    function focusables() {
        if (!dialog) { return []; }
        var nodes = dialog.querySelectorAll(
            "button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])"
        );
        return Array.prototype.filter.call(nodes, function (n) { return !n.disabled && n.offsetParent !== null; });
    }

    function onKeydown(event) {
        if (event.key === "Escape") {
            event.preventDefault();
            close();
            return;
        }
        if (event.key !== "Tab") { return; }
        var list = focusables();
        if (!list.length) { return; }
        var first = list[0];
        var last = list[list.length - 1];
        if (event.shiftKey && document.activeElement === first) {
            event.preventDefault(); last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault(); first.focus();
        }
    }

    function open() {
        if (dialog) { return; }
        lastFocus = document.activeElement;
        dialog = el("div", "sl-diag-dialog-backdrop");
        dialog.setAttribute("role", "presentation");
        var panel = el("div", "sl-diag-dialog");
        panel.setAttribute("role", "dialog");
        panel.setAttribute("aria-modal", "true");
        panel.setAttribute("aria-label", t("slides-diagrams-title"));
        panel.tabIndex = -1;
        dialog.appendChild(panel);

        var head = el("div", "sl-diag-dialog-head");
        head.appendChild(el("h2", "sl-diag-dialog-title", t("slides-diagrams-title")));
        var badge = el("span", "sl-diag-local-badge", t("slides-diagrams-local-only"));
        badge.setAttribute("role", "status");
        badge.setAttribute("aria-live", "polite");
        head.appendChild(badge);
        head.appendChild(button(t("slides-diagrams-close"), "sl-diag-close", close));
        panel.appendChild(head);

        status = el("div", "sl-diag-status");
        status.setAttribute("role", "status");
        status.setAttribute("aria-live", "polite");
        panel.appendChild(status);

        var body = el("div", "sl-diag-dialog-body");
        panel.appendChild(body);
        buildPresets(body);
        buildNodes(body);
        buildOrg(body);
        buildActions(body);

        document.body.appendChild(dialog);
        document.addEventListener("keydown", onKeydown, true);
        syncActive();
        renderAll();
        panel.focus();
        announce(t("slides-diagrams-status-ready"));
    }

    function close() {
        if (!dialog) { return; }
        document.removeEventListener("keydown", onKeydown, true);
        if (dialog.parentNode) { dialog.parentNode.removeChild(dialog); }
        dialog = null;
        status = null;
        active = null;
        selectedNode = null;
        refreshCanvas();
        if (lastFocus && lastFocus.focus) { lastFocus.focus(); }
        lastFocus = null;
    }

    function refreshCanvas() {
        if (window.GBSlidesDiagramsCanvas && window.GBSlidesDiagramsCanvas.refresh) {
            window.GBSlidesDiagramsCanvas.refresh();
        }
    }

    function commit() {
        GB.upsert(active);
        renderAll();
    }

    function buildPresets(parent) {
        var section = el("fieldset", "sl-diag-section");
        section.appendChild(el("legend", "sl-diag-legend", t("slides-diagrams-preset-label")));
        PRESET_KEYS.forEach(function (key) {
            var name = presets.name(key);
            var b = button(name, "sl-diag-preset", function () {
                active = presets.create(key);
                selectedNode = active.nodes.length ? active.nodes[0].id : null;
                commit();
                announce(sdT("slides-diagrams-status-created", "Created { $name }", { name: name }));
            });
            b.setAttribute("data-i18n", "slides-diagrams-preset-" + key);
            section.appendChild(b);
        });
        parent.appendChild(section);
    }

    function buildNodes(parent) {
        var section = el("fieldset", "sl-diag-section");
        section.appendChild(el("legend", "sl-diag-legend", t("slides-diagrams-nodes-label")));
        var list = el("div", "sl-diag-node-list");
        list.setAttribute("role", "list");
        list.id = "sl-diag-node-list";
        section.appendChild(list);

        var controls = el("div", "sl-diag-node-controls");
        controls.appendChild(button(t("slides-diagrams-node-add"), "sl-diag-btn-add", function () {
            if (!active) { return; }
            var node = presets.addNode(active);
            selectedNode = node ? node.id : null;
            commit();
            announce(sdT("slides-diagrams-status-node-added", "Node added"));
        }));
        controls.appendChild(button(t("slides-diagrams-node-remove"), "sl-diag-btn-remove", function () {
            if (!active || !selectedNode) { return; }
            presets.removeNode(active, selectedNode);
            selectedNode = active.nodes.length ? active.nodes[0].id : null;
            commit();
            announce(sdT("slides-diagrams-status-node-removed", "Node removed"));
        }));
        controls.appendChild(button(t("slides-diagrams-node-promote"), "sl-diag-btn-promote", function () {
            if (!active || !selectedNode) { return; }
            presets.promote(active, selectedNode);
            commit();
            announce(sdT("slides-diagrams-status-promoted", "Node promoted"));
        }));
        controls.appendChild(button(t("slides-diagrams-node-demote"), "sl-diag-btn-demote", function () {
            if (!active || !selectedNode) { return; }
            presets.demote(active, selectedNode);
            commit();
            announce(sdT("slides-diagrams-status-demoted", "Node demoted"));
        }));
        section.appendChild(controls);

        var editor = el("div", "sl-diag-node-editor");
        editor.id = "sl-diag-node-editor";
        section.appendChild(editor);
        parent.appendChild(section);
    }

    function buildOrg(parent) {
        var section = el("fieldset", "sl-diag-section");
        section.appendChild(el("legend", "sl-diag-legend", t("slides-diagrams-org-title")));
        section.appendChild(el("p", "sl-diag-hint", t("slides-diagrams-org-hint")));
        var area = el("textarea", "sl-diag-org-input");
        area.id = "sl-diag-org-input";
        area.rows = 5;
        area.setAttribute("placeholder", t("slides-diagrams-org-placeholder"));
        area.setAttribute("aria-label", t("slides-diagrams-org-title"));
        section.appendChild(area);
        section.appendChild(button(t("slides-diagrams-org-generate"), "sl-diag-btn-org", function () {
            if (!org || !org.build) { return; }
            var result = org.build(area.value);
            if (!result.count) {
                announce(sdT("slides-diagrams-status-org-empty", "No valid data rows found"));
                return;
            }
            active = result.diagram;
            selectedNode = active.nodes.length ? active.nodes[0].id : null;
            commit();
            announce(sdT("slides-diagrams-status-org-loaded", "Loaded { $n } people", { n: String(result.count) }));
            if (result.errors.length) {
                announce(sdT("slides-diagrams-status-org-errors", "{ $n } row(s) skipped", { n: String(result.errors.length) }));
            }
        }));
        parent.appendChild(section);
    }

    function buildActions(parent) {
        var section = el("div", "sl-diag-actions");
        section.appendChild(button(t("slides-diagrams-undo"), "sl-diag-btn-undo", function () {
            if (!GB.undo()) { return; }
            syncActive();
            renderAll();
            announce(sdT("slides-diagrams-status-undo", "Undo applied"));
        }));
        section.appendChild(button(t("slides-diagrams-redo"), "sl-diag-btn-redo", function () {
            if (!GB.redo()) { return; }
            syncActive();
            renderAll();
            announce(sdT("slides-diagrams-status-redo", "Redo applied"));
        }));

        var flowBtn = button("", "sl-diag-btn-flow", function () {
            if (!active) { return; }
            active.flowMode = !active.flowMode;
            if (flow && flow.recompute) { flow.recompute(active); }
            commit();
            announce(active.flowMode
                ? sdT("slides-diagrams-status-flow-on", "Flowchart connectors on")
                : sdT("slides-diagrams-status-flow-off", "Flowchart connectors off"));
        });
        section.appendChild(flowBtn);

        var groupBtn = button("", "sl-diag-btn-group", function () {
            if (!active) { return; }
            active.grouped = !active.grouped;
            commit();
            announce(active.grouped
                ? sdT("slides-diagrams-status-grouped", "Nodes grouped")
                : sdT("slides-diagrams-status-ungrouped", "Nodes ungrouped"));
        });
        section.appendChild(groupBtn);

        section.appendChild(button(t("slides-diagrams-drill-in"), "sl-diag-btn-drill", function () {
            if (!active || !active.grouped) { return; }
            selectedNode = active.nodes.length ? active.nodes[0].id : null;
            renderAll();
            announce(t("slides-diagrams-drill-in"));
        }));
        parent.appendChild(section);
    }

    function syncActive() {
        var current = GB.active();
        active = current ? sdClone(current) : null;
        selectedNode = active && active.nodes.length ? active.nodes[0].id : null;
    }

    function currentNode() {
        if (!active || !selectedNode || !presets) { return null; }
        return presets.byId(active, selectedNode);
    }

    function renderAll() {
        var views = window.SDDialogViews;
        if (views) {
            views.nodeList(active, selectedNode, function (id) {
                selectedNode = id;
                renderAll();
            });
            views.nodeEditor(currentNode(), commit, function () {
                presets.reflow(active);
                commit();
            });
            views.actionStates(dialog, active);
        }
        refreshCanvas();
    }

    window.SDDialogUI = { t: t, sdT: sdT, el: el, button: button };

    window.GBSlidesDiagramsDialog = {
        open: open,
        close: close,
        isOpen: function () { return !!dialog; },
        refreshFromState: function () {
            if (!dialog) { return; }
            syncActive();
            renderAll();
        }
    };
}());
