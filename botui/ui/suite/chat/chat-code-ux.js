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

    function escapeHtml(text) {
        var div = document.createElement("div");
        div.textContent = text == null ? "" : String(text);
        return div.innerHTML;
    }

    function highlight(text, lang) {
        var rules = LANGUAGE_TOKENIZERS[lang || ""] || LANGUAGE_TOKENIZERS.js;
        var out = escapeHtml(text);
        for (var r = 0; r < rules.length; r++) {
            var rule = rules[r];
            if (!rule.re) continue;
            rule.re.lastIndex = 0;
            if (rule.re.test(out)) {
                rule.re.lastIndex = 0;
                out = out.replace(rule.re, function (match) {
                    return '<span class="gb-' + rule.named + '">' + match + "</span>";
                });
            }
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

    function enhance(container) {
        if (!container) return;
        var pres = container.querySelectorAll("pre");
        for (var i = 0; i < pres.length; i++) {
            var pre = pres[i];
            if (pre.classList.contains("gb-enhanced")) continue;
            var code = pre.querySelector("code");
            if (!code) continue;
            var text = code.textContent || "";
            if (!text.trim()) continue;
            pre.classList.add("gb-enhanced");
            var lang = "";
            var m = (code.className || "").match(/language-([a-zA-Z0-9_-]+)/);
            if (m) lang = m[1].toLowerCase();
            if (!lang && /^diff\s|^diff --git/.test(text)) lang = "diff";
            var diffMode = lang === "diff" || isDiff(text);
            var body = diffMode ? renderDiff(text) : highlight(text, lang);
            pre.innerHTML =
                '<div class="gb-code-head">' +
                '<span class="gb-code-lang">' + escapeHtml(lang || "code") + "</span>" +
                '<span class="gb-code-count">' + text.split("\n").length + " lines</span>" +
                '<button type="button" class="gb-code-copy">Copy</button>' +
                "</div>" +
                '<code class="gb-code-body">' + body + "</code>";
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
        highlight: highlight,
        renderDiff: renderDiff,
        isDiff: isDiff,
        toolCardHtml: toolCardHtml,
        extractToolCards: extractToolCards,
        appendCard: appendCard
    };
})();