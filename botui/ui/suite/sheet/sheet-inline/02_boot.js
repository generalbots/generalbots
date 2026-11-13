/* Boot: load sheet data from per-window storage (avoids race with multiple windows) */
(function(){
  var D = window.SheetDefaults = window.SheetDefaults || {};
  window.__SHEET_INSTANCES__ = window.__SHEET_INSTANCES__ || {};
  var bodyEl = document.currentScript ? document.currentScript.closest('[id^="window-body-"]') : null;
  var winId = bodyEl ? (bodyEl.dataset.windowId || bodyEl.id.replace('window-body-', '')) : '';
  var sheetData = (window.__SHEET_DATA_MAP || {})[winId];
  var inst = null;
  if (winId && D.ensureSheetInstance) {
    inst = D.ensureSheetInstance(winId);
    inst.body = bodyEl;
  } else if (winId) {
    inst = window.__SHEET_INSTANCES__[winId] = window.__SHEET_INSTANCES__[winId] || { windowId: winId, body: bodyEl, worksheetIndex: 0 };
  }
  function q(sel) { return bodyEl ? bodyEl.querySelector(sel) : document.querySelector(sel); }
  function setState(obj) {
    if (inst) {
      if ('loadedSheet' in obj) inst.loadedSheet = obj.loadedSheet;
      if ('sheetId' in obj) inst.sheetId = obj.sheetId;
      if ('bootPromise' in obj) inst.bootPromise = obj.bootPromise;
      if ('loadError' in obj) inst.loadError = obj.loadError;
      if ('bootCell' in obj) inst.bootCell = obj.bootCell;
      if (D.setActiveSheetInstance) D.setActiveSheetInstance(inst);
    }
    if ('loadedSheet' in obj) window.__LOADED_SHEET = obj.loadedSheet;
    if ('sheetId' in obj) window.__SHEET_INITIAL_ID = obj.sheetId;
    if ('bootPromise' in obj) window.__SHEET_BOOT = obj.bootPromise;
    if ('loadError' in obj) window.__SHEET_LOAD_ERROR = obj.loadError;
    if ('bootCell' in obj) window.__SHEET_BOOT_CELL = obj.bootCell;
  }

  // #2269 — the Drive file could not be read: keep the window in an explicit
  // error state. Booting the blank stand-in here made a failed open look like
  // an empty spreadsheet whose edits and "saved" toast went nowhere.
  if (sheetData && sheetData.loadError) {
    setState({
      loadError: { message: sheetData.loadError, bucket: sheetData.requestedBucket || '', path: sheetData.requestedPath || '' },
      sheetId: null,
      bootPromise: Promise.resolve()
    });
    return;
  }

  if (sheetData && sheetData.loadedSheet) {
    var sidebar = q('#sheet-sidebar');
    if (sidebar) sidebar.style.display = 'none';
    var ls = sheetData.loadedSheet;
    setState({ loadedSheet: ls, sheetId: ls.id, bootPromise: Promise.resolve() });
    if (window.SheetCore && window.SheetCore.refreshWidths) window.SheetCore.refreshWidths();
    var nameInput = q('.sheet-name-input') || q('#sheetName');
    if (nameInput && ls.name) nameInput.value = ls.name;
    return;
  }

  var qs = window.__SHEET_URL_PARAMS || window.location.search;
  if (qs && !qs.startsWith('?')) qs = '?' + qs;
  var gp = window.__gbAppParams__ || {};
  var params = new URLSearchParams(qs);
  var bucket = params.get('bucket') || gp.bucket;
  var path = params.get('path') || gp.file;

  var sheetRef = params.get('file_id') || params.get('id') || gp.file_id || gp.id || '';
  var bootCell = params.get('cell') || gp.cell || '';
  if (bootCell) setState({ bootCell: bootCell });
  if (!bucket && !path && sheetRef) {
    setState({ sheetId: sheetRef });
    var refToken = (typeof window.getGBAccessToken === 'function' && window.getGBAccessToken()) || '';
    setState({ bootPromise: fetch('/api/sheet/load?id=' + encodeURIComponent(sheetRef), {
      headers: refToken ? { Authorization: 'Bearer ' + refToken } : {}
    }).then(function (r) {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.json();
    }).then(function (sheet) {
      setState({ loadedSheet: sheet, sheetId: (sheet && sheet.id) || sheetRef });
      if (window.SheetCore && window.SheetCore.refreshWidths) window.SheetCore.refreshWidths();
      var refNameInput = q('.sheet-name-input') || q('#sheetName');
      if (refNameInput && sheet && sheet.name) refNameInput.value = sheet.name;
    }).catch(function (err) {
      console.error('Sheet load error:', err);
    }) });
    return;
  }

  if (!bucket || !path) {
    setState({ sheetId: "current", bootPromise: Promise.resolve() });
    setTimeout(function () { if (typeof loadSidebar === 'function') loadSidebar(bodyEl || inst); }, 100);
    return;
  }

  setState({ sheetId: "loading" });
  var fileName = path.split('/').pop() || 'Spreadsheet';
  var nameInput = q('.sheet-name-input') || q('#sheetName');
  if (nameInput) nameInput.value = fileName;

  setState({ bootPromise: fetch('/api/sheet/load-from-drive', {
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
    setState({ loadedSheet: sheet, sheetId: sheet.id });
    if (window.SheetCore && window.SheetCore.refreshWidths) window.SheetCore.refreshWidths();
    if (nameInput) nameInput.value = sheet.name || fileName;
  }).catch(function(err){
    console.error('Sheet load error:', err);
    setState({ sheetId: "current" });
  }) });
})();
