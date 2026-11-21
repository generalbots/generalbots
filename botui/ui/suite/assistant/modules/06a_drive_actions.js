"use strict";
/**
 * 06a_drive_actions.js - Drive file actions: context menu, rename,
 * download, delete, preview, search.
 * Shell rendering, upload and folder creation in 06b_drive_shell.js.
 *
 * Reads shared state from window.GBDrive (set by 06_drive.js).
 */
(function () {
    var A = window.GBAssistant;
    var D = window.GBDrive;
    if (!D) { console.error("[drive] 06_drive.js not loaded"); return; }
    var esc = D.esc;

    /* ---- Context menu ---- */
    D.showCtxMenu = function (evt, path, name, isDir, opts) {
        var trigger = evt.currentTarget;
        if (D.ctxTrigger === trigger) { D.closeCtx(true); return; }
        D.closeCtx();
        D.ctxTrigger = trigger;
        trigger.setAttribute("aria-expanded", "true");
        D.ctxEl = document.createElement("div");
        D.ctxEl.className = "gb-drv-ctx";
        opts = opts || {};

        var pt = document.documentElement.lang.toLowerCase().startsWith("pt");
        var items = [];
        /* "Usar numa conversa" action: attach this file to the chat composer */
        if (!isDir && !opts.noUseInChat) {
            items.push({ label: pt ? "Usar numa conversa" : "Use in a conversation", icon: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z", action: "use-in-chat" });
        }
        items.push({ label: A.t("drive.download", "Download"), icon: "M12 3v12M7 10l5 5 5-5M5 21h14", action: "download" });
        if (!opts.noRename) {
            items.push({ label: A.t("drive.rename", "Rename"), icon: "M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z", action: "rename" });
        }
        if (!opts.noDelete) {
            items.push({ label: pt ? "Apagar" : "Delete", icon: "M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2", action: "delete", danger: true });
        }

        var html = "";
        for (var i = 0; i < items.length; i++) {
            html += '<button type="button" class="gb-drv-ctx-item' +
                (items[i].danger ? " gb-drv-ctx-danger" : "") +
                '" data-action="' + items[i].action + '">' +
                D.svg(items[i].icon, 14, 1.8) + '<span>' + esc(items[i].label) + '</span></button>';
        }
        D.ctxEl.innerHTML = html;
        document.body.appendChild(D.ctxEl);

        /* Client coordinates are divided by the app zoom: left/top are CSS pixels. */
        var zoom = window.GBZoom ? window.GBZoom.factor() : 1;
        var anchor = trigger.getBoundingClientRect();
        /* The menu hangs 2px past the trigger's right edge, 32px below its top. */
        var x = anchor.right / zoom + 2;
        var y = anchor.top / zoom + 32;
        var w = D.ctxEl.offsetWidth;
        var h = D.ctxEl.offsetHeight;
        x = Math.max(8, Math.min(window.innerWidth / zoom - w - 8, x - w));
        if (y + h > window.innerHeight / zoom - 8) y = Math.max(8, anchor.top / zoom - h - 4);
        D.ctxEl.style.left = x + "px";
        D.ctxEl.style.top = y + "px";

        D.ctxEl.addEventListener("click", function (e) {
            var btn = e.target.closest("[data-action]");
            if (!btn) return;
            var action = btn.getAttribute("data-action");
            D.closeCtx(true);
            if (action === "rename") startRename(path, name);
            else if (action === "download") downloadFile(path, name);
            else if (action === "delete") confirmDelete(path, name, isDir);
            else if (action === "use-in-chat") useFileInChat(path, name);
        });
        D.ctxEl.addEventListener("keydown", function (e) {
            if (e.key !== "Escape") return;
            e.preventDefault();
            e.stopPropagation();
            D.closeCtx(true);
        });
        D.ctxEl.querySelector("button").focus();
    };

    /* ---- Inline rename ---- */
    function startRename(path, oldName) {
        var row = document.querySelector('.gb-drv-row[data-path="' + CSS.escape(path) + '"]');
        if (!row) return;
        var nameCell = row.querySelector(".gb-drv-name-text");
        if (!nameCell || nameCell.querySelector("input")) return;
        var original = nameCell.textContent;
        var input = document.createElement("input");
        input.type = "text";
        input.className = "gb-drv-rename-input";
        input.value = original;
        input.maxLength = 200;
        nameCell.textContent = "";
        nameCell.appendChild(input);
        input.focus();
        input.select();

        function commit() {
            var val = input.value.trim();
            if (!val || val === original) {
                cleanup();
                nameCell.textContent = original;
                return;
            }
            /* Check for duplicate name */
            var exists = D.files.some(function (f) {
                return f.path !== path && (f.name || "").toLowerCase() === val.toLowerCase();
            });
            if (exists) {
                D.showToast(A.t("drive.rename-exists", "Já existe um item com esse nome"), "error");
                input.classList.add("gb-drv-input-error");
                input.select();
                return;
            }
            cleanup();
            doRename(path, val);
        }
        function cleanup() {
            input.removeEventListener("keydown", onKey);
            input.removeEventListener("blur", onBlur);
        }
        function onKey(e) {
            if (e.key === "Enter") { e.preventDefault(); commit(); }
            if (e.key === "Escape") { e.preventDefault(); cleanup(); nameCell.textContent = original; }
        }
        function onBlur() { commit(); }
        input.addEventListener("keydown", onKey);
        input.addEventListener("blur", onBlur);
    }

    function doRename(path, newName) {
        var parentPath = path.substring(0, path.lastIndexOf("/"));
        var newPath = parentPath ? parentPath + "/" + newName : newName;
        D.req("/move", {
            method: "POST",
            body: JSON.stringify({
                source_bucket: D.bucket || undefined,
                source_path: path,
                dest_bucket: D.bucket || undefined,
                dest_path: newPath,
                scope: D.scope
            })
        }).then(function () { D.refreshFiles(); })
          .catch(function (e) {
              D.showToast(A.t("drive.rename-error", "Erro ao renomear") + ": " + (e.message || e), "error");
              D.refreshFiles();
          });
    }

    /* ---- Download ---- */
    function downloadFile(path, name) {
        A.apiFetch(D.API + "/download-binary", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                bucket: D.bucket || undefined,
                path: path,
                scope: D.scope
            })
        }).then(function (r) {
            if (!r.ok) throw new Error("HTTP " + r.status);
            return r.blob();
        }).then(function (blob) {
            if (window.GBShared && typeof window.GBShared.saveBlob === "function") {
                window.GBShared.saveBlob(blob, name);
            } else {
                var url = URL.createObjectURL(blob);
                var a = document.createElement("a");
                a.href = url; a.download = name;
                document.body.appendChild(a); a.click(); a.remove();
                URL.revokeObjectURL(url);
            }
        }).catch(function (e) {
            D.showToast(A.t("drive.download-error", "Erro ao baixar") + ": " + (e.message || e), "error");
        });
    }

    /* ---- Delete (inline confirmation) ---- */
    function confirmDelete(path, name) {
        var row = document.querySelector('.gb-drv-row[data-path="' + CSS.escape(path) + '"]');
        if (!row) return;
        if (row.querySelector(".gb-drv-confirm")) return;
        var savedHtml = row.innerHTML;

        var bar = document.createElement("div");
        bar.className = "gb-drv-confirm";
        bar.innerHTML =
            '<span class="gb-drv-confirm-text">' +
                esc(A.t("drive.delete-confirm", "Excluir") + ' "' + name + '"?') +
            '</span>' +
            '<button class="gb-drv-confirm-btn gb-drv-confirm-danger" data-act="yes">' +
                esc(A.t("assistant.delete", "Excluir")) + '</button>' +
            '<button class="gb-drv-confirm-btn" data-act="no">' +
                esc(A.t("assistant.cancel", "Cancelar")) + '</button>';

        row.innerHTML = "";
        row.appendChild(bar);

        function restore() { row.innerHTML = savedHtml; D.bindOneRow(row); }

        bar.addEventListener("click", function (e) {
            e.stopPropagation();
            var act = e.target.closest("[data-act]");
            if (!act) return;
            if (act.getAttribute("data-act") === "no") { restore(); return; }
            act.disabled = true;
            act.textContent = "...";
            D.req("/trash/move", {
                method: "POST",
                body: JSON.stringify({
                    bucket: D.bucket || undefined,
                    path: path,
                    scope: D.scope
                })
            }).then(function () { D.refreshFiles(); })
              .catch(function (e) {
                  bar.innerHTML =
                      '<span class="gb-drv-confirm-text gb-drv-confirm-error">' +
                      esc(e.message || "Error") + '</span>' +
                      '<button class="gb-drv-confirm-btn" data-act="no">OK</button>';
                  bar.querySelector('[data-act="no"]').addEventListener("click", function (ev) {
                      ev.stopPropagation(); restore();
                  });
              });
        });

        var cancelBtn = bar.querySelector('[data-act="no"]');
        if (cancelBtn) cancelBtn.focus();
    }

    /* ---- Preview (reuses 13_attachpanel.js) ---- */
    D.previewFile = function (path, name) {
        /* Inbox files use the attachment preview endpoint which knows
           the bot's Drive layout. Regular files use download-inline. */
        var isInbox = D.inboxMode || (path && path.indexOf("inbox/") >= 0);
        if (isInbox && typeof A.makeAttachClickable === "function") {
            if (typeof A.closeAttachPanel === "function") A.closeAttachPanel();
            var ghost = document.createElement("div");
            ghost.style.display = "none";
            document.body.appendChild(ghost);
            A.makeAttachClickable(ghost, { name: name, size: 0 },
                { drivePath: path.indexOf("inbox/") >= 0 ? path : "inbox/" + name });
            ghost.click();
            ghost.remove();
            return;
        }
        A.apiFetch(D.API + "/download-inline", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                bucket: D.bucket || undefined,
                path: path,
                scope: D.scope
            })
        }).then(function (r) {
            if (!r.ok) throw new Error("HTTP " + r.status);
            return r.blob();
        }).then(function (blob) {
            var file;
            try { file = new File([blob], name, { type: blob.type }); }
            catch (e) { file = blob; file.name = name; }
            if (typeof A.closeAttachPanel === "function") A.closeAttachPanel();
            var ghost = document.createElement("div");
            ghost.style.display = "none";
            document.body.appendChild(ghost);
            A.makeAttachClickable(ghost, file, {});
            ghost.click();
            ghost.remove();
        }).catch(function (e) {
            D.showToast(A.t("drive.preview-error", "Erro ao visualizar") + ": " + (e.message || e), "error");
        });
    };

    /* ---- Use file in conversation (item 4) ---- */
    function useFileInChat(path, name) {
        var MAX_ATTACH = 50 * 1024 * 1024;
        D.showToast(A.t("drive.fetching", "Carregando arquivo..."));
        /* Inbox files live in the bot's Drive layout (resolved by the server
           via bot_drive_location). The generic /api/files/download-binary with
           scope "bot" does NOT resolve that layout, so inbox downloads must
           use the preview endpoint that does (/api/chat/attachments/preview).
           For binary types (PDF, images) it returns raw bytes; for text/office
           types it returns HTML, but the blob still works as a chat attachment
           since the server re-reads the inbox copy on send. User-scope files
           use the standard download-binary endpoint. */
        var isInbox = D.inboxMode || (path && path.indexOf("inbox/") >= 0);
        var fetchPromise;
        if (isInbox) {
            var inboxName = path.indexOf("inbox/") >= 0
                ? path.substring(path.indexOf("inbox/") + 6) : name;
            var previewUrl = "/api/chat/attachments/preview?download=1&bot=" +
                encodeURIComponent(A.botName || "default") +
                "&name=" + encodeURIComponent(inboxName);
            fetchPromise = A.apiFetch(previewUrl);
        } else {
            fetchPromise = A.apiFetch(D.API + "/download-binary", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    bucket: D.bucket || undefined,
                    path: path,
                    scope: D.scope
                })
            });
        }
        fetchPromise.then(function (r) {
            if (!r.ok) throw new Error("HTTP " + r.status);
            return r.blob();
        }).then(function (blob) {
            if (blob.size > MAX_ATTACH) {
                D.showToast(
                    A.t("attach.too-big", "Arquivo grande demais (limite de 50 MB por arquivo): ") + name,
                    "error"
                );
                return;
            }
            var file;
            try { file = new File([blob], name, { type: blob.type }); }
            catch (e) { file = blob; file.name = name; file.size = blob.size; }
            A.pendingFiles.push(file);
            /* Switch to chat and trigger chip rendering */
            A.switchView("chat");
            if (typeof A.initComposer === "function") {
                /* Re-render file chips in the composer */
                var container = document.getElementById("gbFileChips");
                if (container && typeof A.attachCard === "function") {
                    container.innerHTML = "";
                    for (var i = 0; i < A.pendingFiles.length; i++) {
                        (function (idx) {
                            var card = A.attachCard(A.pendingFiles[idx], {
                                removable: true,
                                onRemove: function () {
                                    A.pendingFiles.splice(idx, 1);
                                    useFileInChat._renderChips();
                                }
                            });
                            container.appendChild(card);
                        })(i);
                    }
                }
            }
            D.showToast(A.t("drive.attached", "Arquivo anexado à conversa"));
            /* Focus the input */
            var inp = document.getElementById("gbInputDock") || document.getElementById("gbInputEmpty");
            if (inp) inp.focus();
        }).catch(function (e) {
            D.showToast(A.t("drive.download-error", "Erro ao baixar") + ": " + (e.message || e), "error");
        });
    }
    useFileInChat._renderChips = function () {
        var container = document.getElementById("gbFileChips");
        if (!container || typeof A.attachCard !== "function") return;
        container.innerHTML = "";
        for (var i = 0; i < A.pendingFiles.length; i++) {
            (function (idx) {
                var card = A.attachCard(A.pendingFiles[idx], {
                    removable: true,
                    onRemove: function () {
                        A.pendingFiles.splice(idx, 1);
                        useFileInChat._renderChips();
                    }
                });
                container.appendChild(card);
            })(i);
        }
    };
    D.useFileInChat = useFileInChat;

    /* ---- Inbox files (chat attachments from conversation history) ---- */
    D.collectInboxFiles = function () {
        return A.apiFetch("/api/chat/history/sessions?limit=50&bot_name=" +
            encodeURIComponent(A.botName || "default"))
        .then(function (r) { if (!r.ok) throw new Error("HTTP " + r.status); return r.json(); })
        .then(function (data) {
            var seen = {}, allFiles = [];
            return Promise.all((data.sessions || []).slice(0, 20).map(function (s) {
                var sid = s.session_id || s.id;
                return A.apiFetch("/api/chat/history/sessions/" + encodeURIComponent(sid) +
                    "/messages?limit=100&offset=0")
                .then(function (r) { return r.ok ? r.json() : { messages: [] }; })
                .then(function (d) {
                    (d.messages || []).forEach(function (m) {
                        if (m.role !== "user" || !m.attachments) return;
                        m.attachments.forEach(function (att) {
                            var key = att.drive_path || att.name;
                            if (!key || seen[key]) return;
                            seen[key] = true;
                            allFiles.push({ name: att.name,
                                path: att.drive_path || ("inbox/" + att.name),
                                size: att.size || 0, modified: s.updated_at,
                                is_dir: false, _inbox: true });
                        });
                    });
                }).catch(function () { return null; });
            })).then(function () { return allFiles; });
        });
    };

    D.loadInboxFiles = function () {
        D.inboxMode = true;
        var request = D.listRequest = (D.listRequest || 0) + 1;
        D.showLoading();
        D.collectInboxFiles().then(function (files) {
            if (request !== D.listRequest) return;
            D.files = D.sortFiles(files);
            D.renderFileList();
        }).catch(function (e) {
            if (request !== D.listRequest) return;
            console.error("[drive] inbox list failed:", e);
            D.showError(e.message);
        });
    };

    /* ---- Search ---- */
    D.searchFiles = function (query) {
        D.searching = true;
        var request = D.listRequest = (D.listRequest || 0) + 1;
        D.showLoading();
        var params = new URLSearchParams();
        params.set("query", query);
        params.set("scope", D.scope);
        params.set("all_buckets", "false");
        if (D.bucket) params.set("bucket", D.bucket);

        D.req("/search?" + params.toString())
            .then(function (files) {
                if (request !== D.listRequest) return;
                D.files = D.sortFiles(files || []);
                D.renderFileList();
            })
            .catch(function (e) {
                if (request !== D.listRequest) return;
                console.error("[drive] search failed:", e);
                D.showError(e.message);
            });
    };
})();
