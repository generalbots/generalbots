"use strict";
/**
 * 10k_settings_agent_runtime.js -- Settings > Modo Agente.
 *
 * Renders the agent runtime card, Tauri bridge calls, polling during
 * install, action binding and the removal confirmation dialog.
 * Text helpers live in 10j_settings_agent_text.js (A._artText).
 *
 * Polls agent_runtime_status every 1.5 s while installing (the assistant
 * shell does not listen to Tauri events yet).
 */
(function () {
var A = window.GBAssistant;
var T = A._artText || {};
var tr = T.tr || function (pt, en) { return A.locale && A.locale.startsWith("pt") ? pt : en; };
var esc = T.esc || function (s) { return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;"); };

var localPaused = false; /* fallback if status lacks paused field */

/* ---- Tauri / API bridge ---- */

/* The shell page is served by the local server; the desktop app injects
 * `__TAURI_INTERNALS__` into it (same bridge as 09d_settings_profile.js). */
function bridge() { return window.__TAURI_INTERNALS__ || (window.__TAURI__ && window.__TAURI__.core) || null; }
function hasTauri() { var b = bridge(); return !!(b && typeof b.invoke === "function"); }
function invoke(cmd, args) { return bridge().invoke(cmd, args || {}); }

/* The server's codes (/api/product) mapped to the card's states. */
var SERVER_STATE = {
    ready: "ready", unsupported: "unsupported", wsl_missing: "needs_wsl",
    distro_missing: "not_installed", runtime_not_installed: "not_installed"
};

function serverStatus() {
    return A.apiFetch("/api/product").then(function (r) {
        return r.ok ? r.json() : null;
    }).then(function (d) {
        if (!d) { return null; }
        return { state: SERVER_STATE[d.agent_runtime_state] || "unsupported", managed: true };
    }).catch(function () { return null; });
}

function fetchStatus() {
    if (hasTauri()) {
        return invoke("agent_runtime_status").catch(function () { return serverStatus(); });
    }
    return serverStatus();
}

function invokeInstall() { return invoke("agent_runtime_install"); }
function invokeCancel() { return invoke("agent_runtime_cancel"); }
function invokeRemove(keepExport) {
    return invoke("agent_runtime_remove", { request: { keep_export: !!keepExport } });
}
function invokeCheckUpdate() { return invoke("agent_runtime_check_update"); }

/* ---- polling ---- */

var pollTimer = null;

function startPoll(panel) {
    stopPoll();
    pollTimer = setInterval(function () {
        if (!document.body.contains(panel)) { stopPoll(); return; }
        fetchStatus().then(function (st) {
            if (!st) { return; }
            A.agentRuntimeState = st.state;
            if (st.state !== "installing") { stopPoll(); }
            renderCard(panel, st);
        });
    }, 1500);
}

function stopPoll() {
    if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }
}

/* ---- card rendering ---- */

function renderCard(panel, status) {
    var isPaused = status.paused || localPaused || A._artPaused;
    var state = isPaused && status.state !== "installing" && status.state !== "ready"
        ? "paused" : status.state;
    var html = '<div class="gb-set-card"><h2 class="gb-set-title">' +
        esc(tr("Modo Agente", "Agent Mode")) + '</h2>';

    /* Manual setup: short text, no actions */
    if (status.managed === false) {
        html += chipHtml("ready") +
            '<p class="gb-set-desc" style="margin-top:8px">' +
            esc(tr("Configurado manualmente. O ambiente de agentes está sendo gerenciado fora deste aplicativo.",
                    "Manually configured. The agent environment is managed outside this application.")) +
            '</p></div>';
        panel.innerHTML = html;
        return;
    }

    /* Intro line */
    html += '<p class="gb-set-desc">' +
        esc(tr("O Modo Agente permite usar assistentes por assinatura (Claude Code, Codex, Antigravity) que executam comandos em um ambiente isolado neste computador.",
                "Agent Mode lets you use subscription assistants (Claude Code, Codex, Antigravity) that run commands in an isolated environment on this computer.")) +
        '</p>';

    /* Short chip */
    html += chipHtml(state);

    /* Body description */
    var desc = T.stateDesc(state, status);
    if (desc) {
        desc.split("\n").forEach(function (line) {
            html += '<p class="gb-set-desc" style="margin-top:4px">' + esc(line) + '</p>';
        });
    }

    /* Tech details (collapsed) for install states */
    if (state === "not_installed" || state === "needs_wsl") {
        html += '<details class="gb-art-tech" style="margin-top:4px"><summary>' +
            esc(tr("Detalhes técnicos", "Technical details")) +
            '</summary><p class="gb-set-hint">' + esc(T.techDetails()) + '</p></details>';
    }

    /* Version + disk for ready */
    if (state === "ready") {
        if (status.runtime_version) {
            html += '<div class="gb-set-field"><span class="gb-set-label">' +
                esc(tr("Versão", "Version")) + '</span>' +
                '<span class="gb-set-value">' + esc(status.runtime_version) + '</span></div>';
        }
        if (status.disk_bytes) {
            html += '<div class="gb-set-field"><span class="gb-set-label">' +
                esc(tr("Espaço usado", "Space used")) + '</span>' +
                '<span class="gb-set-value">' + esc(T.fmtBytes(status.disk_bytes)) + '</span></div>';
        }
    }

    /* Progress during install */
    if (state === "installing") { html += T.progressHtml(status); }

    /* Error detail from status.last_error: { step, code, detail } */
    var lastErr = status.last_error || (status.last_error_code
        ? { code: status.last_error_code, detail: status.last_error_detail || status.detail } : null);
    if (state === "broken" && lastErr) {
        html += '<p class="gb-set-warn" style="margin-top:8px">' +
            esc(T.errorText(lastErr.code, status)) + '</p>';
        if (lastErr.detail) {
            html += '<details class="gb-art-tech"><summary>' +
                esc(tr("Detalhes técnicos", "Technical details")) +
                '</summary><p class="gb-set-hint">' + esc(lastErr.detail) + '</p></details>';
        }
    }

    /* Policy-blocked: IT text */
    if (state === "policy_blocked") {
        html += '<div class="gb-art-it-box">' +
            '<p class="gb-set-desc" style="font-weight:500;margin-top:8px">' +
            esc(tr("Texto para enviar ao time de TI:", "Text to send to the IT team:")) +
            '</p><div class="gb-art-it-text" id="gbArtItText">' +
            esc(T.policyText()) + '</div>' +
            '<button type="button" class="gb-set-btn-sm" id="gbArtCopyIt">' +
            esc(tr("Copiar texto", "Copy text")) + '</button></div>';
    }

    /* needs_reboot: instruction text */
    if (state === "needs_reboot") {
        /* no extra text needed: stateDesc already has the instruction */
    }

    /* Experimental note for install-offering states */
    if (state === "not_installed" || state === "needs_wsl" || state === "paused" || state === "broken") {
        html += '<p class="gb-set-hint" style="margin-top:8px">' +
            esc(T.experimentalNote()) + '</p>';
    }

    /* Actions */
    if (hasTauri()) {
        html += '<div class="gb-art-actions" style="margin-top:12px;display:flex;gap:8px;flex-wrap:wrap">';
        html += actionButtons(state, status);
        html += '</div>';
    } else if (state !== "ready" && state !== "unsupported" && state !== "policy_blocked") {
        html += '<p class="gb-set-hint" style="margin-top:8px">' +
            esc(tr("Ações disponíveis apenas no aplicativo de desktop.",
                    "Actions available only in the desktop application.")) + '</p>';
    }

    html += '</div>';
    panel.innerHTML = html;
    bindActions(panel, status);
    if (state === "installing") { startPoll(panel); }
}

function chipHtml(state) {
    return '<div class="gb-set-field" style="margin-top:4px"><span class="gb-set-label">' +
        esc(tr("Estado", "Status")) + '</span>' +
        '<span class="gb-badge ' + T.chipClass(state) + '">' +
        esc(T.chipLabel(state)) + '</span></div>';
}

function actionButtons(state) {
    var recheck = '<button type="button" class="gb-set-btn-sm" id="gbArtRecheck">' +
        esc(tr("Verificar de novo", "Check again")) + '</button>';
    switch (state) {
    case "not_installed": case "needs_wsl":
        return '<button type="button" class="gb-set-btn-sm gb-set-btn-accent" id="gbArtInstall">' +
            esc(tr("Instalar", "Install")) + '</button>';
    case "needs_reboot":
        return '<button type="button" class="gb-set-btn-sm gb-set-btn-accent" id="gbArtResumeReboot">' +
            esc(tr("Já reiniciei, continuar", "I have restarted, continue")) + '</button>';
    case "needs_virtualization": case "unsupported": case "policy_blocked":
        return recheck;
    case "installing":
        return '<button type="button" class="gb-set-btn-sm gb-set-btn-danger" id="gbArtCancel">' +
            esc(tr("Cancelar", "Cancel")) + '</button>';
    case "paused":
        return '<button type="button" class="gb-set-btn-sm gb-set-btn-accent" id="gbArtContinue">' +
            esc(tr("Continuar", "Continue")) + '</button>';
    case "broken":
        return '<button type="button" class="gb-set-btn-sm gb-set-btn-accent" id="gbArtRetry">' +
            esc(tr("Tentar de novo", "Try again")) + '</button>';
    case "ready":
        return '<button type="button" class="gb-set-btn-sm" id="gbArtUpdate">' +
            esc(tr("Verificar atualização", "Check for update")) + '</button>' +
            '<button type="button" class="gb-set-btn-sm gb-set-btn-danger" id="gbArtRemove">' +
            esc(tr("Remover", "Remove")) + '</button>';
    default: return '';
    }
}

/* ---- action binding ---- */

function refresh(panel) {
    fetchStatus().then(function (st) {
        if (st) { A.agentRuntimeState = st.state; renderCard(panel, st); }
    });
}

function showErr(panel, e) {
    var card = panel.querySelector(".gb-set-card");
    if (!card) { return; }
    var old = card.querySelector(".gb-art-inline-err");
    if (old) { old.remove(); }
    var p = document.createElement("p");
    p.className = "gb-set-warn gb-art-inline-err";
    p.textContent = (e && e.message) ? e.message : String(e);
    card.appendChild(p);
}

function bindActions(panel) {
    bind(panel, "#gbArtInstall", function (btn) {
        btn.disabled = true;
        btn.textContent = tr("Iniciando...", "Starting...");
        localPaused = false;
        invokeInstall().then(function () { refresh(panel); })
            .catch(function (e) { btn.disabled = false; btn.textContent = tr("Instalar", "Install"); showErr(panel, e); });
    });
    bind(panel, "#gbArtCancel", function (btn) {
        btn.disabled = true;
        invokeCancel().then(function () { localPaused = true; refresh(panel); })
            .catch(function (e) { showErr(panel, e); });
    });
    bind(panel, "#gbArtContinue", function (btn) {
        btn.disabled = true;
        localPaused = false;
        invokeInstall().then(function () { refresh(panel); })
            .catch(function (e) { showErr(panel, e); });
    });
    bind(panel, "#gbArtResumeReboot", function (btn) {
        btn.disabled = true;
        invokeInstall().then(function () { refresh(panel); })
            .catch(function (e) { showErr(panel, e); });
    });
    bind(panel, "#gbArtRetry", function (btn) {
        btn.disabled = true;
        localPaused = false;
        invokeInstall().then(function () { refresh(panel); })
            .catch(function (e) { showErr(panel, e); });
    });
    bind(panel, "#gbArtRecheck", function (btn) {
        btn.disabled = true;
        btn.textContent = tr("Verificando...", "Checking...");
        refresh(panel);
    });
    bind(panel, "#gbArtRemove", function () { showRemoveConfirm(panel); });
    bind(panel, "#gbArtUpdate", function (btn) {
        btn.disabled = true;
        btn.textContent = tr("Verificando...", "Checking...");
        invokeCheckUpdate().then(function (st) {
            if (st && st.available_version && st.available_version !== st.runtime_version) {
                A.agentRuntimeState = st.state; renderCard(panel, st);
            } else {
                btn.textContent = tr("Nenhuma atualização", "No update available");
                setTimeout(function () { btn.disabled = false; btn.textContent = tr("Verificar atualização", "Check for update"); }, 2000);
            }
        }).catch(function (e) { btn.disabled = false; btn.textContent = tr("Verificar atualização", "Check for update"); showErr(panel, e); });
    });
    bind(panel, "#gbArtCopyIt", function (btn) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(T.policyText()).then(function () {
                btn.textContent = tr("Copiado", "Copied");
                setTimeout(function () { btn.textContent = tr("Copiar texto", "Copy text"); }, 1500);
            });
        }
    });
}

function bind(panel, sel, fn) {
    var el = panel.querySelector(sel);
    if (el) { el.addEventListener("click", function () { fn(el); }); }
}

/* ---- remove confirmation ---- */

function showRemoveConfirm(panel) {
    var card = panel.querySelector(".gb-set-card");
    if (!card) { return; }
    var box = document.createElement("div");
    box.className = "gb-art-confirm";
    box.innerHTML =
        '<p class="gb-set-desc" style="margin-top:8px;color:var(--id-danger-text)">' +
        esc(tr("Remover o Modo Agente apaga o ambiente Linux e tudo que os agentes criaram dentro dele. Essa ação não pode ser desfeita.",
                "Removing Agent Mode deletes the Linux environment and everything agents created inside it. This action cannot be undone.")) +
        '</p><label style="display:flex;align-items:center;gap:8px;margin:8px 0;font-size:var(--id-text-sm)">' +
        '<input type="checkbox" id="gbArtKeepExport" />' +
        esc(tr("Guardar uma cópia dos dados antes de remover", "Keep a copy of the data before removing")) +
        '</label><div style="display:flex;gap:8px">' +
        '<button type="button" class="gb-set-btn-sm gb-set-btn-danger" id="gbArtConfirmRemove">' +
        esc(tr("Remover", "Remove")) + '</button>' +
        '<button type="button" class="gb-set-btn-sm" id="gbArtCancelRemove">' +
        esc(tr("Cancelar", "Cancel")) + '</button></div>';
    card.appendChild(box);
    box.querySelector("#gbArtCancelRemove").addEventListener("click", function () { box.remove(); });
    box.querySelector("#gbArtConfirmRemove").addEventListener("click", function () {
        var keep = box.querySelector("#gbArtKeepExport").checked;
        box.querySelector("#gbArtConfirmRemove").disabled = true;
        invokeRemove(keep).then(function () { refresh(panel); }).catch(function (e) { showErr(panel, e); });
    });
}

/* ---- section entry point ---- */

function renderAgentRuntime(panel) {
    fetchStatus().then(function (status) {
        if (!status) {
            panel.innerHTML = '<div class="gb-set-card"><h2 class="gb-set-title">' +
                esc(tr("Modo Agente", "Agent Mode")) + '</h2><p class="gb-set-desc">' +
                esc(tr("Não foi possível obter o estado do Modo Agente.",
                        "Could not retrieve Agent Mode status.")) + '</p></div>';
            return;
        }
        A.agentRuntimeState = status.state;
        renderCard(panel, status);
    }).catch(function (e) {
        panel.innerHTML = '<div class="gb-set-error"><p>' + esc(String(e)) + '</p></div>';
    });
}

/* ---- public API for other modules ---- */
A.openAgentRuntimeSettings = function () { A.openSettings("agent-runtime"); };
A.agentRuntimeReady = function () { return A.agentRuntimeState === "ready"; };
A.fetchAgentRuntimeState = function () {
    return fetchStatus().then(function (st) {
        if (st) { A.agentRuntimeState = st.state; }
        return st;
    });
};

/* ---- register ---- */
A.registerSettingsSection({
    id: "agent-runtime", icon: "server",
    key: "settings.agent-runtime", render: renderAgentRuntime
});

})();
