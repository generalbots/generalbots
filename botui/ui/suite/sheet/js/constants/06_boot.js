"use strict";
/* sheet boot — toolbar, collaboration, auth and startup */

(function () {
  var D = window.SheetDefaults;
  var colName = D.colName;
  var initSidebar = D.initSidebar;
  var SheetAPI = D.SheetAPI;

  function q(scope, sel) { return scope ? scope.querySelector(sel) : null; }

  function activeGrid(scope) {
    if (scope) {
      var wid = D.getSheetWindowId ? D.getSheetWindowId(scope) : null;
      var inst = wid && D.getSheetInstance ? D.getSheetInstance(wid) : null;
      if (inst && inst.grid) return inst.grid;
    }
    var a = D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    return (a && a.grid) || window.SheetVirtualGrid || D.virtualGrid || null;
  }

  function formatSelectedRange(payload, scope) {
    var grid = activeGrid(scope);
    if (!grid) return Promise.resolve(null);
    var row = grid.selectedRow;
    var col = grid.selectedCol;
    if (row == null || col == null) return Promise.resolve(null);
    var inst = grid.instance || (D.getActiveSheetInstance ? D.getActiveSheetInstance() : null);
    var body = {
      sheet_id: D.currentSheetId(inst),
      worksheet_index: inst && typeof inst.worksheetIndex === "number" ? inst.worksheetIndex : D.WORKSHEET_INDEX,
      start_row: row,
      start_col: col,
      end_row: row,
      end_col: col,
    };
    if (payload.style) body.style = payload.style;
    if (payload.format !== undefined) body.format = payload.format;
    return fetch("/api/sheet/format", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    })
      .then(function (r) { return r.json(); })
      .then(function () {
        var key = row + "," + col;
        var existing = grid.cells.get(key) || {};
        if (payload.format !== undefined) existing.format = payload.format;
        if (payload.style) existing.style = Object.assign({}, existing.style || {}, payload.style);
        grid.cells.set(key, existing);
        grid.lastRenderedRange = null;
        grid.requestRange();
      })
      .catch(function (err) { console.error("[gb-sheet] Failed to save cell format:", err); });
  }

  function initToolbar(scope, grid) {
    var root = scope || document;
    var g = function () { return grid || activeGrid(root); };
    var boldBtn = q(root, "#boldBtn");
    if (boldBtn) boldBtn.addEventListener("click", function (e) {
      e.preventDefault(); var vg = g(); if (vg) vg.formatSelectedCell({ font_weight: "bold" });
    });
    var italicBtn = q(root, "#italicBtn");
    if (italicBtn) italicBtn.addEventListener("click", function (e) {
      e.preventDefault(); var vg = g(); if (vg) vg.formatSelectedCell({ font_style: "italic" });
    });
    var underlineBtn = q(root, "#underlineBtn");
    if (underlineBtn) underlineBtn.addEventListener("click", function (e) {
      e.preventDefault(); var vg = g(); if (vg) vg.formatSelectedCell({ text_decoration: "underline" });
    });
    var strikeBtn = q(root, "#strikeBtn");
    if (strikeBtn) strikeBtn.addEventListener("click", function (e) {
      e.preventDefault(); var vg = g(); if (vg) vg.formatSelectedCell({ text_decoration: "line-through" });
    });
    function enhanceColor(id) {
      var el = q(root, "#" + id);
      if (!el || !window.GBColorPicker) return el;
      var trigger = root.querySelector('[data-gb-color-for="' + id + '"]');
      window.GBColorPicker.enhance(el, { trigger: trigger });
      return el;
    }
    var fillBtn = enhanceColor("fillColorBtn") || q(root, "#fillColorBtn");
    if (fillBtn) fillBtn.addEventListener("change", function (e) {
      e.preventDefault(); var vg = g(); if (vg) vg.formatSelectedCell({ background: fillBtn.value });
    });
    var textBtn = enhanceColor("textColorBtn") || q(root, "#textColorBtn");
    if (textBtn) textBtn.addEventListener("change", function (e) {
      e.preventDefault(); var vg = g(); if (vg) vg.formatSelectedCell({ color: textBtn.value });
    });
    Array.prototype.forEach.call(
      root.querySelectorAll("#alignLeftBtn, #alignCenterBtn, #alignRightBtn"),
      function (btn) {
        btn.addEventListener("click", function (e) {
          e.preventDefault();
          var align = btn.getAttribute("data-align") || "left";
          root.querySelectorAll("#alignLeftBtn, #alignCenterBtn, #alignRightBtn")
            .forEach(function (other) { other.setAttribute("aria-pressed", other === btn ? "true" : "false"); });
          var vg = g(); if (vg) vg.formatSelectedCell({ text_align: align });
        });
      }
    );
    var NUMBER_FORMATS = { currency: "R$ #,##0.00", percent: "0.00%", decimal: "0.00", general: "" };
    Array.prototype.forEach.call(
      root.querySelectorAll("#currencyBtn, #percentBtn, #decimalBtn, #clearFormatBtn"),
      function (btn) {
        btn.addEventListener("click", function (e) {
          e.preventDefault();
          var key = btn.getAttribute("data-number-format") || "general";
          formatSelectedRange({ format: NUMBER_FORMATS[key] || "" }, root);
        });
      }
    );
    var formulaInput = q(root, "#formulaInput");
    if (formulaInput) {
      var commitFormula = function () {
        var val = formulaInput.value;
        var vg = g();
        if (!vg || vg.selectedRow == null || vg.selectedCol == null) return;
        var ref = colName(vg.selectedCol) + (vg.selectedRow + 1);
        SheetAPI.updateCell(ref, val, vg.instance || vg).then(function (res) {
          if (res && res.success) { vg.lastRenderedRange = null; vg.requestRange(); }
        });
      };
      formulaInput.addEventListener("keydown", function (e) {
        if (e.key === "Enter") { e.preventDefault(); commitFormula(); }
      });
      formulaInput.addEventListener("change", commitFormula);
    }
  }
  window.initToolbar = initToolbar;

  window.addEventListener("resize", function () {
    var reg = window.__SHEET_INSTANCES__ || {};
    Object.keys(reg).forEach(function (id) {
      var g = reg[id] && reg[id].grid;
      if (g && g.root) { g.updateViewport(); g.lastRenderedRange = null; g.requestRange(); }
    });
  });

  function initCollab(scope, inst) {
    if (!window.GBCollab || !inst) return;
    window.GBCollab.connect({
      app: "sheet",
      docId: D.currentSheetId(inst),
      collaboratorsEl: scope ? scope.querySelector("#collaborators") : null,
      opLogUrl: function (docId, since) {
        return "/api/sheet/" + encodeURIComponent(docId) + "/ops?since=" + since;
      },
      opSeqOf: function (op) { return op.seq; },
      onConnect: function () {},
      onDisconnect: function () { if (window.SheetPresence) window.SheetPresence.clearAll(); },
      onMessage: function (msg) {
        if (!msg || !window.SheetPresence) return;
        if (msg.msg_type === "cursor" || msg.msg_type === "cell_select") window.SheetPresence.cursor(msg);
      },
      onTyping: function (msg) {
        if (!msg || !window.SheetPresence) return;
        if (msg.msg_type === "typing_start") window.SheetPresence.typing(msg);
        else if (msg.msg_type === "typing_stop") window.SheetPresence.clearTyping(msg.user_id);
      },
      onPresence: function (users) { if (window.SheetPresence) window.SheetPresence.sync(users); },
      onEdit: function (msg) {
        var grid = inst.grid; if (!msg || !grid) return;
        if (msg.payload !== undefined && msg.seq !== undefined) {
          var pl = msg.payload || {};
          msg.row = pl.row !== undefined ? pl.row : msg.row;
          msg.col = pl.col !== undefined ? pl.col : msg.col;
          msg.content = pl.value !== undefined ? pl.value : msg.content;
        }
        var row, col;
        if (msg.row !== undefined && msg.col !== undefined) { row = msg.row; col = msg.col; }
        else if (msg.position !== undefined) {
          row = Math.floor(msg.position / grid.totalCols);
          col = msg.position % grid.totalCols;
        } else { return; }
        var key = row + "," + col;
        var existing = grid.cells.get(key) || {};
        grid.cells.set(key, Object.assign({}, existing, { value: msg.content }));
        if (grid.lastRenderedRange) {
          var parts = grid.lastRenderedRange.split(":");
          var start = parseInt(parts[0], 10), end = parseInt(parts[1], 10);
          if (row >= start && row < end) {
            var el = grid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + col + '"]');
            if (el) el.textContent = msg.content || "";
          }
        }
      }
    });
  }

  function initAuth(scope) {
    if (window.GBAuthGuard) window.GBAuthGuard.injectLoginButton(scope ? scope.querySelector("#gb-auth-button") : null);
  }

  function bootSheet() {
    var myData = null;
    var myBody = document.currentScript ? document.currentScript.closest('[id^="window-body-"]') : null;
    if (!myBody) {
      var els = document.querySelectorAll('#sheet-content');
      for (var i = 0; i < els.length; i++) {
        var p = els[i].closest('[id^="window-body-"]');
        if (p && p.style.display !== 'none') { myBody = p; break; }
      }
    }
    var winId = myBody ? (myBody.dataset.windowId || myBody.id.replace("window-body-", "")) : null;
    if (winId) myData = (window.__SHEET_DATA_MAP || {})[winId];
    var inst = winId && D.ensureSheetInstance ? D.ensureSheetInstance(winId) : null;
    if (inst) {
      if (myData && myData.loadedSheet) inst.loadedSheet = myData.loadedSheet;
      if (inst.loadedSheet && inst.loadedSheet.id) inst.sheetId = inst.loadedSheet.id;
      inst.body = myBody;
      D.setActiveSheetInstance(inst);
    }

    initSidebar(myBody || document);
    initAuth(myBody || document);
    initCollab(myBody || document, inst);

    var bootPromise = (inst && inst.bootPromise) || window.__SHEET_BOOT || Promise.resolve();
    bootPromise.then(function () {
      var host = myBody ? myBody.querySelector('#sheet-content') : document.getElementById("sheet-content");
      if (inst && inst.loadError) {
        if (host && typeof gbSheetShowLoadError === 'function') gbSheetShowLoadError(host, inst.loadError);
        return;
      }
      if (host && window.__SHEET_LOAD_ERROR && !inst) {
        if (typeof gbSheetShowLoadError === 'function') gbSheetShowLoadError(host, window.__SHEET_LOAD_ERROR);
        return;
      }
      if (host) {
        var grid = D.createVirtualGrid ? D.createVirtualGrid() : D.virtualGrid;
        if (inst) { inst.grid = grid; grid.instance = inst; }
        grid.init(host);
        initToolbar(myBody || host.parentElement || document, grid);
        var loaded = (inst && inst.loadedSheet) || (myData && myData.loadedSheet) || window.__LOADED_SHEET;
        if (loaded && loaded.worksheets && loaded.worksheets.length > 0) {
          var savedIndex = 0;
          try { savedIndex = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10) || 0; } catch (_) { savedIndex = 0; }
          if (savedIndex < 0 || savedIndex >= loaded.worksheets.length) savedIndex = 0;
          D.WORKSHEET_INDEX = savedIndex;
          if (inst) inst.worksheetIndex = savedIndex;
          try { sessionStorage.setItem("sheet_active_index", String(savedIndex)); } catch (_) {}
          var ws = loaded.worksheets[savedIndex];
          if (ws.data) {
            for (var cellRef in ws.data) { grid.cells.set(cellRef, ws.data[cellRef]); }
          }
          grid.requestSeq++;
          grid.totalRows = Math.max(grid.totalRows, 50);
          grid.render();
        }
        if (window.SheetAdvanced && window.SheetAdvanced.init) {
          var sid = (inst && inst.sheetId) || (host.dataset.sheetId) ||
            (loaded && loaded.id) || window.__SHEET_INITIAL_ID || "current";
          var adv = window.SheetAdvanced.init(host, { sheetId: sid, grid: grid, instance: inst });
          if (adv) {
            window.SheetAdvancedInstance = adv;
            if (inst) inst.advanced = adv;
          }
        }
        window.SheetAPI = SheetAPI;
        window.SheetVirtualGrid = grid;
        if (D.setActiveSheetInstance && inst) D.setActiveSheetInstance(inst);
        var bootCell = (inst && inst.bootCell) || window.__SHEET_BOOT_CELL;
        if (bootCell && typeof gbSheetGotoCell === 'function') {
          gbSheetGotoCell(bootCell, inst || grid);
        }
        if (inst) inst.bootCell = null;
        window.__SHEET_BOOT_CELL = null;
      }
    });
  }

  if (document.readyState === "loading") {
    (function(){ var __cb = bootSheet; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    bootSheet();
  }
})();
