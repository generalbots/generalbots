/* A deep link into an ALREADY-OPEN sheet window: the shell merges the params
   into __gbAppParams__ and dispatches gb:deep-link, but the boot block above
   ran long ago, so chat/Drive "Open" was ignored whenever Sheet was open. */
(function () {
  if (window.__gbSheetDeepLinkBound) return;
  window.__gbSheetDeepLinkBound = true;
  // Sheet windows have ONE id namespace: the registry id (`sheet`) for the
  // launcher window, `sheet-<ts>` for each Drive-opened file. Drive used to mint
  // `sheets-<ts>`, a prefix this guard can never match — "sheets-123"
  // .indexOf("sheet-") is -1 — so every deep link aimed at a Drive-opened sheet
  // was dropped silently (#2272). Derive the match from the registry id instead
  // of hard-coding a prefix.
  var SHEET_APP_ID = 'sheet';
  function isSheetWindowId(appId) {
    return appId === SHEET_APP_ID || appId.indexOf(SHEET_APP_ID + '-') === 0;
  }
  document.addEventListener('gb:deep-link', function (event) {
    var detail = event.detail || {};
    var appId = detail.appId || '';
    if (!isSheetWindowId(appId)) return;
    var params = detail.params || {};
    var bucket = params.bucket;
    var path = params.file;
    // A command deep link targets the workbook by id (`app://sheet?file_id=…`).
    // Ignoring it left the previous workbook on screen while the id pointed at
    // the new one, so the requested sheet never appeared (UAT SHEET-01).
    var sheetRef = params.file_id || params.id || '';
    // `cell` names the address the bot is pointing at (#2271).
    var cell = params.cell || '';
    if (!bucket && !path && sheetRef) {
      if (typeof loadSheet === 'function') loadSheet(sheetRef, cell);
      if (window.SheetCore && window.SheetCore.refreshWidths) window.SheetCore.refreshWidths();
      return;
    }
    if (!bucket || !path) {
      // Nothing here addresses a workbook: report it instead of swallowing the
      // request (a silently dropped deep link looks like the app ignoring the
      // user, and is the worst failure mode for this feature).
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
      window.__LOADED_SHEET = sheet;
      window.__SHEET_INITIAL_ID = sheet.id;
      if (window.SheetCore && window.SheetCore.refreshWidths) window.SheetCore.refreshWidths();
      if (typeof loadSheet === 'function') loadSheet(sheet.id, cell);
      var nameInput = document.getElementById('sheetName');
      if (nameInput && sheet.name) nameInput.value = sheet.name;
    }).catch(function (err) {
      console.error('Sheet deep-link load error:', err);
    });
  });
})();
