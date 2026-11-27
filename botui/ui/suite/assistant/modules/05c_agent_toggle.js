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
            A.t("assistant.agent-label", "Agente") + '</span><span class="gb-toggle-track"></span>' +
            '<svg class="gb-agent-lock" width="11" height="11" viewBox="0 0 24 24" fill="none" ' +
            'stroke="currentColor" stroke-width="2.2" stroke-linecap="round" ' +
            'stroke-linejoin="round" aria-hidden="true">' +
            '<rect x="5" y="11" width="14" height="10" rx="1"/>' +
            '<path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg>';

        row.prepend(btn);
        return btn;
    }

    /** Is the selected model a subscription agent? */
    function impliedByModel() {
        return /^sub:/.test(A.selectedModel || "");
    }

    function syncUI(btn, enabled) {
        if (!btn) { return; }
        var locked = impliedByModel();
        var on = locked || enabled;
        btn.setAttribute("aria-pressed", on ? "true" : "false");
        btn.classList.toggle("gb-agent-on", on);
        btn.classList.toggle("gb-agent-locked", locked);
        /* Held on by the chosen agent: it stays a live, focusable control
           that explains itself on click. Only a missing backend disables it. */
        btn.disabled = locked && !backendAvailable();
        if (locked && !btn.disabled) { btn.setAttribute("aria-disabled", "true"); }
        else { btn.removeAttribute("aria-disabled"); }
        if (!locked) { closeHint(); }
        btn.title = locked
            ? A.t("assistant-agent-locked-tip", "The selected agent runs in the Virtual Machine; choose another model to turn it off")
            : on
            ? A.t("assistant.agent-on-tip", "Agente ativado: o assistente pode executar comandos no computador")
            : A.t("assistant.agent-off-tip", "Agente desativado: o assistente responde apenas com texto");
    }

    /* ---- hint shown when the held-on switch is pressed ---- */
    var hint = null;
    var hintTimer = 0;

    function selectedAgentName() {
        var rest = (A.selectedModel || "").replace(/^sub:/, "");
        var provider = rest.split("/")[0];
        var meta = A._agentMeta ? A._agentMeta(provider) : null;
        return meta ? meta.name : provider;
    }

    function closeHint() {
        if (!hint) { return; }
        clearTimeout(hintTimer);
        document.removeEventListener("keydown", onHintKey, true);
        document.removeEventListener("pointerdown", onHintPointer, true);
        var owner = document.querySelector('[aria-describedby="gbAgentHint"]');
        if (owner) { owner.removeAttribute("aria-describedby"); }
        hint.remove();
        hint = null;
    }

    function onHintKey(e) {
        if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); closeHint(); }
    }

    function onHintPointer(e) {
        if (hint && !hint.contains(e.target) && !e.target.closest(".gb-agent-toggle")) { closeHint(); }
    }

    function showLockedHint(btn) {
        closeHint();
        var text = A.t("assistant-agent-locked-hint",
            "{agent} runs in the Virtual Machine, so the Agent stays on while it is selected. To turn it off, choose another model.")
            .replace("{agent}", selectedAgentName());
        hint = document.createElement("div");
        hint.id = "gbAgentHint";
        hint.className = "gb-agent-hint";
        hint.setAttribute("role", "tooltip");
        hint.textContent = text;
        document.body.appendChild(hint);
        /* Client coordinates are divided by the app zoom: left/top are CSS pixels. */
        var zoom = window.GBZoom ? window.GBZoom.factor() : 1;
        var r = btn.getBoundingClientRect();
        var w = hint.offsetWidth, h = hint.offsetHeight;
        var vw = window.innerWidth / zoom, vh = window.innerHeight / zoom;
        var x = Math.max(8, Math.min(vw - w - 8, r.right / zoom - w));
        var y = r.top / zoom - h - 8;
        if (y < 8) { y = Math.min(vh - h - 8, r.bottom / zoom + 8); hint.classList.add("gb-agent-hint--below"); }
        hint.style.left = x + "px";
        hint.style.top = y + "px";
        /* The pointer of the bubble sits over the switch's track. */
        var track = btn.querySelector(".gb-toggle-track") || btn;
        var tr = track.getBoundingClientRect();
        hint.style.setProperty("--gb-hint-arrow", Math.max(10, Math.min(w - 10, (tr.left + tr.width / 2) / zoom - x)) + "px");
        btn.setAttribute("aria-describedby", hint.id);
        A.announce(text);
        hintTimer = setTimeout(closeHint, 5000);
        document.addEventListener("keydown", onHintKey, true);
        document.addEventListener("pointerdown", onHintPointer, true);
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
        function handleClick(e) {
            if (impliedByModel()) { showLockedHint(e.currentTarget); return; }
            if (typeof A.agentRuntimeReady === "function" && !A.agentRuntimeReady()) {
                A.announce(A.t("assistant-agent-mode-disabled",
                    "Agent Mode must be installed to use subscription agents."));
                if (typeof A.openAgentRuntimeSettings === "function") {
                    A.openAgentRuntimeSettings();
                }
                return;
            }
            if (!backendAvailable()) {
                A.announce(A.t("assistant.agent-unavailable",
                    "Agent mode is not available on this computer."));
                return;
            }
            var stored = !readStored();
            writeStored(stored);
            A.agentEnabled = stored;
            syncUI(dockBtn, stored);
            syncUI(emptyBtn, stored);
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

    /* Called from the refusal card's "Ligar Agente" action so the
       person can turn the agent on without reaching the toggle.
       Writes the value to storage and syncs the UI. */
    A.setAgentEnabled = function (value) {
        writeStored(!!value);
        A.syncAgentToggle();
    };

    /* Called when switching conversations so the toggle reflects the
       stored state for the new conversation. Also called after model
       selection changes so the implied-on rule is re-evaluated. */
    A.syncAgentToggle = function () {
        var stored = readStored();
        A.agentEnabled = impliedByModel() || stored;
        var btn = document.querySelector("#gbComposerDock .gb-agent-toggle");
        syncUI(btn, stored);
        var emptyBtn = document.querySelector("#gbComposerCardEmpty .gb-agent-toggle");
        syncUI(emptyBtn, stored);
    };
})();
