/* Load sidebar dynamically from JSON API and render as HTML */
function _t(key, fallback, params) {
    if (window.SheetI18n && window.SheetI18n.t) return window.SheetI18n.t(key, fallback, params);
    var text = fallback || key;
    if (params) {
        Object.keys(params).forEach(function (p) { text = text.split('{' + p + '}').join(params[p]); });
    }
    return text;
}
function _sheetCtx(arg) {
    var D = window.SheetDefaults || {};
    var ev = (typeof event !== 'undefined') ? event : null;
    var node = arg;
    if (node && node.target) node = node.target;
    if (!node || !node.closest) node = ev && ev.target;
    if (!node || !node.closest) node = document.activeElement;
    var body = node && node.closest ? node.closest('[id^="window-body-"]') : null;
    if (!body && arg && arg.windowBody) body = arg.windowBody;
    if (!body && arg && arg.body) body = arg.body;
    var winId = body ? (body.dataset.windowId || body.id.replace('window-body-', '')) : null;
    var inst = winId && D.getSheetInstance ? D.getSheetInstance(winId) : (D.getActiveSheetInstance ? D.getActiveSheetInstance() : null);
    if (!body && inst) body = inst.body;
    if (inst && D.setActiveSheetInstance) D.setActiveSheetInstance(inst);
    return { body: body, inst: inst, winId: winId || (inst && inst.windowId) || null };
}
function _q(ctx, sel) { return ctx && ctx.body ? ctx.body.querySelector(sel) : null; }
/* A1 reference ("B7", "$B$7") → zero-based {row, col}. Deep links carry the
   cell the bot is talking about, while the grid API addresses cells by index. */
function gbSheetParseA1(ref) {
    var m = /^\$?([A-Za-z]+)\$?(\d+)$/.exec(String(ref || '').trim());
    if (!m) return null;
    var letters = m[1].toUpperCase();
    var col = 0;
    for (var i = 0; i < letters.length; i++) {
        col = col * 26 + (letters.charCodeAt(i) - 64);
    }
    var row = parseInt(m[2], 10);
    if (!row || col < 1) return null;
    return { row: row - 1, col: col - 1 };
}
/* Select and scroll to a zero-based grid position. */
function gbSheetApplyCell(pos, ctx) {
    var inst = ctx && (ctx.inst || ctx.grid ? ctx : null);
    if (ctx && ctx.instance) inst = ctx.instance;
    var grid = (inst && inst.grid) || (ctx && ctx.grid) || (window.SheetDefaults && window.SheetDefaults.getActiveSheetInstance && (window.SheetDefaults.getActiveSheetInstance() || {}).grid) || window.SheetVirtualGrid;
    if (!grid || typeof grid.selectCell !== 'function') return false;
    grid.selectCell(pos.row, pos.col);
    return true;
}
/* Select and scroll to an A1 reference after a workbook load (#2271). */
function gbSheetGotoCell(ref, ctx) {
    var pos = gbSheetParseA1(ref);
    if (!pos) {
        console.warn('[sheet] ignoring malformed cell reference:', ref);
        return false;
    }
    var inst = ctx && (ctx.inst || ctx);
    if (inst && inst.windowId && window.__SHEET_INSTANCES__ && window.__SHEET_INSTANCES__[inst.windowId]) inst = window.__SHEET_INSTANCES__[inst.windowId];
    if (inst && inst.grid) {
        inst.pendingCell = ref;
        return gbSheetApplyCell(pos, inst);
    }
    window.__SHEET_PENDING_CELL = ref;
    return gbSheetApplyCell(pos, ctx);
}
/* Runs when the grid's deferred default selection fires: a requested cell wins
   over the default A1, and the request is cleared either way. */
function gbSheetConsumePendingCell() {
    var ref = window.__SHEET_PENDING_CELL;
    if (!ref) return false;
    window.__SHEET_PENDING_CELL = null;
    var pos = gbSheetParseA1(ref);
    return pos ? gbSheetApplyCell(pos) : false;
}
function newSheet(arg) {
    var ctx = _sheetCtx(arg);
    fetch('/api/sheet/new').then(function(r){ return r.json(); }).then(function(s){
        if (ctx.inst) {
            ctx.inst.loadedSheet = s;
            ctx.inst.sheetId = s.id;
            if (window.SheetDefaults) window.SheetDefaults.setActiveSheetInstance(ctx.inst);
        }
        window.__LOADED_SHEET = s;
        window.__SHEET_INITIAL_ID = s.id;
        var host = _q(ctx, '#sheet-content') || document.getElementById('sheet-content');
        if (!host) return;
        host.innerHTML = '';
        var VG = (ctx.inst && ctx.inst.grid) || window.SheetVirtualGrid;
        if (VG) {
            VG.cells = new Map();
            VG.pool = [];
            VG.usedCount = 0;
            VG.requestSeq = 0;
            VG.lastRenderedRange = null;
            VG.selectedRow = 0;
            VG.selectedCol = 0;
            VG.editingCell = null;
            VG.selectionOverlay = null;
            VG.headerColPool = [];
            VG.init(host);
            if (window.SheetAdvanced && window.SheetAdvanced.sync) window.SheetAdvanced.sync();
            if (window.SheetCore && window.SheetCore.refreshWidths) window.SheetCore.refreshWidths();
        }
    });
}
function loadSidebar(arg) {
    var ctx = _sheetCtx(arg);
    var el = _q(ctx, '#sidebar-files') || document.getElementById('sidebar-files');
    if (!el) return;
    fetch('/api/sheet/list').then(function(r){ return r.json(); }).then(function(items){
        if (!items || items.length === 0) {
            el.innerHTML = '<div class="ss-empty-hint" style="padding:24px;text-align:center;color:var(--sheet-muted, #94a3b8);font-style:italic;">' + _t('sheet-none', 'No spreadsheets yet. Create one to start.') + '</div>';
            return;
        }
        var html = '<div class="ss-sidebar"><div class="ss-sidebar-header" style="display:flex;justify-content:space-between;align-items:center;padding:12px;border-bottom:1px solid var(--sheet-border, #334155);"><h3 style="margin:0;color:var(--sheet-fg, #f8fafc);font-size:14px;">' + _t('sheet-my-sheets', 'My Sheets') + '</h3><button class="btn-icon" onclick="newSheet()" style="background:#10b981;color:white;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">' + _t('sheet-new', '+ New') + '</button></div><div class="ss-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;max-height:calc(100vh - 200px);overflow-y:auto;">';
        for (var i = 0; i < items.length; i++) {
            var m = items[i];
            html += '<div class="ss-metadata-card" onclick="loadSheet(\'' + m.id + '\')" style="padding:12px;border:1px solid var(--sheet-border, #334155);border-radius:6px;background:var(--sheet-bg-elevated, #1e293b);cursor:pointer;"><div style="font-weight:600;color:var(--sheet-fg, #f8fafc);">' + escHtml(m.name) + '</div><div style="font-size:12px;color:var(--sheet-muted, #94a3b8);margin-top:4px;">' + _t('sheet-sheets-count', '{0} sheet(s)', { count: m.worksheet_count || 0 }) + '</div></div>';
        }
        html += '</div></div>';
        el.innerHTML = html;
    }).catch(function(){
        el.innerHTML = '<div style="padding:24px;text-align:center;color:var(--sheet-muted, #94a3b8);font-style:italic;">' + _t('sheet-sidebar-load-error', 'Error loading sheets.') + '</div>';
    });
}
function escHtml(s) { return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;'); }
function openSheetPanel(panel) {
    var ctx = _sheetCtx();
    var sheet = (ctx.inst && ctx.inst.loadedSheet) || window.__LOADED_SHEET;
    if (!sheet) {
        var host = _q(ctx, '#sheet-content') || document.getElementById('sheet-content');
        if (host) host.innerHTML = '<div style="padding:20px;color:#f87171;text-align:center;">' + _t('sheet-sidebar-load-first', 'Load a spreadsheet first.') + '</div>';
        return;
    }
    var panelHost = _q(ctx, '#sheet-content') || document.getElementById('sheet-content');
    if (!panelHost) return;
    if (window.GBSheetRefreshPanel) {
        window.GBSheetRefreshPanel(panel).catch(function () {
            panelHost.innerHTML = '<div style="padding:20px;color:#f87171;text-align:center;">' + _t('sheet-panel-load-error', 'Error loading panel.') + '</div>';
        });
        return;
    }
    fetch('/suite/sheet/fragments/' + panel, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(sheet)
    }).then(function(r){ return r.text(); }).then(function(html){ panelHost.innerHTML = html; })
    .catch(function(){ panelHost.innerHTML = '<div style="padding:20px;color:#f87171;text-align:center;">' + _t('sheet-panel-load-error', 'Error loading panel.') + '</div>'; });
}
function loadSheet(id, cell) {
    var ctx = _sheetCtx();
    if (ctx.inst) {
        ctx.inst.sheetId = id;
        ctx.inst.worksheetIndex = 0;
        if (window.SheetDefaults) window.SheetDefaults.setActiveSheetInstance(ctx.inst);
    }
    window.__SHEET_INITIAL_ID = id;
    try { sessionStorage.setItem("sheet_active_index", "0"); } catch (_) {}
    document.dispatchEvent(new CustomEvent("gb-sheet-tab", { detail: { index: 0, windowId: ctx.winId }, bubbles: true }));
    var host = _q(ctx, '#sheet-content') || document.getElementById('sheet-content');
    if (host) host.innerHTML = '<div style="padding:20px;color:var(--sheet-muted, #94a3b8);text-align:center;">' + _t('sheet-loading', 'Loading...') + '</div>';
    fetch('/api/sheet/load?id=' + encodeURIComponent(id)).then(function(r){ return r.json(); }).then(function(sheet){
        if (ctx.inst) { ctx.inst.loadedSheet = sheet; ctx.inst.sheetId = sheet.id || id; }
        window.__LOADED_SHEET = sheet;
        var titleInput = _q(ctx, '#sheetName') || document.getElementById('sheetName');
        if (titleInput && sheet && sheet.name) titleInput.value = sheet.name;
        if (!host) return;
        host.innerHTML = '';
        var VG = (ctx.inst && ctx.inst.grid) || window.SheetVirtualGrid;
        if (VG) {
            VG.cells = new Map();
            VG.pool = [];
            VG.usedCount = 0;
            VG.requestSeq = 0;
            VG.lastRenderedRange = null;
            VG.selectedRow = 0;
            VG.selectedCol = 0;
            VG.editingCell = null;
            VG.selectionOverlay = null;
            VG.headerColPool = [];
            VG.init(host);
            if (window.SheetAdvanced && window.SheetAdvanced.sync) window.SheetAdvanced.sync();
            if (window.SheetCore && window.SheetCore.refreshWidths) window.SheetCore.refreshWidths();
            var loaded = (ctx.inst && ctx.inst.loadedSheet) || window.__LOADED_SHEET;
            if (loaded && loaded.worksheets && loaded.worksheets.length > 0) {
                var ws = loaded.worksheets[0];
                if (ws.data) {
                    for (var cellRef in ws.data) { VG.cells.set(cellRef, ws.data[cellRef]); }
                }
                VG.requestSeq++;
                VG.totalRows = Math.max(VG.totalRows, 50);
                VG.render();
                if (window.SheetAdvanced && window.SheetAdvanced.sync) window.SheetAdvanced.sync();
                document.dispatchEvent(new CustomEvent("gb-sheet-tab", { detail: { index: 0, windowId: ctx.winId }, bubbles: true }));
            }
            if (cell) gbSheetGotoCell(cell, ctx.inst || VG);
        }
    }).catch(function(){
        if (host) host.innerHTML = '<div style="padding:20px;color:#f87171;text-align:center;">Error loading sheet.</div>';
    });
}
function getLiveSheetId(arg) {
    var ctx = _sheetCtx(arg);
    if (ctx.inst) {
        return (ctx.inst.loadedSheet && ctx.inst.loadedSheet.id) || ctx.inst.sheetId || null;
    }
    return (window.__LOADED_SHEET && window.__LOADED_SHEET.id) || window.__SHEET_INITIAL_ID || null;
}
// Toolbar modals hardcode id="current"; rewrite it to the live sheet id once
// per HTMX request so the modal/form targets the open workbook (#2270).
document.addEventListener("htmx:configRequest", function (event) {
    var parameters = (event.detail && event.detail.parameters) || {};
    var ctx = _sheetCtx(event.detail && event.detail.elt);
    var live = getLiveSheetId(ctx);
    if (live && live !== "current") {
        ["id", "sheet_id"].forEach(function (key) {
            if (parameters[key] === "current") parameters[key] = live;
        });
    }
    if (ctx.inst && typeof ctx.inst.worksheetIndex === "number") {
        parameters.worksheet_index = ctx.inst.worksheetIndex;
    } else if (window.SheetCore && typeof window.SheetCore.wsIndex === "function") {
        parameters.worksheet_index = window.SheetCore.wsIndex();
    } else {
        var tabs = _q(ctx, "#worksheet-tabs") || document.getElementById("worksheet-tabs");
        if (tabs && tabs.dataset.activeIndex !== undefined) {
            parameters.worksheet_index = parseInt(tabs.dataset.activeIndex, 10) || 0;
        }
    }
});
// Sheet AI answers are JSON; render the text in #ai-response in that window.
document.addEventListener("htmx:afterRequest", function (event) {
    var detail = event.detail || {};
    var path = (detail.pathInfo && detail.pathInfo.requestPath) || "";
    if (path !== "/api/sheet/ai" || !detail.successful) return;
    var text = "";
    try {
        var data = JSON.parse(detail.xhr.responseText);
        text = data.response || data.result || "";
    } catch (error) {
        text = detail.xhr.responseText;
    }
    var ctx = _sheetCtx(detail.elt);
    var target = _q(ctx, "#ai-response") || document.getElementById("ai-response");
    if (target) target.textContent = text;
});

document.addEventListener("htmx:afterRequest", function (event) {
    var detail = event.detail || {};
    if (!detail.successful) return;
    var path = (detail.pathInfo && detail.pathInfo.requestPath) || "";
    if (path.indexOf("/api/sheet/") !== 0) return;
    if (path === "/api/sheet/load" || path === "/api/sheet/load-from-drive") return;
    var ctx = _sheetCtx(detail.elt);
    var live = getLiveSheetId(ctx);
    if (live && typeof loadSheet === "function") {
        // preserve context by setting active before reloading this window
        if (ctx.inst && window.SheetDefaults) window.SheetDefaults.setActiveSheetInstance(ctx.inst);
        loadSheet(live);
    }
});
function GBSheetRefreshNamedRanges(sheetId) {
    var ctx = _sheetCtx();
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    fetch("/suite/sheet/fragments/named-ranges", {
        method: "POST",
        headers: headers,
        body: JSON.stringify({ id: sheetId || getLiveSheetId(ctx) }),
    })
        .then(function (response) { return response.text(); })
        .then(function (html) {
            var current = _q(ctx, "#named-ranges-panel") || document.getElementById("named-ranges-panel");
            if (!current) return;
            var holder = document.createElement("div");
            holder.innerHTML = html.trim();
            var next = holder.firstElementChild;
            if (next) current.replaceWith(next);
        })
        .catch(function (err) {
            console.debug("[gb-sheet] Named ranges refresh failed:", err);
        });
}
window.GBSheetRefreshNamedRanges = GBSheetRefreshNamedRanges;
function GBSheetCreateNamedRange(sheetId) {
    var ctx = _sheetCtx();
    var name = (window.GBModal||{prompt:prompt}).prompt("Nome do intervalo:", "");
    if (!name || !name.trim()) return;
    var sel = window.SheetAdvanced && window.SheetAdvanced.getSelection
        ? window.SheetAdvanced.getSelection() : null;
    if (!sel) {
        sheetNotify("Selecione as células do intervalo primeiro.", true);
        return;
    }
    var inst = ctx.inst || (window.SheetDefaults && window.SheetDefaults.getActiveSheetInstance ? window.SheetDefaults.getActiveSheetInstance() : null);
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    fetch("/api/sheet/named-range", {
        method: "POST",
        headers: headers,
        body: JSON.stringify({
            sheet_id: sheetId || getLiveSheetId(ctx),
            name: name.trim(),
            scope: "workbook",
            worksheet_index: inst && typeof inst.worksheetIndex === "number" ? inst.worksheetIndex : (window.SheetCore ? window.SheetCore.wsIndex() : 0),
            start_row: sel.startRow,
            start_col: sel.startCol,
            end_row: sel.endRow,
            end_col: sel.endCol,
        }),
    })
        .then(function (response) {
            if (!response.ok) throw new Error("HTTP " + response.status);
            GBSheetRefreshNamedRanges(sheetId || getLiveSheetId(ctx));
        })
        .catch(function (error) {
            sheetNotify("Could not create the named range: " + error.message, true);
        });
}
window.GBSheetCreateNamedRange = GBSheetCreateNamedRange;
function openSheetComments() {
    if (!window.GBCollabComments) return;
    var ctx = _sheetCtx();
    var id = getLiveSheetId(ctx);
    if (!id || id === 'current') {
        alert('Save the spreadsheet first, then open comments.');
        return;
    }
    window.GBCollabComments.open({
        resourceType: 'sheet',
        resourceId: String(id),
        title: 'Spreadsheet comments',
        includeChildren: true,
    });
}
function renameSheet(arg) {
    var ctx = _sheetCtx(arg);
    var nameInput = _q(ctx, '#sheetName') || document.getElementById('sheetName');
    if (!nameInput) return;
    var name = nameInput.value.trim();
    var loaded = (ctx.inst && ctx.inst.loadedSheet) || window.__LOADED_SHEET;
    if (!name) { nameInput.value = loaded ? loaded.name : 'Untitled Spreadsheet'; return; }
    var id = getLiveSheetId(ctx);
    if (!id || id === 'current') {
        if (ctx.inst && ctx.inst.loadedSheet) ctx.inst.loadedSheet.name = name;
        if (window.__LOADED_SHEET) window.__LOADED_SHEET.name = name;
        return;
    }
    fetch('/api/sheet/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: id, name: name, action: 'rename' })
    }).then(function(r) {
        return r.json().then(function(j) {
            if (!r.ok) throw new Error((j && j.error) || 'rename failed');
            return j;
        });
    }).then(function() {
        if (ctx.inst && ctx.inst.loadedSheet) ctx.inst.loadedSheet.name = name;
        if (window.__LOADED_SHEET) window.__LOADED_SHEET.name = name;
        sheetNotify('Renamed');
    }).catch(function(err) {
        sheetNotify('Rename failed: ' + (err && err.message ? err.message : 'unknown error'), true);
    });
}

function sheetNotify(message, isError) {
    var notif = document.createElement('div');
    notif.style.cssText = 'position:fixed;bottom:20px;right:20px;background:' +
        (isError ? '#ef4444' : '#10b981') +
        ';color:white;padding:10px 20px;border-radius:6px;font-size:13px;z-index:9999;box-shadow:0 4px 12px rgba(0,0,0,0.3);';
    notif.textContent = message;
    document.body.appendChild(notif);
    setTimeout(function() { notif.remove(); }, isError ? 5000 : 2000);
}
function gbSheetShowLoadError(host, info) {
    info = info || {};
    if (!host) return;
    host.innerHTML = '';
    var box = document.createElement('div');
    box.className = 'gb-sheet-load-error';
    box.style.cssText = 'padding:24px;text-align:center;color:#f87171;';
    var title = document.createElement('div');
    title.style.cssText = 'font-weight:600;margin-bottom:8px;';
    title.textContent = info.message || 'The spreadsheet could not be loaded.';
    var hint = document.createElement('div');
    hint.style.cssText = 'font-size:13px;color:var(--sheet-muted, #94a3b8);margin-bottom:16px;';
    hint.textContent = 'Nothing was opened and nothing can be saved. The Drive file is unchanged.';
    box.appendChild(title);
    box.appendChild(hint);
    if (info.bucket && info.path && window.WindowManager && window.WindowManager.openDeepLink) {
        var retry = document.createElement('button');
        retry.textContent = 'Retry';
        retry.style.cssText = 'padding:6px 14px;border-radius:6px;border:1px solid var(--sheet-border, #334155);' +
            'background:var(--sheet-bg-elevated, #1e293b);color:inherit;cursor:pointer;';
        retry.addEventListener('click', function () {
            window.__SHEET_LOAD_ERROR = null;
            window.WindowManager.openDeepLink('sheet', { bucket: info.bucket, file: info.path });
        });
        box.appendChild(retry);
    }
    host.appendChild(box);
}
function saveSheet(arg) {
    var ctx = _sheetCtx(arg);
    if (ctx.inst && ctx.inst.loadError) {
        sheetNotify('Not saved: this window has no document — the file failed to load.', true);
        return;
    }
    if (!ctx.inst && window.__SHEET_LOAD_ERROR) {
        sheetNotify('Not saved: this window has no document — the file failed to load.', true);
        return;
    }
    var nameInput = _q(ctx, '#sheetName') || document.getElementById('sheetName');
    var name = nameInput ? nameInput.value : 'Untitled Spreadsheet';
    var id = getLiveSheetId(ctx);
    var payload = { name: name, action: 'save' };
    if (id) payload.id = id;
    fetch('/api/sheet/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
    }).then(function(r) {
        return r.json().then(function(j) {
            if (!r.ok) throw new Error((j && j.error) || 'save failed');
            return j;
        });
    }).then(function(j) {
        if (j && j.id) {
            if (ctx.inst) {
                ctx.inst.sheetId = j.id;
                if (ctx.inst.loadedSheet) ctx.inst.loadedSheet.id = j.id;
            }
            window.__SHEET_INITIAL_ID = j.id;
            if (window.__LOADED_SHEET) window.__LOADED_SHEET.id = j.id;
        }
        var notif = document.createElement('div');
        notif.style.cssText = 'position:fixed;bottom:20px;right:20px;background:#10b981;color:white;padding:10px 20px;border-radius:6px;font-size:13px;z-index:9999;box-shadow:0 4px 12px rgba(0,0,0,0.3);';
        notif.textContent = '✓ Sheet saved';
        document.body.appendChild(notif);
        setTimeout(function() { notif.remove(); }, 2000);
    }).catch(function(err) {
        sheetNotify('Save failed: ' + (err && err.message ? err.message : 'unknown error'), true);
    });
}
document.addEventListener('keydown', function(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        var t = e.target;
        if (t && t.closest && t.closest('[id^="window-body-"]')) {
            e.preventDefault();
            saveSheet(t);
        }
    }
});
if (typeof htmx === 'undefined') {
    var s = document.createElement('script');
    // Absolute path: a relative one resolved against /suite/sheet/sheet.html
    // (/suite/sheet/js/vendor/htmx.min.js) and answered 404, so every toolbar
    // hx-post stayed dead whenever the app was opened without the desktop
    // shell's global htmx (isolated window / direct route).
    s.src = '/suite/js/vendor/htmx.min.js';
    document.body.appendChild(s);
}
