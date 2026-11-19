/* slides diagrams — editor dialog views + fallback strings (#2488).
 * Split from 35_diag_dialog.js to keep every module under the 350-line target. */

"use strict";

var SD_DIALOG_FALLBACKS = {
    "slides-diagrams-title": "Diagram editor",
    "slides-diagrams-close": "Close",
    "slides-diagrams-local-only": "Local only — not saved to the server",
    "slides-diagrams-status-ready": "Choose a preset or build an org chart to begin.",
    "slides-diagrams-status-created": "Created { $name }",
    "slides-diagrams-status-node-added": "Node added",
    "slides-diagrams-status-node-removed": "Node removed",
    "slides-diagrams-status-promoted": "Node promoted",
    "slides-diagrams-status-demoted": "Node demoted",
    "slides-diagrams-status-org-empty": "No valid data rows found",
    "slides-diagrams-status-org-loaded": "Loaded { $n } people",
    "slides-diagrams-status-org-errors": "{ $n } row(s) skipped",
    "slides-diagrams-status-grouped": "Nodes grouped",
    "slides-diagrams-status-ungrouped": "Nodes ungrouped",
    "slides-diagrams-status-flow-on": "Flowchart connectors on",
    "slides-diagrams-status-flow-off": "Flowchart connectors off",
    "slides-diagrams-status-undo": "Undo applied",
    "slides-diagrams-status-redo": "Redo applied",
    "slides-diagrams-preset-label": "Preset",
    "slides-diagrams-nodes-label": "Nodes",
    "slides-diagrams-node-add": "Add node",
    "slides-diagrams-node-remove": "Remove node",
    "slides-diagrams-node-promote": "Promote",
    "slides-diagrams-node-demote": "Demote",
    "slides-diagrams-node-text": "Text",
    "slides-diagrams-node-text-placeholder": "Node text",
    "slides-diagrams-node-icon": "Icon",
    "slides-diagrams-node-color": "Colour",
    "slides-diagrams-node-level": "Level",
    "slides-diagrams-node-default": "Node",
    "slides-diagrams-icon-search-label": "Search icons",
    "slides-diagrams-icon-search-placeholder": "Search icons",
    "slides-diagrams-icon-empty": "No icons match your search.",
    "slides-diagrams-org-title": "Organisation chart from data",
    "slides-diagrams-org-hint": "Paste rows: name, title, reports-to (CSV or tab separated). Local only — no server fetch.",
    "slides-diagrams-org-placeholder": "Ana, Director,  Bruno, Manager, Ana",
    "slides-diagrams-org-generate": "Build chart",
    "slides-diagrams-flow-toggle": "Flowchart connectors",
    "slides-diagrams-group": "Group as one object",
    "slides-diagrams-ungroup": "Ungroup",
    "slides-diagrams-drill-in": "Edit grouped nodes",
    "slides-diagrams-undo": "Undo",
    "slides-diagrams-redo": "Redo",
    "slides-diagrams-empty": "No diagram yet"
};

(function () {
    "use strict";

    var ui = window.SDDialogUI;

    function t(key) { return ui.t(key); }

    function nodeList(active, selectedNode, onSelect) {
        var list = document.getElementById("sl-diag-node-list");
        if (!list) { return; }
        while (list.firstChild) { list.removeChild(list.firstChild); }
        if (!active || !active.nodes.length) {
            list.appendChild(ui.el("p", "sl-diag-empty", t("slides-diagrams-empty")));
            return;
        }
        active.nodes.forEach(function (node, index) {
            var base = t("slides-diagrams-node-default") + " " + (index + 1);
            var b = ui.button(base + (node.text ? ": " + node.text : ""),
                "sl-diag-node-item" + (node.id === selectedNode ? " is-selected" : ""), function () {
                    onSelect(node.id);
                });
            b.setAttribute("role", "listitem");
            b.setAttribute("aria-label", node.text || base);
            list.appendChild(b);
        });
    }

    function iconGrid(grid, query, node, onEdit) {
        var icons = window.SDIcons;
        while (grid.firstChild) { grid.removeChild(grid.firstChild); }
        if (!icons || !icons.search) { return; }
        var list = icons.search(query || "");
        if (!list.length) {
            grid.appendChild(ui.el("p", "sl-diag-icon-empty", t("slides-diagrams-icon-empty")));
            return;
        }
        list.forEach(function (item) {
            var b = ui.button("", "sl-diag-icon" + (node.icon === item.id ? " is-selected" : ""), function () {
                node.icon = item.id;
                onEdit();
            });
            b.setAttribute("data-i18n-aria-label", item.key);
            b.setAttribute("aria-label", icons.label(item));
            var svg = icons.svg(item.id, icons.label(item));
            if (svg) { b.appendChild(svg); }
            grid.appendChild(b);
        });
    }

    function nodeEditor(node, onEdit, onReflow) {
        var host = document.getElementById("sl-diag-node-editor");
        if (!host) { return; }
        while (host.firstChild) { host.removeChild(host.firstChild); }
        if (!node) { return; }

        var textLabel = ui.el("label", "sl-diag-field");
        textLabel.appendChild(ui.el("span", "sl-diag-field-label", t("slides-diagrams-node-text")));
        var text = ui.el("input", "sl-diag-field-input");
        text.type = "text";
        text.value = node.text || "";
        text.setAttribute("placeholder", t("slides-diagrams-node-text-placeholder"));
        text.addEventListener("change", function () {
            node.text = text.value;
            onEdit();
        });
        textLabel.appendChild(text);
        host.appendChild(textLabel);

        var colorLabel = ui.el("label", "sl-diag-field");
        colorLabel.appendChild(ui.el("span", "sl-diag-field-label", t("slides-diagrams-node-color")));
        var color = ui.el("input", "sl-diag-field-color");
        color.type = "color";
        color.value = /^#[0-9a-f]{6}$/i.test(node.color || "") ? node.color : "#3778c2";
        color.addEventListener("change", function () {
            node.color = color.value;
            onEdit();
        });
        colorLabel.appendChild(color);
        host.appendChild(colorLabel);

        var levelLabel = ui.el("label", "sl-diag-field");
        levelLabel.appendChild(ui.el("span", "sl-diag-field-label", t("slides-diagrams-node-level")));
        var level = ui.el("input", "sl-diag-field-level");
        level.type = "number";
        level.min = "0";
        level.value = String(node.level || 0);
        level.addEventListener("change", function () {
            node.level = Math.max(0, parseInt(level.value, 10) || 0);
            onReflow();
        });
        levelLabel.appendChild(level);
        host.appendChild(levelLabel);

        host.appendChild(ui.el("span", "sl-diag-field-label", t("slides-diagrams-node-icon")));
        var search = ui.el("input", "sl-diag-icon-search");
        search.type = "search";
        search.setAttribute("placeholder", t("slides-diagrams-icon-search-placeholder"));
        search.setAttribute("aria-label", t("slides-diagrams-icon-search-label"));
        host.appendChild(search);
        var grid = ui.el("div", "sl-diag-icon-grid");
        grid.id = "sl-diag-icon-grid";
        host.appendChild(grid);
        search.addEventListener("input", function () { iconGrid(grid, search.value, node, onEdit); });
        iconGrid(grid, "", node, onEdit);
    }

    function actionStates(dialog, active) {
        if (!dialog) { return; }
        var flowBtn = dialog.querySelector(".sl-diag-btn-flow");
        if (flowBtn && active) {
            flowBtn.textContent = active.flowMode
                ? ui.sdT("slides-diagrams-status-flow-on", "Flowchart connectors on")
                : ui.sdT("slides-diagrams-status-flow-off", "Flowchart connectors off");
        }
        var groupBtn = dialog.querySelector(".sl-diag-btn-group");
        if (groupBtn && active) {
            groupBtn.textContent = active.grouped
                ? t("slides-diagrams-ungroup")
                : t("slides-diagrams-group");
        }
    }

    window.SDDialogViews = {
        nodeList: nodeList,
        nodeEditor: nodeEditor,
        actionStates: actionStates
    };
}());
