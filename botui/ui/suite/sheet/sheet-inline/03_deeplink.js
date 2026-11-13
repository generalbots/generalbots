/* A deep link into an ALREADY-OPEN sheet window: the shell merges the params
   into __gbAppParams__ and dispatches gb:deep-link, but the boot block above
   ran long ago, so chat/Drive "Open" was ignored whenever Sheet was open. */
(function () {
  if (window.__gbSheetDeepLinkBound) return;
  window.__gbSheetDeepLinkBound = true;
  var SHEET_APP_ID = 'sheet';
  function isSheetWindowId(appId) {
    return appId === SHEET_APP_ID || (appId || '').indexOf(SHEET_APP_ID + '-') === 0;
  }
  document.addEventListener('gb:deep-link', function (event) {
    var detail = event.detail || {};
    var appId = detail.appId || '';
    if (!isSheetWindowId(appId)) return;
    var D = window.SheetDefaults || {};
    var body = document.getElementById('window-body-' + appId);
    var inst = D.getSheetInstance ? D.getSheetInstance(appId) : (window.__SHEET_INSTANCES__ || {})[appId];
    if (inst && D.setActiveSheetInstance) D.setActiveSheetInstance(inst);
    var params = detail.params || {};
    var bucket = params.bucket;
    var path = params.file;
    var sheetRef = params.file_id || params.id || '';
    var cell = params.cell || '';
    if (!bucket && !path && sheetRef) {
      if (typeof loadSheet === 'function') loadSheet(sheetRef, cell);
      if (window.SheetCore && window.SheetCore.refreshWidths) window.SheetCore.refreshWidths();
      return;
    }
    if (!bucket || !path) {
      console.warn('[sheet] deep link names no workbook (file_id/id or bucket+file):', appId, params);
      return;
    }
    var token = (typeof window.getGBAccessToken === 'function' && window.getGBAccessToken()) || '';
    fetch('/api/sheet/load-from-drive', {
      method: 'POST',
      headers: Object.assign(
        { 'Content-Type': 'application/json' },
        token ? { Authorization: 'Bearer ' + token } : {}
      ),
      body: JSON.stringify({ bucket: bucket, path: path })
    }).then(function (r) {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.json();
    }).then(function (sheet) {
      if (!sheet || !sheet.id) throw new Error('no sheet in response');
      if (inst) { inst.loadedSheet = sheet; inst.sheetId = sheet.id; }
      window.__LOADED_SHEET = sheet;
      window.__SHEET_INITIAL_ID = sheet.id;
      if (window.SheetCore && window.SheetCore.refreshWidths) window.SheetCore.refreshWidths();
      if (typeof loadSheet === 'function') loadSheet(sheet.id, cell);
      var nameInput = (body ? body.querySelector('#sheetName') : null) || document.getElementById('sheetName');
      if (nameInput && sheet.name) nameInput.value = sheet.name;
    }).catch(function (err) {
      console.error('Sheet deep-link load error:', err);
    });
  });
})();
