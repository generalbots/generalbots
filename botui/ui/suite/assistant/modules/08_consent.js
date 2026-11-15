"use strict";
/**
 * 08_consent.js - Consent/permission card for the assistant shell.
 *
 * Port of the legacy 40_consent_cards.js protocol, restyled for the
 * assistant shell identity.  The server sends a consent_request in a
 * bot-response frame (message_type 2) or embeds a base64 marker
 * "GB-CONSENT-REQUEST:<base64>" in the answer text.
 *
 * Decisions POST /api/consent/resolve and, on grant, send a continuation
 * message to resume the turn.
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
            var b64 = content.slice(idx + MARKER.length).trim();
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
        var card = document.createElement("div");
        card.className = "gb-consent-card";
        card.setAttribute("data-request-id", String(req.request_id));

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
                '<button type="button" class="gb-consent-btn gb-consent-allow" data-decision="allow_once">' + escHtml(allowLabel) + '</button>' +
                '<button type="button" class="gb-consent-btn gb-consent-deny" data-decision="deny">' + escHtml(denyLabel) + '</button>' +
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
        var buttons = card.querySelectorAll(".gb-consent-btn");
        for (var i = 0; i < buttons.length; i++) { buttons[i].disabled = true; }

        A.apiFetch("/api/consent/resolve", {
            method: "POST",
            body: JSON.stringify({ request_id: requestId, decision: decision })
        }).then(function (r) {
            if (!r.ok) { throw new Error("HTTP " + r.status); }
            swapOutcome(card, decision, true);
            resumeAfterGrant(decision);
        }).catch(function () {
            swapOutcome(card, decision, false);
        });
    }

    function swapOutcome(card, decision, ok) {
        var granted = decision !== "deny";
        var outcomeText = ok
            ? (granted
                ? A.t("assistant.consent-granted", "Allowed")
                : A.t("assistant.consent-denied", "Denied"))
            : A.t("assistant.consent-failed", "Could not record decision");
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

    function resumeAfterGrant(decision) {
        if (decision === "deny") { return; }
        setTimeout(function () {
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

    /* Revive pending requests on load */
    A.reviveConsentCards = function (container) {
        if (A.compiledFeatures.indexOf("consent") === -1) { return; }
        A.apiFetch("/api/consent/pending").then(function (r) {
            if (r.status === 404) { return null; }
            return r.ok ? r.json() : null;
        }).then(function (d) {
            if (!d) { return; }
            var items = Array.isArray(d) ? d : (d.items || d.requests || d.pending || []);
            for (var i = 0; i < items.length; i++) {
                if (!items[i] || !items[i].request_id) { continue; }
                if (container.querySelector('[data-request-id="' + items[i].request_id + '"]')) { continue; }
                var wrap = document.createElement("div");
                wrap.className = "gb-msg-assistant";
                wrap.appendChild(buildCard(items[i]));
                container.appendChild(wrap);
            }
        }).catch(function () { /* best-effort */ });
    };
})();
