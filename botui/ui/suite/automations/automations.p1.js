/* Automations app — part 1: state, helpers, cron preview, schedules API, modal */
"use strict";

const AutoState = {
    schedules: [],
    runs: {},
    expandedRun: null,
    eventSources: {},
    editingId: null,
    pollTimer: null
};

function autoToken() {
    return localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
}

async function autoApi(endpoint, options = {}) {
    const headers = Object.assign({ "Content-Type": "application/json" }, options.headers || {});
    const token = autoToken();
    if (token) headers["Authorization"] = "Bearer " + token;
    const response = await fetch(endpoint, Object.assign({}, options, { headers }));
    if (!response.ok) {
        const body = await response.json().catch(() => ({}));
        throw new Error(body.error || "Request failed (" + response.status + ")");
    }
    return response.json();
}

function autoEsc(value) {
    const div = document.createElement("div");
    div.textContent = value == null ? "" : String(value);
    return div.innerHTML;
}

function autoSetState(message, isError) {
    const el = document.getElementById("auto-state");
    if (!el) return;
    el.textContent = message || "";
    el.classList.toggle("error", !!isError);
}

const AUTO_TIMEZONES = [
    "UTC", "America/Sao_Paulo", "America/New_York", "America/Chicago",
    "America/Los_Angeles", "Europe/London", "Europe/Lisbon", "Europe/Berlin",
    "Europe/Madrid", "Africa/Lagos", "Asia/Dubai", "Asia/Kolkata",
    "Asia/Tokyo", "Asia/Singapore", "Australia/Sydney"
];

const AUTO_DOW_NAMES = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const AUTO_MONTH_NAMES = ["January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"];

function autoParseField(field, domain) {
    const values = new Set();
    for (const part of field.split(",")) {
        let expr = part.trim();
        let step = 1;
        const slash = expr.indexOf("/");
        if (slash !== -1) {
            step = parseInt(expr.slice(slash + 1), 10);
            if (!Number.isFinite(step) || step < 1) return null;
            expr = expr.slice(0, slash);
        }
        let lo = 0;
        let hi = domain - 1;
        if (expr !== "*" && expr !== "") {
            const dash = expr.indexOf("-");
            if (dash !== -1) {
                lo = parseInt(expr.slice(0, dash), 10);
                hi = parseInt(expr.slice(dash + 1), 10);
            } else {
                lo = parseInt(expr, 10);
                hi = step === 1 ? lo : domain - 1;
            }
            if (!Number.isFinite(lo) || !Number.isFinite(hi) || lo < 0 || hi < lo) return null;
        }
        for (let v = lo; v <= hi; v += step) values.add(v % domain);
    }
    return values;
}

function autoCronValues(expr) {
    const fields = String(expr || "").trim().split(/\s+/);
    if (fields.length !== 5) return null;
    const domains = [60, 24, 31, 12, 7];
    const parsed = [];
    for (let i = 0; i < 5; i++) {
        const set = autoParseField(fields[i], domains[i]);
        if (!set || set.size === 0) return null;
        parsed.push([...set].sort((a, b) => a - b));
    }
    return parsed;
}

function autoIsFull(values, size) {
    return values.length === size && values[0] === 0 && values[values.length - 1] === size - 1;
}

function autoStepSize(values, fullSize) {
    if (autoIsFull(values, fullSize)) return 0;
    if (values.length < 2 || values[0] !== 0) return -1;
    const step = values[1] - values[0];
    if (step <= 0) return -1;
    for (let i = 1; i < values.length; i++) {
        if (values[i] !== i * step) return -1;
    }
    return step;
}

function autoPad2(n) { return String(n).padStart(2, "0"); }

function autoDescribeCron(expr) {
    const parts = autoCronValues(expr);
    if (!parts) return "";
    const [mins, hours, doms, months, dows] = parts.map((v, i) => {
        if (i === 4) return v.map(d => (d === 0 ? 7 : d)).sort((a, b) => a - b);
        return v;
    });
    const dowNames = [...new Set(dows)].map(d => AUTO_DOW_NAMES[d - 1]).join(", ");
    const timeText = hours.length === 1 && mins.length === 1
        ? "at " + autoPad2(hours[0]) + ":" + autoPad2(mins[0])
        : "at minute " + mins.join(", ") + " past hour " + hours.join(", ");
    const minStep = autoStepSize(mins, 60);
    if (hours.length === 24 && minStep > 0) {
        return "Every " + minStep + " minutes";
    }
    if (hours.length === 24 && mins.length === 1) {
        return "Hourly at :" + autoPad2(mins[0]);
    }
    if (autoIsFull(doms, 31) && autoIsFull(months, 12) && autoIsFull(dows, 7)) {
        return "Every day " + timeText;
    }
    if (autoIsFull(doms, 31) && autoIsFull(months, 12)) {
        return "Weekly on " + dowNames + " " + timeText;
    }
    if (autoIsFull(months, 12) && dows.length === 7) {
        const dayLabel = autoIsFull(doms, 31)
            ? "every day"
            : "on day " + doms.join(", ") + " of the month";
        return "Monthly — " + dayLabel + " " + timeText;
    }
    if (doms.length === 1 && months.length === 1 && dows.length === 7) {
        return "Yearly on " + AUTO_MONTH_NAMES[months[0]] + " " + doms[0] + " " + timeText;
    }
    return "Runs " + timeText + (dows.length < 7 ? " on " + dowNames : "");
}

function autoFillTimezones() {
    const select = document.getElementById("auto-f-tz");
    if (!select || select.options.length) return;
    for (const tz of AUTO_TIMEZONES) {
        const option = document.createElement("option");
        option.value = tz;
        option.textContent = tz;
        select.appendChild(option);
    }
}

function openScheduleModal(scheduleId) {
    AutoState.editingId = scheduleId || null;
    autoFillTimezones();
    const modal = document.getElementById("auto-modal");
    const form = document.getElementById("auto-form");
    if (!modal || !form) return;
    form.reset();
    document.getElementById("auto-cron-preview").textContent = "";
    const schedule = scheduleId ? AutoState.schedules.find(s => s.id === scheduleId) : null;
    if (schedule) {
        document.getElementById("auto-modal-title").textContent = "Edit Schedule";
        document.getElementById("auto-f-title").value = schedule.title || "";
        document.getElementById("auto-f-goal").value = schedule.goal || "";
        document.getElementById("auto-f-cron").value = schedule.cron_expr || "";
        updateCronPreview();
        const tz = document.getElementById("auto-f-tz");
        if ([...tz.options].some(o => o.value === schedule.timezone)) tz.value = schedule.timezone;
        document.getElementById("auto-f-runtime").value = schedule.max_runtime_secs || 900;
        const delivery = schedule.delivery || {};
        document.getElementById("auto-f-email").checked = delivery.email !== false;
        document.getElementById("auto-f-sms").checked = !!delivery.sms;
        document.getElementById("auto-delete").classList.remove("hidden");
    } else {
        document.getElementById("auto-modal-title").textContent = "New Schedule";
        document.getElementById("auto-delete").classList.add("hidden");
        const localTz = Intl.DateTimeFormat().resolvedOptions().timeZone;
        document.getElementById("auto-f-tz").value = AUTO_TIMEZONES.includes(localTz) ? localTz : "UTC";
    }
    modal.classList.add("open");
    document.getElementById("auto-f-title").focus();
}

function closeScheduleModal() {
    const modal = document.getElementById("auto-modal");
    if (modal) modal.classList.remove("open");
    AutoState.editingId = null;
}

function updateCronPreview() {
    const input = document.getElementById("auto-f-cron");
    const preview = document.getElementById("auto-cron-preview");
    if (!input || !preview) return;
    const description = autoDescribeCron(input.value);
    preview.textContent = description || "Invalid or incomplete cron expression.";
    preview.classList.toggle("invalid", !description);
}

async function saveSchedule(event) {
    event.preventDefault();
    const payload = {
        title: document.getElementById("auto-f-title").value.trim(),
        goal: document.getElementById("auto-f-goal").value.trim(),
        cron_expr: document.getElementById("auto-f-cron").value.trim(),
        timezone: document.getElementById("auto-f-tz").value || "UTC",
        max_runtime_secs: parseInt(document.getElementById("auto-f-runtime").value, 10) || 900,
        delivery: {
            email: document.getElementById("auto-f-email").checked,
            sms: document.getElementById("auto-f-sms").checked
        },
        enabled: true
    };
    if (!payload.title || !payload.goal || !autoCronValues(payload.cron_expr)) {
        autoSetState("Title, goal and a valid cron expression are required.", true);
        return;
    }
    const saveBtn = document.getElementById("auto-save");
    saveBtn.disabled = true;
    try {
        if (AutoState.editingId) {
            await autoApi("/api/automations/schedules/" + encodeURIComponent(AutoState.editingId), {
                method: "PUT",
                body: JSON.stringify(payload)
            });
        } else {
            await autoApi("/api/automations/schedules", { method: "POST", body: JSON.stringify(payload) });
        }
        closeScheduleModal();
        await loadSchedulesAndRender();
    } catch (err) {
        autoSetState("Save failed: " + err.message, true);
    } finally {
        saveBtn.disabled = false;
    }
}

async function deleteSelectedSchedule() {
    if (!AutoState.editingId) return;
    const id = AutoState.editingId;
    if (!window.confirm("Delete this schedule? Its run history will be kept." )) return;
    try {
        await autoApi("/api/automations/schedules/" + encodeURIComponent(id), { method: "DELETE" });
        closeScheduleModal();
        await loadSchedulesAndRender();
    } catch (err) {
        autoSetState("Delete failed: " + err.message, true);
    }
}
