/* A deep link into an ALREADY-OPEN sheet window: the shell merges the params
   into __gbAppParams__ and dispatches gb:deep-link, but the boot block above
   ran long ago, so chat/Drive "Open" was ignored whenever Sheet was open. */
(function () {
  if (window.__gbSheetDeepLinkBound) return;
  window.__gbSheetDeepLinkBound = true;
  document.addEventListener('gb:deep-link', function (event) {
    var detail = event.detail || {};
    var appId = detail.appId || '';
    if (appId !== 'sheet' && appId.indexOf('sheet-') !== 0) return;
    var params = detail.params || {};
    var bucket = params.bucket;
    var path = params.file;
    // A command deep link targets the workbook by id (`app://sheet?file_id=…`).
    // Ignoring it left the previous workbook on screen while the id pointed at
    // the new one, so the requested sheet never appeared (UAT SHEET-01).
    var sheetRef = params.file_id || params.id || '';
    if (!bucket && !path && sheetRef) {
      if (typeof loadSheet === 'function') loadSheet(sheetRef);
      return;
    }
    if (!bucket || !path) return;
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
      window.__LOADED_SHEET = sheet;
      window.__SHEET_INITIAL_ID = sheet.id;
      if (typeof loadSheet === 'function') loadSheet(sheet.id);
      var nameInput = document.getElementById('sheetName');
      if (nameInput && sheet.name) nameInput.value = sheet.name;
    }).catch(function (err) {
      console.error('Sheet deep-link load error:', err);
    });
  });
})();
