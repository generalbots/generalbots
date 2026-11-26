/* Split from 01_sidebar.js for #2305 (450-line limit). */
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
        if (typeof window.gbPerfSignal === "function") window.gbPerfSignal("sheet", "save");
        var notif = document.createElement('div');
        notif.style.cssText = 'position:fixed;bottom:20px;right:20px;background:var(--id-success);color:white;padding:10px 20px;border-radius:6px;font-size:13px;z-index:var(--z-system);box-shadow:0 4px 12px color-mix(in srgb, var(--id-ink) 30%, transparent);';
        notif.textContent = (typeof window.gbT === "function" ? window.gbT("sheet-saved", "✓ Sheet saved") : "✓ Sheet saved");
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
