"use strict";
/**
 * 18_file_panel.js - A file the assistant made, opened beside the
 * conversation.
 *
 * Answers link generated files as `vmfile:<conversation>/output/<name>`.
 * A click opens this panel instead of a dead browser link: name, type and
 * size; a preview for PDF (the webview's own viewer), images, text,
 * Markdown and CSV; office packages show the server's rendering when the
 * workspace can make one, otherwise a plain choice to download or open with
 * the computer's program. "Baixar" and "Salvar no Drive" (while no Drive
 * copy exists) are always at hand. Sources: 18a_file_store.js.
 */
(function () {
    var A = window.GBAssistant;
    var F = A.files;
    var I = window.GBIcon || function () { return ""; };
    var t = F.t;
    var panel = null, opener = null, current = null;

    function esc(s) {
        return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;")
            .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
    }

    function reason(err) {
        var st = err && err.status;
        if (st === 409) {
            return t("A Máquina Virtual desta conversa está desligada e o arquivo ainda não foi salvo no Drive.",
                "This conversation's virtual machine is off and the file has not been saved to Drive yet.");
        }
        if (st === 404) { return t("O arquivo não foi encontrado.", "The file was not found."); }
        if (st === 401 || st === 403) { return t("Sem permissão para ler este arquivo.", "No permission to read this file."); }
        return t("Não foi possível abrir o arquivo.", "The file could not be opened.") + (st ? " (HTTP " + st + ")" : "");
    }

    function close() {
        if (!panel) { return; }
        var hadFocus = panel.contains(document.activeElement);
        if (panel._url) { URL.revokeObjectURL(panel._url); }
        panel.remove();
        panel = null;
        current = null;
        var main = document.getElementById("gb-main");
        if (main) { main.classList.remove("gb-panel-open", "gb-file-panel-open"); }
        document.removeEventListener("keydown", onKey, true);
        if (hadFocus && opener && opener.isConnected) { opener.focus(); }
    }
    F.closePanel = close;

    function onKey(e) {
        if (e.key === "Escape" && panel && !document.querySelector(".gb-lightbox")) {
            e.preventDefault();
            e.stopPropagation();
            close();
        }
    }

    function setMeta(state) {
        if (!panel) { return; }
        var bits = [F.typeName(state.src.name)];
        if (state.size != null) { bits.push(F.fmtBytes(state.size)); }
        var copy = state.src.kind === "drive" ? state.src : state.copy;
        if (copy && copy.folder) { bits.push(t("No Drive", "In Drive") + ": " + copy.folder); }
        else if (state.src.kind === "vm") { bits.push(t("Na área de trabalho da conversa", "In the conversation workspace")); }
        panel.querySelector(".gb-file-meta").textContent = bits.join(" · ");
    }

    function notice(state, text) {
        var box = panel.querySelector(".gb-file-preview");
        box.className = "gb-file-preview gb-file-preview--notice";
        box.innerHTML = '<div class="gb-file-notice">' + I("file", 28) + '<p>' + esc(text) + '</p></div>';
    }

    function showText(box, text, kind) {
        if (kind === "markdown" && typeof A.mdToHtml === "function") {
            box.innerHTML = '<div class="gb-file-md">' + A.mdToHtml(text) + '</div>';
            return;
        }
        if (kind === "csv") {
            var sep = text.indexOf("\t") !== -1 && text.indexOf(",") === -1 ? "\t" : (text.split("\n")[0].indexOf(";") !== -1 ? ";" : ",");
            var rows = text.split(/\r?\n/).filter(Boolean).slice(0, 300).map(function (line) { return line.split(sep); });
            box.innerHTML = '<table class="gb-attach-panel-table">' + rows.map(function (r, i) {
                var tag = i === 0 ? "th" : "td";
                return "<tr>" + r.map(function (c) { return "<" + tag + ">" + esc(c.replace(/^"|"$/g, "")) + "</" + tag + ">"; }).join("") + "</tr>";
            }).join("") + "</table>";
            return;
        }
        box.innerHTML = '<pre class="gb-file-text">' + esc(text.slice(0, 200000)) + '</pre>';
    }

    function preview(state) {
        var box = panel.querySelector(".gb-file-preview");
        var kind = F.kind(state.src.name);
        var mine = panel;
        F.fetchBlob(state.src).then(function (blob) {
            if (panel !== mine) { return; }
            state.blob = blob;
            state.size = blob.size;
            if (state.src.driveCopy) { state.copy = state.src.driveCopy; paintSave(state); }
            setMeta(state);
            box.className = "gb-file-preview gb-file-preview--" + kind;
            if (kind === "pdf" || kind === "image") {
                var typed = blob.type ? blob : new Blob([blob], { type: kind === "pdf" ? "application/pdf" : "image/" + F.ext(state.src.name) });
                panel._url = URL.createObjectURL(typed);
                box.innerHTML = kind === "pdf"
                    ? '<iframe class="gb-file-frame" title="' + esc(state.src.name) + '" src="' + panel._url + '"></iframe>'
                    : '<img class="gb-file-img" alt="' + esc(state.src.name) + '" src="' + panel._url + '">';
                return;
            }
            if (kind === "text" || kind === "markdown" || kind === "csv") {
                return blob.text().then(function (text) { if (panel === mine) { showText(box, text, kind); } });
            }
            if (kind === "office" && F.canPreviewOffice(state.src)) {
                return F.fetchOfficeHtml(state.src).then(function (html) {
                    if (panel !== mine) { return; }
                    box.innerHTML = '<iframe class="gb-file-frame gb-file-frame--doc" sandbox="" title="' + esc(state.src.name) + '"></iframe>';
                    box.querySelector("iframe").srcdoc = html;
                }).catch(function () { if (panel === mine) { officeNotice(state); } });
            }
            if (kind === "office") { officeNotice(state); return; }
            notice(state, t("Este tipo de arquivo não tem visualização aqui. Baixe o arquivo para abri-lo.",
                "This kind of file has no preview here. Download it to open it."));
        }).catch(function (err) {
            if (panel !== mine) { return; }
            state.error = err;
            notice(state, reason(err));
            panel.querySelectorAll('[data-act="download"],[data-act="open"]').forEach(function (b) { b.disabled = true; });
        });
    }

    function officeNotice(state) {
        notice(state, F.isDesktop()
            ? t("Este arquivo abre no aplicativo do computador. Use “Abrir com o aplicativo do computador” ou “Baixar”.",
                "This file opens in the computer's program. Use “Open with the computer's program” or “Download”.")
            : t("A visualização deste tipo de arquivo não está disponível aqui. Use “Baixar” para abri-lo no seu computador.",
                "This kind of file cannot be previewed here. Use “Download” to open it on your computer."));
    }

    function paintSave(state) {
        var btn = panel && panel.querySelector('[data-act="save"]');
        if (!btn) { return; }
        var saved = state.src.kind === "drive" || !!state.copy;
        btn.hidden = !saved && !F.canSaveToDrive();
        btn.disabled = saved || state.saving;
        btn.classList.toggle("gb-file-saved", saved);
        btn.innerHTML = saved ? I("check-circle", 14) + "<span>" + esc(t("Salvo no Drive", "Saved to Drive")) + "</span>"
            : I("folder", 14) + "<span>" + esc(state.saving ? t("Salvando…", "Saving…") : t("Salvar no Drive", "Save to Drive")) + "</span>";
    }

    function withBlob(state) {
        return state.blob ? Promise.resolve(state.blob) : F.fetchBlob(state.src).then(function (b) { state.blob = b; return b; });
    }

    function status(text, kind) {
        var el = panel && panel.querySelector(".gb-file-status");
        if (!el) { return; }
        el.textContent = text || "";
        el.dataset.kind = kind || "";
        if (text) { A.announce(text); }
    }

    function act(state, which, btn) {
        if (which === "download") {
            btn.disabled = true;
            withBlob(state).then(function (blob) { return F.saveBlob(blob, state.src.name); }).then(function (path) {
                status(path ? t("Salvo em ", "Saved to ") + path : "", "ok");
            }).catch(function (err) { status(reason(err), "error"); }).then(function () { btn.disabled = false; });
            return;
        }
        if (which === "open") {
            btn.disabled = true;
            withBlob(state).then(function (blob) { return F.openOutside(blob, state.src.name); }).then(function (path) {
                status(path === true ? t("Aberto no aplicativo do computador.", "Opened in the computer's program.")
                    : path ? t("Aberto a partir de ", "Opened from ") + path : "", "ok");
            }).catch(function (err) {
                status(err && err.savedTo
                    ? t("Salvo em ", "Saved to ") + err.savedTo + t(". Abra-o pelo Explorador de Arquivos.", ". Open it from File Explorer.")
                    : reason(err), err && err.savedTo ? "ok" : "error");
            }).then(function () { btn.disabled = false; });
            return;
        }
        if (which === "save" && state.src.kind === "vm" && !state.copy) {
            state.saving = true;
            paintSave(state);
            F.saveToDrive(state.src).then(function (data) {
                state.saving = false;
                var hit = (data.saved || [])[0];
                if (!hit) { throw new Error("none"); }
                state.copy = { kind: "drive", name: state.src.name, path: hit.path, folder: data.folder || "" };
                paintSave(state);
                setMeta(state);
                status(t("Salvo no Drive, na pasta ", "Saved to Drive, in the folder ") + (data.folder || ""), "ok");
            }).catch(function (err) {
                state.saving = false;
                paintSave(state);
                status(t("Não foi possível salvar no Drive.", "Could not save to Drive.") + (err && err.status ? " (HTTP " + err.status + ")" : ""), "error");
            });
        }
    }

    /* src: { kind:"vm", conversationId, path, name } or
            { kind:"drive", path, name, size, folder } */
    F.open = function (src, from) {
        close();
        opener = from || document.activeElement;
        var state = current = { src: src, size: src.size != null ? src.size : null, copy: null };
        var kind = F.kind(src.name);
        panel = document.createElement("aside");
        panel.className = "gb-attach-panel gb-file-panel";
        panel.setAttribute("role", "dialog");
        panel.setAttribute("aria-labelledby", "gbFilePanelTitle");
        var openBtn = F.isDesktop() && kind !== "image"
            ? '<button type="button" class="gb-block-btn" data-act="open">' + I("arrow-square-out", 14) + '<span>' +
                esc(t("Abrir com o aplicativo do computador", "Open with the computer's program")) + '</span></button>' : "";
        panel.innerHTML =
            '<div class="gb-attach-panel-header">' +
                '<span class="gb-file-icon" aria-hidden="true">' + I(A.fileTypeIcon ? A.fileTypeIcon(src.name) : "file", 18) + '</span>' +
                '<div class="gb-file-head"><h2 class="gb-attach-panel-title" id="gbFilePanelTitle"></h2><p class="gb-file-meta"></p></div>' +
                '<button type="button" class="gb-attach-panel-close" aria-label="' + esc(t("Fechar", "Close")) + '">' + I("x", 16) + '</button>' +
            '</div>' +
            '<div class="gb-file-preview gb-file-preview--loading"><p class="gb-attach-panel-loading">' + esc(t("Carregando…", "Loading…")) + '</p></div>' +
            '<div class="gb-file-actions">' +
                '<button type="button" class="gb-block-btn gb-block-btn--acc" data-act="download">' + I("download", 14) + '<span>' + esc(t("Baixar", "Download")) + '</span></button>' +
                openBtn +
                '<button type="button" class="gb-block-btn" data-act="save" hidden></button>' +
            '</div>' +
            '<p class="gb-file-status" role="status"></p>';
        panel.querySelector("#gbFilePanelTitle").textContent = src.name;
        panel.querySelector(".gb-attach-panel-close").addEventListener("click", close);
        panel.querySelector(".gb-file-actions").addEventListener("click", function (e) {
            var btn = e.target.closest("[data-act]");
            if (btn && !btn.disabled) { act(state, btn.getAttribute("data-act"), btn); }
        });
        if (typeof A.closeAttachPanel === "function") { A.closeAttachPanel(); }
        var main = document.getElementById("gb-main") || document.body;
        main.appendChild(panel);
        main.classList.add("gb-panel-open", "gb-file-panel-open");
        document.addEventListener("keydown", onKey, true);
        setMeta(state);
        paintSave(state);
        panel.querySelector(".gb-attach-panel-close").focus();
        if (src.kind === "vm") {
            F.findDriveCopy(src).then(function (copy) {
                if (current !== state || !copy) { return; }
                state.copy = copy;
                paintSave(state);
                setMeta(state);
            });
        }
        preview(state);
        return panel;
    };

    /* Every generated-file link, in any answer, opens the panel. */
    document.addEventListener("click", function (e) {
        var link = e.target.closest && e.target.closest('a[href^="vmfile:"]');
        if (!link) { return; }
        e.preventDefault();
        e.stopPropagation();
        F.open(F.parseLink(link.getAttribute("href")), link);
    }, true);

    /* An answer that links a file re-reads its run's header (12b). */
    var pending = 0;
    function watch() {
        var msgs = document.getElementById("gbMessages");
        if (!msgs) { return; }
        new MutationObserver(function () {
            clearTimeout(pending);
            pending = setTimeout(function () {
                if (A.repaintRunsIn && msgs.querySelector('a[href^="vmfile:"]')) { A.repaintRunsIn(msgs); }
            }, 120);
        }).observe(msgs, { childList: true, subtree: true });
    }
    if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", watch); } else { watch(); }
})();
