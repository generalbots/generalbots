"use strict";
/**
 * 16b_machine_view.js - Agent machine page: the section list and the one
 * section on screen (heading, live values, controls). State and requests
 * live in 16_machine.js (A.machine), the rail rows in 16c_machine_nav.js.
 */
(function () {
    var A = window.GBAssistant;
    var M = A.machine;
    if (!M) return;
    var t = M.t;
    var SUBTITLE = {
        running: "subRunning", stopped: "subStopped", provisioning: "subProvisioning", error: "subError",
        full: "subFull", unavailable: "subUnavailable", loading: "subLoading", restoring: "subProvisioning",
        starting: "subStarting"
    };
    /* One rail row each. A "live" section keeps the 5 s poll going while it
       is on screen; the others refresh once when they open. */
    M.SECTIONS = [
        {id: "overview", title: "secA", head: "title", lead: "lead", live: true, icon: "M3 4h18v12H3zM8 20h8M12 16v4"},
        {id: "usage", title: "secB", head: "headB", live: true, icon: "M22 12h-4l-3 9L9 3l-3 9H2"},
        {id: "limits", title: "secC", head: "headC", icon: "M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6"},
        {id: "recovery", title: "secD", head: "headD", icon: "M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5"},
        {id: "contents", title: "secE", head: "headE", icon: "M21 8 12 3 3 8v8l9 5 9-5zM3 8l9 5 9-5M12 13v8"},
        {id: "activity", title: "secF", head: "headF", live: true, icon: "M12 7v5l3 3M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z"},
        {id: "care", title: "secG", head: "headG", live: true,
            icon: "M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"}
    ];
    M.current = function () {
        return M.SECTIONS.find(function (s) { return s.id === M.section; }) || M.SECTIONS[0];
    };

    function el(tag, cls, text) {
        var node = document.createElement(tag);
        if (cls) node.className = cls;
        if (text != null) node.textContent = text;
        return node;
    }
    function date(value) { return value ? new Date(value).toLocaleString(A.locale, {dateStyle: "short", timeStyle: "short"}) : null; }
    function button(parent, text, handler, opts) {
        opts = opts || {};
        var btn = el("button", "gb-btn" + (opts.cls ? " " + opts.cls : ""), text);
        btn.type = "button";
        btn.disabled = M.busy || !!opts.disabled;
        if (handler) btn.addEventListener("click", handler);
        parent.appendChild(btn);
        return btn;
    }
    function soon(parent, text, cls) {
        var btn = button(parent, text, null, {disabled: true, cls: cls});
        btn.append(el("span", "gb-machine-soon", t("soon")));
        btn.title = t("soon");
        return btn;
    }
    function confirmThen(text, onYes) {
        var shared = A._modelShared;
        if (shared && typeof shared.inlineConfirm === "function") shared.inlineConfirm(text, onYes);
    }
    function facts(parent, rows) {
        var list = el("dl", "gb-machine-facts");
        rows.filter(function (row) { return row[1] != null && row[1] !== ""; }).forEach(function (row) {
            var cell = el("div");
            cell.append(el("dt", "", row[0]), el("dd", "", String(row[1])));
            list.appendChild(cell);
        });
        if (list.children.length) parent.appendChild(list);
    }
    function row(parent, label, value) {
        var line = el("div", "gb-machine-row");
        line.append(el("span", "", label), el("strong", "", value == null || value === "" ? "—" : String(value)));
        parent.appendChild(line);
    }
    function level(ratio) {
        if (ratio == null) return "";
        return ratio >= 0.85 ? t("high") : ratio >= 0.6 ? t("moderate") : t("light");
    }
    function meter(parent, label, ratio, value) {
        var cell = el("div", "gb-machine-meter");
        var head = el("div", "gb-machine-meter-head");
        head.append(el("span", "", label), el("em", "", level(ratio)));
        var bar = el("div", "gb-machine-bar");
        bar.setAttribute("role", "img");
        bar.setAttribute("aria-label", label + ": " + (ratio == null ? "—" : Math.round(ratio * 100) + "%"));
        if (ratio != null) {
            bar.dataset.level = ratio >= 0.85 ? "danger" : ratio >= 0.6 ? "warn" : "ok";
            var fill = el("i"); fill.style.width = Math.min(100, ratio * 100).toFixed(1) + "%"; bar.appendChild(fill);
        }
        cell.append(head, el("strong", "", value || "—"), bar);
        parent.appendChild(cell);
    }
    function buttons(parent) { var bar = el("div", "gb-machine-buttons"); parent.appendChild(bar); return bar; }

    var FILL = {
        overview: function (panel, model, vm, metrics) {
            var body = el("div", "gb-machine-hero");
            var art = el("div", "gb-machine-art"); art.setAttribute("aria-hidden", "true");
            art.append(el("i"), el("i"), el("i"), el("b"));
            var text = el("div", "gb-machine-hero-text");
            text.append(el("h2", "gb-machine-state", M.stateLabel(model.status)), el("p", "", t(SUBTITLE[model.status] || "subError")));
            if (model.status !== "unavailable") text.appendChild(el("p", "gb-machine-note", t("isolated")));
            var actions = el("div", "gb-machine-actions");
            var canToggle = ["running", "stopped", "full", "error"].includes(model.status);
            var on = model.status !== "stopped";
            var sessionId = A.currentSessionId || A.sessionId;
            if (canToggle && on) soon(actions, t("restart"));
            if (canToggle) {
                var label = on ? t("stop") : t("start");
                button(actions, label, function () {
                    var go = function () {
                        M.operate(label, "/api/agent/sessions/mode?bot_id=" + encodeURIComponent(A.botId || ""),
                            {enabled: !on, session_id: sessionId});
                    };
                    if (on) confirmThen(t("confirmStop"), go); else go();
                }, {cls: "gb-btn-pri", disabled: !sessionId});
            }
            button(actions, t("refresh"), function () { A.refreshMachineStatus(true); });
            body.append(art, text, actions);
            panel.appendChild(body);
            facts(panel, [
                [t("lastActive"), date(vm.last_active_at)],
                [t("processes"), typeof metrics.processes === "number" ? metrics.processes : null],
                [t("arch"), metrics.architecture],
                [t("name"), vm.vm_name]
            ]);
        },
        usage: function (panel, model, vm, metrics) {
            var grid = el("div", "gb-machine-meters");
            var cores = M.cores(metrics.cpu_limit);
            var cpuRatio = model.cpu == null ? null : model.cpu / 100;
            var cpuText = model.cpu == null ? null : (cpuRatio * cores).toLocaleString(A.locale, {maximumFractionDigits: 1}) +
                " " + t("of") + " " + cores + " " + (cores === 1 ? t("core") : t("cores"));
            meter(grid, t("cpu"), cpuRatio, cpuText);
            var memLimit = M.parseSize(metrics.memory_limit);
            var mem = metrics.memory_bytes;
            meter(grid, t("memory"), typeof mem === "number" && memLimit ? mem / memLimit : null,
                typeof mem === "number" ? M.bytes(mem) + (memLimit ? " " + t("of") + " " + M.bytes(memLimit) : "") : null);
            var disk = metrics.disk_bytes, total = metrics.disk_total_bytes || M.parseSize(metrics.disk_limit);
            meter(grid, t("disk"), typeof disk === "number" && total ? disk / total : null,
                typeof disk === "number" ? M.bytes(disk) + (total ? " " + t("of") + " " + M.bytes(total) : "") : null);
            panel.appendChild(grid);
        },
        limits: function (panel, model, vm, metrics) {
            row(panel, t("cpuLimit"), metrics.cpu_limit || (model.row ? t("noLimit") : null));
            row(panel, t("memLimit"), metrics.memory_limit || (model.row ? t("noLimit") : null));
            row(panel, t("diskLimit"), metrics.disk_limit || (model.row ? t("noLimit") : null));
            soon(buttons(panel), t("changeLimits"));
        },
        recovery: function (panel, model, vm, metrics, snaps) {
            if (Array.isArray(snaps)) snaps.forEach(function (s) {
                var line = el("div", "gb-machine-row");
                line.append(el("span", "", s.label || s.incus_snapshot), el("strong", "", date(s.created_at) || "—"));
                button(line, t("restore"), function () {
                    confirmThen(t("confirmRestore"), function () {
                        M.model.status = "restoring";
                        M.operate(t("snapRestored"), "/api/agent/snapshots/" + encodeURIComponent(s.id) + "/restore", {});
                    });
                });
                panel.appendChild(line);
            });
            if (!snaps || !snaps.length) panel.appendChild(el("p", "", snaps ? t("noSnaps") : t("snapsOff")));
            var bar = buttons(panel);
            if (vm.session_id) button(bar, t("createSnap"), function () {
                M.operate(t("snapCreated"), "/api/agent/sessions/" + encodeURIComponent(vm.session_id) + "/snapshots",
                    {label: new Date().toLocaleString(A.locale)});
            });
            soon(bar, t("export"));
        },
        contents: function (panel, model, vm, metrics) {
            row(panel, t("image"), metrics.image);
            var accounts = model.accounts || [];
            accounts.forEach(function (a) { row(panel, a.name || a.id, a.status); });
            if (!accounts.length) panel.appendChild(el("p", "", t("noAccounts")));
            button(buttons(panel), t("configure"), function () { A.openSettings("model"); });
        },
        activity: function (panel, model, vm) {
            row(panel, t("lastActive"), date(vm.last_active_at));
            M.activity.forEach(function (entry) { row(panel, entry.label, new Date(entry.time).toLocaleTimeString(A.locale)); });
            if (!M.activity.length) panel.appendChild(el("p", "", t("noActivity")));
        },
        care: function (panel, model, vm, metrics, snaps) {
            row(panel, t("diskUsed"), M.bytes(metrics.disk_bytes));
            row(panel, t("snapCount"), Array.isArray(snaps) ? snaps.length : null);
            var bar = buttons(panel);
            soon(bar, t("cleanTemp")); soon(bar, t("erase"), "gb-btn-danger");
        }
    };

    /* The line under the heading: what the section's numbers refer to. */
    function meta(id, model) {
        var metrics = model.resources || {}, snaps = model.snapshots;
        if (id === "overview" && metrics.image) return t("image") + ": " + metrics.image;
        if (id === "usage" && model.row) return t("every");
        if (id === "recovery" && Array.isArray(snaps)) return t("snapCount") + ": " + snaps.length;
        return "";
    }

    /* The poll redraws every 5 s: a focused control keeps its focus. */
    function focusedIndex(root) {
        var active = document.activeElement;
        return active && root.contains(active) ? Array.prototype.indexOf.call(root.querySelectorAll("button"), active) : -1;
    }

    M.draw = function () {
        var root = document.getElementById("gbMachineView");
        if (!root) return;
        var model = M.model, section = M.current();
        var scroll = root.scrollTop, focused = focusedIndex(root);
        root.dataset.state = model.status;
        var content = el("div", "gb-machine-content");
        content.dataset.section = section.id;
        content.innerHTML = A.settingsHeading(t(section.title), t(section.head), section.lead ? t(section.lead) : "");
        var line = meta(section.id, model);
        if (line) content.appendChild(el("p", "gb-set-meta gb-machine-meta", line));
        if (model.error) {
            var warning = el("p", "gb-machine-error", model.error);
            warning.setAttribute("role", "alert"); content.appendChild(warning);
        }
        var panel = el("section", "gb-machine-panel");
        panel.setAttribute("aria-label", t(section.title));
        FILL[section.id](panel, model, model.row || {}, model.resources || {}, model.snapshots);
        content.appendChild(panel);
        root.replaceChildren(content);
        root.scrollTop = scroll;
        var again = focused >= 0 && root.querySelectorAll("button")[focused];
        if (again && !again.disabled) again.focus({preventScroll: true});
    };
})();
