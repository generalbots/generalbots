"use strict";
/**
 * 08c_consent_vm_bar.js - "Permitir tudo" bar and waiting conversations.
 *
 * - A slim bar above the composer of the conversation that holds an
 *   "allow all" grant (row in /api/consent/permissions with action_class
 *   "vm_all" and app_id "vm:<agent>:<thread>"): "Permissão total: <agente>
 *   · Revogar" (DELETE /api/consent/permissions/<id>).
 * - Every conversation with a card still waiting gets "Aguardando sua
 *   permissão" in its sidebar row. After a reload, when the shell opens on
 *   an empty conversation and another one has a card waiting, that one is
 *   opened so the card can be answered.
 */

(function () {
    var A = window.GBAssistant;
    var SHIELD = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>';

    function esc(s) {
        return String(s == null ? "" : s)
            .replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }

    function activeThread() { return String(A.currentSessionId || A.sessionId || ""); }

    /* ---- "Permitir tudo" bar ---- */
    var grantsFor = null;
    var grants = [];

    function liveAllGrants(rows, thread) {
        var now = Date.now();
        return rows.filter(function (row) {
            var app = String(row.app_id || "");
            return row.action_class === "vm_all" && row.granted !== false && app.indexOf("vm:") === 0 &&
                app.slice(-thread.length - 1) === ":" + thread &&
                (!row.expires_at || Date.parse(row.expires_at) > now);
        });
    }

    function barEl() {
        var bar = document.getElementById("gbVmAllBar");
        var dock = document.getElementById("gbComposerDock");
        if (bar || !dock) { return bar; }
        bar = document.createElement("div");
        bar.id = "gbVmAllBar";
        bar.className = "gb-vm-allbar";
        bar.setAttribute("role", "status");
        bar.hidden = true;
        dock.parentNode.insertBefore(bar, dock);
        bar.addEventListener("click", function (e) {
            var btn = e.target.closest(".gb-vm-allbar-revoke");
            if (btn) { revoke(btn.getAttribute("data-id"), btn); }
        });
        return bar;
    }

    function renderBar() {
        var bar = barEl();
        if (!bar) { return; }
        var dock = document.getElementById("gbComposerDock");
        var shown = grantsFor === activeThread() && dock && dock.style.display !== "none" ? grants : [];
        bar.hidden = !shown.length;
        bar.innerHTML = shown.map(function (row) {
            var agent = String(row.app_id).split(":")[1] || "chat";
            return '<span class="gb-vm-allbar-item">' + SHIELD +
                '<strong>' + esc(A.t("assistant.vm-chip", "Permissão total: {agent}").replace("{agent}", A.vmAgentName(agent))) + '</strong>' +
                '<span class="gb-vm-allbar-scope">' + esc(A.t("assistant.vm-all-scope", "só dentro da Máquina Virtual")) + '</span>' +
                '<span aria-hidden="true">·</span><button type="button" class="gb-vm-allbar-revoke" data-id="' + esc(row.id) + '">' +
                esc(A.t("assistant.vm-revoke", "Revogar")) + '</button></span>';
        }).join("");
    }

    function refresh(force) {
        var thread = activeThread();
        if (!thread || A.compiledFeatures.indexOf("consent") === -1) { grantsFor = thread; grants = []; renderBar(); return; }
        if (!force && grantsFor === thread) { renderBar(); return; }
        A.apiFetch("/api/consent/permissions").then(function (r) { return r.ok ? r.json() : null; }).then(function (d) {
            if (activeThread() !== thread) { return; }
            var rows = Array.isArray(d) ? d : ((d && (d.permissions || d.items || d.grants)) || []);
            grantsFor = thread;
            grants = liveAllGrants(rows, thread);
            renderBar();
        }).catch(function () { /* the bar stays as it was */ });
    }

    function revoke(id, btn) {
        if (!id) { return; }
        btn.disabled = true;
        A.apiFetch("/api/consent/permissions/" + encodeURIComponent(id), { method: "DELETE" }).then(function (r) {
            if (!r.ok) { throw new Error("revoke"); }
            grants = grants.filter(function (row) { return String(row.id) !== String(id); });
            renderBar();
            var input = document.querySelector("#gbComposerDock textarea");
            if (input) { input.focus(); }
            A.announce(A.t("assistant.vm-revoked", "Permissão total revogada"));
        }).catch(function () {
            btn.disabled = false;
            A.announce(A.t("assistant.consent-failed", "Não foi possível registrar a decisão"));
        });
    }

    A.onVmGrantChanged = function (decision) {
        if (decision === "allow_all") { refresh(true); }
    };

    /* ---- Conversations with a card waiting ---- */
    function markRows() {
        var waiting = A.pendingConsentThreads ? A.pendingConsentThreads() : {};
        var label = A.t("assistant.vm-waiting-row", "Aguardando sua permissão");
        document.querySelectorAll("#gbConversations .gb-conv-item[data-session-id]").forEach(function (row) {
            var on = !!waiting[row.getAttribute("data-session-id")];
            var mark = row.querySelector(".gb-conv-wait");
            if (on === !!mark && row.classList.contains("gb-conv-item--waiting") === on) { return; }
            row.classList.toggle("gb-conv-item--waiting", on);
            if (!on) { if (mark) { mark.remove(); } return; }
            if (!mark) {
                mark = document.createElement("span");
                mark.className = "gb-conv-wait";
                mark.textContent = label;
                (row.querySelector(".gb-conv-item-text") || row).appendChild(mark);
            }
        });
    }

    function emptyView() {
        var empty = document.getElementById("gbEmptyState");
        var msgs = document.getElementById("gbMessages");
        return !!empty && empty.style.display !== "none" && (!msgs || !msgs.children.length);
    }

    /* Once per page: a conversation with a card waiting is reopened when the
       shell starts on an empty one. */
    var checked = false;
    function firstCheck() {
        if (checked || !A.reviveConsentCards) { return; }
        checked = true;
        A.reviveConsentCards().then(function () {
            markRows();
            var waiting = A.pendingConsentThreads ? Object.keys(A.pendingConsentThreads()) : [];
            if (!waiting.length || waiting.indexOf(activeThread()) !== -1 || !emptyView()) { return; }
            var row = document.querySelector('#gbConversations .gb-conv-item[data-session-id="' + CSS.escape(waiting[0]) + '"]');
            if (row) { row.click(); }
        });
    }

    var observed = null;
    function watchList() {
        var list = document.getElementById("gbConversations");
        if (!list || observed === list) { return; }
        observed = list;
        new MutationObserver(function () {
            markRows();
            if (list.querySelector(".gb-conv-item[data-session-id]")) { firstCheck(); }
        }).observe(list, { childList: true, subtree: true });
    }

    A.onConsentPendingChanged = markRows;

    var baseSync = A.syncPermissionDock;
    A.syncPermissionDock = function () {
        if (baseSync) { baseSync(); }
        refresh(false);
        watchList();
    };

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", watchList);
    } else {
        watchList();
    }
})();
