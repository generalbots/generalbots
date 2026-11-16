"use strict";
/**
 * 03d_math.js - LaTeX math rendering via KaTeX.
 *
 * Renders $...$, $$...$$, \(...\), \[...\] as typeset maths after the
 * markdown pass, without breaking code blocks or currency like "R$ 10".
 *
 * KaTeX 0.19.1 (MIT) vendored in vendor/katex/.
 */

(function () {
    var A = window.GBAssistant;

    if (typeof katex === "undefined") {
        A.renderMath = function () {};
        return;
    }

    /* ---- pre-process: extract LaTeX before markdown ---- */

    var mathSlots = [];
    var slotPrefix = "GBMATH";

    function slotTag(id) {
        return slotPrefix + id + "END";
    }

    /**
     * Extract LaTeX delimiters from raw text, replace with slot
     * markers that survive markdown, return the modified text.
     * Code fences are skipped.
     */
    function extractMath(text) {
        if (!text) { return text; }
        mathSlots = [];
        var id = 0;

        /* Protect code fences: split on ``` boundaries. */
        var parts = text.split(/(```[\s\S]*?```)/);
        for (var p = 0; p < parts.length; p++) {
            /* Odd indices are inside code fences -- skip them. */
            if (p % 2 === 1) { continue; }
            var seg = parts[p];

            /* Display: $$...$$ */
            seg = seg.replace(
                /\$\$([\s\S]+?)\$\$/g,
                function (match, tex) {
                    var tag = slotTag(id);
                    mathSlots.push({ id: id, tex: tex, display: true });
                    id++;
                    return tag;
                }
            );

            /* Display: \[...\] */
            seg = seg.replace(
                /\\\[([\s\S]+?)\\\]/g,
                function (match, tex) {
                    var tag = slotTag(id);
                    mathSlots.push({ id: id, tex: tex, display: true });
                    id++;
                    return tag;
                }
            );

            /* Inline: \(...\) */
            seg = seg.replace(
                /\\\(([\s\S]+?)\\\)/g,
                function (match, tex) {
                    var tag = slotTag(id);
                    mathSlots.push({ id: id, tex: tex, display: false });
                    id++;
                    return tag;
                }
            );

            /* Inline: $...$
               Not preceded by alnum or $. Not followed by digit
               (currency: "R$ 10"). Non-empty inside. */
            seg = seg.replace(
                /(^|[^a-zA-Z0-9$])\$(?!\d)([^$\n]+?)\$(?=[^a-zA-Z0-9$]|$)/gm,
                function (match, before, tex) {
                    if (!tex || !tex.trim()) { return match; }
                    var tag = slotTag(id);
                    mathSlots.push({ id: id, tex: tex, display: false });
                    id++;
                    return before + tag;
                }
            );

            parts[p] = seg;
        }
        return parts.join("");
    }

    /**
     * Render a single LaTeX string to KaTeX HTML.
     */
    function renderOne(tex, displayMode) {
        try {
            return katex.renderToString(tex.trim(), {
                displayMode: displayMode,
                throwOnError: false,
                strict: false,
                trust: false,
                output: "htmlAndMathml"
            });
        } catch (e) {
            var esc = A.escapeHtml ? A.escapeHtml(tex) : tex;
            return '<span class="gb-math-error" title="LaTeX error">' +
                esc + '</span>';
        }
    }

    /* ---- Wrap mdToHtml to extract and restore math ---- */

    var origMdToHtml = A.mdToHtml;
    A.mdToHtml = function (text) {
        if (!text) { return origMdToHtml(text); }

        /* Quick check: any math delimiters at all? */
        if (text.indexOf("$") === -1 &&
            text.indexOf("\\(") === -1 &&
            text.indexOf("\\[") === -1) {
            return origMdToHtml(text);
        }

        var protected_ = extractMath(text);
        var html = origMdToHtml(protected_);

        /* Replace slot markers with rendered KaTeX. */
        for (var i = 0; i < mathSlots.length; i++) {
            var slot = mathSlots[i];
            var tag = slotTag(slot.id);
            var cls = slot.display
                ? "gb-math-display" : "gb-math-inline";
            var rendered = '<span class="' + cls + '">' +
                renderOne(slot.tex, slot.display) + '</span>';
            html = html.replace(tag, rendered);
        }
        mathSlots = [];
        return html;
    };

    /**
     * Post-process a container for any remaining math (e.g. from
     * history where the content already went through mdToHtml).
     * This is a safety net; the wrapped mdToHtml handles most cases.
     */
    A.renderMath = function (container) {
        /* Nothing to do: the wrapped mdToHtml already rendered.
           Keep the function so callers don't break. */
    };
})();
