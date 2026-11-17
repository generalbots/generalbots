"use strict";
/**
 * 03c_markdown.js - Sanitizer, markdown conversion, code block enhancement.
 * Split from 03_render.js to stay under 450 lines each.
 */

(function () {
    var A = window.GBAssistant;

    /* ================================================================
       Sanitizer (template-based, mirrors legacy GBSanitize)
       ================================================================ */
    var DROP = "script,style,iframe,frame,frameset,object,embed,applet,link,meta,base,form,input,textarea,select,option,button,foreignObject,use,set";
    var URI_ATTRS = ["href","src","xlink:href","action","formaction","poster","background","data","srcset","srcdoc"];

    function uriAllowed(val) {
        var v = String(val == null ? "" : val).replace(/[\u0000-  ]/g, "").toLowerCase();
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
            return inlineCodeInHtml(sanitizeHtml(window.marked.parse(clean)));
        } catch (e) { return escapeHtml(clean); }
    }
    A.mdToHtml = mdToHtml;

    /* Some models answer with HTML lists and still write `code` spans in
       them. Markdown is not parsed inside HTML blocks, so the backticks would
       show: turn those spans into code elements, outside code and pre only. */
    function inlineCodeInHtml(html) {
        if (html.indexOf("`") === -1) { return html; }
        var box = document.createElement("template");
        box.innerHTML = html;
        var walker = document.createTreeWalker(box.content, NodeFilter.SHOW_TEXT);
        var nodes = [];
        while (walker.nextNode()) {
            var n = walker.currentNode;
            if (n.nodeValue.indexOf("`") !== -1 && !n.parentElement.closest("code, pre")) { nodes.push(n); }
        }
        nodes.forEach(function (n) {
            var parts = n.nodeValue.split(/`([^`]+)`/);
            if (parts.length < 3) { return; }
            var frag = document.createDocumentFragment();
            parts.forEach(function (part, i) {
                if (i % 2 === 1) {
                    var code = document.createElement("code");
                    code.textContent = part;
                    frag.appendChild(code);
                } else if (part) {
                    frag.appendChild(document.createTextNode(part));
                }
            });
            n.parentNode.replaceChild(frag, n);
        });
        return box.innerHTML;
    }

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
       Syntax highlighting (highlight.js, if loaded)
       ================================================================ */
    A.highlightCodeBlocks = function (container) {
        if (!window.hljs) { return; }
        var codes = container.querySelectorAll("pre code[class*='language-']");
        for (var i = 0; i < codes.length; i++) {
            if (codes[i].dataset.highlighted) { continue; }
            try {
                window.hljs.highlightElement(codes[i]);
                codes[i].dataset.highlighted = "1";
            } catch (e) { /* unsupported language, skip */ }
        }
        /* Also try auto-detect on untagged code blocks */
        var plain = container.querySelectorAll("pre code:not([class*='language-']):not([data-highlighted])");
        for (var j = 0; j < plain.length; j++) {
            var text = plain[j].textContent || "";
            if (text.length > 20 && text.length < 50000) {
                try {
                    var result = window.hljs.highlightAuto(text);
                    if (result.relevance > 5) {
                        plain[j].innerHTML = result.value;
                        plain[j].classList.add("hljs");
                        plain[j].dataset.highlighted = "1";
                    }
                } catch (e) { /* skip */ }
            }
        }
    };

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
    A.leakedNoticeHtml = leakedNoticeHtml;

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
    A.citationsHtml = citationsHtml;

    /** Sources that arrive after the answer go under the last answer. */
    A.onCitations = function (citations) {
        var all = document.querySelectorAll("#gbMessages > .gb-msg-assistant");
        var el = all.length ? all[all.length - 1] : null;
        if (!el || !citations.length || el.querySelector(".gb-citations")) { return; }
        var toolbar = el.querySelector(".gb-msg-toolbar");
        if (toolbar) { toolbar.insertAdjacentHTML("beforebegin", citationsHtml(citations)); }
        else { el.insertAdjacentHTML("beforeend", citationsHtml(citations)); }
    };
})();
