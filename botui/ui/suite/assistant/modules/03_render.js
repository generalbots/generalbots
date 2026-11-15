"use strict";
/**
 * 03_render.js - Sanitizer, markdown, message rendering, WS handler.
 * Streaming/scroll/thinking in 03a_stream.js (loaded after).
 */

(function () {
    var A = window.GBAssistant;
    var DEDUP_WINDOW = 60000;
    /** True after the first user message is sent in this session.
     *  A bot completion that arrives before any user turn is a greeting
     *  from start.bas — it goes into the hero subtitle, not the transcript. */
    A.hasUserSent = false;

    /* ================================================================
       Sanitizer (template-based, mirrors legacy GBSanitize)
       ================================================================ */
    var DROP = "script,style,iframe,frame,frameset,object,embed,applet,link,meta,base,form,input,textarea,select,option,button,foreignObject,use,set";
    var URI_ATTRS = ["href","src","xlink:href","action","formaction","poster","background","data","srcset","srcdoc"];

    function uriAllowed(val) {
        var v = String(val == null ? "" : val).replace(/[\u0000-  ]/g, "").toLowerCase();
        if (!v) { return true; }
        if (/^(javascript|vbscript|livescript):/.test(v)) { return false; }
        if (v.indexOf("data:") === 0) {
            return /^data:image\/(png|jpe?g|gif|webp|bmp)[;,]/.test(v);
        }
        return true;
    }

    function sanitizeHtml(input) {
        var tpl = document.createElement("template");
        tpl.innerHTML = String(input == null ? "" : input);
        var rm = tpl.content.querySelectorAll(DROP);
        for (var i = 0; i < rm.length; i++) {
            if (rm[i].parentNode) { rm[i].parentNode.removeChild(rm[i]); }
        }
        var all = tpl.content.querySelectorAll("*");
        for (var j = 0; j < all.length; j++) {
            var el = all[j];
            var attrs = Array.prototype.slice.call(el.attributes);
            for (var k = 0; k < attrs.length; k++) {
                var n = attrs[k].name.toLowerCase();
                if (n.indexOf("on") === 0) { el.removeAttribute(attrs[k].name); continue; }
                if (URI_ATTRS.indexOf(n) !== -1 && !uriAllowed(attrs[k].value)) {
                    el.removeAttribute(attrs[k].name);
                }
            }
            if (el.tagName && el.tagName.toLowerCase() === "a") {
                el.setAttribute("rel", "noopener noreferrer");
                el.setAttribute("target", "_blank");
            }
        }
        return tpl.innerHTML;
    }
    A.sanitizeHtml = sanitizeHtml;

    /* ================================================================
       Markdown
       ================================================================ */
    if (window.marked) {
        window.marked.setOptions({ breaks: true, gfm: true, headerIds: false, mangle: false });
    }

    function escapeHtml(s) {
        return String(s || "").replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }
    A.escapeHtml = escapeHtml;

    function mdToHtml(text) {
        if (!text) { return ""; }
        var clean = typeof A.cleanProtocol === "function" ? A.cleanProtocol(text) : text;
        if (typeof A.consentStripMarker === "function") {
            clean = A.consentStripMarker(clean);
        }
        if (!window.marked) { return escapeHtml(clean); }
        try {
            return sanitizeHtml(window.marked.parse(clean));
        } catch (e) { return escapeHtml(clean); }
    }
    A.mdToHtml = mdToHtml;

    /* ================================================================
       Code block enhancement
       ================================================================ */
    function enhanceCodeBlocks(container) {
        var pres = container.querySelectorAll("pre");
        for (var i = 0; i < pres.length; i++) {
            if (pres[i].querySelector(".gb-code-header")) { continue; }
            var code = pres[i].querySelector("code");
            var lc = code ? (code.className || "") : "";
            var lm = lc.match(/language-(\w+)/);
            var lang = lm ? lm[1] : "";

            var hdr = document.createElement("div");
            hdr.className = "gb-code-header";
            var lbl = document.createElement("span");
            lbl.className = "gb-code-lang";
            lbl.textContent = lang || "code";
            hdr.appendChild(lbl);

            var cpBtn = document.createElement("button");
            cpBtn.className = "gb-code-copy";
            cpBtn.textContent = A.t("assistant.copy", "Copy");
            (function (pre, btn) {
                btn.addEventListener("click", function () {
                    var c = pre.querySelector("code");
                    navigator.clipboard.writeText(c ? c.textContent : pre.textContent).then(function () {
                        btn.textContent = A.t("assistant.copied", "Copied");
                        setTimeout(function () { btn.textContent = A.t("assistant.copy", "Copy"); }, 1500);
                    });
                });
            })(pres[i], cpBtn);
            hdr.appendChild(cpBtn);

            var wrBtn = document.createElement("button");
            wrBtn.className = "gb-code-wrap";
            wrBtn.textContent = A.t("assistant.wrap", "Wrap");
            (function (pre, btn) {
                btn.addEventListener("click", function () {
                    pre.classList.toggle("gb-code-wrapped");
                    btn.textContent = pre.classList.contains("gb-code-wrapped")
                        ? A.t("assistant.nowrap", "No wrap") : A.t("assistant.wrap", "Wrap");
                });
            })(pres[i], wrBtn);
            hdr.appendChild(wrBtn);
            pres[i].insertBefore(hdr, pres[i].firstChild);
        }
    }
    A.enhanceCodeBlocks = enhanceCodeBlocks;

    /* ================================================================
       Reasoning disclosure
       ================================================================ */
    function reasoningHtml(text, elapsed) {
        if (!text) { return ""; }
        var summary = elapsed > 0
            ? escapeHtml(A.t("assistant.thought-for", "Thought for")) + " " + elapsed + " s"
            : escapeHtml(A.t("assistant.reasoning", "Reasoning"));
        return '<details class="gb-reasoning"><summary>' +
            '<svg class="gb-reasoning-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>' +
            '<span>' + summary + '</span></summary>' +
            '<div class="gb-reasoning-body">' + escapeHtml(text) + '</div></details>';
    }
    A.reasoningHtml = reasoningHtml;

    /* ================================================================
       Notice card (leaked/unavailable tool)
       ================================================================ */
    function leakedNoticeHtml(raw) {
        var msg = A.t("assistant.tool-unavailable", "Tool unavailable in this edition");
        return '<div class="gb-notice-card">' +
            '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>' +
            '<span>' + escapeHtml(msg) + '</span>' +
            '<details class="gb-notice-raw"><summary>' +
                escapeHtml(A.t("assistant.details", "Details")) +
            '</summary><pre>' + escapeHtml(raw) + '</pre></details></div>';
    }

    /* ================================================================
       Citations / sources
       ================================================================ */
    function citationsHtml(citations) {
        if (!citations || !citations.length) { return ""; }
        var items = "";
        for (var i = 0; i < citations.length; i++) {
            var c = citations[i];
            var title = c.title || c.name || c.source || ("Source " + (i + 1));
            var url = c.url || c.link || "";
            if (url) {
                items += '<a class="gb-citation-link" href="' + escapeHtml(url) +
                    '" target="_blank" rel="noopener noreferrer">' + escapeHtml(title) + '</a>';
            } else {
                items += '<span class="gb-citation-item">' + escapeHtml(title) + '</span>';
            }
        }
        return '<details class="gb-citations"><summary>' +
            escapeHtml(A.t("assistant.sources", "Sources")) +
            ' (' + citations.length + ')</summary>' +
            '<div class="gb-citations-list">' + items + '</div></details>';
    }

    /* ================================================================
       Per-message actions (hover)
       ================================================================ */
    function addMessageActions(el, content, retryable) {
        var actions = document.createElement("div");
        actions.className = "gb-msg-actions";
        if (content) {
            var cpBtn = document.createElement("button");
            cpBtn.className = "gb-msg-action";
            cpBtn.textContent = A.t("assistant.copy", "Copy");
            cpBtn.addEventListener("click", function () {
                navigator.clipboard.writeText(content).then(function () {
                    cpBtn.textContent = A.t("assistant.copied", "Copied");
                    setTimeout(function () { cpBtn.textContent = A.t("assistant.copy", "Copy"); }, 1500);
                });
            });
            actions.appendChild(cpBtn);
        }
        if (retryable) {
            var rtBtn = document.createElement("button");
            rtBtn.className = "gb-msg-action gb-msg-retry";
            rtBtn.textContent = A.t("assistant.retry", "Try again");
            rtBtn.addEventListener("click", function () {
                var msgs = document.getElementById("gbMessages");
                if (!msgs) { return; }
                var users = msgs.querySelectorAll(".gb-msg-user");
                var last = users.length ? users[users.length - 1] : null;
                var text = last ? (last.dataset.originalContent || last.textContent || "").trim() : "";
                if (text) { A.sendMessage(text); }
            });
            actions.appendChild(rtBtn);
        }
        el.appendChild(actions);
    }

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
            for (var i = 0; i < files.length; i++) {
                var chip = document.createElement("span");
                chip.className = "gb-file-chip-inline";
                chip.textContent = files[i].name || files[i];
                bubble.appendChild(chip);
            }
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
        var el = document.createElement("div");
        el.className = "gb-msg-assistant";
        var elapsed = A.thinkingElapsed();
        if (reasoning) { el.insertAdjacentHTML("beforeend", reasoningHtml(reasoning, elapsed)); }
        var card = typeof A.consentFromContent === "function" ? A.consentFromContent(raw) : null;
        if (card) { el.appendChild(card); }
        if (leaked && !card) { el.insertAdjacentHTML("beforeend", leakedNoticeHtml(raw)); }
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
        document.getElementById("gbMessages").appendChild(el);
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
            } else {
                el.className = "gb-msg-assistant";
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
