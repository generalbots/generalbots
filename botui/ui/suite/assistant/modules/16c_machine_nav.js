"use strict";
/**
 * 16c_machine_nav.js - The rail while the machine page is open, in the
 * Settings pattern: the page name, "Back to conversation" and one numbered
 * row per section. The chosen section is remembered in this browser
 * profile. Arrow keys, Home and End move between the rows.
 */
(function () {
    var A = window.GBAssistant;
    var M = A.machine;
    var host = document.getElementById("gbMachineNav");
    if (!M || !M.SECTIONS || !host) return;
    var KEY = "gb-machine-section";

    function known(id) { return M.SECTIONS.some(function (s) { return s.id === id; }); }
    function stored() {
        try { return localStorage.getItem(KEY); } catch (e) { return null; }
    }
    function remember(id) {
        try { localStorage.setItem(KEY, id); } catch (e) { /* storage unavailable */ }
    }
    M.section = known(stored()) ? stored() : M.SECTIONS[0].id;

    function mark() {
        host.querySelectorAll("[data-msection]").forEach(function (row) {
            var active = row.dataset.msection === M.section;
            row.classList.toggle("active", active);
            if (active) row.setAttribute("aria-current", "page"); else row.removeAttribute("aria-current");
        });
    }

    M.drawNav = function () {
        var esc = A.settingsEsc, icon = A.settingsIcon;
        var html = '<div class="gb-set-nav"><strong class="gb-mach-label">' + esc(M.t("machine")) + '</strong>' +
            '<button type="button" class="gb-set-back">' + icon("m15 6-6 6 6 6", 14, 3) +
            '<span>' + esc(A.t("settings.back", "Back to conversation")) + '</span></button>';
        M.SECTIONS.forEach(function (section, index) {
            html += '<button type="button" class="gb-set-tab" data-msection="' + section.id +
                '"><i class="gb-set-gutter" aria-hidden="true"></i><span class="gb-set-tab-in" data-num="0' + (index + 1) + '">' +
                icon(section.icon, 15) + '<span class="gb-set-tab-label">' + esc(M.t(section.title)) + '</span></span></button>';
        });
        host.innerHTML = html + '</div>';
        mark();
    };

    /* Show one section: a live one resumes the poll, the others ask once. */
    M.select = function (id) {
        M.section = known(id) ? id : M.SECTIONS[0].id;
        remember(M.section);
        mark();
        if (A.updateBreadcrumb) A.updateBreadcrumb();
        var root = document.getElementById("gbMachineView");
        if (root) root.scrollTop = 0;
        M.draw();
        if (M.current().live) M.wake(); else A.refreshMachineStatus(true);
    };

    host.addEventListener("click", function (e) {
        var row = e.target.closest("button");
        if (!row || !host.contains(row)) return;
        if (row.classList.contains("gb-set-back")) A.switchView("chat");
        else if (row.dataset.msection) M.select(row.dataset.msection);
    });
    host.addEventListener("keydown", function (e) {
        var step = {ArrowDown: 1, ArrowUp: -1}[e.key];
        if ((!step && e.key !== "Home" && e.key !== "End") || e.altKey || e.ctrlKey || e.metaKey) return;
        var rows = Array.prototype.slice.call(host.querySelectorAll("button"));
        var at = rows.indexOf(document.activeElement);
        if (at < 0) return;
        e.preventDefault();
        var next = e.key === "Home" ? 0 : e.key === "End" ? rows.length - 1 : (at + step + rows.length) % rows.length;
        rows[next].focus();
    });
})();
