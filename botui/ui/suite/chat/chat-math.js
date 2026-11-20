"use strict";
/**
 * Chat — LaTeX typesetting (KaTeX, vendored locally in /suite/vendor/katex).
 *
 * Answers arrive as markdown and are sanitized after the markdown pass, so
 * maths cannot be rendered into the string that the sanitizer sees. Instead the
 * delimited LaTeX is replaced by a plain-text slot before markdown, and the
 * slots are swapped for KaTeX markup after sanitizing. Code fences are never
 * touched, and a `$` that looks like currency ("R$ 10") stays literal.
 */

(function () {
  var SLOT_PREFIX = "GBMATHSLOT";
  var SLOT_SUFFIX = "END";

  // KaTeX is ~300 KB of CSS+JS. Inject it from JS rather than as a static tag
  // in chat.html so it stays out of the parser-blocking boot graph (#2495).
  // Both tags are added once; extract()/restore() degrade to plain text until
  // the library is ready, so a slow load never drops a message.
  (function loadKatex() {
    if (!document || typeof document.createElement !== "function") return;
    if (typeof window.katex !== "undefined") return;
    if (!document.querySelector("link[data-gb-katex]")) {
      var link = document.createElement("link");
      link.rel = "stylesheet";
      link.href = "/suite/js/vendor/katex/katex.min.css";
      link.setAttribute("data-gb-katex", "1");
      (document.head || document.documentElement).appendChild(link);
    }
    if (!document.querySelector("script[data-gb-katex]")) {
      var script = document.createElement("script");
      script.src = "/suite/js/vendor/katex/katex.min.js";
      script.async = false;
      script.setAttribute("data-gb-katex", "1");
      (document.head || document.documentElement).appendChild(script);
    }
  })();

  function katexAvailable() {
    return typeof window.katex !== "undefined" &&
      typeof window.katex.renderToString === "function";
  }

  function hasMath(text) {
    var value = String(text || "");
    return value.indexOf("$") !== -1 ||
      value.indexOf("\\(") !== -1 ||
      value.indexOf("\\[") !== -1;
  }

  function slotTag(index) {
    return SLOT_PREFIX + index + SLOT_SUFFIX;
  }

  function pushSlot(slots, tex, display) {
    slots.push({ tex: tex, display: display });
    return slotTag(slots.length - 1);
  }

  /** Extract every display/inline expression from one non-code segment. */
  function extractSegment(segment, slots) {
    var out = segment;
    out = out.replace(/\$\$([\s\S]+?)\$\$/g, function (match, tex) {
      return pushSlot(slots, tex, true);
    });
    out = out.replace(/\\\[([\s\S]+?)\\\]/g, function (match, tex) {
      return pushSlot(slots, tex, true);
    });
    out = out.replace(/\\\(([\s\S]+?)\\\)/g, function (match, tex) {
      return pushSlot(slots, tex, false);
    });
    // Inline $...$: never preceded by a word character or another $, never
    // followed by a digit, so "R$ 10" and "$5 to $9" are left alone.
    out = out.replace(
      /(^|[^a-zA-Z0-9$])\$(?!\d)([^$\n]+?)\$(?=[^a-zA-Z0-9$]|$)/gm,
      function (match, before, tex) {
        if (!tex || !tex.trim()) return match;
        return before + pushSlot(slots, tex, false);
      }
    );
    return out;
  }

  /**
   * Replace LaTeX with slot tags so it survives markdown and sanitizing.
   * Returns { text, slots }; `slots` is empty when there is nothing to render.
   */
  function extract(text) {
    var source = String(text == null ? "" : text);
    var slots = [];
    if (!katexAvailable() || !hasMath(source)) return { text: source, slots: slots };
    var parts = source.split(/(```[\s\S]*?```)/);
    for (var index = 0; index < parts.length; index++) {
      // Odd indices are inside fenced code and must stay verbatim.
      if (index % 2 === 1) continue;
      parts[index] = extractSegment(parts[index], slots);
    }
    return { text: parts.join(""), slots: slots };
  }

  function renderOne(tex, display) {
    try {
      return window.katex.renderToString(String(tex).trim(), {
        displayMode: !!display,
        throwOnError: false,
        strict: false,
        trust: false,
        output: "htmlAndMathml"
      });
    } catch (error) {
      // Invalid LaTeX is shown as its source, never dropped silently.
      var escaped = String(tex)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
      return '<code class="gb-math-error">' + escaped + "</code>";
    }
  }

  /** Swap the slot tags left in sanitized HTML for rendered KaTeX markup. */
  function restore(html, slots) {
    if (!html || !slots || !slots.length) return html;
    var out = String(html);
    for (var index = 0; index < slots.length; index++) {
      var tag = slotTag(index);
      if (out.indexOf(tag) === -1) continue;
      var slot = slots[index];
      var className = slot.display ? "gb-math-display" : "gb-math-inline";
      var rendered = '<span class="' + className + '">' +
        renderOne(slot.tex, slot.display) + "</span>";
      out = out.split(tag).join(rendered);
    }
    return out;
  }

  window.GBChatMath = {
    enabled: katexAvailable,
    hasMath: hasMath,
    extract: extract,
    restore: restore
  };
})();
