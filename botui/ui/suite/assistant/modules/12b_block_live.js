"use strict";
/**
 * 12b_block_live.js - Live run frames into blocks.
 *
 * gb.activity frames become one block per activity id (updated in place).
 * Workspace stages arrive as AG-UI step frames (botserver
 * pipeline/workspace_progress.rs):
 *   { gb_event, type:"STEP_STARTED",  run_id, data:{ step_id:"tool:execute",
 *     name, phase, step, detail, started_at } }
 *   { gb_event, type:"STEP_FINISHED", run_id, data:{ step_id, status:"ok"|"error",
 *     elapsed_ms, duration_ms, phase, step, detail, name } }
 * plus CUSTOM gb.tool_progress { tool:"vm_exec", phase:"start"|"done", ok }
 * and RUN_FINISHED { state }. A run gets a "Machine run" block with steps
 * and a bar only when something really ran in the Machine; a plain model
 * turn (phase "agent" only) shows no block.
 */
(function () {
var A = window.GBAssistant;
var B = A._blk;
var esc = B.esc, t = B.t;
/* Phases that run in the Machine; "agent" is the model writing, which a
   plain answer also sends. Labels: done (pt, en), then running (pt, en). */
var VM_PHASES = { infra: 1, tool: 1 };
var STEP_LABELS = {
    queued: ["Entrou na fila da Máquina", "Queued on the Machine", "Na fila da Máquina", "Waiting for the Machine"],
    image_pull: ["Baixou a imagem da Máquina", "Pulled the Machine image", "Baixando a imagem da Máquina", "Pulling the Machine image"],
    container_start: ["Ligou a Máquina", "Started the Machine", "Ligando a Máquina", "Starting the Machine"],
    bootstrap: ["Preparou a Máquina", "Prepared the Machine", "Preparando a Máquina", "Preparing the Machine"],
    package_install: ["Instalou pacotes", "Installed packages", "Instalando pacotes", "Installing packages"],
    execute: ["Executou o programa", "Ran the program", "Executando o programa", "Running the program"],
    save: ["Guardou o resultado", "Saved the output", "Guardando o resultado", "Saving the output"],
    publish: ["Publicou no Drive", "Published to Drive", "Publicando no Drive", "Publishing to Drive"],
    generate: ["Gerou a resposta", "Wrote the answer", "Gerando a resposta", "Writing the answer"]
};
var runs = {};

/* The activity stack of the current turn (after the last user message). */
function turnStack() {
    var msgs = document.getElementById("gbMessages");
    if (!msgs) { return null; }
    var children = msgs.children, lastUser = -1, ci;
    for (ci = children.length - 1; ci >= 0; ci--) {
        if (children[ci].classList.contains("gb-msg-user")) { lastUser = ci; break; }
    }
    for (ci = children.length - 1; ci > lastUser && ci >= Math.max(0, children.length - 4); ci--) {
        if (children[ci].classList.contains("gb-msg-assistant")) {
            var found = children[ci].querySelector(".gb-block-stack");
            if (found) { return found; }
        }
    }
    var stack = document.createElement("div");
    stack.className = "gb-block-stack";
    var turn = document.createElement("div");
    turn.className = "gb-msg-assistant";
    turn.appendChild(stack);
    if (A.addMessageHeading) { A.addMessageHeading(turn, false, Date.now()); }
    var streamingEl = msgs.querySelector(".gb-streaming");
    if (streamingEl && streamingEl.parentElement === msgs) { msgs.insertBefore(turn, streamingEl); }
    else { msgs.appendChild(turn); }
    return stack;
}

function place(el) {
    var stack = turnStack();
    if (!stack) { return; }
    stack.appendChild(el);
    if (!A.isUserScrolledUp || !A.isUserScrolledUp()) { A.scrollToBottom(false); }
}

/* ---- gb.activity ---- */
function onActivity(payload) {
    var permission = payload.summary || {};
    if (payload.kind === "permission" && permission.request_id && A.queueConsentCard) {
        if (payload.status === "needs_input") { A.reviveConsentCards(permission.request_id); }
        else if (payload.status === "completed" || payload.status === "failed") { A.removeConsentCard(permission.request_id); }
    }
    var ev = { id: payload.id, kind: payload.kind, status: payload.status,
        summary: payload.summary || {}, data: payload.summary || {} };
    if (payload.error) { ev.error = payload.error; ev.data.error = payload.error; }
    var existing = document.querySelector('[data-activity-id="' + CSS.escape(payload.id) + '"]');
    if (!existing) {
        var fresh = A.renderSingleBlock(ev);
        if (fresh) { place(fresh); }
        return;
    }
    /* The completed frame carries the final summary: rebuild the block; it
       folds like a fresh one unless the person opened or closed it. */
    var touched = existing.dataset.userToggled === "1";
    var wasOpen = existing.classList.contains("open");
    var rebuilt = A.renderSingleBlock(ev);
    if (!rebuilt) { B.updateStatus(existing, B.mapStatus(payload.status)); return; }
    if (payload.error && payload.kind !== "attachment_read") {
        rebuilt.querySelector(".gb-block-body").innerHTML = B.noteRow(payload.error, "danger");
    }
    if (touched) { rebuilt.dataset.userToggled = "1"; }
    if (touched && wasOpen !== rebuilt.classList.contains("open")) {
        rebuilt.classList.toggle("open", wasOpen);
        rebuilt.querySelector(".gb-block-header").setAttribute("aria-expanded", String(wasOpen));
    }
    existing.replaceWith(rebuilt);
}

/* ---- Workspace run ----
   Steps are tracked per run without drawing anything; the card appears only
   once a step really runs in the Machine (infra/tool phase or vm_exec). */
function stepLabel(step) {
    var l = STEP_LABELS[step.key];
    if (!l) { return step.name || step.key || t("Etapa", "Step"); }
    return step.state === "running" ? t(l[2], l[3]) : t(l[0], l[1]);
}

function runFor(runId) {
    var key = runId || "run";
    if (!runs[key]) { runs[key] = { key: key, el: null, steps: [], ok: null, vm: false }; }
    return runs[key];
}

function showRun(run) {
    if (!run.vm) { return; }
    if (!run.el || !document.body.contains(run.el)) {
        run.el = B.block({ kind: "machine", icon: "terminal", title: t("Executando na Máquina", "Running on the Machine"),
            summary: "", status: "running", open: true });
        run.el.dataset.runId = run.key;
        place(run.el);
    }
    paintRun(run);
}

function paintRun(run) {
    var done = run.steps.filter(function (s) { return s.state !== "running"; }).length;
    var failed = run.steps.some(function (s) { return s.state === "failed"; }) || run.ok === false;
    var current = run.steps.filter(function (s) { return s.state === "running"; })[0];
    var status = failed ? "failed" : (!current && run.ok !== null ? "done" : "running");
    var el = run.el;
    el.querySelector(".gb-block-title").textContent = status === "running"
        ? t("Executando na Máquina", "Running on the Machine")
        : (failed ? t("Falhou na Máquina", "Failed on the Machine") : t("Executou na Máquina", "Ran on the Machine"));
    el.querySelector(".gb-block-summary").textContent = current ? (current.detail || stepLabel(current)) : "";
    var count = el.querySelector(".gb-block-count");
    if (!count) {
        count = document.createElement("span");
        count.className = "gb-block-count";
        el.querySelector(".gb-block-chip").before(count);
    }
    count.textContent = done + " " + t("de", "of") + " " + run.steps.length;
    B.updateStatus(el, status);
    if (status !== "running" && el.dataset.userToggled !== "1") {
        el.classList.remove("open");
        el.querySelector(".gb-block-header").setAttribute("aria-expanded", "false");
    }
    var pct = run.steps.length ? Math.round(done * 100 / run.steps.length) : 0;
    el.querySelector(".gb-block-body").innerHTML = '<div class="gb-block-list">' + run.steps.map(function (s) {
        var mark = s.state === "ok" ? "✓" : (s.state === "failed" ? "✕" : "");
        return '<div class="gb-run-step gb-run-step--' + s.state + '"><span class="gb-run-mark" aria-hidden="true">' + mark + '</span>' +
            '<span class="gb-run-label">' + esc(stepLabel(s)) + '</span>' +
            '<span class="gb-run-time">' + (s.ms != null ? esc(Math.max(1, Math.round(s.ms / 1000)) + " s") : "") + '</span></div>';
    }).join("") +
        '<div class="gb-run-bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="' + pct + '" aria-label="' +
        esc(t("Progresso", "Progress")) + '"><span style="width:' + pct + '%"></span></div></div>';
}

/* STEP_FINISHED may come without phase/step (the model step sends only
   step_id): it is matched to the started step by step_id. */
function onStep(frame) {
    var data = frame.data || {};
    var run = runFor(frame.run_id);
    var id = data.step_id || (data.phase + ":" + data.step);
    var step = run.steps.filter(function (s) { return s.id === id; })[0];
    if (!step) {
        if (frame.type !== "STEP_STARTED" && !data.phase) { return; }
        step = { id: id, key: data.step || "", name: data.name || "", detail: "", state: "running", ms: null };
        run.steps.push(step);
    }
    if (VM_PHASES[data.phase]) { run.vm = true; }
    if (frame.type === "STEP_STARTED") {
        step.state = "running";
        step.detail = data.detail || "";
    } else {
        step.state = data.status === "ok" || data.status === "completed" || data.status === "succeeded" ? "ok" : "failed";
        step.ms = data.elapsed_ms != null ? data.elapsed_ms : data.duration_ms;
    }
    showRun(run);
}

function closeRun(run, ok) {
    run.ok = ok;
    run.steps.forEach(function (s) { if (s.state === "running") { s.state = ok ? "ok" : "failed"; } });
    showRun(run);
}

function onToolProgress(frame) {
    var p = (frame.data || {}).payload || {};
    if (p.tool !== "vm_exec") { return; }
    var run = runFor(frame.run_id);
    run.vm = true;
    if (p.phase === "done") { closeRun(run, p.ok !== false); } else { showRun(run); }
}

/* RUN_FINISHED { state:"succeeded"|"failed"|... } closes what is still open. */
function onRunFinished(frame) {
    var run = runs[frame.run_id || "run"];
    if (!run) { return; }
    closeRun(run, (frame.data || {}).state === "succeeded");
    delete runs[run.key];
}

/* ---- Live frame entry point (02_api routes every gb_event here) ---- */
A.onActivityEvent = function (frame) {
    var type = frame.type || frame.event_type || "";
    var data = frame.data || {};
    if (type === "STEP_STARTED" || type === "STEP_FINISHED") { onStep(frame); return; }
    if (type === "RUN_FINISHED") { onRunFinished(frame); return; }
    if (data.name === "gb.tool_progress") { onToolProgress(frame); return; }
    if (data.name !== "gb.activity") { return; }
    var payload = data.payload;
    if (payload && payload.id) { onActivity(payload); }
};
})();
