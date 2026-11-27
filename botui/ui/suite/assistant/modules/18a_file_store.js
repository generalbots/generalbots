"use strict";
/**
 * 18a_file_store.js - Where the files the assistant made live, and what can
 * be done with them.
 *
 * Two places hold them:
 * - the conversation's workspace on the Virtual Machine, read through
 *   GET /api/agent/vm/file?conversation_id=&path=output/<name>&raw=1 (bytes)
 *   or &preview=1 (office packages rendered as script-free HTML);
 * - the Drive copy, `<bot>.gbdrive/conversations/<folder>/<name>`, listed by
 *   GET /api/agent/drive/conversations (each folder with its conversation id,
 *   title and files) and read with GET /api/agent/drive/file?path=<relative
 *   to .gbdrive>&raw=1 or &preview=1. A server without those endpoints (404)
 *   is read the older way: GET /api/files/list?scope=bot&path= and
 *   POST /api/files/download-binary, with the folder matched to a
 *   conversation by its name (title slug, or `<slug>-<first 8 characters of
 *   the conversation id>`).
 *
 * Saving to Drive is POST /api/agent/drive/save { conversation_id, files }.
 * The panel (18_file_panel.js) and the Drive tab (06d) both use this.
 */
(function () {
    var A = window.GBAssistant;
    var F = A.files = A.files || {};

    function t(pt, en) { return A.locale && A.locale.indexOf("pt") === 0 ? pt : en; }
    function gt(key, en) { return window.gbT ? window.gbT(key, en) : en; }
    F.t = t;

    var KINDS = {
        pdf: "pdf", png: "image", jpg: "image", jpeg: "image", gif: "image", webp: "image", bmp: "image",
        txt: "text", log: "text", json: "text", xml: "text", yaml: "text", yml: "text", py: "text", html: "text",
        md: "markdown", markdown: "markdown", csv: "csv", tsv: "csv",
        docx: "office", xlsx: "office", pptx: "office", odt: "office", ods: "office", odp: "office"
    };
    var TYPES = {
        pdf: ["Documento PDF", "PDF document"], docx: ["Documento de texto (DOCX)", "Text document (DOCX)"],
        xlsx: [gt("file-kind-spreadsheet-xlsx", "Spreadsheet (XLSX)"), "Spreadsheet (XLSX)"], pptx: [gt("file-kind-presentation-pptx", "Presentation (PPTX)"), "Presentation (PPTX)"],
        odt: [gt("file-kind-text-document-odt", "Text document (ODT)"), "Text document (ODT)"], ods: [gt("file-kind-spreadsheet-ods", "Spreadsheet (ODS)"), "Spreadsheet (ODS)"],
        odp: ["Apresentação (ODP)", "Presentation (ODP)"], csv: ["Tabela (CSV)", "Table (CSV)"],
        tsv: ["Tabela (TSV)", "Table (TSV)"], md: ["Texto formatado (Markdown)", "Formatted text (Markdown)"],
        txt: [gt("file-kind-text", "Text"), "Text"], json: [gt("file-kind-data-json", "Data (JSON)"), "Data (JSON)"], zip: [gt("file-kind-archive", "Archive"), "Archive"]
    };

    F.ext = function (name) {
        var m = /\.([a-z0-9]+)$/i.exec(String(name || ""));
        return m ? m[1].toLowerCase() : "";
    };
    F.kind = function (name) { return KINDS[F.ext(name)] || "other"; };
    F.typeName = function (name) {
        var e = F.ext(name), known = TYPES[e];
        if (known) { return t(known[0], known[1]); }
        if (F.kind(name) === "image") { return t("Imagem", "Image") + " (" + e.toUpperCase() + ")"; }
        return e ? t("Arquivo", "File") + " " + e.toUpperCase() : t("Arquivo", "File");
    };
    F.fmtBytes = function (b) {
        if (!b && b !== 0) { return ""; }
        if (b < 1024) { return b + " B"; }
        var units = ["KB", "MB", "GB"], v = b / 1024, i = 0;
        while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
        return gbFormatNumber(v, { maximumFractionDigits: v < 10 ? 1 : 0 }, A.locale || undefined) + " " + units[i];
    };

    function httpError(status) {
        var e = new Error("HTTP " + status);
        e.status = status;
        return e;
    }

    /* `vmfile:<conversation>/output/<name>`; a link without a conversation
       (`vmfile:output/<name>`) belongs to the open one. */
    F.parseLink = function (href) {
        var value = String(href || "").replace(/^vmfile:/i, "");
        try { value = decodeURIComponent(value); } catch (e) { /* keep as sent */ }
        var slash = value.indexOf("/");
        var head = slash === -1 ? "" : value.slice(0, slash);
        var local = !head || head === "output" || head === "program" || head === "data";
        var path = local ? value : value.slice(slash + 1);
        return {
            kind: "vm",
            conversationId: local ? String(A.currentSessionId || A.sessionId || "") : head,
            path: path,
            name: path.split("/").pop()
        };
    };

    F.vmUrl = function (src, mode) {
        return "/api/agent/vm/file?conversation_id=" + encodeURIComponent(src.conversationId) +
            "&path=" + encodeURIComponent(src.path) + "&" + mode + "=1";
    };

    F.driveFileUrl = function (item, mode) {
        return "/api/agent/drive/file?path=" + encodeURIComponent(item.path) + "&" + mode + "=1";
    };

    function driveBlob(item) {
        if (item.api) {
            return A.apiFetch(F.driveFileUrl(item, "raw")).then(function (r) {
                if (!r.ok) { throw httpError(r.status); }
                return r.blob();
            });
        }
        return A.apiFetch("/api/files/download-binary", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ path: item.path, scope: "bot", bucket: item.bucket || undefined })
        }).then(function (r) {
            if (!r.ok) { throw httpError(r.status); }
            return r.blob();
        });
    }

    /* The bytes: the workspace first; when the Virtual Machine is off (409)
       or no longer has it (404), the Drive copy. */
    F.fetchBlob = function (src) {
        if (src.kind === "drive") { return driveBlob(src); }
        return A.apiFetch(F.vmUrl(src, "raw")).then(function (r) {
            if (r.ok) { return r.blob(); }
            var failed = httpError(r.status);
            return F.findDriveCopy(src).then(function (copy) {
                if (!copy) { throw failed; }
                src.driveCopy = copy;
                return driveBlob(copy);
            });
        });
    };

    /* Office packages preview from the workspace, or from a Drive copy the
       server can render; the older Drive read has no preview. */
    function previewItem(src) { return src.kind === "drive" ? src : src.driveCopy; }
    F.canPreviewOffice = function (src) {
        var item = previewItem(src);
        return item ? !!item.api : src.kind === "vm";
    };
    F.fetchOfficeHtml = function (src) {
        if (!F.canPreviewOffice(src)) { return Promise.reject(httpError(415)); }
        var item = previewItem(src);
        return A.apiFetch(item ? F.driveFileUrl(item, "preview") : F.vmUrl(src, "preview")).then(function (r) {
            if (!r.ok) { throw httpError(r.status); }
            return r.text();
        });
    };

    /* ---- Drive copies ---- */
    F.driveRoot = function () { return (A.botName || "default") + ".gbdrive/conversations"; };

    function list(path) {
        return A.apiFetch("/api/files/list?scope=bot&path=" + encodeURIComponent(path)).then(function (r) {
            if (!r.ok) { throw httpError(r.status); }
            return r.json();
        }).then(function (items) { return Array.isArray(items) ? items : []; });
    }

    /* Same rule as the server's folder name: ASCII letters and digits kept,
       separators folded into one dash, at most 60 characters. */
    F.slug = function (text) {
        var out = "";
        String(text || "").trim().split("").some(function (ch) {
            if (/[A-Za-z0-9]/.test(ch)) { out += ch; }
            else if (" -_.:()".indexOf(ch) !== -1 && out.slice(-1) !== "-") { out += "-"; }
            return out.length >= 60;
        });
        return out.replace(/^-+|-+$/g, "");
    };

    /* Which conversation a folder belongs to: `<slug>-<id8>` and
       `conversation-<id8>` name it exactly; a bare slug matches a title. */
    F.folderSession = function (folder) {
        var name = String(folder || "");
        var sessions = A.sessions || [];
        var m = /(?:^|-)([0-9a-f]{8})$/i.exec(name);
        if (m) {
            var id8 = m[1].toLowerCase();
            var byId = sessions.filter(function (s) { return String(s.id || "").toLowerCase().indexOf(id8) === 0; })[0];
            if (byId) { return byId; }
        }
        return sessions.filter(function (s) {
            var slug = F.slug(s.title || "");
            return slug && slug.toLowerCase() === name.toLowerCase();
        })[0] || null;
    };

    F.folderMatches = function (folder, conversationId) {
        var id = String(conversationId || "").toLowerCase();
        if (!id) { return false; }
        var m = /(?:^|-)([0-9a-f]{8})$/i.exec(String(folder || ""));
        if (m) { return id.indexOf(m[1].toLowerCase()) === 0; }
        var s = F.folderSession(folder);
        return !!s && String(s.id).toLowerCase() === id;
    };

    var cache = null;
    F.invalidateFolders = function () { cache = null; };

    /* The server's own list; null when this server does not have it. */
    function serverFolders() {
        return A.apiFetch("/api/agent/drive/conversations").then(function (r) {
            if (r.status === 404 || r.status === 405) { return null; }
            if (!r.ok) { throw httpError(r.status); }
            return r.json().then(function (data) { return data && Array.isArray(data.folders) ? data.folders : null; }, function () { return null; });
        }).then(function (folders) {
            return folders && folders.map(function (g) {
                return { folder: String(g.folder || ""), conversationId: String(g.conversation_id || ""), title: String(g.title || ""),
                    files: (g.files || []).map(function (f) {
                        return { name: f.name, path: f.path, size: f.size, modified: f.modified, api: true };
                    }) };
            });
        });
    }

    function sessionById(id) {
        var key = String(id || "").toLowerCase();
        return key ? (A.sessions || []).filter(function (s) { return String(s.id || "").toLowerCase() === key; })[0] || null : null;
    }

    function listedFolders() {
        return list(F.driveRoot()).then(function (items) {
            var dirs = items.filter(function (i) { return i.is_dir; }).slice(0, 80);
            return Promise.all(dirs.map(function (d) {
                return list(d.path).then(function (files) {
                    return { folder: d.name, path: d.path, files: files.filter(function (f) { return !f.is_dir; }) };
                }).catch(function () { return { folder: d.name, path: d.path, files: [] }; });
            }));
        });
    }

    /* Every conversation folder with its files, read at most every 30 s. */
    F.conversationFolders = function (force) {
        if (!force && cache && Date.now() - cache.at < 30000) { return cache.promise; }
        var promise = serverFolders().then(function (groups) { return groups || listedFolders(); }).then(function (groups) {
            return groups.filter(function (g) { return g.files.length; }).map(function (g) {
                g.session = sessionById(g.conversationId) || F.folderSession(g.folder);
                return g;
            });
        });
        cache = { at: Date.now(), promise: promise };
        promise.catch(function () { cache = null; });
        return promise;
    };

    /* The Drive copy of a workspace file: same name, in this conversation's
       folder when one is known, otherwise the only folder holding that name
       whose conversation the server did not name. */
    F.findDriveCopy = function (src) {
        return F.conversationFolders().then(function (groups) {
            var holding = groups.map(function (g) {
                var file = g.files.filter(function (f) { return f.name === src.name; })[0];
                return file ? { group: g, file: file } : null;
            }).filter(Boolean);
            var own = holding.filter(function (h) {
                var id = h.group.conversationId;
                return id ? id.toLowerCase() === String(src.conversationId || "").toLowerCase() : F.folderMatches(h.group.folder, src.conversationId);
            })[0];
            var unnamed = holding.length === 1 && !holding[0].group.conversationId;
            var pick = own || (unnamed ? holding[0] : null);
            return pick ? { kind: "drive", api: !!pick.file.api, path: pick.file.path, name: pick.file.name, size: pick.file.size, folder: pick.group.folder } : null;
        }).catch(function () { return null; });
    };

    F.canSaveToDrive = function () {
        return (A.compiledFeatures || []).indexOf("agent-vm") !== -1;
    };

    F.saveToDrive = function (src) {
        return A.apiFetch("/api/agent/drive/save", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ conversation_id: src.conversationId, files: [src.name] })
        }).then(function (r) {
            if (!r.ok) { throw httpError(r.status); }
            return r.json();
        }).then(function (data) {
            F.invalidateFolders();
            return data || {};
        });
    };

    /* ---- Leaving the app: the person's own Save dialog and programs ---- */
    function tauri() {
        var T = window.__TAURI_INTERNALS__;
        return T && typeof T.invoke === "function" ? T : null;
    }
    F.isDesktop = function () { return !!tauri(); };

    function anchorSave(blob, name) {
        var url = URL.createObjectURL(blob);
        var link = document.createElement("a");
        link.href = url;
        link.download = name || "";
        document.body.appendChild(link);
        link.click();
        link.remove();
        setTimeout(function () { URL.revokeObjectURL(url); }, 15000);
        return Promise.resolve(null);
    }

    /* Resolves with the saved path (desktop), null when cancelled or in the
       browser build. */
    F.saveBlob = function (blob, name) {
        var T = tauri();
        if (!T) { return anchorSave(blob, name); }
        return blob.arrayBuffer().then(function (buffer) {
            return T.invoke("save_download", { fileName: name, url: null, bytes: Array.from(new Uint8Array(buffer)) });
        });
    };

    /* Opens the file with the program the computer uses for that type:
       the desktop's open_download command when it has one (resolves with the
       path it used, or true), otherwise a copy saved where the person
       chooses and then opened. */
    F.openOutside = function (blob, name) {
        var T = tauri();
        if (!T) { return anchorSave(blob, name); }
        return blob.arrayBuffer().then(function (buffer) {
            return T.invoke("open_download", { fileName: name, bytes: Array.from(new Uint8Array(buffer)) });
        }).then(function (path) { return path || true; }, function (err) {
            if (!/not found|unknown command/i.test(String((err && err.message) || err))) { throw err; }
            return saveAndOpen(T, blob, name);
        });
    };

    function saveAndOpen(T, blob, name) {
        return F.saveBlob(blob, name).then(function (saved) {
            if (!saved) { return null; }
            return T.invoke("plugin:opener|open_path", { path: saved }).then(function () { return saved; }, function () {
                var e = new Error("open_path");
                e.savedTo = saved;
                throw e;
            });
        });
    }
})();
