/* Boot: load sheet data from per-window storage (avoids race with multiple windows) */
(function(){
  var bodyEl = document.currentScript ? document.currentScript.closest('[id^="window-body-"]') : null;
  var winId = bodyEl ? bodyEl.dataset.windowId || bodyEl.id.replace('window-body-', '') : '';
  var sheetData = (window.__SHEET_DATA_MAP || {})[winId];

  if (sheetData && sheetData.loadedSheet) {
    // Opened from Drive — use pre-loaded data
    var sidebar = document.getElementById('sheet-sidebar');
    if (sidebar) sidebar.style.display = 'none';
    var ls = sheetData.loadedSheet;
    window.__LOADED_SHEET = ls;
    window.__SHEET_INITIAL_ID = ls.id;
    window.__SHEET_BOOT = Promise.resolve();
    var nameInput = document.querySelector('.sheet-name-input');
    if (nameInput && ls.name) nameInput.value = ls.name;
    return;
  }

  // Direct URL params fallback
  var qs = window.__SHEET_URL_PARAMS || window.location.search;
  if (qs && !qs.startsWith('?')) qs = '?' + qs;
  // A desktop deep link (chat "Open", Drive) carries the params in
  // __gbAppParams__ — the shell URL has no query string of its own.
  var gp = window.__gbAppParams__ || {};
  var params = new URLSearchParams(qs);
  var bucket = params.get('bucket') || gp.bucket;
  var path = params.get('path') || gp.file;

  // A chat/command deep link carries the workbook id, not a Drive path
  // (`app://sheet?file_id=<id>`, used by the sheet.open/get/save/cell.set
  // commands). This case was missing, so every such link fell through to the
  // "no file" branch below: the grid bound itself to the literal id "current"
  // and the workbook the bot had just created was never fetched — the sheet
  // opened empty while still accepting edits, which then landed on a
  // different, blank workbook (UAT SHEET-01 / SHEET-03).
  var sheetRef = params.get('file_id') || params.get('id') || gp.file_id || gp.id || '';
  if (!bucket && !path && sheetRef) {
    window.__SHEET_INITIAL_ID = sheetRef;
    var refToken = (typeof window.getGBAccessToken === 'function' && window.getGBAccessToken()) || '';
    window.__SHEET_BOOT = fetch('/api/sheet/load?id=' + encodeURIComponent(sheetRef), {
      headers: refToken ? { Authorization: 'Bearer ' + refToken } : {}
    }).then(function (r) {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.json();
    }).then(function (sheet) {
      window.__LOADED_SHEET = sheet;
      if (sheet && sheet.id) window.__SHEET_INITIAL_ID = sheet.id;
      var refNameInput = document.querySelector('.sheet-name-input');
      if (refNameInput && sheet && sheet.name) refNameInput.value = sheet.name;
    }).catch(function (err) {
      // Keep the requested id: edits must still target the same workbook.
      console.error('Sheet load error:', err);
    });
    return;
  }

  if (!bucket || !path) {
    window.__SHEET_INITIAL_ID = "current";
    window.__SHEET_BOOT = Promise.resolve();
    setTimeout(loadSidebar, 100);
    return;
  }

  window.__SHEET_INITIAL_ID = "loading";
  var fileName = path.split('/').pop() || 'Spreadsheet';
  var nameInput = document.querySelector('.sheet-name-input');
  if (nameInput) nameInput.value = fileName;

  window.__SHEET_BOOT = fetch('/api/sheet/load-from-drive', {
    method: 'POST',
    headers: Object.assign(
      { 'Content-Type': 'application/json' },
      (function () {
        var token = (typeof window.getGBAccessToken === 'function' && window.getGBAccessToken()) || '';
        return token ? { Authorization: 'Bearer ' + token } : {};
      })()
    ),
    body: JSON.stringify({ bucket: bucket, path: path })
  }).then(function(r){ return r.json(); }).then(function(sheet){
    window.__LOADED_SHEET = sheet;
    window.__SHEET_INITIAL_ID = sheet.id;
    if (nameInput) nameInput.value = sheet.name || fileName;
  }).catch(function(err){
    console.error('Sheet load error:', err);
    window.__SHEET_INITIAL_ID = "current";
  });
})();
