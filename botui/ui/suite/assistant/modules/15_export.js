"use strict";
/**
 * 15_export.js - Export conversation (Markdown) and print/PDF.
 */

(function () {
    var A = window.GBAssistant;
    var I = window.GBIcon || function () { return ""; };

    /** Export the current conversation as a Markdown file. */
    A.exportConversation = function () {
        var sessId = A.currentSessionId || A.sessionId;
        if (!sessId) { return; }
        A.loadSessionMessages(sessId).then(function (msgs) {
            if (!msgs || !msgs.length) { return; }
            var title = conversationTitle();
            var date = new Date().toISOString().slice(0, 10);
            var lines = ["# " + title, "", "_" + date + "_", ""];
            for (var i = 0; i < msgs.length; i++) {
                var m = msgs[i];
                var role = m.role === "user"
                    ? (A.userName || A.t("assistant.user", "User"))
                    : (productName());
                lines.push("**" + role + ":**");
                lines.push("");
                /* Keep code blocks intact, strip reasoning */
                var content = (m.content || "").trim();
                if (content) { lines.push(content); }
                /* Attachments */
                if (m.attachments && m.attachments.length) {
                    for (var a = 0; a < m.attachments.length; a++) {
                        var att = m.attachments[a];
                        var name = att.name || att.filename || "attachment";
                        lines.push("  - " + name);
                    }
                }
                lines.push("");
                lines.push("---");
                lines.push("");
            }
            var md = lines.join("\n");
            var slug = title.replace(/[^a-zA-Z0-9À-ɏ]+/g, "-").toLowerCase().slice(0, 60);
            triggerDownload(md, slug + ".md", "text/markdown;charset=utf-8");
        }).catch(function (err) {
            console.warn("[export] failed:", err);
        });
    };

    /** Print / PDF via browser print dialog with a clean print stylesheet. */
    A.printConversation = function () {
        window.print();
    };

    /* ---- helpers ---- */
    function conversationTitle() {
        if (A.sessions && A.currentSessionId) {
            var s = A.sessions.filter(function (x) { return x.id === A.currentSessionId; })[0];
            if (s && s.title) { return s.title; }
        }
        return A.t("assistant.untitled", "Untitled");
    }

    function productName() {
        return (A.productConfig && A.productConfig.name) || "General Bots";
    }

    function triggerDownload(text, filename, mime) {
        /* Try the Tauri save-download command first (desktop app). */
        if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === "function") {
            var encoder = new TextEncoder();
            var bytes = Array.from(encoder.encode(text));
            window.__TAURI_INTERNALS__.invoke("save_download", {
                filename: filename,
                data: bytes,
                mimeType: mime
            }).catch(function () { fallbackDownload(text, filename, mime); });
            return;
        }
        fallbackDownload(text, filename, mime);
    }

    /** Add export/print buttons next to the titlebar title. */
    A.initExportActions = function () {
        var titleEl = document.getElementById("gbTitlebarTitle");
        if (!titleEl) { return; }
        var wrap = document.createElement("span");
        wrap.className = "gb-export-actions";
        wrap.id = "gbExportActions";

        var exportBtn = document.createElement("button");
        exportBtn.className = "gb-export-btn";
        exportBtn.title = A.t("assistant.export-md", "Export Markdown");
        exportBtn.setAttribute("aria-label", exportBtn.title);
        exportBtn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>';
        exportBtn.addEventListener("click", function () { A.exportConversation(); });

        var printBtn = document.createElement("button");
        printBtn.className = "gb-export-btn";
        printBtn.title = A.t("assistant.print", "Print / PDF");
        printBtn.setAttribute("aria-label", printBtn.title);
        printBtn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="6 9 6 2 18 2 18 9"/><path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/><rect x="6" y="14" width="12" height="8"/></svg>';
        printBtn.addEventListener("click", function () { A.printConversation(); });

        wrap.appendChild(exportBtn);
        wrap.appendChild(printBtn);
        titleEl.parentNode.insertBefore(wrap, titleEl.nextSibling);
    };

    function fallbackDownload(text, filename, mime) {
        var blob = new Blob([text], { type: mime });
        var url = URL.createObjectURL(blob);
        var a = document.createElement("a");
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        setTimeout(function () {
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        }, 100);
    }
})();
