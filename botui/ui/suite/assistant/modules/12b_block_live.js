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
/* Steps that belong to the virtual machine (not: local_engine_start, generate). */
var VM_STEPS = { queued: 1, image_pull: 1, container_start: 1, bootstrap: 1, package_install: 1, execute: 1, repair: 1, save: 1, publish: 1, vm_action: 1 };
/* Start-up checks the server sends before every agent turn. They show only
   when the machine really had to start (the step runs past QUIET_MS or
   fails, the same quiet window as the server's workspace start); a machine
   that is already on stays out of the answer. */
var STARTUP = { queued: 1, image_pull: 1, container_start: 1, bootstrap: 1 };
var QUIET_MS = 2000;
var STEP_LABELS = {
    queued: ["Entrou na fila da Máquina Virtual", "Queued on the virtual machine", "Na fila da Máquina Virtual", "Waiting for the virtual machine"],
    image_pull: ["Baixou a imagem da Máquina Virtual", "Pulled the virtual machine image", "Baixando a imagem da Máquina Virtual", "Pulling the virtual machine image"],
    container_start: ["Ligou a Máquina Virtual", "Started the virtual machine", "Ligando a Máquina Virtual", "Starting the virtual machine"],
    local_engine_start: ["Ligou o modelo local", "Started the local model", "Ligando o modelo local", "Starting the local model"],
    bootstrap: ["Preparou a Máquina Virtual", "Prepared the virtual machine", "Preparando a Máquina Virtual", "Preparing the virtual machine"],
    package_install: ["Instalou pacotes", "Installed packages", "Instalando pacotes", "Installing packages"],
    execute: ["Executou o programa", "Ran the program", "Executando o programa", "Running the program"],
    repair: ["Corrigiu o programa", "Fixed the program", "Corrigindo o programa", "Fixing the program"],
    save: ["Guardou o resultado", "Saved the output", "Guardando o resultado", "Saving the output"],
    publish: ["Publicou no Drive", "Published to Drive", "Publicando no Drive", "Publishing to Drive"],
    generate: ["Gerou a resposta", "Wrote the answer", "Gerando a resposta", "Writing the answer"],
    vm_action: ["Ação na Máquina Virtual", "Virtual Machine action", "Ação na Máquina Virtual", "Virtual Machine action"],
    web_fetch: ["Leu a página", "Read the web page", "Lendo a página", "Reading the web page"],
    web_search: ["Pesquisou na web", "Searched the web", "Pesquisando na web", "Searching the web"]
};
/* Web steps carry data, not words: the query or the site, shown as sent. */
var WEB_STEPS = { web_fetch: 1, web_search: 1 };
var SITES_SHOWN = 4;
/* A step id the shell does not know: a neutral line, never the server's
   own (English) name or detail. */
var OTHER_STEP = ["Concluiu uma etapa", "Finished a step", "Executando uma etapa", "Running a step"];
/* Agent actions (vm_consent): labels and details in 12d_block_vm.js. */
function vm() { return A._vmStep; }
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

/* Where a Virtual Machine permission card goes: in the answer that waits
   for it, right after its steps (08_consent.js mounts the card). */
A.consentSlot = function (requestId) {
    var msgs = document.getElementById("gbMessages");
    /* Not while the conversation is being drawn: the history comes first. */
    if (!msgs || !msgs.children.length || msgs.getAttribute("aria-busy") === "true") { return null; }
    var record = msgs.querySelector('.gb-block[data-permission-id="' + CSS.escape(requestId) + '"]');
    if (record) { record.style.display = "none"; }
    var slot = document.createElement("div");
    slot.className = "gb-consent-slot";
    slot.dataset.requestId = requestId;
    place(slot);
    return slot.isConnected ? slot : null;
};

/* ---- gb.activity ---- */
function onActivity(payload) {
    var permission = payload.summary || {};
    if (payload.kind === "permission" && permission.request_id && A.queueConsentCard) {
        if (payload.status === "needs_input") { A.reviveConsentCards(permission.request_id); }
        else if (payload.status === "completed" || payload.status === "failed") { A.removeConsentCard(permission.request_id); }
        /* A Virtual Machine action: the card and its step row tell it all. */
        if (vm().verbs[permission.action]) { return; }
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
   once a step really runs in the virtual machine (infra/tool phase or vm_exec). */
function stepLabel(step) {
    if (step.key === "vm_action" && step.action) { return vm().label(step); }
    /* A stage the program itself names (vm_trace markers): its own words. */
    if (step.key.indexOf("trace:") === 0 && step.name) { return t("Etapa do programa: ", "Program step: ") + step.name; }
    var l = STEP_LABELS[step.key] || OTHER_STEP;
    var label = step.state === "running" ? t(l[2], l[3]) : t(l[0], l[1]);
    if (!WEB_STEPS[step.key] || !step.detail) { return label; }
    var sites = step.sites || [];
    var more = sites.length > SITES_SHOWN ? ", …" : "";
    return label + ": " + step.detail + (sites.length ? " (" + sites.slice(0, SITES_SHOWN).join(", ") + more + ")" : "");
}

/* The run header's second line: the shell's label; package names and the
   site of a page read are the only server details shown (data, not words). */
function stepSummary(step) {
    var label = stepLabel(step);
    return step.key === "package_install" && step.detail ? label + ": " + step.detail : label;
}

function runFor(runId) {
    var key = runId || "run";
    if (!runs[key]) { runs[key] = { key: key, el: null, steps: [], ok: null, vm: false, local: false, web: false }; }
    return runs[key];
}

function runTitle(run, status) {
    /* Pages read from the web (phase "web"): not the virtual machine. */
    if (run.web && !run.vm && !run.local) {
        var searchedOnly = run.steps.every(function (s) { return s.key === "web_search"; });
        if (searchedOnly) {
            if (status === "running") { return t("Pesquisando na web", "Searching the web"); }
            return status === "failed" ? t("A pesquisa na web falhou", "The web search failed") : t("Pesquisou na web", "Searched the web");
        }
        if (status === "running") { return t("Lendo a web", "Reading the web"); }
        return status === "failed" ? t("Não conseguiu ler a página", "Could not read the page") : t("Leu a web", "Read the web");
    }
    if (run.local && !run.vm) {
        if (status === "running") { return t("Preparando o modelo local", "Preparing the local model"); }
        return status === "failed" ? t("Falhou ao preparar o modelo local", "Failed to prepare the local model")
            : t("Preparou o modelo local", "Prepared the local model");
    }
    if (status === "needs-you") { return t("Aguardando sua permissão", "Waiting for your permission"); }
    if (status === "denied") { return t("Nada foi executado na Máquina Virtual", "Nothing ran on the virtual machine"); }
    if (status === "running") { return t("Executando na Máquina Virtual", "Running on the virtual machine"); }
    return status === "failed" ? t("Falhou na Máquina Virtual", "Failed on the virtual machine")
        : t("Executou na Máquina Virtual", "Ran on the virtual machine");
}

function showRun(run) {
    if (!run.vm && !run.local && !run.web) { return; }
    if (!run.el || !document.body.contains(run.el)) {
        run.el = B.block({ kind: "machine", icon: "terminal", title: runTitle(run, "running"),
            summary: "", status: "running", open: true });
        run.el.dataset.runId = run.key;
        run.el.addEventListener("click", function (e) {
            var more = e.target.closest(".gb-run-more");
            if (!more) { return; }
            var step = run.steps.filter(function (s) { return s.id === more.getAttribute("data-step-id"); })[0];
            if (!step) { return; }
            step.open = !step.open;
            paintRun(run);
            var again = run.el.querySelector('.gb-run-more[data-step-id="' + CSS.escape(step.id) + '"]');
            if (again) { again.focus(); }
        });
        /* A card that waits (also one restored after a reload) keeps its
           run in the same answer, right above it. */
        var slots = document.querySelectorAll("#gbMessages .gb-consent-slot");
        var slot = slots.length ? slots[slots.length - 1] : null;
        if (slot && slot.parentNode) { slot.parentNode.insertBefore(run.el, slot); } else { place(run.el); }
    }
    run.el._gbRun = run;
    paintRun(run);
}

/* A finished run whose answer arrives later with a file link: the header
   is read again once (18_file_panel.js calls this). */
A.repaintRunsIn = function (root) {
    (root || document).querySelectorAll(".gb-block[data-run-id]").forEach(function (el) {
        if (!el._gbRun || el.dataset.fileSeen === "1" || !answerHasFile(el)) { return; }
        el.dataset.fileSeen = "1";
        paintRun(el._gbRun);
    });
};

function shown(run) { return run.steps.filter(function (s) { return !s.quiet; }); }

function wakeStep(run, step) {
    clearTimeout(step.timer);
    if (!step.quiet) { return; }
    step.quiet = false;
    run.vm = true;
}

/* A step that failed and was followed by a later action that worked was
   corrected within the run (a retried command, a repaired program). */
var ACTION_STEPS = { vm_action: 1, execute: 1, repair: 1, package_install: 1, save: 1, publish: 1 };
function fixedSteps(steps) {
    var fixed = {};
    steps.forEach(function (s, i) {
        if (s.state !== "failed" || s.denied) { return; }
        for (var j = i + 1; j < steps.length; j++) {
            if (steps[j].state === "ok" && ACTION_STEPS[steps[j].key]) { fixed[s.id] = true; return; }
        }
    });
    return fixed;
}

/* The answer of this run links a file it made. */
function answerHasFile(el) {
    var node = el.closest(".gb-msg-assistant") || el.closest(".gb-block-stack");
    var sel = 'a[href^="vmfile:"]';
    if (!node) { return false; }
    if (node.querySelector(sel)) { return true; }
    for (var n = node.nextElementSibling; n && !n.classList.contains("gb-msg-user"); n = n.nextElementSibling) {
        if (n.querySelector(sel)) { return true; }
    }
    return false;
}

function paintRun(run) {
    var steps = shown(run);
    var done = steps.filter(function (s) { return s.state !== "running"; }).length;
    var fixed = fixedSteps(steps);
    var saved = steps.some(function (s) { return s.state === "ok" && s.files && s.files.length; }) || answerHasFile(run.el);
    /* The header tells the outcome: a run that recovered and saved its file
       succeeded; the corrected step keeps its own mark in the list. */
    var failed = steps.some(function (s) { return s.state === "failed" && !fixed[s.id]; }) || (run.ok === false && !saved);
    var current = steps.filter(function (s) { return s.waiting && s.state === "running"; })[0] ||
        steps.filter(function (s) { return s.state === "running"; })[0];
    var status = failed ? "failed" : (!current && run.ok !== null ? "done" : "running");
    var waiting = steps.some(function (s) { return s.waiting && s.state === "running"; });
    var denied = failed && steps.some(function (s) { return s.denied; }) &&
        steps.every(function (s) { return s.state !== "failed" || s.denied; });
    if (waiting) { status = "needs-you"; }
    var el = run.el;
    el.querySelector(".gb-block-title").textContent = runTitle(run, denied ? "denied" : status);
    el.querySelector(".gb-block-summary").textContent = current ? stepSummary(current) : "";
    var count = el.querySelector(".gb-block-count");
    if (!count) {
        count = document.createElement("span");
        count.className = "gb-block-count";
        el.querySelector(".gb-block-chip").before(count);
    }
    count.textContent = done + " " + t("de", "of") + " " + steps.length;
    B.updateStatus(el, status);
    if (denied) {
        var last = steps.filter(function (s) { return s.denied; }).pop();
        var chip = vm().chip[last.denied] || vm().chip.denied;
        el.querySelector(".gb-block-chip").textContent = t(chip[0], chip[1]);
    }
    /* A run with an agent action stays open: what ran, and how, stays in view. */
    var acted = steps.some(function (s) { return s.key === "vm_action"; });
    if (acted && el.dataset.userToggled !== "1" && !el.classList.contains("open")) {
        el.classList.add("open");
        el.querySelector(".gb-block-header").setAttribute("aria-expanded", "true");
    }
    if (status !== "running" && el.dataset.userToggled !== "1" && !acted) {
        el.classList.remove("open");
        el.querySelector(".gb-block-header").setAttribute("aria-expanded", "false");
    }
    var pct = steps.length ? Math.round(done * 100 / steps.length) : 0;
    el.querySelector(".gb-block-body").innerHTML = '<div class="gb-block-list">' + steps.map(function (s) {
        var mark = s.state === "ok" ? "✓" : (s.state === "failed" ? "✕" : "");
        return '<div class="gb-run-step gb-run-step--' + s.state + (fixed[s.id] ? " gb-run-step--fixed" : "") +
            '"><span class="gb-run-mark" aria-hidden="true">' + mark + '</span>' +
            '<span class="gb-run-label">' + esc(stepLabel(s)) +
            (fixed[s.id] ? ' <span class="gb-run-fixed">' + esc(t("corrigido", "fixed")) + '</span>' : '') + '</span>' +
            '<span class="gb-run-time">' + (s.ms != null ? esc(Math.max(1, Math.round(s.ms / 1000)) + " s") : "") + '</span></div>' + vm().extra(s);
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
        step = { id: id, key: data.step || "", name: data.name || "", detail: "", state: "running", ms: null,
            quiet: !!STARTUP[data.step || ""] };
        run.steps.push(step);
        if (step.quiet) {
            step.timer = setTimeout(function () {
                if (step.state === "running") { wakeStep(run, step); showRun(run); }
            }, QUIET_MS);
        }
    }
    if (VM_PHASES[data.phase] && VM_STEPS[data.step || ""] && !STARTUP[data.step || ""]) { run.vm = true; }
    if ((data.step || "") === "local_engine_start") { run.local = true; }
    if (data.phase === "web") { run.web = true; }
    if (data.action) { step.action = data.action; }
    if (data.allowed_by) { step.allowedBy = data.allowed_by; }
    if (data.waiting != null) { step.waiting = !!data.waiting; }
    /* Withdrawn by the server (turn stopped or a new turn): the card is
       gone from the pending list, a full read closes it. */
    if (data.denied) { step.denied = data.denied; if (A.reviveConsentCards) { A.reviveConsentCards(); } }
    if (Array.isArray(data.output)) { step.output = data.output; }
    if (Array.isArray(data.files)) { step.files = data.files; }
    if (Array.isArray(data.sites)) { step.sites = data.sites.map(String); }
    if (data.reason) { step.reason = data.reason; }
    if (data.exit_code != null) { step.exit = data.exit_code; }
    if (frame.type === "STEP_STARTED") {
        step.state = "running";
    }
    /* STEP_FINISHED may repeat the detail or leave it out: keep the start's. */
    if (data.detail) { step.detail = data.detail; }
    if (frame.type !== "STEP_STARTED") {
        step.state = data.status === "ok" || data.status === "completed" || data.status === "succeeded" ? "ok" : "failed";
        step.ms = data.elapsed_ms != null ? data.elapsed_ms : data.duration_ms;
        if (step.quiet) {
            clearTimeout(step.timer);
            if (step.state === "failed" || (step.ms || 0) >= QUIET_MS) { wakeStep(run, step); }
        }
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
