"use strict";
/**
 * 05c_agent_toggle.js — Agent mode toggle in the composer.
 *
 * Off by default: the assistant answers from its knowledge and the API
 * command catalog. When turned on, the assistant can run commands in an
 * isolated workspace on this computer (requires WSL2 + Incus on Windows).
 * The state is remembered per conversation in localStorage.
 */
(function () {
    var A = window.GBAssistant;
    var STORAGE_PREFIX = "gb-agent-mode:";

    /* ---- persistence (per conversation) ---- */
    function storageKey() {
        var sid = A.currentSessionId || A.sessionId || "global";
        return STORAGE_PREFIX + (A.botName || "default") + ":" + sid;
    }

    function readStored() {
        try {
            var raw = localStorage.getItem(storageKey());
            if (raw === "true") { return true; }
        } catch (e) { /* private window */ }
        return false;
    }

    function writeStored(value) {
        try { localStorage.setItem(storageKey(), value ? "true" : "false"); }
        catch (e) { /* */ }
    }

    /* ---- availability ---- */
    function backendAvailable() {
        var cfg = A.productConfig;
        if (!cfg) { return false; }
        /* The server reports agent_backend_unavailable as null when the
           backend is usable, or a reason string when it is not. */
        var reason = cfg.agent_backend_unavailable;
        return reason === null || reason === undefined;
    }

    function unavailableReason() {
        var cfg = A.productConfig;
        if (!cfg) { return null; }
        return cfg.agent_backend_unavailable || null;
    }

    /* ---- UI ---- */
    function createToggle(containerId) {
        var container = document.getElementById(containerId);
        if (!container) { return null; }
        /* The toggle goes in the .gb-composer-row (or the footer row in the
           empty-state card), next to the model picker. */
        var row = container.querySelector(".gb-engine-toggles");
        if (!row) { return null; }

        var btn = document.createElement("button");
        btn.type = "button";
        btn.className = "gb-agent-toggle gb-engine-toggle";
        btn.setAttribute("aria-pressed", "false");
        btn.title = A.t("assistant.agent-label", "Agente");

        /* Icon: a simple terminal/command prompt glyph */
        btn.innerHTML =
            '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" ' +
            'stroke="currentColor" stroke-width="2" stroke-linecap="round" ' +
            'stroke-linejoin="round">' +
            '<polyline points="4 17 10 11 4 5"/>' +
            '<line x1="12" y1="19" x2="20" y2="19"/>' +
            '</svg>' +
            '<span class="gb-agent-label">' +
            A.t("assistant.agent-label", "Agente") + '</span><span class="gb-toggle-track"></span>';

        row.prepend(btn);
        return btn;
    }

    function syncUI(btn, enabled) {
        if (!btn) { return; }
        btn.setAttribute("aria-pressed", enabled ? "true" : "false");
        btn.classList.toggle("gb-agent-on", enabled);
        btn.title = enabled
            ? A.t("assistant.agent-on-tip", "Agente ativado: o assistente pode executar comandos no computador")
            : A.t("assistant.agent-off-tip", "Agente desativado: o assistente responde apenas com texto");
    }

    /* ---- init ---- */
    A.initAgentToggle = function () {
        /* The toggle is only meaningful when the server compiled agent-vm. */
        var features = A.compiledFeatures || [];
        var hasFeature = features.indexOf("agent-vm") !== -1;
        if (!hasFeature) { return; }

        A.agentEnabled = readStored();

        var dockBtn = createToggle("gbComposerDock") ?
            document.querySelector("#gbComposerDock .gb-agent-toggle") : null;
        var emptyBtn = createToggle("gbComposerCardEmpty") ?
            document.querySelector("#gbComposerCardEmpty .gb-agent-toggle") : null;

        /* When the backend is not available, show the reason on click
           instead of toggling. */
        function handleClick() {
            if (!backendAvailable()) {
                A.announce(A.t("assistant.agent-unavailable",
                    "Agent mode is not available on this computer."));
                return;
            }
            A.agentEnabled = !A.agentEnabled;
            writeStored(A.agentEnabled);
            syncUI(dockBtn, A.agentEnabled);
            syncUI(emptyBtn, A.agentEnabled);
        }

        if (dockBtn) {
            dockBtn.addEventListener("click", handleClick);
            syncUI(dockBtn, A.agentEnabled);
        }
        if (emptyBtn) {
            emptyBtn.addEventListener("click", handleClick);
            syncUI(emptyBtn, A.agentEnabled);
        }
    };

    /* Turned on from the model menu when a subscription agent is picked. */
    A.setAgentEnabled = function (value) {
        writeStored(!!value);
        A.syncAgentToggle();
    };

    /* Called when switching conversations so the toggle reflects the
       stored state for the new conversation. */
    A.syncAgentToggle = function () {
        A.agentEnabled = readStored();
        var btn = document.querySelector("#gbComposerDock .gb-agent-toggle");
        syncUI(btn, A.agentEnabled);
        var emptyBtn = document.querySelector("#gbComposerCardEmpty .gb-agent-toggle");
        syncUI(emptyBtn, A.agentEnabled);
    };
})();
