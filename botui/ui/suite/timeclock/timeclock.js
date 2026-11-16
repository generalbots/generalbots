if (window.GBAppLifecycle) GBAppLifecycle.begin("timeclock");
"use strict";

(function() {
    function getTabs() {
        return Array.prototype.slice.call(document.querySelectorAll('.tc-tabs [data-tab-trigger]'));
    }
    function activateTab(btn, focus) {
        var tabs = getTabs();
        tabs.forEach(function(b) {
            var isActive = b === btn;
            b.classList.toggle("active", isActive);
            b.setAttribute("aria-selected", isActive ? "true" : "false");
            b.tabIndex = isActive ? 0 : -1;
        });
        var panel = document.getElementById("tc-content");
        if (panel && btn && btn.id) panel.setAttribute("aria-labelledby", btn.id);
        if (focus && btn) btn.focus();
    }
    document.body.addEventListener("click", function(e) {
        var t = e.target.closest(".tc-tabs [data-tab-trigger]");
        if (t) activateTab(t, false);
    });
    document.body.addEventListener("keydown", function(e) {
        var current = e.target.closest(".tc-tabs [data-tab-trigger]");
        if (!current) return;
        var tabs = getTabs();
        var idx = tabs.indexOf(current);
        if (idx === -1) return;
        var next = null;
        if (e.key === "ArrowRight") next = tabs[(idx + 1) % tabs.length];
        else if (e.key === "ArrowLeft") next = tabs[(idx - 1 + tabs.length) % tabs.length];
        else if (e.key === "Home") next = tabs[0];
        else if (e.key === "End") next = tabs[tabs.length - 1];
        if (next) {
            e.preventDefault();
            next.click();
            activateTab(next, true);
        }
    });

    function tick() {
        var c = document.getElementById("tc-clock");
        var d = document.getElementById("tc-date");
        var z = document.getElementById("tc-zone");
        var now = new Date();
        if (c) {
            var h = String(now.getHours()).padStart(2, "0");
            var m = String(now.getMinutes()).padStart(2, "0");
            var s = String(now.getSeconds()).padStart(2, "0");
            c.textContent = h + ":" + m + ":" + s;
        }
        if (d) {
            try {
                var opts = { weekday: "long", year: "numeric", month: "long", day: "numeric" };
                d.textContent = gbFormatDate(now, opts);
            } catch (e) {
                d.textContent = now.toDateString();
            }
        }
        if (z && !z.textContent) {
            // #2404 — the clock is the payroll surface: name the timezone and
            // the UTC offset instead of showing an ambiguous local time.
            var tz = "UTC";
            var offsetMin = -now.getTimezoneOffset();
            try { tz = Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"; } catch (e) { tz = "UTC"; }
            var sign = offsetMin < 0 ? "-" : "+";
            var abs = Math.abs(offsetMin);
            z.textContent = tz + " (UTC" + sign + String(Math.floor(abs / 60)).padStart(2, "0") + ":" + String(abs % 60).padStart(2, "0") + ")";
        }
    }
    tick();
    if (window.GBAppLifecycle && GBAppLifecycle.interval) {
        GBAppLifecycle.interval("timeclock", tick, 1000);
    } else {
        setInterval(tick, 1000);
    }

    function openOtModal() {
        var m = document.getElementById("tc-ot-modal");
        if (m) {
            m.hidden = false;
            var first = m.querySelector("input, textarea, button");
            if (first) first.focus();
        }
    }
    function closeOtModal() {
        var m = document.getElementById("tc-ot-modal");
        if (m) m.hidden = true;
    }
    function t(key, fallback) {
        return window.gbT ? window.gbT(key, fallback) : fallback;
    }
    function setStatus(message, isError) {
        var el = document.getElementById("tc-status");
        if (!el) return;
        el.textContent = message || "";
        el.classList.toggle("tc-status-error", !!isError);
    }
    document.body.addEventListener("click", function(e) {
        var backdrop = e.target.closest(".tc-modal");
        if (backdrop && e.target === backdrop) backdrop.hidden = true;
    });
    document.addEventListener("keydown", function(e) {
        if (e.key !== "Escape") return;
        var m = document.getElementById("tc-ot-modal");
        if (m && !m.hidden) closeOtModal();
    });
    var openBtn = document.getElementById("tc-ot-open");
    var cancelBtn = document.getElementById("tc-ot-cancel");
    var otForm = document.getElementById("tc-ot-form");
    if (openBtn) openBtn.addEventListener("click", openOtModal);
    if (cancelBtn) cancelBtn.addEventListener("click", closeOtModal);

    function submitOtForm(event) {
        if (event) event.preventDefault();
        var form = event.target;
        var data = {};
        Array.prototype.forEach.call(form.elements, function(el) {
            if (el.name && !el.disabled) data[el.name] = el.value;
        });
        // #2404 — stamp the request with the client's zone/offset so the server
        // can resolve the real instant instead of trusting a bare local date.
        data.client_timezone = (function() {
            try { return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"; } catch (e) { return "UTC"; }
        })();
        data.client_offset_minutes = -new Date().getTimezoneOffset();
        fetch("/api/timeclock/forms/overtime", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(data)
        })
            .then(function(r) {
                if (!r.ok) throw new Error("HTTP " + r.status);
                return r.json();
            })
            .then(function(resp) {
                if (resp && resp.ok) {
                    form.closest(".tc-modal").hidden = true;
                    form.reset();
                    setStatus(t("timeclock-requested", "Overtime request sent."), false);
                    if (window.htmx) htmx.trigger("#tc-content", "load");
                } else {
                    setStatus(t("timeclock-send-failed", "Could not send the request.") + " " + ((resp && resp.error) || ""), true);
                }
            })
            .catch(function(err) {
                setStatus(t("timeclock-network-error", "Network error. Please try again."), true);
            });
        return false;
    }
    if (otForm) otForm.addEventListener("submit", submitOtForm);
})();
