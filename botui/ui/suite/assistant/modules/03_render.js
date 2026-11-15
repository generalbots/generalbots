"use strict";
/**
 * 03_render.js - Message rendering, streaming, markdown.
 *
 * Streaming protocol (matches the legacy 70_stream.js rules):
 *  - Chunk frame:  is_complete=false, content non-empty  -> append to stream
 *  - Thinking frame: thinking truthy                     -> keep indicator
 *  - Completion frame: is_complete=true                  -> finalize
 *    * If content is non-empty, USE IT (replace accumulated stream)
 *    * If content is empty, fall back to accumulated stream
 *    * Remove the streaming element, add the final message
 *    * Dedup: skip if same content within 60 s
 */

(function () {
    var A = window.GBAssistant;
    var STREAM_THROTTLE_MS = 120;
    var lastStreamRender = 0;
    var pendingStreamFrame = null;
    var DEDUP_WINDOW = 60000;

    /* ---- marked configuration ---- */
    if (window.marked) {
        window.marked.setOptions({ breaks: true, gfm: true, headerIds: false, mangle: false });
    }

    function sanitize(html) {
        return html
            .replace(/<script[\s\S]*?<\/script>/gi, "")
            .replace(/<iframe[\s\S]*?<\/iframe>/gi, "")
            .replace(/\son\w+\s*=\s*["'][^"']*["']/gi, "");
    }

    function mdToHtml(text) {
        if (!text) { return ""; }
        if (!window.marked) { return escapeHtml(text); }
        try { return sanitize(window.marked.parse(text)); }
        catch (e) { return escapeHtml(text); }
    }

    function escapeHtml(s) {
        return s.replace(/&/g, "&amp;").replace(/</g, "&lt;")
                .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }

    function addCopyButtons(container) {
        var pres = container.querySelectorAll("pre");
        for (var i = 0; i < pres.length; i++) {
            if (pres[i].querySelector(".gb-code-copy")) { continue; }
            var btn = document.createElement("button");
            btn.className = "gb-code-copy";
            btn.textContent = A.t("assistant.copy", "Copy");
            btn.setAttribute("aria-label", A.t("assistant.copy-code", "Copy code"));
            (function (pre, b) {
                b.addEventListener("click", function () {
                    var code = pre.querySelector("code");
                    var text = code ? code.textContent : pre.textContent;
                    navigator.clipboard.writeText(text).then(function () {
                        b.textContent = A.t("assistant.copied", "Copied");
                        setTimeout(function () {
                            b.textContent = A.t("assistant.copy", "Copy");
                        }, 1500);
                    });
                });
            })(pres[i], btn);
            pres[i].appendChild(btn);
        }
    }

    function scrollToBottom(smooth) {
        var conv = document.getElementById("gbConversation");
        if (!conv) { return; }
        conv.scrollTo({ top: conv.scrollHeight, behavior: smooth ? "smooth" : "auto" });
    }

    /* ---- Reasoning disclosure ---- */
    function reasoningHtml(text) {
        if (!text) { return ""; }
        return '<details class="gb-reasoning"><summary>' +
            escapeHtml(A.t("assistant.reasoning", "Reasoning")) +
            '</summary><div class="gb-reasoning-body">' +
            escapeHtml(text) + '</div></details>';
    }

    /* ---- User message ---- */
    A.renderUserMessage = function (text) {
        showConversation();
        var el = document.createElement("div");
        el.className = "gb-msg-user";
        el.textContent = text;
        document.getElementById("gbMessages").appendChild(el);
        scrollToBottom(true);
    };

    /* ---- Final assistant message ---- */
    A.renderAssistantMessage = function (text, reasoning) {
        showConversation();
        var el = document.createElement("div");
        el.className = "gb-msg-assistant";
        el.innerHTML = reasoningHtml(reasoning) + mdToHtml(text);
        addCopyButtons(el);
        document.getElementById("gbMessages").appendChild(el);
        scrollToBottom(true);
    };

    /* ---- Streaming ---- */

    A.startStream = function () {
        showConversation();
        A.isStreaming = true;
        A.streamingContent = "";
        A.streamingReasoning = "";
        var el = document.createElement("div");
        el.className = "gb-msg-assistant gb-streaming-cursor";
        el.setAttribute("aria-busy", "true");
        document.getElementById("gbMessages").appendChild(el);
        A.streamingEl = el;
        toggleStopBtn(true);
        scrollToBottom(false);
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
        A.streamingEl.innerHTML =
            reasoningHtml(A.streamingReasoning) + mdToHtml(A.streamingContent);
        addCopyButtons(A.streamingEl);
        scrollToBottom(false);
    }

    /** Finalize streaming: remove the streaming element. */
    function finalizeStreaming() {
        if (pendingStreamFrame) {
            cancelAnimationFrame(pendingStreamFrame);
            pendingStreamFrame = null;
        }
        if (A.streamingEl) {
            A.streamingEl.remove();
            A.streamingEl = null;
        }
        A.isStreaming = false;
        toggleStopBtn(false);
    }

    A.endStream = function () {
        finalizeStreaming();
        A.streamingContent = "";
        A.streamingReasoning = "";
        scrollToBottom(true);
    };

    /* ---- Error ---- */
    A.renderError = function (text) {
        showConversation();
        var el = document.createElement("div");
        el.className = "gb-msg-error";
        el.setAttribute("role", "alert");
        el.textContent = text;
        document.getElementById("gbMessages").appendChild(el);
        scrollToBottom(true);
    };

    /* ---- Show/hide empty vs conversation ---- */
    function showConversation() {
        var empty = document.getElementById("gbEmptyState");
        var conv = document.getElementById("gbConversation");
        var dock = document.getElementById("gbComposerDock");
        if (empty) { empty.style.display = "none"; }
        if (conv) { conv.style.display = ""; }
        if (dock) { dock.style.display = ""; }
    }

    A.showEmptyState = function () {
        var empty = document.getElementById("gbEmptyState");
        var conv = document.getElementById("gbConversation");
        var dock = document.getElementById("gbComposerDock");
        var msgs = document.getElementById("gbMessages");
        if (empty) { empty.style.display = ""; }
        if (conv) { conv.style.display = "none"; }
        if (dock) { dock.style.display = "none"; }
        if (msgs) { msgs.innerHTML = ""; }
    };

    /* ---- Stop button ---- */
    function toggleStopBtn(show) {
        var sendBtn = document.getElementById("gbSendDock");
        var stopBtn = document.getElementById("gbStopBtn");
        if (sendBtn) { sendBtn.style.display = show ? "none" : ""; }
        if (stopBtn) { stopBtn.style.display = show ? "" : "none"; }
    }

    /* ---- Render full history ---- */
    A.renderHistory = function (messages) {
        var msgs = document.getElementById("gbMessages");
        if (!msgs) { return; }
        msgs.innerHTML = "";
        if (!messages || messages.length === 0) {
            A.showEmptyState();
            return;
        }
        showConversation();
        for (var i = 0; i < messages.length; i++) {
            var m = messages[i];
            var el = document.createElement("div");
            if (m.role === "user") {
                el.className = "gb-msg-user";
                el.textContent = m.content;
            } else {
                el.className = "gb-msg-assistant";
                el.innerHTML = mdToHtml(m.content);
                addCopyButtons(el);
            }
            msgs.appendChild(el);
        }
        scrollToBottom(false);
    };

    /* ---- WS message handler (matches legacy 70_stream.js rules) ---- */
    A.onBotMessage = function (data) {
        var content = data.content || "";
        var reasoning = data.reasoning || "";
        var isComplete = data.is_complete === true;

        /* Thinking frame: no content, keep indicator alive */
        if (data.thinking && !content && !reasoning) { return; }

        /* Streaming chunk: append */
        if (!isComplete && (content || reasoning)) {
            if (!A.isStreaming) { A.startStream(); }
            A.appendStream(content, reasoning);
            return;
        }

        /* Completion frame */
        if (isComplete) {
            var finalContent = content || A.streamingContent;
            var finalReasoning = reasoning || A.streamingReasoning;

            /* Empty completion with no accumulated content: ignore */
            if (!finalContent && !finalReasoning && !A.isStreaming) { return; }

            /* Dedup: skip if same content within 60 s */
            var now = Date.now();
            if (finalContent && finalContent === A.lastCompletedContent &&
                (now - A.lastCompletedTime) < DEDUP_WINDOW) {
                finalizeStreaming();
                return;
            }
            A.lastCompletedContent = finalContent;
            A.lastCompletedTime = now;

            /* Remove the streaming element */
            finalizeStreaming();

            /* Add the final permanent message */
            if (finalContent || finalReasoning) {
                A.renderAssistantMessage(finalContent, finalReasoning);
            }

            /* Show suggestions after the turn */
            if (data.suggestions && data.suggestions.length) {
                A.suggestions = data.suggestions;
                if (typeof A.onSuggestions === "function") {
                    A.onSuggestions(data.suggestions);
                }
            }

            /* Refresh sidebar */
            if (typeof A.refreshSidebar === "function") {
                A.refreshSidebar();
            }
            return;
        }

        /* Frames with only suggestions (no content, not complete) */
        /* Already handled in dispatch */
    };

    A.onStreamEnd = function () { A.endStream(); };

    A.onChatStatus = function (data) {
        if (data.text || data.status) {
            A.announce(data.text || data.status);
        }
    };
})();
