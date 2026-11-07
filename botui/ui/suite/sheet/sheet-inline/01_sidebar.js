/* Load sidebar dynamically from JSON API and render as HTML */
function _t(key, fallback, params) {
    if (window.SheetI18n && window.SheetI18n.t) return window.SheetI18n.t(key, params);
    var text = fallback || key;
    if (params) {
        Object.keys(params).forEach(function (p) { text = text.split('{' + p + '}').join(params[p]); });
    }
    return text;
}
function newSheet() {
    fetch('/api/sheet/new').then(function(r){ return r.json(); }).then(function(s){
        window.__LOADED_SHEET = s;
        window.__SHEET_INITIAL_ID = s.id;
        var host = document.getElementById('sheet-content');
        if (!host) return;
        host.innerHTML = '';
        if (window.SheetVirtualGrid) {
            var VG = window.SheetVirtualGrid;
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
        }
    });
}
function loadSidebar() {
    var el = document.getElementById('sidebar-files');
    if (!el) return;
    fetch('/api/sheet/list').then(function(r){ return r.json(); }).then(function(items){
        if (!items || items.length === 0) {
            el.innerHTML = '<div class="ss-empty-hint" style="padding:24px;text-align:center;color:#94a3b8;font-style:italic;">' + _t('sidebar.none', 'No spreadsheets yet. Create one to start.') + '</div>';
            return;
        }
        var html = '<div class="ss-sidebar" id="sheet-list-sidebar"><div class="ss-sidebar-header" style="display:flex;justify-content:space-between;align-items:center;padding:12px;border-bottom:1px solid #334155;"><h3 style="margin:0;color:#f8fafc;font-size:14px;">' + _t('sidebar.my_sheets', 'My Sheets') + '</h3><button class="btn-icon" onclick="newSheet()" style="background:#10b981;color:white;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">' + _t('sidebar.new', '+ New') + '</button></div><div class="ss-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;max-height:calc(100vh - 200px);overflow-y:auto;">';
        for (var i = 0; i < items.length; i++) {
            var m = items[i];
            html += '<div class="ss-metadata-card" onclick="loadSheet(\'' + m.id + '\')" style="padding:12px;border:1px solid #334155;border-radius:6px;background:#1e293b;cursor:pointer;"><div style="font-weight:600;color:#f8fafc;">' + escHtml(m.name) + '</div><div style="font-size:12px;color:#94a3b8;margin-top:4px;">' + _t('sidebar.sheets_count', '{0} sheet(s)', { '0': m.worksheet_count || 0 }) + '</div></div>';
        }
        html += '</div></div>';
        el.innerHTML = html;
    }).catch(function(){
        el.innerHTML = '<div style="padding:24px;text-align:center;color:#94a3b8;font-style:italic;">' + _t('sidebar.load_error', 'Error loading sheets.') + '</div>';
    });
}
function escHtml(s) { return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;'); }
function openSheetPanel(panel) {
    var sheet = window.__LOADED_SHEET;
    if (!sheet) {
        var host = document.getElementById('sheet-content');
        if (host) host.innerHTML = '<div style="padding:20px;color:#f87171;text-align:center;">' + _t('sidebar.load_first', 'Load a spreadsheet first.') + '</div>';
        return;
    }
    var panelHost = document.getElementById('sheet-content');
    if (!panelHost) return;
    // The module reloads the workbook before rendering: posting the cached
    // copy showed stale protection/chart state after every mutation.
    if (window.GBSheetRefreshPanel) {
        window.GBSheetRefreshPanel(panel).catch(function () {
            panelHost.innerHTML = '<div style="padding:20px;color:#f87171;text-align:center;">' + _t('panel.load_error', 'Error loading panel.') + '</div>';
        });
        return;
    }
    fetch('/suite/sheet/fragments/' + panel, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(sheet)
    }).then(function(r){ return r.text(); }).then(function(html){ panelHost.innerHTML = html; })
    .catch(function(){ panelHost.innerHTML = '<div style="padding:20px;color:#f87171;text-align:center;">' + _t('panel.load_error', 'Error loading panel.') + '</div>'; });
}
function loadSheet(id) {
    window.__SHEET_INITIAL_ID = id;
    // Each workbook starts on its first tab: the persisted index belonged to
    // the previously open sheet and could point at a missing worksheet.
    try { sessionStorage.setItem("sheet_active_index", "0"); } catch (_) {}
    document.dispatchEvent(new CustomEvent("gb-sheet-tab", { detail: { index: 0 }, bubbles: true }));
    var host = document.getElementById('sheet-content');
    if (host) host.innerHTML = '<div style="padding:20px;color:#94a3b8;text-align:center;">' + _t('common.loading', 'Loading...') + '</div>';
    // Load the worksheet tabs HTML from server, then init VirtualGrid
    fetch('/api/sheet/load?id=' + encodeURIComponent(id)).then(function(r){ return r.json(); }).then(function(sheet){
        window.__LOADED_SHEET = sheet;
        // Switching workbooks must refresh the title input: it kept the
        // previous workbook's name, and the next Save then renamed the newly
        // opened sheet to that stale value.
        var titleInput = document.getElementById('sheetName');
        if (titleInput && sheet && sheet.name) titleInput.value = sheet.name;
        if (!host) return;
        host.innerHTML = '';
        // Re-initialize VirtualGrid with fresh state
        if (window.SheetVirtualGrid) {
            // Reset VirtualGrid state completely
            var VG = window.SheetVirtualGrid;
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
            if (window.__LOADED_SHEET && window.__LOADED_SHEET.worksheets && window.__LOADED_SHEET.worksheets.length > 0) {
                var ws = window.__LOADED_SHEET.worksheets[0];
                if (ws.data) {
                    for (var cellRef in ws.data) {
                        VG.cells.set(cellRef, ws.data[cellRef]);
                    }
                }
                VG.requestSeq++;
                VG.totalRows = Math.max(VG.totalRows, 50);
                VG.render();
                if (window.SheetAdvanced && window.SheetAdvanced.sync) window.SheetAdvanced.sync();
            }
        }
    }).catch(function(){
        if (host) host.innerHTML = '<div style="padding:20px;color:#f87171;text-align:center;">Error loading sheet.</div>';
    });
}
function getLiveSheetId() {
    return (window.__LOADED_SHEET && window.__LOADED_SHEET.id) || window.__SHEET_INITIAL_ID || null;
}
// Toolbar modals hardcode id="current"; rewrite it to the live sheet id once
// per HTMX request so the modal (and the form it renders) targets the open
// workbook instead of a session literal.
document.addEventListener("htmx:configRequest", function (event) {
    var parameters = (event.detail && event.detail.parameters) || {};
    var live = getLiveSheetId();
    if (live && live !== "current") {
        ["id", "sheet_id"].forEach(function (key) {
            if (parameters[key] === "current") parameters[key] = live;
        });
    }
    // Mutations must target the active worksheet tab, not always the first.
    if (window.SheetCore && typeof window.SheetCore.wsIndex === "function") {
        parameters.worksheet_index = window.SheetCore.wsIndex();
    } else {
        var tabs = document.getElementById("worksheet-tabs");
        if (tabs && tabs.dataset.activeIndex !== undefined) {
            parameters.worksheet_index = parseInt(tabs.dataset.activeIndex, 10) || 0;
        }
    }
});
// Re-render the grid after a successful htmx mutation (CF/DV/format/chart/
// share) so the change is visible without a reload. Cell edits use fetch.
// Sheet AI answers are JSON; render the text in #ai-response instead of
// swapping the object into the panel.
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
    var target = document.getElementById("ai-response");
    if (target) target.textContent = text;
});

document.addEventListener("htmx:afterRequest", function (event) {
    var detail = event.detail || {};
    if (!detail.successful) return;
    var path = (detail.pathInfo && detail.pathInfo.requestPath) || "";
    if (path.indexOf("/api/sheet/") !== 0) return;
    if (path === "/api/sheet/load" || path === "/api/sheet/load-from-drive") return;
    var live = getLiveSheetId();
    if (live && typeof loadSheet === "function") loadSheet(live);
});
function GBSheetRefreshNamedRanges(sheetId) {
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    fetch("/suite/sheet/fragments/named-ranges", {
        method: "POST",
        headers: headers,
        body: JSON.stringify({ id: sheetId }),
    })
        .then(function (response) { return response.text(); })
        .then(function (html) {
            var current = document.getElementById("named-ranges-panel");
            if (!current) return;
            var holder = document.createElement("div");
            holder.innerHTML = html.trim();
            var next = holder.firstElementChild;
            if (next) current.replaceWith(next);
        })
        .catch(function () {});
}
window.GBSheetRefreshNamedRanges = GBSheetRefreshNamedRanges;
function GBSheetCreateNamedRange(sheetId) {
    var name = (window.GBModal||{prompt:prompt}).prompt("Nome do intervalo:", "");
    if (!name || !name.trim()) return;
    var sel = window.SheetAdvanced && window.SheetAdvanced.getSelection
        ? window.SheetAdvanced.getSelection() : null;
    if (!sel) {
        sheetNotify("Selecione as células do intervalo primeiro.", true);
        return;
    }
    var token = localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    var headers = { "Content-Type": "application/json" };
    if (token) headers.Authorization = "Bearer " + token;
    fetch("/api/sheet/named-range", {
        method: "POST",
        headers: headers,
        body: JSON.stringify({
            sheet_id: sheetId,
            name: name.trim(),
            scope: "workbook",
            worksheet_index: window.SheetCore ? window.SheetCore.wsIndex() : 0,
            start_row: sel.startRow,
            start_col: sel.startCol,
            end_row: sel.endRow,
            end_col: sel.endCol,
        }),
    })
        .then(function (response) {
            if (!response.ok) throw new Error("HTTP " + response.status);
            GBSheetRefreshNamedRanges(sheetId);
        })
        .catch(function (error) {
            sheetNotify("Could not create the named range: " + error.message, true);
        });
}
window.GBSheetCreateNamedRange = GBSheetCreateNamedRange;
function openSheetComments() {
    if (!window.GBCollabComments) return;
    var id = getLiveSheetId();
    if (!id || id === 'current') {
        alert('Save the spreadsheet first, then open comments.');
        return;
    }
    window.GBCollabComments.open({
        resourceType: 'sheet',
        resourceId: String(id),
        title: 'Spreadsheet comments',
        // Aggregate every cell-anchored comment (sheet:cell) under this sheet
        // so the doc-level panel shows the full discussion, not just a few
        // sheet-level comments.
        includeChildren: true,
    });
}
function renameSheet() {
    var nameInput = document.getElementById('sheetName');
    if (!nameInput) return;
    var name = nameInput.value.trim();
    if (!name) { nameInput.value = window.__LOADED_SHEET ? window.__LOADED_SHEET.name : 'Untitled Spreadsheet'; return; }
    var id = getLiveSheetId();
    if (!id || id === 'current') {
        // No live document yet — persist the title with the next save.
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
        if (window.__LOADED_SHEET) window.__LOADED_SHEET.name = name;
        var notif = document.createElement('div');
        notif.style.cssText = 'position:fixed;bottom:20px;right:20px;background:#3b82f6;color:white;padding:10px 20px;border-radius:6px;font-size:13px;z-index:9999;box-shadow:0 4px 12px rgba(0,0,0,0.3);';
        notif.textContent = 'Renamed';
        document.body.appendChild(notif);
        setTimeout(function() { notif.remove(); }, 2000);
    }).catch(function(err) {
        sheetNotify('Rename failed: ' + (err && err.message ? err.message : 'unknown error'), true);
    });
}

// Save/rename failures used to be swallowed by an empty .catch; show them in
// the same place successes appear so a failed save is never silent (#1712).
function sheetNotify(message, isError) {
    var notif = document.createElement('div');
    notif.style.cssText = 'position:fixed;bottom:20px;right:20px;background:' +
        (isError ? '#ef4444' : '#10b981') +
        ';color:white;padding:10px 20px;border-radius:6px;font-size:13px;z-index:9999;box-shadow:0 4px 12px rgba(0,0,0,0.3);';
    notif.textContent = message;
    document.body.appendChild(notif);
    setTimeout(function() { notif.remove(); }, isError ? 5000 : 2000);
}
function saveSheet() {
    var nameInput = document.getElementById('sheetName');
    var name = nameInput ? nameInput.value : 'Untitled Spreadsheet';
    var id = getLiveSheetId();
    var payload = { name: name, action: 'save' };
    // "current" is a real live session: every cell edit was written to it.
    // Dropping it here made the server create a fresh blank workbook, losing
    // the edits the user had just made.
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
        e.preventDefault();
        saveSheet();
    }
});
if (typeof htmx === 'undefined') {
    var s = document.createElement('script');
    s.src = 'js/vendor/htmx.min.js';
    document.body.appendChild(s);
}
