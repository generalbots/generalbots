"use strict";
/* 03_render.js — message rendering, WS handler, history replay.
   Sanitizer/markdown/code/citations: 03c_markdown.js.
   Streaming/scroll/thinking: 03a_stream.js. Per-message actions: 03b_actions.js. */
(function () {
    var A = window.GBAssistant;
    var DEDUP_WINDOW = 60000;
    A.hasUserSent = false;      // true after the first user message in this session
    A.greetingReceived = false; // at most one frame per empty-state cycle is a greeting

    /* Shared error-card markup (renderError and renderErrorCode). */
    function errorCardHtml(msg) {
        return '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>' +
            '<span>' + A.escapeHtml(msg) + '</span>';
    }

    /* User message */
    A.renderUserMessage = function (text, files) {
        A.hasUserSent = true;
        A.lastCompletedContent = ""; A.lastCompletedTime = 0; // reset dedup
        A.showConversation(); A.hideThinking(); hideDockSuggestions();
        var el = document.createElement("div");
        el.className = "gb-msg-user"; el.dataset.originalContent = text;
        var bubble = document.createElement("div");
        bubble.className = "gb-msg-user-bubble"; bubble.textContent = text;
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
        A.addMessageHeading(el, true, Date.now());
        addUserActions(el, text);
        /* #2562 — sequential index for truncation. */
        var allMsgs = document.querySelectorAll("#gbMessages > .gb-msg-user, #gbMessages > .gb-msg-assistant");
        el.dataset.messageIndex = String(allMsgs.length);
        document.getElementById("gbMessages").appendChild(el);
        if (!A.isUserScrolledUp()) { A.scrollToBottom(false); }
    };

    /* Assistant message (final) */
    A.renderAssistantMessage = function (text, reasoning, opts) {
        opts = opts || {};
        A.showConversation(); A.hideThinking();
        var raw = text || "";
        var leaked = typeof A.detectLeakedProtocol === "function" ? A.detectLeakedProtocol(raw) : null;
        var msgs = document.getElementById("gbMessages");
        /* Adopt the current turn's activity stack: look only AFTER the last
           user message so a previous turn's element is never adopted. */
        var el = null;
        if (msgs) {
            var children = msgs.children, lastUserIdx = -1;
            for (var ui = children.length - 1; ui >= 0; ui--) {
                if (children[ui].classList.contains("gb-msg-user")) { lastUserIdx = ui; break; }
            }
            for (var ci = children.length - 1; ci > lastUserIdx && ci >= Math.max(0, children.length - 4); ci--) {
                var ch = children[ci];
                if (ch.classList.contains("gb-msg-assistant") && ch.querySelector(".gb-block-stack")) { el = ch; break; }
            }
        }
        if (!el) {
            el = document.createElement("div");
            el.className = "gb-msg-assistant";
            if (msgs) { msgs.appendChild(el); }
        }
        var stack = el.querySelector(".gb-block-stack");
        var elapsed = A._thoughtSeconds || A.thinkingElapsed();
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
        if (A.addMessageHeading) { A.addMessageHeading(el, false, Number(el.dataset.streamStarted) || Date.now()); }
        var card = typeof A.consentFromContent === "function" ? A.consentFromContent(raw) : null;
        if (card) { A.queueConsentCard(card); }
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
            body.className = "gb-msg-body"; body.innerHTML = html;
            A.enhanceCodeBlocks(body); A.highlightCodeBlocks(body);
            if (typeof A.renderMath === "function") { A.renderMath(body); }
            el.appendChild(body);
        }
        if (opts.citations && opts.citations.length) { el.insertAdjacentHTML("beforeend", A.citationsHtml(opts.citations)); }
        el.dataset.originalContent = raw;
        el.dataset.originalReasoning = reasoning || "";
        /* #2562 — index for truncation. */
        if (!el.dataset.messageIndex) {
            var allLive = document.querySelectorAll("#gbMessages > .gb-msg-user, #gbMessages > .gb-msg-assistant");
            for (var mi = 0; mi < allLive.length; mi++) {
                if (allLive[mi] === el) { el.dataset.messageIndex = String(mi); break; }
            }
        }
        if (!card) {
            addAssistantActions(el, raw, !!(opts.retryable || opts.isError));
            if (!opts.isError && raw && A.tagAnswer) { A.tagAnswer(el, raw, opts.model); }
        }
        if (!A.isUserScrolledUp()) { A.scrollToBottom(true); }
        A.resetThinkingTimer();
    };

    /* Error card */
    A.renderError = function (text) {
        A.showConversation(); A.hideThinking();
        var el = document.createElement("div");
        el.className = "gb-msg-assistant gb-msg-error";
        if (typeof A.blockError === "function") {
            el.appendChild(A.blockError({ summary: { message: text || A.t("assistant.error-generic", "An error occurred.") }, status: "failed" }));
            document.getElementById("gbMessages").appendChild(el);
            if (!A.isUserScrolledUp()) { A.scrollToBottom(true); }
            return;
        }
        var c = document.createElement("div");
        c.className = "gb-error-card"; c.setAttribute("role", "alert");
        c.innerHTML = errorCardHtml(text || A.t("assistant.error-generic", "An error occurred."));
        el.appendChild(c);
        addAssistantActions(el, null, true);
        document.getElementById("gbMessages").appendChild(el);
        if (!A.isUserScrolledUp()) { A.scrollToBottom(true); }
    };

    /* Classified error card (#2562) */
    A.renderErrorCode = function (code, fallbackMsg) {
        A.showConversation(); A.hideThinking(); A._finalizeStreaming();
        var msgs = document.getElementById("gbMessages");
        var el = document.createElement("div");
        el.className = "gb-msg-assistant";
        var i18nMap = {
            model_unreachable: "assistant.error-model-unreachable",
            model_auth: "assistant.error-model-auth",
            model_rate_limited: "assistant.error-model-rate-limited",
            model_error: "assistant.error-model-error",
            timeout: "assistant.error-timeout", context_exceeded: "assistant.error-context-exceeded",
        };
        var key = i18nMap[code] || "assistant.error-generic";
        var msg = A.t(key, fallbackMsg || A.t("assistant.error-generic", "An error occurred."));
        var card = document.createElement("div");
        card.className = "gb-error-card"; card.setAttribute("role", "alert");
        card.innerHTML = errorCardHtml(msg);
        el.appendChild(card);
        var bar = document.createElement("div"); // action buttons
        bar.className = "gb-error-actions";
        var retryBtn = document.createElement("button"); // Retry
        retryBtn.className = "gb-btn-retry";
        retryBtn.textContent = A.t("assistant.retry", "Tentar novamente");
        retryBtn.addEventListener("click", function () {
            var all = msgs.querySelectorAll(".gb-msg-user");
            var last = all.length ? all[all.length - 1] : null;
            if (!last) { return; }
            var text = (last.dataset.originalContent || "").trim();
            if (!text) { return; }
            el.remove();
            A.lastCompletedContent = ""; A.lastCompletedTime = 0;
            A.showThinking();
            if (!A.sendMessage(text)) { A.hideThinking(); A.renderErrorCode("model_unreachable"); }
        });
        bar.appendChild(retryBtn);
        if (code === "model_unreachable" || code === "model_auth") { // settings link
            var settingsBtn = document.createElement("button");
            settingsBtn.className = "gb-btn-settings";
            settingsBtn.textContent = A.t("assistant.open-model-settings", "Abrir configurações do modelo");
            settingsBtn.addEventListener("click", function () {
                if (typeof A.openSettings === "function") { A.openSettings("model"); }
            });
            bar.appendChild(settingsBtn);
        }
        el.appendChild(bar);
        msgs.appendChild(el);
        if (!A.isUserScrolledUp()) { A.scrollToBottom(true); }
    };

    /* History replay */
    A.renderHistory = function (messages) {
        var msgs = document.getElementById("gbMessages");
        if (!msgs) { return; }
        msgs.innerHTML = "";
        if (!messages || messages.length === 0) { A.showEmptyState(); return; }
        A.showConversation();
        for (var h = 0; h < messages.length; h++) {
            if (messages[h].role === "user") { A.hasUserSent = true; break; }
        }
        /* #2562 — merge activities from _lastHistoryActivities into messages,
           grouped by the preceding assistant message. */
        var activities = A._lastHistoryActivities || [];
        if (activities.length) {
            var assistantIndices = [];
            for (var ai = 0; ai < messages.length; ai++) {
                if (messages[ai].role === "assistant") { assistantIndices.push(ai); }
            }
            if (assistantIndices.length && !messages[assistantIndices[0]].activities) {
                for (var aj = 0; aj < assistantIndices.length; aj++) { messages[assistantIndices[aj]].activities = []; }
                activities.forEach(function (act) {
                    var target = assistantIndices[assistantIndices.length - 1]; // best-effort
                    if (act.message_index != null) {
                        for (var ak = 0; ak < assistantIndices.length; ak++) {
                            if (assistantIndices[ak] >= act.message_index) { target = assistantIndices[ak]; break; }
                        }
                    }
                    if (messages[target]) {
                        messages[target].activities = messages[target].activities || [];
                        messages[target].activities.push(act);
                    }
                });
            }
        }
        for (var i = 0; i < messages.length; i++) {
            var m = messages[i], el = document.createElement("div");
            if (m.role === "user") {
                el.className = "gb-msg-user"; el.dataset.originalContent = m.content || "";
                if (m.message_index != null) { el.dataset.messageIndex = String(m.message_index); }
                var bub = document.createElement("div");
                bub.className = "gb-msg-user-bubble"; bub.textContent = m.content;
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
                el.dataset.originalReasoning = m.reasoning || "";
                el.dataset.messageIndex = String(i);
                /* #2562 — reconstruct activity step cards and reasoning from history */
                if (m.activities && m.activities.length && typeof A.renderBlockStack === "function") {
                    var stk = A.renderBlockStack(m.activities);
                    if (stk) { el.appendChild(stk); }
                }
                if (m.reasoning && typeof A.blockThinking === "function") {
                    var thinkStack = el.querySelector(".gb-block-stack");
                    /* The stored turn carries no duration unless the server adds
                       `reasoning_ms`; without it the card shows no time. */
                    var thinkEl = A.blockThinking(m.reasoning_ms > 0 ? Math.floor(m.reasoning_ms / 1000) : null, m.reasoning);
                    if (thinkStack) {
                        thinkStack.appendChild(thinkEl);
                    } else {
                        thinkStack = document.createElement("div");
                        thinkStack.className = "gb-block-stack";
                        thinkStack.appendChild(thinkEl);
                        el.insertBefore(thinkStack, el.firstChild);
                    }
                }
                var ht = A.mdToHtml(m.content);
                if (ht) {
                    var bd = document.createElement("div");
                    bd.className = "gb-msg-body"; bd.innerHTML = ht;
                    A.enhanceCodeBlocks(bd); A.highlightCodeBlocks(bd);
                    if (typeof A.renderMath === "function") { A.renderMath(bd); }
                    el.appendChild(bd);
                }
                if (A.renderTurnExtras) { A.renderTurnExtras(el, m); }
                addAssistantActions(el, m.content, false);
                if (A.answerModel) { A.attachModelTag(el, A.answerModel(m.content) || m.model); }
            }
            A.addMessageHeading(el, m.role === "user", m.created_at || m.timestamp);
            msgs.appendChild(el);
        }
        A.scrollToBottom(false); hideDockSuggestions();
        if (typeof A.reviveConsentCards === "function") { A.reviveConsentCards(msgs); }
    };

    /* Per-message action hooks (delegated to 03b_actions.js) */
    function addAssistantActions(el, content, retryable) {
        if (typeof A.addAssistantToolbar === "function") { A.addAssistantToolbar(el, content, retryable); }
        else if (typeof A.addMsgToolbar === "function") { A.addMsgToolbar(el, content, retryable); }
    }
    function addUserActions(el, content) {
        if (typeof A.addUserToolbar === "function") { A.addUserToolbar(el, content); }
    }

    /* Suggestion chips: hide in dock once conversation has messages */
    function hideDockSuggestions() {
        var dock = document.getElementById("gbSuggestionsDock");
        if (dock) { dock.innerHTML = ""; }
    }
    A._hideDockSuggestions = hideDockSuggestions;

    /* WS message handler */
    A.onBotMessage = function (data) {
        var content = data.content || "", reasoning = data.reasoning || "";
        var isComplete = data.is_complete === true;
        if (typeof A.onConsentFrame === "function") { A.onConsentFrame(data); }
        var isError = !!(data.is_error || data.error || data.status === "failed" ||
            data.turn_status === "failed" || data.turn_status === "error");
        /* Suppress ALL frames after a cancel so partial text stays and the
           composer is not re-locked by a late streaming chunk (#2562). */
        if (A._cancelledAt && (Date.now() - A._cancelledAt) < 5000) {
            if (isComplete) { A._cancelledAt = 0; A._finalizeStreaming(); A.hideThinking(); }
            return;
        }
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
            // Clear streaming leftovers so a later complete frame (e.g.
            // suggestions-only) does not re-render the same answer (#2562).
            A.streamingContent = ""; A.streamingReasoning = ""; A._greetingBuffer = "";
            if (!fc && !fr && !A.isStreaming) { return; }
            var now = Date.now();
            if (fc && fc === A.lastCompletedContent && (now - A.lastCompletedTime) < DEDUP_WINDOW) {
                A._finalizeStreaming(); A.hideThinking(); return;
            }
            A.lastCompletedContent = fc; A.lastCompletedTime = now;
            A._finalizeStreaming();
            if (!A.hasUserSent && !A.greetingReceived && fc && !A.isStreaming) {
                A.greetingReceived = true; A.hideThinking();
                var sub = document.getElementById("gbGreetingSub");
                if (sub) { sub.textContent = fc.replace(/<[^>]*>/g, "").trim(); }
                if (data.suggestions && data.suggestions.length) {
                    A.suggestions = data.suggestions;
                    if (typeof A.onSuggestions === "function") { A.onSuggestions(data.suggestions); }
                }
                return;
            }
            if (fc || fr) {
                /* #2562 — classified error from the server: show the right card. */
                if (isError && data.error_code) {
                    if (!A.renderEngineError || !A.renderEngineError(data.error_code, data.error_params)) {
                    A.renderErrorCode(data.error_code, fc); }
                } else {
                    A.renderAssistantMessage(fc, fr, {
                        citations: data.citations, model: data.llm_model || data.model,
                        retryable: !!(data.retryable || isError), isError: isError
                    });
                }
            } else if (isError) {
                if (!A.renderEngineError || !A.renderEngineError(data.error_code, data.error_params)) {
                A.renderErrorCode(data.error_code || "model_error", data.content || ""); }
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
