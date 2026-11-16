"use strict";
/* Split from 03_render.js for #2305 (450-line limit). */
(function () {
  var A = window.GBAssistant;
  if (!A) return;

    /* ================================================================
       User message
       ================================================================ */
    A.renderUserMessage = function (text, files) {
        A.hasUserSent = true;
        A.showConversation();
        A.hideThinking();
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
        /* Adopt the current turn's activity stack if one exists (created by
           onActivityEvent during the same turn). This keeps activities and the
           thinking block in a single container above the answer text. */
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
        /* Use block for thinking/reasoning when available */
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
            el.insertAdjacentHTML("beforeend", reasoningHtml(reasoning, elapsed));
        }
        var card = typeof A.consentFromContent === "function" ? A.consentFromContent(raw) : null;
        if (card) { el.appendChild(card); }
        /* Use block for leaked-markup notice */
        if (leaked && !card) {
            if (typeof A.blockNotice === "function") {
                el.appendChild(A.blockNotice({ summary: { message: raw }, status: "completed" }));
            } else {
                el.insertAdjacentHTML("beforeend", leakedNoticeHtml(raw));
            }
        }
        var html = mdToHtml(raw);
        if (html && !card && !leaked) {
            var body = document.createElement("div");
            body.className = "gb-msg-body";
            body.innerHTML = html;
            enhanceCodeBlocks(body);
            el.appendChild(body);
        }
        if (opts.citations && opts.citations.length) {
            el.insertAdjacentHTML("beforeend", citationsHtml(opts.citations));
        }
        addMessageActions(el, raw, !!(opts.retryable || opts.isError));
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
            '<span>' + escapeHtml(text || A.t("assistant.error-generic", "An error occurred.")) + '</span>';
        el.appendChild(c);
        addMessageActions(el, null, true);
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
                /* Attachments from history */
                if (m.attachments && m.attachments.length && typeof A.attachCard === "function") {
                    var ac = document.createElement("div");
                    ac.className = "gb-attach-list";
                    m.attachments.forEach(function (att) { ac.appendChild(A.attachCard(att, { removable: false })); });
                    el.appendChild(ac);
                }
            } else {
                el.className = "gb-msg-assistant";
                /* Activities from history */
                if (m.activities && m.activities.length && typeof A.renderBlockStack === "function") {
                    var stack = A.renderBlockStack(m.activities);
                    if (stack) el.appendChild(stack);
                }
                var h = mdToHtml(m.content);
                if (h) {
                    var bd = document.createElement("div");
                    bd.className = "gb-msg-body";
                    bd.innerHTML = h;
                    enhanceCodeBlocks(bd);
                    el.appendChild(bd);
                }
                addMessageActions(el, m.content, false);
            }
            msgs.appendChild(el);
        }
        A.scrollToBottom(false);
        if (typeof A.reviveConsentCards === "function") { A.reviveConsentCards(msgs); }
    };

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
            A.hideThinking();
            if (!A.isStreaming) { A.startStream(); }
            A.appendStream(content, reasoning);
            return;
        }
        if (isComplete) {
            var fc = content || A.streamingContent || "";
            var fr = reasoning || A.streamingReasoning || "";
            if (!fc && !fr && !A.isStreaming) { return; }
            var now = Date.now();
            if (fc && fc === A.lastCompletedContent && (now - A.lastCompletedTime) < DEDUP_WINDOW) {
                A._finalizeStreaming(); A.hideThinking(); return;
            }
            A.lastCompletedContent = fc;
            A.lastCompletedTime = now;
            A._finalizeStreaming();
            /* Greeting from start.bas: a bot completion before any user
               turn.  Show it as the hero's secondary line, not in the
               transcript; the empty state stays visible with its chips. */
            if (!A.hasUserSent && fc && !A.isStreaming) {
                A.hideThinking();
                var sub = document.getElementById("gbGreetingSub");
                if (sub) { sub.textContent = fc.replace(/<[^>]*>/g, "").trim(); }
                /* Show suggestions that came with the greeting */
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
