/**
 * Chat Code UX (#748) — syntax-highlighted code blocks, unified diff viewer,
 * and tool progress cards rendered inline in chat messages.
 */
(function () {
    "use strict";

    var LANGUAGE_TOKENIZERS = {
        js: [{ named: "kw", re: /\b(?:const|let|var|function|return|if|else|for|while|async|await|class|new|import|from|export|try|catch|throw|switch|case|break|continue|null|undefined|true|false|typeof|instanceof|of|in|this|yield|static|extends|super)\b/g },
             { named: "str", re: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`/g },
             { named: "num", re: /\b\d+(?:\.\d+)?\b/g },
             { named: "com", re: /\/\/[^\n]*|\/\*[\s\S]*?\*\//g }],
        rust: [{ named: "kw", re: /\b(?:fn|let|mut|const|struct|enum|impl|trait|pub|use|mod|async|await|match|if|else|loop|while|for|return|self|Self|Box|Vec|String|Result|Option|Some|None|Ok|Err|true|false|where|dyn|crate|super|ref|move|break|continue)\b/g },
               { named: "str", re: /"(?:\\.|[^"\\])*"/g },
               { named: "num", re: /\b\d+(?:\.\d+)?\b/g },
               { named: "com", re: /\/\/[^\n]*/g }],
        python: [{ named: "kw", re: /\b(?:def|class|return|if|elif|else|for|while|in|not|and|or|is|None|True|False|import|from|as|try|except|finally|with|lambda|pass|yield|global|nonlocal|raise)\b/g },
                 { named: "str", re: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g },
                 { named: "num", re: /\b\d+(?:\.\d+)?\b/g },
                 { named: "com", re: /#[^\n]*/g }],
        bash: [{ named: "kw", re: /\b(?:if|then|else|fi|for|while|do|done|case|esac|function|export|local|echo|printf|return|exit|cd|rm|cp|mv|mkdir|grep|sed|awk|git|npm|npx|sudo|sh)\b/g },
               { named: "arg", re: /--?[a-zA-Z][a-zA-Z0-9_-]*/g },
               { named: "str", re: /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g },
               { named: "com", re: /#[^\n]*/g }],
        json: [{ named: "key", re: /"(?:\\.|[^"\\])*"(?=\s*:)/g },
               { named: "str", re: /"(?:\\.|[^"\\])*"/g },
               { named: "kw", re: /\b(?:true|false|null)\b/g },
               { named: "num", re: /-?\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b/g }],
        html: [{ named: "tag", re: /<\/?[a-zA-Z][a-zA-Z0-9]*|\/?>/g },
               { named: "attr", re: /\b[a-zA-Z-]+=("[^"]*"|'[^']*')/g },
               { named: "com", re: /<!--[\s\S]*?-->/g }],
        css: [{ named: "sel", re: /[.#]?[a-zA-Z-]+(?=\s*\{)/g },
              { named: "prop", re: /\b[a-z-]+(?=\s*:)/g },
              { named: "com", re: /\/\*[\s\S]*?\*\//g }]
    };
    LANGUAGE_TOKENIZERS.ts = LANGUAGE_TOKENIZERS.js;
    LANGUAGE_TOKENIZERS.typescript = LANGUAGE_TOKENIZERS.js;
    LANGUAGE_TOKENIZERS.py = LANGUAGE_TOKENIZERS.python;
    LANGUAGE_TOKENIZERS.sh = LANGUAGE_TOKENIZERS.bash;
    LANGUAGE_TOKENIZERS.shell = LANGUAGE_TOKENIZERS.bash;
    LANGUAGE_TOKENIZERS.basic = LANGUAGE_TOKENIZERS.bash;
    LANGUAGE_TOKENIZERS.vbs = LANGUAGE_TOKENIZERS.js;
    LANGUAGE_TOKENIZERS.text = [];
    LANGUAGE_TOKENIZERS.plain = [];
    LANGUAGE_TOKENIZERS.plaintext = [];

    function escapeHtml(value) {
      if (window.GBShared && window.GBShared.html) return window.GBShared.html.escape(value);
      return String(value == null ? "" : value)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
    }

    function highlight(text, lang) {
        var rules = LANGUAGE_TOKENIZERS[lang || ""] || LANGUAGE_TOKENIZERS.js;
        if (!rules.length) return escapeHtml(text);
        // Tokenize the raw source in one pass. The previous version ran each
        // rule over the HTML produced by the earlier ones, so the string rule
        // matched class="gb-kw" and code rendered as `"gb-kw">def`.
        var out = "";
        var pos = 0;
        while (pos < text.length) {
            var best = null;
            for (var r = 0; r < rules.length; r++) {
                var rule = rules[r];
                if (!rule.re) continue;
                var scanner = new RegExp(rule.re.source, rule.re.flags || "g");
                scanner.lastIndex = pos;
                var match = scanner.exec(text);
                if (!match || match[0].length === 0) continue;
                if (!best || match.index < best.index) {
                    best = { index: match.index, value: match[0], named: rule.named };
                }
            }
            if (!best) {
                out += escapeHtml(text.slice(pos));
                break;
            }
            out += escapeHtml(text.slice(pos, best.index));
            out += '<span class="gb-' + best.named + '">' + escapeHtml(best.value) + "</span>";
            pos = best.index + best.value.length;
        }
        return out;
    }

    function renderDiff(text) {
        var lines = text.split("\n");
        var out = [];
        var adds = 0;
        var dels = 0;
        for (var i = 0; i < lines.length; i++) {
            var line = lines[i];
            var cls = "";
            if (/^[+][^+]/.test(line) || line === "+") { cls = "gb-diff-add"; adds++; }
            else if (/^[-][^-]/.test(line) || line === "-") { cls = "gb-diff-del"; dels++; }
            else if (/^@@/.test(line)) { cls = "gb-diff-hunk"; }
            else if (/^diff --git|^index |^new file|^deleted file|^--- |^\+\+\+ /.test(line)) { cls = "gb-diff-meta"; }
            out.push('<div class="gb-diff-line ' + cls + '">' + escapeHtml(line) + "</div>");
        }
        return '<div class="gb-diff-stats">' +
            '<span class="gb-diff-add-count">+' + adds + "</span> " +
            '<span class="gb-diff-del-count">-' + dels + "</span></div>" +
            '<div class="gb-diff-body">' + out.join("") + "</div>";
    }

    function isDiff(text) {
        if (!text) return false;
        var lines = text.split("\n");
        var first = lines[0] || "";
        if (/^diff( |$)|^Index: /.test(first)) return true;
        if (lines.length > 1 && /^--- (?:a\/)?/.test(first) && /^\+\+\+/.test(lines[1])) return true;
        return false;
    }

    var COLLAPSE_AFTER_LINES = 14;

    function enhance(container) {
        if (!container) return;
        var pres = container.querySelectorAll("pre");
        for (var i = 0; i < pres.length; i++) {
            var pre = pres[i];
            if (pre.classList.contains("gb-enhanced")) continue;
            var code = pre.querySelector("code");
            if (!code) {
                // Bare <pre> blocks (tool output, plainer renderers) were skipped,
                // so the longest listings were the ones that never collapsed.
                code = document.createElement("code");
                code.textContent = pre.textContent || "";
                pre.textContent = "";
                pre.appendChild(code);
            }
            var text = code.textContent || "";
            if (!text.trim()) continue;
            pre.classList.add("gb-enhanced");
            var lang = "";
            var m = (code.className || "").match(/language-([a-zA-Z0-9_-]+)/);
            if (m) lang = m[1].toLowerCase();
            if (!lang && /^diff\s|^diff --git/.test(text)) lang = "diff";
            var diffMode = lang === "diff" || isDiff(text);
            var body = diffMode ? renderDiff(text) : highlight(text, lang);
            // A long listing used to fill the whole window and push the answer
            // itself off screen, so long blocks start collapsed with an explicit
            // "show all" (and the copy button stays available either way).
            var lines = text.split("\n").length;
            // One enormous line wraps into dozens of visual rows, so length
            // counts as much as the line count when deciding to collapse.
            var collapsible = lines > COLLAPSE_AFTER_LINES || text.length > 900;
            pre.innerHTML =
                '<div class="gb-code-head">' +
                '<span class="gb-code-lang">' + escapeHtml(lang || "code") + "</span>" +
                '<span class="gb-code-count">' + lines + " lines</span>" +
                (collapsible
                    ? '<button type="button" class="gb-code-toggle">Show all</button>'
                    : "") +
                '<button type="button" class="gb-code-copy">Copy</button>' +
                "</div>" +
                '<code class="gb-code-body">' + body + "</code>";
            if (collapsible) {
                pre.classList.add("gb-code-collapsed");
                var toggle = pre.querySelector(".gb-code-toggle");
                toggle.addEventListener("click", function () {
                    var collapsed = pre.classList.toggle("gb-code-collapsed");
                    this.textContent = collapsed ? "Show all" : "Collapse";
                });
            }
            var copy = pre.querySelector(".gb-code-copy");
            copy.addEventListener("click", function () {
                var src = this.parentElement.nextElementSibling.textContent;
                if (navigator.clipboard && navigator.clipboard.writeText) {
                    navigator.clipboard.writeText(src).then(function () {
                        this.textContent = "Copied";
                        var btn = this;
                        setTimeout(function () { btn.textContent = "Copy"; }, 1500);
                    }.bind(this), function () {});
                }
            });
        }
    }

    // A single bot turn can still flood the window when its content is plain
    // text (not fenced code) — e.g. a tool that dumps a long listing. Collapse
    // such turns behind a "Show all" toggle so the answer stays readable.
    function collapseLong(container) {
        if (!container) return;
        var content = container.querySelector(".message-content");
        if (!content) return;
        if (content.classList.contains("gb-long-handled")) return;
        if (content.scrollHeight <= 480) return;
        content.classList.add("gb-long-handled", "gb-long-collapsed");
        var toggle = document.createElement("button");
        toggle.type = "button";
        toggle.className = "gb-long-toggle";
        toggle.textContent = "Show all";
        toggle.addEventListener("click", function () {
            var collapsed = content.classList.toggle("gb-long-collapsed");
            this.textContent = collapsed ? "Show all" : "Collapse";
        });
        container.appendChild(toggle);
    }

    /* Tool progress cards */

    function toolStatusIcon(status) {
        switch (status) {
            case "done": return "\u2713";
            case "error": return "\u2717";
            case "awaiting": return "\u23F3";
            default: return "";
        }
    }

    function toolCardHtml(tool) {
        var status = tool.status || "running";
        var icon = toolStatusIcon(status);
        var indicator = status === "running"
            ? '<span class="gb-tool-spinner"></span>'
            : '<span class="gb-tool-icon gb-tool-' + status + '">' + icon + "</span>";
        var detail = tool.argsDetail
            ? '<div class="gb-tool-args">' + escapeHtml(tool.argsDetail) + "</div>"
            : "";
        var summary = tool.summary
            ? '<div class="gb-tool-summary">' + escapeHtml(tool.summary) + "</div>"
            : "";
        return '<div class="gb-tool-card gb-tool-' + status + '" data-tool-id="' +
            escapeHtml(tool.id || "") + '">' + indicator +
            '<div class="gb-tool-info">' +
            '<div class="gb-tool-name">' + escapeHtml(tool.tool_name || tool.name || "tool") + "</div>" +
            detail + summary + "</div></div>";
    }

    function extractToolCards(text) {
        var cards = [];
        if (!text) return cards;
        var pattern = /\{[^{}]*"(?:tool_name|name|tool)"[^{}]*\}/g;
        var guard = 0;
        var match;
        while ((match = pattern.exec(text)) !== null && guard++ < 20) {
            var piece = match[0];
            if (!/"status"|"summary"|"args_detail"/.test(piece)) continue;
            try {
                var obj = JSON.parse(piece);
                if (obj.tool_name || obj.name) cards.push(obj);
            } catch (e) { /* not JSON */ }
        }
        if (cards.length === 0) {
            var re = /\b(?:tool|executed)\s*:\s*([A-Za-z0-9_\/\- ]{1,40})\s*(-{1,2}|–)\s*(running|done|awaited|awaiting|error|failed)/gi;
            var cm;
            var rest = text;
            while ((cm = re.exec(rest)) !== null) {
                var st = cm[3].toLowerCase();
                cards.push({
                    tool_name: cm[1].trim(),
                    status: st === "failed" ? "error" : st,
                    argsDetail: ""
                });
            }
        }
        return cards;
    }

    function appendCard(tool) {
        var messages = document.getElementById("messages");
        if (!messages) return;
        var id = tool.id || "";
        var existing = messages.querySelector('.gb-tool-card[data-tool-id="' + id + '"]');
        if (existing) {
            var card = document.createElement("div");
            card.innerHTML = toolCardHtml(tool);
            var html = card.firstChild;
            existing.replaceWith(html);
            return html;
        }
        var wrapper = document.createElement("div");
        wrapper.className = "message bot";
        wrapper.innerHTML = '<div class="message-content bot-message">' + toolCardHtml(tool) + "</div>";
        messages.appendChild(wrapper);
        return wrapper.querySelector(".gb-tool-card");
    }

    window.ChatCodeUX = {
        enhance: enhance,
        collapseLong: collapseLong,
        highlight: highlight,
        renderDiff: renderDiff,
        isDiff: isDiff,
        toolCardHtml: toolCardHtml,
        extractToolCards: extractToolCards,
        appendCard: appendCard
    };
})();