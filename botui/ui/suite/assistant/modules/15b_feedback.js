"use strict";
/**
 * 15b_feedback.js - Thumbs up/down feedback on assistant answers.
 *
 * Stored locally (localStorage) keyed by session + message index.
 * Restored when a conversation is reopened.
 */

(function () {
    var A = window.GBAssistant;
    var I = window.GBIcon || function () { return ""; };
    var STORE_KEY = "gb-feedback";

    function pt() { return A.locale && A.locale.startsWith("pt"); }

    /* ---- localStorage wrapper ---- */
    function loadStore() {
        try {
            var raw = localStorage.getItem(STORE_KEY);
            return raw ? JSON.parse(raw) : {};
        } catch (e) { return {}; }
    }

    function saveStore(store) {
        try { localStorage.setItem(STORE_KEY, JSON.stringify(store)); }
        catch (e) { /* quota */ }
    }

    function feedbackKey(sessId, idx) {
        return sessId + ":" + idx;
    }

    /** Get the stored feedback for a message: {vote:"up"|"down", comment?} or null. */
    A.getFeedback = function (sessId, idx) {
        var store = loadStore();
        return store[feedbackKey(sessId, idx)] || null;
    };

    /** Set feedback for a message. vote = "up" | "down" | null (clear). */
    A.setFeedback = function (sessId, idx, vote, comment) {
        var store = loadStore();
        var key = feedbackKey(sessId, idx);
        if (!vote) {
            delete store[key];
        } else {
            store[key] = { vote: vote };
            if (comment) { store[key].comment = comment.slice(0, 500); }
        }
        saveStore(store);
    };

    function voteButton(title, svg, on) {
        var b = document.createElement("button");
        b.type = "button";
        b.className = "gb-msg-toolbar-btn gb-feedback-btn";
        b.title = title;
        b.setAttribute("aria-label", title);
        b.innerHTML = svg;
        mark(b, on);
        return b;
    }

    function mark(b, on) {
        b.classList.toggle("active", on);
        b.setAttribute("aria-pressed", on ? "true" : "false");
    }

    /** Add the thumbs to the answer's action row, after Copy and Regenerate,
     *  so every action shares one row. The comment box opens under the row. */
    A.addFeedbackButtons = function (el, bar) {
        var sessId = A.currentSessionId || A.sessionId || "";
        var idx = el.dataset.messageIndex;
        var wrap = bar || el.querySelector(".gb-msg-toolbar");
        if (idx == null || !sessId || !wrap) { return; }
        idx = parseInt(idx, 10);

        var existing = A.getFeedback(sessId, idx);
        var up = voteButton(pt() ? "Boa resposta" : "Good answer", thumbUpSvg(14),
            !!(existing && existing.vote === "up"));
        var down = voteButton(pt() ? "Resposta ruim" : "Bad answer", thumbDownSvg(14),
            !!(existing && existing.vote === "down"));

        up.addEventListener("click", function () {
            var cur = A.getFeedback(sessId, idx);
            if (cur && cur.vote === "up") {
                A.setFeedback(sessId, idx, null);
                mark(up, false);
            } else {
                A.setFeedback(sessId, idx, "up");
                mark(up, true);
                mark(down, false);
                removeCommentBox(wrap);
            }
        });

        down.addEventListener("click", function () {
            var cur = A.getFeedback(sessId, idx);
            if (cur && cur.vote === "down") {
                A.setFeedback(sessId, idx, null);
                mark(down, false);
                removeCommentBox(wrap);
            } else {
                A.setFeedback(sessId, idx, "down");
                mark(down, true);
                mark(up, false);
                showCommentBox(wrap, sessId, idx);
            }
        });

        wrap.appendChild(up);
        wrap.appendChild(down);
    };

    function showCommentBox(wrap, sessId, idx) {
        var next = wrap.nextElementSibling;
        if (next && next.classList.contains("gb-feedback-comment")) { return; }
        var box = document.createElement("div");
        box.className = "gb-feedback-comment";
        var input = document.createElement("input");
        input.type = "text";
        input.className = "gb-feedback-input";
        input.placeholder = pt() ? "O que poderia melhorar?" : "What could be better?";
        input.maxLength = 500;
        var send = document.createElement("button");
        send.className = "gb-feedback-send";
        send.textContent = pt() ? "Enviar" : "Send";
        send.addEventListener("click", function () {
            var text = input.value.trim();
            A.setFeedback(sessId, idx, "down", text || undefined);
            box.remove();
        });
        input.addEventListener("keydown", function (e) {
            if (e.key === "Enter") { send.click(); }
            if (e.key === "Escape") { box.remove(); }
        });
        box.appendChild(input);
        box.appendChild(send);
        wrap.after(box);
        input.focus();
    }

    function removeCommentBox(wrap) {
        var box = wrap.nextElementSibling;
        if (box && box.classList.contains("gb-feedback-comment")) { box.remove(); }
    }

    function thumbUpSvg(size) {
        return '<svg width="' + size + '" height="' + size +
            '" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
            '<path d="M14 9V5a3 3 0 0 0-3-3l-4 9v11h11.28a2 2 0 0 0 2-1.7l1.38-9a2 2 0 0 0-2-2.3H14zM7 22H4a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2h3"/>' +
            '</svg>';
    }

    function thumbDownSvg(size) {
        return '<svg width="' + size + '" height="' + size +
            '" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
            '<path d="M10 15v4a3 3 0 0 0 3 3l4-9V2H5.72a2 2 0 0 0-2 1.7l-1.38 9a2 2 0 0 0 2 2.3H10zM17 2h2.67A2.31 2.31 0 0 1 22 4v7a2.31 2.31 0 0 1-2.33 2H17"/>' +
            '</svg>';
    }
})();
