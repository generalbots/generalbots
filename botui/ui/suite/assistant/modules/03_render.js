"use strict";
/**
 * 03_render.js - Message rendering, WS handler, history replay.
 * Sanitizer, markdown, code blocks and citations in 03c_markdown.js.
 * Streaming/scroll/thinking in 03a_stream.js.
 * Per-message actions in 03b_actions.js.
 */

(function () {
    var A = window.GBAssistant;
    var DEDUP_WINDOW = 60000;
    /** True after the first user message is sent in this session. */
    A.hasUserSent = false;
    /** At most one frame per empty-state cycle is treated as a greeting. */
    A.greetingReceived = false;

    /* ================================================================
       User message
       ================================================================ */
    A.renderUserMessage = function (text, files) {
        A.hasUserSent = true;
        A.showConversation();
        A.hideThinking();
        hideDockSuggestions();
        var el = document.createElement("div");
        el.className = "gb-msg-user";
        el.dataset.originalContent = text;
        var bubble = document.createElement("div");
        bubble.className = "gb-msg-user-bubble";
        bubble.textContent = text;
        el.appendChild(bubble);
        if (files && files.length) {
            var cards = document.createElement("div");
            cards.className = "gb-attach-list";
            for (var i = 0; i < files.length; i++) {
                if (typeof A.attachCard === "function") {
                    cards.appendChild(A.attachCard(files[i], { removable: false }));
                } else {
                    var chip = document.createElement("span");
                    chip.className = "gb-file-chip-inline";
                    chip.textContent = files[i].name || files[i];
                    cards.appendChild(chip);
                }
            }
            el.appendChild(cards);
        }
        addUserActions(el, text);
        document.getElementById("gbMessages").appendChild(el);
        if (!A.isUserScrolledUp()) { A.scrollToBottom(false); }
    };

    /* ================================================================
       Assistant message (final)
       ================================================================ */
    A.renderAssistantMessage = function (text, reasoning, opts) {
        opts = opts || {};
        A.showConversation();
        A.hideThinking();
        var raw = text || "";
        var leaked = typeof A.detectLeakedProtocol === "function"
            ? A.detectLeakedProtocol(raw) : null;
        var msgs = document.getElementById("gbMessages");
        /* Adopt the current turn's activity stack if one exists */
        var el = null;
        if (msgs) {
            var children = msgs.children;
            for (var ci = children.length - 1; ci >= Math.max(0, children.length - 4); ci--) {
                var ch = children[ci];
                if (ch.classList.contains("gb-msg-assistant") && ch.querySelector(".gb-block-stack")) {
                    el = ch; break;
                }
            }
        }
        if (!el) {
            el = document.createElement("div");
            el.className = "gb-msg-assistant";
            if (msgs) { msgs.appendChild(el); }
        }
        /* Find or create the block stack for this turn */
        var stack = el.querySelector(".gb-block-stack");
        var elapsed = A.thinkingElapsed();
        if (reasoning && typeof A.blockThinking === "function") {
            var thinkEl = A.blockThinking(elapsed, reasoning);
            if (stack) {
                stack.appendChild(thinkEl);
            } else {
                stack = document.createElement("div");
                stack.className = "gb-block-stack";
                stack.appendChild(thinkEl);
                el.insertBefore(stack, el.firstChild);
            }
        } else if (reasoning) {
            el.insertAdjacentHTML("beforeend", A.reasoningHtml(reasoning, elapsed));
        }
        var card = typeof A.consentFromContent === "function" ? A.consentFromContent(raw) : null;
        if (card) { el.appendChild(card); }
        if (leaked && !card) {
            if (typeof A.blockNotice === "function") {
                el.appendChild(A.blockNotice({ summary: { message: raw }, status: "completed" }));
            } else {
                el.insertAdjacentHTML("beforeend", A.leakedNoticeHtml(raw));
            }
        }
        var html = A.mdToHtml(raw);
        if (html && !card && !leaked) {
            var body = document.createElement("div");
            body.className = "gb-msg-body";
            body.innerHTML = html;
            A.enhanceCodeBlocks(body);
            A.highlightCodeBlocks(body);
            el.appendChild(body);
        }
        if (opts.citations && opts.citations.length) {
            el.insertAdjacentHTML("beforeend", A.citationsHtml(opts.citations));
        }
        el.dataset.originalContent = raw;
        el.dataset.originalReasoning = reasoning || "";
        addAssistantActions(el, raw, !!(opts.retryable || opts.isError));
        if (!A.isUserScrolledUp()) { A.scrollToBottom(true); }
        A.resetThinkingTimer();
    };

    /* ================================================================
       Error card
       ================================================================ */
    A.renderError = function (text) {
        A.showConversation();
        A.hideThinking();
        var el = document.createElement("div");
        el.className = "gb-msg-assistant";
        if (typeof A.blockError === "function") {
            el.appendChild(A.blockError({ summary: { message: text || A.t("assistant.error-generic", "An error occurred.") }, status: "failed" }));
            document.getElementById("gbMessages").appendChild(el);
            if (!A.isUserScrolledUp()) { A.scrollToBottom(true); }
            return;
        }
        var c = document.createElement("div");
        c.className = "gb-error-card";
        c.setAttribute("role", "alert");
        c.innerHTML =
            '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>' +
            '<span>' + A.escapeHtml(text || A.t("assistant.error-generic", "An error occurred.")) + '</span>';
        el.appendChild(c);
        addAssistantActions(el, null, true);
        document.getElementById("gbMessages").appendChild(el);
        if (!A.isUserScrolledUp()) { A.scrollToBottom(true); }
    };

    /* ================================================================
       History replay
       ================================================================ */
    A.renderHistory = function (messages) {
        var msgs = document.getElementById("gbMessages");
        if (!msgs) { return; }
        msgs.innerHTML = "";
        if (!messages || messages.length === 0) { A.showEmptyState(); return; }
        A.showConversation();
        for (var h = 0; h < messages.length; h++) {
            if (messages[h].role === "user") { A.hasUserSent = true; break; }
        }
        for (var i = 0; i < messages.length; i++) {
            var m = messages[i];
            var el = document.createElement("div");
            if (m.role === "user") {
                el.className = "gb-msg-user";
                el.dataset.originalContent = m.content || "";
                var bub = document.createElement("div");
                bub.className = "gb-msg-user-bubble";
                bub.textContent = m.content;
                el.appendChild(bub);
                if (m.attachments && m.attachments.length && typeof A.attachCard === "function") {
                    var ac = document.createElement("div");
                    ac.className = "gb-attach-list";
                    m.attachments.forEach(function (att) { ac.appendChild(A.attachCard(att, { removable: false })); });
                    el.appendChild(ac);
                }
                addUserActions(el, m.content || "");
            } else {
                el.className = "gb-msg-assistant";
                el.dataset.originalContent = m.content || "";
                if (m.activities && m.activities.length && typeof A.renderBlockStack === "function") {
                    var stk = A.renderBlockStack(m.activities);
                    if (stk) { el.appendChild(stk); }
                }
                var ht = A.mdToHtml(m.content);
                if (ht) {
                    var bd = document.createElement("div");
                    bd.className = "gb-msg-body";
                    bd.innerHTML = ht;
                    A.enhanceCodeBlocks(bd);
                    A.highlightCodeBlocks(bd);
                    el.appendChild(bd);
                }
                addAssistantActions(el, m.content, false);
            }
            msgs.appendChild(el);
        }
        A.scrollToBottom(false);
        hideDockSuggestions();
        if (typeof A.reviveConsentCards === "function") { A.reviveConsentCards(msgs); }
    };

    /* ================================================================
       Per-message action hooks (delegated to 03b_actions.js)
       ================================================================ */
    function addAssistantActions(el, content, retryable) {
        if (typeof A.addAssistantToolbar === "function") {
            A.addAssistantToolbar(el, content, retryable);
        } else if (typeof A.addMsgToolbar === "function") {
            A.addMsgToolbar(el, content, retryable);
        }
    }
    function addUserActions(el, content) {
        if (typeof A.addUserToolbar === "function") {
            A.addUserToolbar(el, content);
        }
    }

    /* ================================================================
       Suggestion chips: hide in dock once conversation has messages
       ================================================================ */
    function hideDockSuggestions() {
        var dock = document.getElementById("gbSuggestionsDock");
        if (dock) { dock.innerHTML = ""; }
    }
    A._hideDockSuggestions = hideDockSuggestions;

    /* ================================================================
       WS message handler
       ================================================================ */
    A.onBotMessage = function (data) {
        var content = data.content || "";
        var reasoning = data.reasoning || "";
        var isComplete = data.is_complete === true;
        if (typeof A.onConsentFrame === "function") { A.onConsentFrame(data); }
        var isError = !!(data.is_error || data.error || data.status === "failed" ||
            data.turn_status === "failed" || data.turn_status === "error");
        if (data.thinking && !content && !reasoning) { A.showThinking(); return; }
        if (!isComplete && (content || reasoning)) {
            if (!A.hasUserSent && !A.greetingReceived) {
                A._greetingBuffer = (A._greetingBuffer || "") + (content || ""); return;
            }
            A.hideThinking();
            if (!A.isStreaming) { A.startStream(); }
            A.appendStream(content, reasoning);
            return;
        }
        if (isComplete) {
            var fc = content || A.streamingContent || A._greetingBuffer || "";
            var fr = reasoning || A.streamingReasoning || "";
            A._greetingBuffer = "";
            if (!fc && !fr && !A.isStreaming) { return; }
            var now = Date.now();
            if (fc && fc === A.lastCompletedContent && (now - A.lastCompletedTime) < DEDUP_WINDOW) {
                A._finalizeStreaming(); A.hideThinking(); return;
            }
            A.lastCompletedContent = fc;
            A.lastCompletedTime = now;
            A._finalizeStreaming();
            if (!A.hasUserSent && !A.greetingReceived && fc && !A.isStreaming) {
                A.greetingReceived = true;
                A.hideThinking();
                var sub = document.getElementById("gbGreetingSub");
                if (sub) { sub.textContent = fc.replace(/<[^>]*>/g, "").trim(); }
                if (data.suggestions && data.suggestions.length) {
                    A.suggestions = data.suggestions;
                    if (typeof A.onSuggestions === "function") { A.onSuggestions(data.suggestions); }
                }
                return;
            }
            if (fc || fr) {
                A.renderAssistantMessage(fc, fr, {
                    citations: data.citations,
                    retryable: !!(data.retryable || isError), isError: isError
                });
            }
            if (data.suggestions && data.suggestions.length) {
                A.suggestions = data.suggestions;
                if (typeof A.onSuggestions === "function") { A.onSuggestions(data.suggestions); }
            }
            if (typeof A.refreshSidebar === "function") { A.refreshSidebar(); }
        }
    };
    A.onStreamEnd = function () { A.endStream(); };
    A.onChatStatus = function (data) {
        if (data.text || data.status) { A.announce(data.text || data.status); }
    };
})();
