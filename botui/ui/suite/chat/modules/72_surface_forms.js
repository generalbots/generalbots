/**
 * Tier 1 surface renderer (#1669) — form, plan, diff and tabs components.
 * Split from 72_surface_renderer.js for #1731; helpers come from
 * window.SurfaceRenderer, created by the previous file.
 */
(function () {
    "use strict";

    var SR = window.SurfaceRenderer;
    var element = SR._element;
    var isObject = SR._isObject;

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
                    panel.appendChild(SR._renderComponent(child, context, onSubmit));
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


    SR._renderForm = renderForm;
    SR._renderPlan = renderPlan;
    SR._renderDiff = renderDiff;
    SR._renderTabs = renderTabs;
})();
