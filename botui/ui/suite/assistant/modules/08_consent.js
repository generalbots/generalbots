"use strict";
/**
 * 08_consent.js - Consent/permission card for the assistant shell.
 *
 * Port of the legacy 40_consent_cards.js protocol, restyled for the
 * assistant shell identity.  The server sends a consent_request in a
 * bot-response frame (message_type 2) or embeds a base64 marker
 * "GB-CONSENT-REQUEST:<base64>" in the answer text.
 *
 * Decisions POST /api/consent/resolve. A blocking request (Virtual Machine
 * action) wakes the waiting tool call; any other grant sends a
 * continuation message to resume the turn.
 */

(function () {
    var A = window.GBAssistant;

    var MARKER = "GB-CONSENT-REQUEST:";
    var MARKER_RE = /GB-CONSENT-REQUEST:[A-Za-z0-9+/=_-]+/g;

    function escHtml(s) {
        return String(s == null ? "" : s)
            .replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }

    /* ---- Decode base64 marker ---- */
    function decodeMarker(content) {
        if (typeof content !== "string") { return null; }
        var idx = content.indexOf(MARKER);
        if (idx === -1) { return null; }
        try {
            var b64 = (content.slice(idx + MARKER.length).match(/^[A-Za-z0-9+/=_-]+/) || [""])[0];
            var std = b64.replace(/-/g, "+").replace(/_/g, "/");
            while (std.length % 4 !== 0) { std += "="; }
            var bin = atob(std);
            var json = decodeURIComponent(
                Array.prototype.map.call(bin, function (c) {
                    return "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2);
                }).join("")
            );
            var req = JSON.parse(json);
            return req && req.request_id ? req : null;
        } catch (e) { return null; }
    }

    /* ---- Describe the detail object in plain language ---- */
    function describeDetail(detail) {
        if (detail == null) { return ""; }
        if (typeof detail === "string") { return detail; }
        if (typeof detail !== "object") { return String(detail); }
        try {
            var parts = [];
            var command = detail.command || detail.tool || detail.name;
            if (command) { parts.push(String(command)); }
            var params = detail.params;
            if (params && typeof params === "object") {
                var method = params.method ? String(params.method).toUpperCase() + " " : "";
                var path = params.path || params.url || "";
                if (method || path) { parts.push(" → " + method + path); }
            }
            var text = parts.join("").replace(/\s+/g, " ").trim();
            return text || JSON.stringify(detail).slice(0, 200);
        } catch (e) { return ""; }
    }

    /* ---- Build the card DOM ---- */
    function buildCard(req) {
        if (req.detail && req.detail.vm && A.buildVmConsentCard) {
            return A.buildVmConsentCard(req, resolve, activeThread());
        }
        var card = document.createElement("div");
        card.className = "gb-consent-card";
        card.dataset.requestId = req.request_id;
        card.dataset.threadId = req.thread_id || activeThread();

        var detail = req.detail && typeof req.detail === "object" ? req.detail : {};
        var appName = req.app_id || detail.app_id || "workspace";
        var action = req.action_class || detail.action_class || "";

        var titleText = A.t("assistant.consent-title", "Permission request");
        var subtitleText = appName + (action ? " · " + action : "");
        var detailText = describeDetail(req.detail);
        var allowLabel = A.t("assistant.consent-allow", "Allow once");
        var denyLabel = A.t("assistant.consent-deny", "Deny");

        card.innerHTML =
            '<div class="gb-consent-head">' +
                '<svg class="gb-consent-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>' +
                '<div class="gb-consent-headings">' +
                    '<div class="gb-consent-title">' + escHtml(titleText) + '</div>' +
                    '<div class="gb-consent-subtitle">' + escHtml(subtitleText) + '</div>' +
                '</div>' +
            '</div>' +
            (detailText ? '<div class="gb-consent-detail">' + escHtml(detailText) + '</div>' : '') +
            '<div class="gb-consent-actions">' +
                '<button type="button" class="gb-consent-btn gb-consent-deny" data-decision="deny">' + escHtml(denyLabel) + '</button>' +
                '<button type="button" class="gb-consent-btn gb-consent-allow" data-decision="allow_once">' + escHtml(allowLabel) + '</button>' +
            '</div>';

        card.addEventListener("click", function (e) {
            var btn = e.target.closest(".gb-consent-btn");
            if (!btn) { return; }
            resolve(card, req.request_id, btn.getAttribute("data-decision"));
        });

        return card;
    }

    /* ---- Resolve ---- */
    function resolve(card, requestId, decision) {
        var hadFocus = card.contains(document.activeElement);
        var buttons = card.querySelectorAll(".gb-consent-btn");
        for (var i = 0; i < buttons.length; i++) { buttons[i].disabled = true; }

        A.apiFetch("/api/consent/resolve", {
            method: "POST",
            body: JSON.stringify({ request_id: requestId, decision: decision, thread_id: card.dataset.threadId || undefined })
        }).then(function (r) {
            return r.json().catch(function () { return null; }).then(function (body) { return { ok: r.ok, status: r.status, body: body }; });
        }).then(function (res) {
            // The server refuses a decision that does not belong to this
            // conversation: the card stays open for a valid answer.
            if (res.status === 403 || (res.body && res.body.error === "forbidden")) {
                refuseHere(card);
                return;
            }
            if (!res.ok || (res.body && res.body.error)) {
                // #2561 — expired or unknown request: show a clear message
                // and release the composer so the conversation is not stuck.
                swapExpired(card);
                retireCard(card, "failed");
                return;
            }
            swapOutcome(card, decision, true);
            retireCard(card, "done");
            // A blocking request wakes the waiting tool call on the server.
            if (card.dataset.blocking === "1") {
                if (A.onVmConsentDecision) { A.onVmConsentDecision(decision, hadFocus); }
            } else {
                resumeAfterGrant(decision, card.dataset.threadId);
            }
        }).catch(function () {
            card.querySelectorAll(".gb-consent-btn").forEach(function (button) { button.disabled = false; });
            A.announce(A.t("assistant.consent-failed", "Could not record decision"));
        });
    }

    function refuseHere(card) {
        card.querySelectorAll(".gb-consent-btn").forEach(function (button) { button.disabled = false; });
        var line = card.querySelector(".gb-consent-refused");
        if (!line) {
            line = document.createElement("p");
            line.className = "gb-consent-note gb-consent-warn gb-consent-refused";
            line.setAttribute("role", "alert");
            card.appendChild(line);
        }
        line.textContent = A.t("assistant.consent-forbidden", "Esta decisão não pôde ser registrada aqui.");
    }

    /* Expired or unknown (the turn that asked is gone): the card closes. */
    function swapExpired(card) {
        var label = A.t("assistant.consent-expired", "Este pedido expirou.");
        card.classList.add("gb-consent-resolved");
        card.innerHTML =
            '<div class="gb-consent-outcome">' +
                '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">' +
                    '<circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>' +
                '</svg>' +
                '<span>' + escHtml(label) + '</span>' +
            '</div>';
    }

    function swapOutcome(card, decision, ok) {
        var granted = decision !== "deny";
        var vmText = ok && card.dataset.blocking === "1" && A.vmOutcomeText ? A.vmOutcomeText(decision) : null;
        var outcomeText = vmText || (ok
            ? (granted
                ? A.t("assistant.consent-granted", "Allowed")
                : A.t("assistant.consent-denied", "Denied"))
            : A.t("assistant.consent-failed", "Could not record decision"));
        card.classList.add("gb-consent-resolved");
        card.innerHTML =
            '<div class="gb-consent-outcome">' +
                '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">' +
                    (ok && granted
                        ? '<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/>'
                        : '<circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/>') +
                '</svg>' +
                '<span>' + escHtml(outcomeText) + '</span>' +
            '</div>';
    }

    function resumeAfterGrant(decision, thread) {
        if (decision === "deny") { return; }
        setTimeout(function () {
            if (activeThread() !== thread) { return; }
            try {
                A.sendMessage("Approved. Continue and run the previously requested command now with the same parameters.");
            } catch (e) { /* grant still applies */ }
        }, 200);
    }

    /* ---- Public API ---- */
    A.consentDecodeMarker = decodeMarker;
    A.consentStripMarker = function (text) {
        if (typeof text !== "string") { return text; }
        return text.replace(MARKER_RE, "").replace(/[ \t]+$/gm, "").replace(/\n{3,}/g, "\n\n").trim();
    };
    A.consentBuildCard = buildCard;

    /* Last request from WS frame */
    var lastConsentRequest = null;

    A.onConsentFrame = function (data) {
        if (data.consent_request && data.consent_request.request_id) {
            lastConsentRequest = data.consent_request;
        }
    };

    /**
     * Check if a message contains a consent request. Returns the card
     * element or null.
     */
    A.consentFromContent = function (content) {
        var req = lastConsentRequest || decodeMarker(content);
        if (!req) { return null; }
        lastConsentRequest = null;
        return buildCard(req);
    };

    /* Every pending card the shell knows, of every conversation. A card is
       shown only in its own conversation: a Virtual Machine request inside
       the answer that waits for it (a slot in the steps), any other request
       in the dock above the composer. */
    var cards = new Map();
    var settled = new Set();

    function activeThread() {
        return String(A.currentSessionId || A.sessionId || "");
    }

    function conversationShown() {
        var conv = document.getElementById("gbConversation");
        return !!conv && conv.style.display !== "none";
    }

    function slotFor(card) {
        var msgs = document.getElementById("gbMessages");
        if (!msgs || !conversationShown()) { return null; }
        var slot = msgs.querySelector('.gb-consent-slot[data-request-id="' + CSS.escape(card.dataset.requestId) + '"]');
        return slot || (A.consentSlot ? A.consentSlot(card.dataset.requestId) : null);
    }

    A.syncPermissionDock = function () {
        var thread = activeThread();
        var docked = [];
        cards.forEach(function (card) {
            var here = card.dataset.threadId === thread;
            if (card.dataset.blocking !== "1") { if (here) { docked.push(card); } return; }
            var slot = here ? slotFor(card) : null;
            if (!slot) { if (card.parentNode) { card.remove(); } return; }
            if (card.parentNode !== slot) {
                slot.appendChild(card);
                if (A.onConsentCardShown) { A.onConsentCardShown(card); }
            }
        });
        var dock = document.getElementById("gbPermissionDock");
        if (!dock) { return; }
        if (docked.length !== dock.children.length || docked.some(function (card, i) { return dock.children[i] !== card; })) {
            dock.replaceChildren.apply(dock, docked);
        }
        dock.hidden = docked.length === 0;
    };

    A.queueConsentCard = function (card) {
        var id = card.dataset.requestId;
        if (cards.has(id) || settled.has(id)) { return; }
        cards.set(id, card);
        A.syncPermissionDock();
        if (A.onConsentPendingChanged) { A.onConsentPendingChanged(); }
    };

    function drop(id) {
        var card = cards.get(id);
        settled.add(id);
        cards.delete(id);
        if (card) {
            var slot = card.parentNode;
            card.remove();
            if (slot && slot.classList && slot.classList.contains("gb-consent-slot")) { slot.remove(); }
        }
        A.syncPermissionDock();
        if (A.onConsentPendingChanged) { A.onConsentPendingChanged(); }
    }

    /* The server closes the card when the request is decided. A card the
       person just answered keeps its outcome line for a moment first. */
    A.removeConsentCard = function (id) {
        id = String(id);
        var card = cards.get(id);
        if (card && card.dataset.retiring === "1") { return; }
        drop(id);
    };

    /* Conversations with a card still waiting for an answer, and how. */
    A.pendingConsentThreads = function () {
        var threads = {};
        cards.forEach(function (card) {
            if (card.classList.contains("gb-consent-resolved")) { return; }
            /* "wait": a turn is paused on it; "open": an offer after the turn. */
            var id = card.dataset.threadId;
            threads[id] = card.dataset.blocking === "1" || threads[id] === "wait" ? "wait" : "open";
        });
        return threads;
    };

    A.openConsentCard = function () {
        var thread = activeThread();
        var open = null;
        cards.forEach(function (card) {
            if (!open && card.dataset.threadId === thread && card.isConnected && !card.classList.contains("gb-consent-resolved")) { open = card; }
        });
        return open;
    };

    function retireCard(card, status) {
        var id = card.dataset.requestId;
        settled.add(id);
        card.dataset.retiring = "1";
        if (A.setPermissionActivityStatus) { A.setPermissionActivityStatus(id, status); }
        if (A.onConsentPendingChanged) { A.onConsentPendingChanged(); }
        setTimeout(function () { drop(id); }, 2500);
    }

    /* Reads every pending request. With an id, only that one is added; a
       full read also forgets cards the server no longer holds (decided or
       expired elsewhere). Unscoped older requests join the open
       conversation. Returns a promise. */
    A.reviveConsentCards = function (requestId) {
        if (A.compiledFeatures.indexOf("consent") === -1) { return Promise.resolve(); }
        return A.apiFetch("/api/consent/pending").then(function (r) {
            return r.ok ? r.json() : null;
        }).then(function (d) {
            if (!d) { return; }
            var items = Array.isArray(d) ? d : (d.items || d.requests || d.pending || []);
            var live = {};
            items.forEach(function (request) {
                if (!request || !request.request_id) { return; }
                live[request.request_id] = true;
                if (!requestId || request.request_id === requestId) { A.queueConsentCard(buildCard(request)); }
            });
            if (!requestId) {
                Array.from(cards.keys()).forEach(function (id) {
                    if (!live[id] && cards.get(id).dataset.retiring !== "1") { drop(id); }
                });
            }
            A.syncPermissionDock();
        }).catch(function () { /* best-effort */ });
    };
})();
