"use strict";

(function () {
    var A = window.GBAssistant;
    function id(name) { return document.getElementById(name); }

    A.updateBreadcrumb = function () {
        var screen = A.currentView;
        var section = screen === "settings" ? A.t("assistant.settings") :
            screen === "machine" ? A.t("shell.machine") : screen === "drive" ? "Drive" : "Chat";
        var active;
        if (screen === "settings") active = id("gbSettingsNav").querySelector(".active, [aria-current=page]");
        if (screen === "drive") active = id("gbDriveNav").querySelector(".active");
        var current = active ? active.textContent.trim() : section;
        if (screen === "chat") {
            var conversation = A.sessions.find(function (s) { return s.id === A.currentSessionId; });
            current = (conversation && conversation.title) || A.t("assistant.new-chat");
        }
        if (screen === "machine") current = A.t("shell.workspace");
        id("gbCrumbSection").textContent = section;
        id("gbCrumbCurrent").textContent = current;
    };

    A.renderEdition = function () {
        var version = A.productConfig && A.productConfig.version;
        if (version && id("gbBuildVersion")) id("gbBuildVersion").textContent = "v" + version.replace(/^v/i, "");
    };

    /* Machine row: mini processor chart. One sample every 10 s while the
       window is visible and the sidebar open; the per-poll share comes from
       the cpu_ns delta. Hidden while the workspace is absent or unavailable. */
    var CPU_EVERY = 10000, CPU_IDLE = 60000, CPU_BARS = 14;
    var cpu = { hist: [], last: null, timer: 0, polled: 0, off: false };
    function cpuShown() {
        var bar = id("gb-sidebar");
        return !document.hidden && !!bar && !bar.classList.contains("collapsed");
    }
    function cpuCores(limit) {
        var text = String(limit == null ? "" : limit).trim(), range = /^(\d+)-(\d+)$/.exec(text);
        if (/^\d+$/.test(text) && +text > 0) return +text;
        if (range) return +range[2] - +range[1] + 1;
        return navigator.hardwareConcurrency || 1;
    }
    function drawCpu() {
        var box = id("gbRailCpu");
        if (!box) return;
        box.hidden = cpu.hist.length === 0;
        box.replaceChildren.apply(box, cpu.hist.map(function (value, i, all) {
            var bar = document.createElement("b");
            bar.style.height = (3 + Math.round(value / 100 * 11)) + "px";
            if (i === all.length - 1) bar.className = "now";
            return bar;
        }));
        var pt = A.locale && A.locale.startsWith("pt"), now = cpu.hist[cpu.hist.length - 1];
        box.title = cpu.hist.length ? (pt ? "Processador: " : "Processor: ") + now + "% · " +
            (pt ? "últimos " : "last ") + Math.round(cpu.hist.length * CPU_EVERY / 1000) + " s" : "";
    }
    function scheduleCpu(delay) {
        clearTimeout(cpu.timer);
        cpu.timer = cpu.off || !cpuShown() ? 0 : setTimeout(sampleCpu, delay);
    }
    async function sampleCpu() {
        cpu.timer = 0;
        var cfg = A.productConfig;
        if (!cfg) { scheduleCpu(3000); return; }
        if (!(cfg.compiled_features || []).includes("agent-vm") || cfg.agent_backend_unavailable) {
            cpu.off = true; cpu.hist = []; drawCpu(); return;
        }
        if (!cpuShown()) return;
        var delay = CPU_EVERY;
        cpu.polled = performance.now();
        try {
            var res = await A.apiFetch("/api/agent/vm/resources");
            var data = res.ok ? await res.json() : null, at = performance.now();
            if (data && typeof data.cpu_ns === "number") {
                if (cpu.last && data.cpu_ns >= cpu.last.ns && at > cpu.last.at) {
                    var share = (data.cpu_ns - cpu.last.ns) / ((at - cpu.last.at) * 1e6 * cpuCores(data.cpu_limit)) * 100;
                    cpu.hist = cpu.hist.concat(Math.max(0, Math.min(100, Math.round(share)))).slice(-CPU_BARS);
                }
                cpu.last = { ns: data.cpu_ns, at: at };
            } else { cpu.hist = []; cpu.last = null; delay = CPU_IDLE; }
        } catch (err) { cpu.hist = []; cpu.last = null; delay = CPU_IDLE; }
        drawCpu();
        scheduleCpu(delay);
    }
    function resumeCpu() {
        if (!cpuShown()) { clearTimeout(cpu.timer); cpu.timer = 0; return; }
        if (!cpu.timer && !cpu.off) scheduleCpu(Math.max(0, CPU_EVERY - (performance.now() - cpu.polled)));
    }
    A.startRailCpu = function () {
        document.addEventListener("visibilitychange", resumeCpu);
        new MutationObserver(resumeCpu).observe(id("gb-sidebar"), { attributes: true, attributeFilter: ["class"] });
        scheduleCpu(0);
    };

    /* Status bar: server port, interface language, clock and date. */
    function initStatusBar() {
        var port = location.port;
        try { port = new URL(A.origin || location.href, location.href).port || port; } catch (err) { port = location.port; }
        id("gbStatusPort").textContent = "botserver" + (port ? " :" + port : "");
        id("gbStatusLang").textContent = String(A.locale || document.documentElement.lang || "").toUpperCase();
        var crumb = document.querySelector(".gb-breadcrumb");
        if (crumb) {
            new MutationObserver(function () {
                var meta = crumb.querySelector('.gb-crumb-meta[data-for="drive"]');
                id("gbStatusUsage").textContent = meta && !meta.hidden ? meta.textContent : "";
            }).observe(crumb, {subtree: true, childList: true, characterData: true, attributes: true, attributeFilter: ["hidden"]});
        }
        function pad(n) { return String(n).padStart(2, "0"); }
        (function tick() {
            var now = new Date();
            id("gbStatusClock").textContent = pad(now.getHours()) + ":" + pad(now.getMinutes());
            id("gbStatusDate").textContent = pad(now.getDate()) + "/" + pad(now.getMonth() + 1) + "/" + now.getFullYear();
            setTimeout(tick, 60000 - now.getSeconds() * 1000 - now.getMilliseconds() + 50);
        })();
    }

    function dockExportActions() {
        var crumb = document.querySelector(".gb-breadcrumb");
        var bar = id("gbTitlebarTitle");
        if (!crumb || !bar) return;
        var dock = new MutationObserver(function () {
            var actions = id("gbExportActions");
            if (!actions) return;
            dock.disconnect();
            crumb.appendChild(actions);
        });
        dock.observe(bar.parentNode, {childList: true});
    }

    A.initMantisShell = function () {
        document.documentElement.lang = A.locale;
        document.querySelectorAll("[data-icon]").forEach(function (el) {
            el.innerHTML = window.GBIcon(el.dataset.icon, 17);
        });
        var dialog = id("gbAppearanceDialog");
        id("gbNavAppearance").addEventListener("click", function () {
            A.renderMantisAppearance(id("gbAppearanceContent"));
            dialog.showModal();
        });
        id("gbAppearanceClose").addEventListener("click", function () { dialog.close(); });
        dialog.addEventListener("click", function (event) {
            if (event.target !== dialog) return;
            var rect = dialog.getBoundingClientRect();
            if (event.clientX < rect.left || event.clientX > rect.right ||
                event.clientY < rect.top || event.clientY > rect.bottom) dialog.close();
        });
        var observer = new MutationObserver(A.updateBreadcrumb);
        ["gbSettingsNav", "gbDriveNav", "gbConversations", "gbTitlebarTitle"].forEach(function (name) {
            if (id(name)) observer.observe(id(name), {subtree: true, childList: true,
                attributes: true, attributeFilter: ["class", "aria-current"]});
        });
        dockExportActions();
        A.updateBreadcrumb();
        A.startRailCpu();
        initStatusBar();
    };
})();
