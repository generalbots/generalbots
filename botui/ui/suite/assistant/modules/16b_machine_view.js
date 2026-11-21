"use strict";
/**
 * 16b_machine_view.js - Agent machine page: hero card, section chips, the
 * lettered sections A-G and the sidebar section list. State and requests
 * live in 16_machine.js (A.machine).
 */
(function () {
    var A = window.GBAssistant;
    var M = A.machine;
    if (!M) return;
    var t = M.t;
    var LETTERS = ["A", "B", "C", "D", "E", "F", "G"];
    var TITLES = ["secA", "secB", "secC", "secD", "secE", "secF", "secG"];
    var SUBTITLE = {
        running: "subRunning", stopped: "subStopped", provisioning: "subProvisioning", error: "subError",
        full: "subFull", unavailable: "subUnavailable", loading: "subLoading", restoring: "subProvisioning",
        starting: "subStarting"
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

    function hero(panel, model, vm, metrics) {
        var body = el("div", "gb-machine-hero");
        var art = el("div", "gb-machine-art"); art.setAttribute("aria-hidden", "true");
        art.append(el("i"), el("i"), el("i"), el("b"));
        var text = el("div", "gb-machine-hero-text");
        var state = el("h3", "gb-machine-state", M.stateLabel(model.status));
        text.append(state, el("p", "", t(SUBTITLE[model.status] || "subError")));
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
    }

    function usage(panel, model, metrics) {
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
    }

    function sections(model) {
        var vm = model.row || {}, metrics = model.resources || {};
        var snaps = model.snapshots;
        var meta = [metrics.image, model.row ? t("every") : "", "", Array.isArray(snaps) ? String(snaps.length) : "",
            "", "", ""];
        var fill = [
            function (p) { hero(p, model, vm, metrics); },
            function (p) { usage(p, model, metrics); },
            function (p) {
                row(p, t("cpuLimit"), metrics.cpu_limit || (model.row ? t("noLimit") : null));
                row(p, t("memLimit"), metrics.memory_limit || (model.row ? t("noLimit") : null));
                row(p, t("diskLimit"), metrics.disk_limit || (model.row ? t("noLimit") : null));
                var bar = el("div", "gb-machine-buttons"); soon(bar, t("changeLimits")); p.appendChild(bar);
            },
            function (p) {
                if (Array.isArray(snaps)) snaps.forEach(function (s) {
                    var line = el("div", "gb-machine-row");
                    line.append(el("span", "", s.label || s.incus_snapshot), el("strong", "", date(s.created_at) || "—"));
                    button(line, t("restore"), function () {
                        confirmThen(t("confirmRestore"), function () {
                            M.model.status = "restoring";
                            M.operate(t("snapRestored"), "/api/agent/snapshots/" + encodeURIComponent(s.id) + "/restore", {});
                        });
                    });
                    p.appendChild(line);
                });
                if (!snaps || !snaps.length) p.appendChild(el("p", "", snaps ? t("noSnaps") : t("snapsOff")));
                var bar = el("div", "gb-machine-buttons");
                if (vm.session_id) button(bar, t("createSnap"), function () {
                    M.operate(t("snapCreated"), "/api/agent/sessions/" + encodeURIComponent(vm.session_id) + "/snapshots",
                        {label: new Date().toLocaleString(A.locale)});
                });
                soon(bar, t("export"));
                p.appendChild(bar);
            },
            function (p) {
                row(p, t("image"), metrics.image);
                var accounts = model.accounts || [];
                accounts.forEach(function (a) { row(p, a.name || a.id, a.status); });
                if (!accounts.length) p.appendChild(el("p", "", t("noAccounts")));
                var bar = el("div", "gb-machine-buttons");
                button(bar, t("configure"), function () { A.openSettings("model"); });
                p.appendChild(bar);
            },
            function (p) {
                row(p, t("lastActive"), date(vm.last_active_at));
                M.activity.forEach(function (entry) { row(p, entry.label, new Date(entry.time).toLocaleTimeString(A.locale)); });
                if (!M.activity.length) p.appendChild(el("p", "", t("noActivity")));
            },
            function (p) {
                row(p, t("diskUsed"), M.bytes(metrics.disk_bytes));
                row(p, t("snapCount"), Array.isArray(snaps) ? snaps.length : null);
                var bar = el("div", "gb-machine-buttons");
                soon(bar, t("cleanTemp")); soon(bar, t("erase"), "gb-btn-danger");
                p.appendChild(bar);
            }
        ];
        return {meta: meta, fill: fill};
    }

    function letter(text) { return el("span", "gb-letter", text); }

    M.draw = function () {
        var root = document.getElementById("gbMachineView"), nav = document.getElementById("gbMachineNav");
        if (!root || !nav) return;
        var model = M.model;
        var scroll = root.scrollTop;
        root.replaceChildren(); nav.replaceChildren();
        root.dataset.state = model.status;
        var content = el("div", "gb-machine-content"); root.appendChild(content);
        content.append(el("small", "gb-machine-kicker", t("kicker")), el("h1", "", t("title")), el("p", "gb-machine-lead", t("lead")));
        if (model.error) {
            var warning = el("p", "gb-machine-error", model.error);
            warning.setAttribute("role", "alert"); content.appendChild(warning);
        }
        var tray = el("div", "gb-machine-tray"); content.appendChild(tray);
        var chips = el("div", "gb-machine-chips");
        chips.appendChild(el("span", "gb-machine-chips-label", t("sections")));
        tray.appendChild(chips);
        var list = el("div", "gb-machine-list"); tray.appendChild(list);
        nav.appendChild(el("span", "gb-strip", t("sections")));
        var built = sections(model);
        LETTERS.forEach(function (key, i) {
            var title = t(TITLES[i]);
            var panel = el("section", "gb-machine-panel"); panel.id = "gbMachineSection" + key;
            var heading = el("h2");
            heading.append(letter(key), el("span", "gb-machine-panel-title", title));
            if (built.meta[i]) heading.appendChild(el("small", "", built.meta[i]));
            panel.appendChild(heading);
            built.fill[i](panel);
            list.appendChild(panel);
            function jump() { root.scrollTop += panel.getBoundingClientRect().top - root.getBoundingClientRect().top - 8; }
            var chip = el("button", "gb-machine-chip"); chip.type = "button";
            chip.append(letter(key), document.createTextNode(title));
            chip.addEventListener("click", jump); chips.appendChild(chip);
            var link = el("button", "gb-machine-jump"); link.type = "button";
            link.append(letter(key), document.createTextNode(title));
            link.addEventListener("click", jump); nav.appendChild(link);
        });
        root.scrollTop = scroll;
    };
})();
