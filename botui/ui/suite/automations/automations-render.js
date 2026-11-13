/* Automations app — part 2: board rendering, runs, SSE checklist, polling */
"use strict";

function autoStatusClass(status) {
    return "st-" + String(status || "queued").toLowerCase().replace(/[^a-z]/g, "");
}

function autoFmtElapsed(startedAt, finishedAt) {
    if (!startedAt) return "";
    const end = finishedAt ? new Date(finishedAt) : new Date();
    const secs = Math.max(0, Math.round((end - new Date(startedAt)) / 1000));
    if (secs < 60) return secs + "s";
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    if (m < 60) return m + "m " + s + "s";
    return Math.floor(m / 60) + "h " + (m % 60) + "m";
}

function autoFmtWhen(value) {
    if (!value) return "";
    const d = new Date(value);
    return isNaN(d.getTime()) ? "" : gbFormatDateTime(d);
}

async function loadSchedulesAndRender() {
    try {
        const data = await autoApi("/api/automations/schedules");
        AutoState.schedules = data.items || data.schedules || [];
        await loadRuns();
    } catch (err) {
        autoSetState("Failed to load schedules: " + err.message, true);
        renderBoard();
    }
    await loadConversationJobs();
    renderConversationJobs();
}

async function loadRuns() {
    try {
        const data = await autoApi("/api/automations/runs");
        const runs = data.runs || data.items || [];
        AutoState.runs = {};
        for (const run of runs) {
            const key = run.schedule_id || "_unassigned";
            if (!AutoState.runs[key]) AutoState.runs[key] = [];
            AutoState.runs[key].push(run);
        }
        autoSetState("");
        renderBoard();
    } catch (err) {
        autoSetState("Failed to load runs: " + err.message, true);
        renderBoard();
    }
}

async function loadConversationJobs() {
    try {
        const data = await autoApi("/api/conversation-autotasks");
        AutoState.conversationJobs = data.autotasks || data.items || [];
    } catch (err) {
        AutoState.conversationJobs = [];
    }
}

function autoConversationJobHtml(job) {
    const status = job.status || "pending";
    const isActive = status === "running" || status === "scheduled" || status === "pending";
    let html = '<div class="auto-run-card ' + autoStatusClass(status) + '" data-job-id="' + autoEsc(job.id) + '">';
    html += '<div class="auto-run-top">';
    html += '<span class="auto-run-status">' + autoEsc(status) + "</span>";
    html += '<span class="auto-run-trigger">' + autoEsc(job.kind || "") + " · " + autoEsc(job.title || "") + "</span>";
    html += "</div>";
    if (job.schedule) html += '<div class="auto-run-summary">' + autoEsc(job.schedule) + (job.schedule_human ? " — " + autoEsc(job.schedule_human) : "") + "</div>";
    if (job.last_error) html += '<div class="auto-run-error">' + autoEsc(job.last_error) + "</div>";
    html += '<div class="auto-run-actions">';
    if (isActive) {
        html += '<button type="button" class="auto-link-btn danger" data-auto-cancel-job="' + autoEsc(job.id) + '">Cancel</button>';
    }
    html += '<button type="button" class="auto-link-btn" data-auto-open-conversation="' + autoEsc(job.conversation_id) + '">Open conversation</button>';
    html += '<button type="button" class="auto-link-btn danger" data-auto-delete-job="' + autoEsc(job.id) + '">Delete</button>';
    html += "</div></div>";
    return html;
}

function renderConversationJobs() {
    const container = document.getElementById("auto-conversation-jobs");
    if (!container) return;
    const jobs = AutoState.conversationJobs || [];
    if (!jobs.length) {
        container.innerHTML = '<div class="auto-empty">No conversation jobs.</div>';
        return;
    }
    container.innerHTML = jobs.map(autoConversationJobHtml).join("");
}

function renderBoard() {
    const board = document.getElementById("auto-board");
    if (!board) return;
    document.getElementById("auto-count").textContent =
        AutoState.schedules.length + " schedule" + (AutoState.schedules.length === 1 ? "" : "s");
    if (!AutoState.schedules.length) {
        board.innerHTML = '<div class="auto-empty">No schedules yet. Create one with the New Schedule button.</div>';
        return;
    }
    board.innerHTML = AutoState.schedules.map(autoLaneHtml).join("");
}

function autoRunCardHtml(run) {
    const status = run.status || "queued";
    const isRunning = status === "running";
    const elapsed = autoFmtElapsed(run.started_at, run.finished_at);
    let html = '<div class="auto-run-card ' + autoStatusClass(status) + '" data-run-id="' + autoEsc(run.id) + '">';
    html += '<div class="auto-run-top">';
    html += '<span class="auto-run-status">' + autoEsc(status) + "</span>";
    html += '<span class="auto-run-trigger">' + autoEsc(run.trigger_kind || "") + " · " + autoEsc(autoFmtWhen(run.created_at)) + "</span>";
    if (elapsed) html += '<span class="auto-run-time" data-elapsed-start="' + autoEsc(run.started_at || "") + '">' + autoEsc(elapsed) + "</span>";
    html += "</div>";
    if (run.result_summary) html += '<div class="auto-run-summary">' + autoEsc(run.result_summary) + "</div>";
    if (run.error) html += '<div class="auto-run-error">' + autoEsc(run.error) + "</div>";
    html += '<div class="auto-run-actions">';
    if (isRunning || status === "queued") {
        html += '<button type="button" class="auto-link-btn danger" data-auto-cancel="' + autoEsc(run.id) + '">Cancel</button>';
    } else {
        html += '<button type="button" class="auto-link-btn" data-auto-expand="' + autoEsc(run.id) + '">Steps</button>';
    }
    html += "</div>";
    html += '<div class="auto-steps hidden" data-steps-for="' + autoEsc(run.id) + '"></div>';
    html += "</div>";
    return html;
}

function autoLaneHtml(schedule) {
    const delivery = schedule.delivery || {};
    const channels = [];
    if (delivery.email !== false && delivery.email) channels.push("email");
    if (delivery.sms) channels.push("SMS");
    if (Array.isArray(delivery.channels)) channels.push(...delivery.channels);
    const runs = (AutoState.runs[schedule.id] || []).slice(0, 8);
    let html = '<section class="auto-lane" aria-label="Schedule ' + autoEsc(schedule.title) + '">';
    html += '<div class="auto-lane-head">';
    html += '<div class="auto-lane-title-row">';
    html += '<h4 class="auto-lane-name">' + autoEsc(schedule.title) + "</h4>";
    html += '<span class="auto-enabled-badge' + (schedule.enabled ? "" : " off") + '">' + (schedule.enabled ? "Active" : "Paused") + "</span>";
    html += "</div>";
    html += '<div class="auto-lane-meta">';
    html += '<span class="auto-lane-cron">' + autoEsc(schedule.cron_expr) + "</span>";
    html += "<span>" + autoEsc(autoDescribeCron(schedule.cron_expr)) + "</span>";
    html += "<span>" + autoEsc(schedule.timezone || "UTC") + "</span>";
    if (schedule.next_run_at) html += "<span>next: " + autoEsc(autoFmtWhen(schedule.next_run_at)) + "</span>";
    if (channels.length) html += "<span>via " + autoEsc(channels.join(", ")) + "</span>";
    html += "</div>";
    html += '<div class="auto-lane-actions">';
    html += '<button type="button" class="auto-btn small" data-auto-run-now="' + autoEsc(schedule.id) + '">Run now</button>';
    html += '<button type="button" class="auto-btn small ghost" data-auto-toggle-enable="' + autoEsc(schedule.id) + '">' + (schedule.enabled ? "Pause" : "Resume") + "</button>";
    html += '<button type="button" class="auto-btn small ghost" data-auto-edit="' + autoEsc(schedule.id) + '">Edit</button>';
    html += "</div></div>";
    html += '<div class="auto-lane-body">';
    html += runs.length
        ? runs.map(autoRunCardHtml).join("")
        : '<div class="auto-lane-empty">No recent runs.</div>';
    html += "</div></section>";
    return html;
}

function renderStepsFor(runId, steps) {
    const container = document.querySelector('[data-steps-for="' + runId + '"]');
    if (!container) return null;
    container.innerHTML = steps.map(function (step) {
        const cls = step.status === "ok" ? "done" : (step.status === "error" ? "failed" : "running");
        const tokens = step.tokens_out != null ? '<span class="auto-step-tokens">' + autoEsc(step.tokens_in + "/" + step.tokens_out) + "</span>" : "";
        return '<div class="auto-step ' + cls + '"><span class="auto-step-mark"></span>' +
            '<span class="auto-step-name">' + autoEsc(step.name || step.kind || "step") + "</span>" + tokens + "</div>";
    }).join("");
    container.classList.remove("hidden");
    return container;
}

function closeAllEventSources() {
    for (const id of Object.keys(AutoState.eventSources)) {
        AutoState.eventSources[id].close();
        delete AutoState.eventSources[id];
    }
}

function openRunEvents(runId) {
    if (AutoState.expandedRun === runId) {
        closeAllEventSources();
        AutoState.expandedRun = null;
        const container = document.querySelector('[data-steps-for="' + runId + '"]');
        if (container) container.classList.add("hidden");
        return;
    }
    closeAllEventSources();
    AutoState.expandedRun = runId;
    renderStepsFor(runId, [{ name: "Waiting for events…", status: "running" }]);
    const token = autoToken();
    const url = "/api/automations/runs/" + encodeURIComponent(runId) + "/events" + (token ? "?token=" + encodeURIComponent(token) : "");
    const source = new EventSource(url);
    AutoState.eventSources[runId] = source;
    source.onmessage = function (event) {
        let payload;
        try { payload = JSON.parse(event.data); } catch (e) { return; }
        if (payload.run && payload.run.status) updateRunCardStatus(payload.run);
        const span = payload.span || (payload.type === "span" ? payload : null);
        if (!span) return;
        const container = document.querySelector('[data-steps-for="' + runId + '"]');
        if (!container) return;
        if (container.dataset.loaded !== "1") {
            container.dataset.loaded = "1";
            container.innerHTML = "";
        }
        const existing = container.querySelectorAll(".auto-step").length;
        const temp = document.createElement("div");
        temp.innerHTML = '<div class="auto-step"><span class="auto-step-mark"></span><span class="auto-step-name"></span></div>';
        const row = temp.firstElementChild;
        row.className = "auto-step " + (span.status === "ok" ? "done" : (span.status === "error" ? "failed" : "running"));
        row.querySelector(".auto-step-name").textContent = span.name || span.kind || ("step " + (existing + 1));
        if (span.tokens_out != null) {
            const t = document.createElement("span");
            t.className = "auto-step-tokens";
            t.textContent = span.tokens_in + "/" + span.tokens_out;
            row.appendChild(t);
        }
        container.appendChild(row);
    };
    source.onerror = function () {
        source.close();
        delete AutoState.eventSources[runId];
    };
}

function updateRunCardStatus(run) {
    const card = document.querySelector('.auto-run-card[data-run-id="' + run.id + '"]');
    if (!card) return;
    card.className = "auto-run-card " + autoStatusClass(run.status);
    const badge = card.querySelector(".auto-run-status");
    if (badge) badge.textContent = run.status;
    if (run.error) {
        let errEl = card.querySelector(".auto-run-error");
        if (!errEl) {
            errEl = document.createElement("div");
            errEl.className = "auto-run-error";
            card.appendChild(errEl);
        }
        errEl.textContent = run.error;
    }
}

async function cancelRun(runId) {
    try {
        await autoApi("/api/automations/runs/" + encodeURIComponent(runId) + "/cancel", { method: "POST", body: "{}" });
        await loadRuns();
    } catch (err) {
        autoSetState("Cancel failed: " + err.message, true);
    }
}

async function cancelConversationJob(jobId) {
    try {
        await autoApi("/api/conversation-autotasks/" + encodeURIComponent(jobId) + "/cancel", { method: "POST", body: "{}" });
        await loadConversationJobs();
        renderConversationJobs();
    } catch (err) {
        autoSetState("Cancel failed: " + err.message, true);
    }
}

async function deleteConversationJob(jobId) {
    if (!(window.GBModal || { confirm: confirm }).confirm("Delete this conversation job?")) return;
    try {
        await autoApi("/api/conversation-autotasks/" + encodeURIComponent(jobId), { method: "DELETE" });
        await loadConversationJobs();
        renderConversationJobs();
    } catch (err) {
        autoSetState("Delete failed: " + err.message, true);
    }
}

function openConversationJob(conversationId) {
    if (!conversationId) return;
    if (window.GBSidebarConvos && typeof window.GBSidebarConvos.openConversation === "function") {
        window.GBSidebarConvos.openConversation(conversationId);
        return;
    }
    if (window.GBSidebar && typeof window.GBSidebar.openConversation === "function") {
        window.GBSidebar.openConversation(conversationId);
        return;
    }
    if (typeof window.openDeepLink === "function") {
        window.openDeepLink("chat", { session: conversationId });
    }
}

async function runScheduleNow(scheduleId) {
    try {
        await autoApi("/api/automations/schedules/" + encodeURIComponent(scheduleId) + "/run", { method: "POST", body: "{}" });
        await loadRuns();
    } catch (err) {
        autoSetState("Manual run failed: " + err.message, true);
    }
}

async function toggleScheduleEnabled(scheduleId) {
    const schedule = AutoState.schedules.find(s => s.id === scheduleId);
    if (!schedule) return;
    try {
        await autoApi("/api/automations/schedules/" + encodeURIComponent(scheduleId), {
            method: "PUT",
            body: JSON.stringify({ enabled: !schedule.enabled })
        });
        await loadSchedulesAndRender();
    } catch (err) {
        autoSetState("Update failed: " + err.message, true);
    }
}

function tickElapsedTimers() {
    document.querySelectorAll("[data-elapsed-start]").forEach(function (el) {
        const start = el.getAttribute("data-elapsed-start");
        if (!start) return;
        const card = el.closest(".auto-run-card");
        if (card && /st-(running|queued)/.test(card.className)) {
            el.textContent = autoFmtElapsed(start, null);
        }
    });
}

function startPolling() {
    stopPolling();
    AutoState.pollTimer = setInterval(loadSchedulesAndRender, 15000);
    setInterval(tickElapsedTimers, 1000);
    const dot = document.getElementById("auto-poll-dot");
    if (dot) dot.classList.remove("paused");
}

function stopPolling() {
    if (AutoState.pollTimer) clearInterval(AutoState.pollTimer);
    AutoState.pollTimer = null;
}

document.addEventListener("click", function (event) {
    const target = event.target.closest("[data-auto-close],[data-auto-edit],[data-auto-expand],[data-auto-cancel],[data-auto-run-now],[data-auto-toggle-enable],[data-auto-cancel-job],[data-auto-open-conversation],[data-auto-delete-job]");
    if (!target) return;
    if (target.hasAttribute("data-auto-close")) { closeScheduleModal(); return; }
    if (target.hasAttribute("data-auto-edit")) { openScheduleModal(target.getAttribute("data-auto-edit")); return; }
    if (target.hasAttribute("data-auto-expand")) { openRunEvents(target.getAttribute("data-auto-expand")); return; }
    if (target.hasAttribute("data-auto-cancel")) { cancelRun(target.getAttribute("data-auto-cancel")); return; }
    if (target.hasAttribute("data-auto-run-now")) { runScheduleNow(target.getAttribute("data-auto-run-now")); return; }
    if (target.hasAttribute("data-auto-toggle-enable")) { toggleScheduleEnabled(target.getAttribute("data-auto-toggle-enable")); return; }
    if (target.hasAttribute("data-auto-cancel-job")) { cancelConversationJob(target.getAttribute("data-auto-cancel-job")); return; }
    if (target.hasAttribute("data-auto-open-conversation")) { openConversationJob(target.getAttribute("data-auto-open-conversation")); return; }
    if (target.hasAttribute("data-auto-delete-job")) { deleteConversationJob(target.getAttribute("data-auto-delete-job")); return; }
});

(function initAutomationsApp() {
    const root = document.getElementById("automations-app");
    if (!root) return;
    document.getElementById("auto-new").addEventListener("click", function () { openScheduleModal(null); });
    document.getElementById("auto-refresh").addEventListener("click", loadSchedulesAndRender);
    document.getElementById("auto-form").addEventListener("submit", saveSchedule);
    document.getElementById("auto-delete").addEventListener("click", deleteSelectedSchedule);
    const cronInput = document.getElementById("auto-f-cron");
    cronInput.addEventListener("input", updateCronPreview);
    cronInput.addEventListener("change", updateCronPreview);
    startPolling();
    loadSchedulesAndRender();
})();
