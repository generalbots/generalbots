"use strict";
/**
 * 03a_stream.js - Streaming, thinking indicator, scroll management.
 * Split from 03_render.js to stay under 450 lines each.
 */

(function () {
    var A = window.GBAssistant;
    var STREAM_THROTTLE_MS = 100;
    var lastStreamRender = 0;
    var pendingStreamFrame = null;
    var userScrolledUp = false;

    /* ================================================================
       Scroll management
       ================================================================ */
    function getConv() { return document.getElementById("gbConversation"); }

    function isNearBottom() {
        var conv = getConv();
        if (!conv) { return true; }
        return (conv.scrollHeight - conv.scrollTop - conv.clientHeight) < 200;
    }

    A.scrollToBottom = function (smooth) {
        var conv = getConv();
        if (!conv) { return; }
        conv.scrollTo({ top: conv.scrollHeight, behavior: smooth ? "smooth" : "auto" });
        userScrolledUp = false;
        updateScrollPill();
    };

    A.isUserScrolledUp = function () { return userScrolledUp; };

    function updateScrollPill() {
        var pill = document.getElementById("gbScrollPill");
        if (!pill) { return; }
        pill.style.display = userScrolledUp && !isNearBottom() ? "" : "none";
    }

    A.initScroll = function () {
        var conv = getConv();
        if (!conv) { return; }
        conv.addEventListener("scroll", function () {
            userScrolledUp = !isNearBottom();
            updateScrollPill();
        });
        var existing = document.getElementById("gbScrollPill");
        if (!existing) {
            var pill = document.createElement("button");
            pill.id = "gbScrollPill";
            pill.className = "gb-scroll-pill";
            pill.style.display = "none";
            pill.textContent = A.t("assistant.scroll-bottom", "Go to end");
            pill.addEventListener("click", function () { A.scrollToBottom(true); });
            var above = document.getElementById("gbDockAbove");
            if (above) { above.insertBefore(pill, above.firstChild); }
        }
    };

    /* ================================================================
       Thinking indicator (live, with elapsed seconds)
       ================================================================ */
    var thinkingTimer = null;
    var thinkingStartTime = 0;

    A.showThinking = function () {
        if (!A.isStreaming && !document.getElementById("gbThinking")) {
            var msgs = document.getElementById("gbMessages");
            if (!msgs) { return; }
            thinkingStartTime = Date.now();
            var el = document.createElement("div");
            el.id = "gbThinking";
            el.className = "gb-thinking-row";
            el.innerHTML =
                '<div class="gb-thinking-pulse" aria-hidden="true"></div>' +
                '<span class="gb-thinking-label">' +
                    A.escapeHtml(A.t("assistant.thinking", "Thinking")) +
                '</span>' +
                '<span class="gb-thinking-elapsed"></span>';
            msgs.appendChild(el);
            if (!userScrolledUp) { A.scrollToBottom(false); }
            thinkingTimer = setInterval(function () {
                var secs = Math.floor((Date.now() - thinkingStartTime) / 1000);
                var elapsedEl = el.querySelector(".gb-thinking-elapsed");
                if (elapsedEl) { elapsedEl.textContent = secs + " s"; }
            }, 1000);
        }
    };

    A.hideThinking = function () {
        var el = document.getElementById("gbThinking");
        if (el) { el.remove(); }
        if (thinkingTimer) { clearInterval(thinkingTimer); thinkingTimer = null; }
    };

    A.thinkingElapsed = function () {
        if (!thinkingStartTime) { return 0; }
        return Math.floor((Date.now() - thinkingStartTime) / 1000);
    };

    A.resetThinkingTimer = function () { thinkingStartTime = 0; };

    /* ================================================================
       Streaming
       ================================================================ */
    A.startStream = function () {
        A.showConversation();
        A.hideThinking();
        A.isStreaming = true;
        A.streamingContent = "";
        A.streamingReasoning = "";
        var el = document.createElement("div");
        el.className = "gb-msg-assistant gb-streaming";
        el.setAttribute("aria-busy", "true");
        document.getElementById("gbMessages").appendChild(el);
        A.streamingEl = el;
        toggleStopBtn(true);
        if (!userScrolledUp) { A.scrollToBottom(false); }
    };

    A.appendStream = function (chunk, reasoning) {
        if (!A.streamingEl) { return; }
        if (chunk) { A.streamingContent += chunk; }
        if (reasoning) { A.streamingReasoning += reasoning; }
        var now = Date.now();
        if (now - lastStreamRender < STREAM_THROTTLE_MS) {
            if (!pendingStreamFrame) {
                pendingStreamFrame = requestAnimationFrame(function () {
                    pendingStreamFrame = null;
                    renderStreamContent();
                });
            }
            return;
        }
        lastStreamRender = now;
        renderStreamContent();
    };

    function renderStreamContent() {
        if (!A.streamingEl) { return; }
        var elapsed = A.thinkingElapsed();
        var html = "";
        if (A.streamingReasoning) {
            html += A.reasoningHtml(A.streamingReasoning, elapsed);
        }
        var cleaned = typeof A.cleanProtocol === "function"
            ? A.cleanProtocol(A.streamingContent) : A.streamingContent;
        if (typeof A.consentStripMarker === "function") {
            cleaned = A.consentStripMarker(cleaned);
        }
        html += A.mdToHtml(cleaned);
        A.streamingEl.innerHTML = html;
        A.enhanceCodeBlocks(A.streamingEl);
        if (!userScrolledUp) { A.scrollToBottom(false); }
    }

    function finalizeStreaming() {
        if (pendingStreamFrame) {
            cancelAnimationFrame(pendingStreamFrame);
            pendingStreamFrame = null;
        }
        if (A.streamingEl) { A.streamingEl.remove(); A.streamingEl = null; }
        A.isStreaming = false;
        toggleStopBtn(false);
    }
    A._finalizeStreaming = finalizeStreaming;

    A.endStream = function () {
        finalizeStreaming();
        A.streamingContent = "";
        A.streamingReasoning = "";
        if (!userScrolledUp) { A.scrollToBottom(true); }
    };

    function toggleStopBtn(show) {
        var sendBtn = document.getElementById("gbSendDock");
        var stopBtn = document.getElementById("gbStopBtn");
        if (sendBtn) { sendBtn.style.display = show ? "none" : ""; }
        if (stopBtn) { stopBtn.style.display = show ? "" : "none"; }
    }

    /* ================================================================
       Show/hide empty vs conversation
       ================================================================ */
    A.showConversation = function () {
        var empty = document.getElementById("gbEmptyState");
        var conv = document.getElementById("gbConversation");
        var dock = document.getElementById("gbComposerDock");
        var above = document.getElementById("gbDockAbove");
        if (empty) { empty.style.display = "none"; }
        if (conv) { conv.style.display = ""; }
        if (dock) { dock.style.display = ""; }
        if (above) { above.style.display = ""; }
    };

    A.showEmptyState = function () {
        var empty = document.getElementById("gbEmptyState");
        var conv = document.getElementById("gbConversation");
        var dock = document.getElementById("gbComposerDock");
        var above = document.getElementById("gbDockAbove");
        var msgs = document.getElementById("gbMessages");
        if (empty) { empty.style.display = ""; }
        if (conv) { conv.style.display = "none"; }
        if (dock) { dock.style.display = "none"; }
        if (above) { above.style.display = "none"; }
        if (msgs) { msgs.innerHTML = ""; }
        A.hideThinking();
        A.hasUserSent = false;
        var sub = document.getElementById("gbGreetingSub");
        if (sub) { sub.textContent = ""; }
    };
})();
