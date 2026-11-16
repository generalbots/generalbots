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

    /** Add feedback buttons to an assistant message element. */
    A.addFeedbackButtons = function (el) {
        var sessId = A.currentSessionId || A.sessionId || "";
        var idx = el.dataset.messageIndex;
        if (idx == null || !sessId) { return; }
        idx = parseInt(idx, 10);

        var wrap = document.createElement("span");
        wrap.className = "gb-feedback-wrap";

        var existing = A.getFeedback(sessId, idx);

        var up = document.createElement("button");
        up.className = "gb-feedback-btn" + (existing && existing.vote === "up" ? " active" : "");
        up.title = pt() ? "Boa resposta" : "Good answer";
        up.setAttribute("aria-label", up.title);
        up.innerHTML = thumbUpSvg(14);

        var down = document.createElement("button");
        down.className = "gb-feedback-btn" + (existing && existing.vote === "down" ? " active" : "");
        down.title = pt() ? "Resposta ruim" : "Bad answer";
        down.setAttribute("aria-label", down.title);
        down.innerHTML = thumbDownSvg(14);

        up.addEventListener("click", function () {
            var cur = A.getFeedback(sessId, idx);
            if (cur && cur.vote === "up") {
                A.setFeedback(sessId, idx, null);
                up.classList.remove("active");
            } else {
                A.setFeedback(sessId, idx, "up");
                up.classList.add("active");
                down.classList.remove("active");
                removeCommentBox(wrap);
            }
        });

        down.addEventListener("click", function () {
            var cur = A.getFeedback(sessId, idx);
            if (cur && cur.vote === "down") {
                A.setFeedback(sessId, idx, null);
                down.classList.remove("active");
                removeCommentBox(wrap);
            } else {
                A.setFeedback(sessId, idx, "down");
                down.classList.add("active");
                up.classList.remove("active");
                showCommentBox(wrap, sessId, idx);
            }
        });

        wrap.appendChild(up);
        wrap.appendChild(down);
        el.appendChild(wrap);
    };

    function showCommentBox(wrap, sessId, idx) {
        if (wrap.querySelector(".gb-feedback-comment")) { return; }
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
        wrap.appendChild(box);
        input.focus();
    }

    function removeCommentBox(wrap) {
        var box = wrap.querySelector(".gb-feedback-comment");
        if (box) { box.remove(); }
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
