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
    var thinkingWatchdog = null;
    var THINKING_TIMEOUT_S = 90;

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
        // Safety watchdog: if no server frame arrives within the timeout,
        // replace the indicator with a timeout error card (#2562). Every
        // call (the server's keep-alive frames included) starts it again.
        if (document.getElementById("gbThinking")) {
            if (thinkingWatchdog) { clearTimeout(thinkingWatchdog); }
            thinkingWatchdog = setTimeout(function () {
                if (document.getElementById("gbThinking")) {
                    A.hideThinking();
                    if (typeof A.renderErrorCode === "function") {
                        A.renderErrorCode("timeout",
                            A.t("assistant.error-timeout",
                                "The server did not respond in time."));
                    }
                }
            }, THINKING_TIMEOUT_S * 1000);
        }
    };

    A.hideThinking = function () {
        var el = document.getElementById("gbThinking");
        if (el) { el.remove(); }
        if (thinkingTimer) { clearInterval(thinkingTimer); thinkingTimer = null; }
        if (thinkingWatchdog) { clearTimeout(thinkingWatchdog); thinkingWatchdog = null; }
    };

    A.thinkingElapsed = function () {
        if (!thinkingStartTime) { return 0; }
        return Math.floor((Date.now() - thinkingStartTime) / 1000);
    };

    A.resetThinkingTimer = function () { thinkingStartTime = 0; A._thoughtSeconds = 0; };

    /* ================================================================
       Streaming
       ================================================================ */
    A.startStream = function () {
        A.showConversation();
        A.hideThinking();
        A.isStreaming = true;
        A.streamingContent = "";
        A.streamingReasoning = "";
        A._thoughtSeconds = 0;
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
        /* The thinking ends when the answer starts: keep how long it took. */
        if (chunk && !A.streamingContent && A.streamingReasoning) { A._thoughtSeconds = A.thinkingElapsed(); }
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
            var answering = !!A.streamingContent;
            html += A.reasoningHtml(A.streamingReasoning, answering ? (A._thoughtSeconds || elapsed) : elapsed, !answering);
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
        /* When cancelled with partial content, keep the text on screen
           by promoting the streaming element to a normal assistant
           message.  We do a final render, strip the streaming markers,
           and add the action toolbar, so the partial behaves like any
           other assistant message. */
        var partial = A.streamingContent || "";
        var partialR = A.streamingReasoning || "";
        if (partial && A.streamingEl) {
            renderStreamContent(); /* flush the last frame */
            A.streamingEl.classList.remove("gb-streaming");
            A.streamingEl.removeAttribute("aria-busy");
            A.streamingEl.dataset.originalContent = partial;
            A.streamingEl.dataset.originalReasoning = partialR;
            if (typeof A.addAssistantToolbar === "function") {
                A.addAssistantToolbar(A.streamingEl, partial, false);
            }
            A.streamingEl = null;
        }
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
        A.greetingReceived = false;
        // Clear streaming leftovers so a suggestions-only frame from the new
        // session does not pick up the previous conversation's last answer.
        A.streamingContent = "";
        A.streamingReasoning = "";
        A._greetingBuffer = "";
        A.lastCompletedContent = "";
        var sub = document.getElementById("gbGreetingSub");
        if (sub) { sub.textContent = ""; }
    };
})();
