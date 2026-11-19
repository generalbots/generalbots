"use strict";
/* sheet boot — toolbar, collaboration, auth and startup */

(function () {
  var D = window.SheetDefaults;
  var colName = D.colName;
  var initSidebar = D.initSidebar;
  var SheetAPI = D.SheetAPI;
  var fallbackVirtualGrid = D.virtualGrid;

  function activeInstance(context) {
    if (context && context.windowId && (context.grid || context.body)) return context;
    if (D.getSheetInstance) {
      var inst = D.getSheetInstance(context);
      if (inst) return inst;
    }
    return D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
  }
  function activeGrid(context) {
    var inst = activeInstance(context);
    if (context && context.cells && context.selectCell) return context;
    return (inst && inst.grid) || window.SheetVirtualGrid || D.virtualGrid || fallbackVirtualGrid;
  }
  function scopeEl(id, instOrBody) {
    var body = instOrBody && instOrBody.nodeType ? instOrBody : (instOrBody && instOrBody.body ? instOrBody.body : null);
    if (!body && instOrBody && instOrBody.windowId) body = document.getElementById('window-body-' + instOrBody.windowId);
    if (!body && D.getActiveSheetInstance) { var a = D.getActiveSheetInstance(); body = a && a.body; }
    if (body && body.querySelector) {
      var byId = body.querySelector('#' + id);
      if (byId) return byId;
    }
    return document.getElementById(id);
  }
  function scopeAll(sel, instOrBody) {
    var body = instOrBody && instOrBody.nodeType ? instOrBody : (instOrBody && instOrBody.body ? instOrBody.body : null);
    if (!body && instOrBody && instOrBody.windowId) body = document.getElementById('window-body-' + instOrBody.windowId);
    if (!body && D.getActiveSheetInstance) { var a = D.getActiveSheetInstance(); body = a && a.body; }
    if (body && body.querySelectorAll) return body.querySelectorAll(sel);
    return document.querySelectorAll(sel);
  }

  // Toolbar format button handlers
  /* Formats the selected cell with a payload that can carry either a `style`
     object or a `format` string, then mirrors it in the local cell cache so the
     grid repaints immediately (#2184). */
  function formatSelectedRange(payload, gridOrCtx) {
    var inst = activeInstance(gridOrCtx);
    var grid = activeGrid(gridOrCtx || inst);
    var row = grid && grid.selectedRow;
    var col = grid && grid.selectedCol;
    if (row == null || col == null || !grid) return Promise.resolve(null);
    var body = {
      sheet_id: D.currentSheetId(inst || grid),
      worksheet_index: (inst && typeof inst.worksheetIndex === "number") ? inst.worksheetIndex : D.WORKSHEET_INDEX,
      start_row: row,
      start_col: col,
      end_row: row,
      end_col: col,
    };
    if (payload && payload.style) body.style = payload.style;
    if (payload && payload.format !== undefined) body.format = payload.format;
    return fetch("/api/sheet/format", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    })
      .then(function (r) { return r.json(); })
      .then(function () {
        var key = row + "," + col;
        var existing = grid.cells.get(key) || {};
        if (payload && payload.format !== undefined) existing.format = payload.format;
        if (payload && payload.style) {
          existing.style = Object.assign({}, existing.style || {}, payload.style);
        }
        grid.cells.set(key, existing);
        grid.lastRenderedRange = null;
        if (grid.requestRange) grid.requestRange();
      })
      .catch(function (err) {
        console.warn("[sheet] format apply failed", err);
      });
  }

  function initToolbar(scopeBody) {
    var inst = activeInstance(scopeBody);
    var gridFn = function () { return activeGrid(inst || scopeBody); };
    var byId = function (id) { return scopeEl(id, inst || scopeBody); };
    var all = function (sel) { return scopeAll(sel, inst || scopeBody); };
    var boldBtn = byId("boldBtn");
    if (boldBtn) {
      boldBtn.addEventListener("click", function (e) {
        e.preventDefault();
        var g = gridFn(); if (g && g.formatSelectedCell) g.formatSelectedCell({ font_weight: "bold" });
      });
    }
    var italicBtn = byId("italicBtn");
    if (italicBtn) {
      italicBtn.addEventListener("click", function (e) {
        e.preventDefault();
        var g = gridFn(); if (g && g.formatSelectedCell) g.formatSelectedCell({ font_style: "italic" });
      });
    }
    var underlineBtn = byId("underlineBtn");
    if (underlineBtn) {
      underlineBtn.addEventListener("click", function (e) {
        e.preventDefault();
        var g = gridFn(); if (g && g.formatSelectedCell) g.formatSelectedCell({ text_decoration: "underline" });
      });
    }
    var strikeBtn = byId("strikeBtn");
    if (strikeBtn) {
      strikeBtn.addEventListener("click", function (e) {
        e.preventDefault();
        var g = gridFn(); if (g && g.formatSelectedCell) g.formatSelectedCell({ text_decoration: "line-through" });
      });
    }
    var fontSizeInput = byId("fontSizeInput");
    if (fontSizeInput) {
      var parseFontSize = function (raw) {
        var text = String(raw == null ? "" : raw).trim().toLowerCase();
        var m = text.match(/(\d+(?:\.\d+)?)/);
        if (!m) return null;
        var n = Math.round(parseFloat(m[1]) * 100) / 100;
        if (!isFinite(n) || n < 1 || n > 200) return null;
        return n;
      };
      var applyFontSize = function () {
        var size = parseFontSize(fontSizeInput.value);
        if (size == null) return;
        fontSizeInput.value = String(size);
        var g = gridFn(); if (g && g.formatSelectedCell) g.formatSelectedCell({ font_size: size });
      };
      fontSizeInput.addEventListener("change", applyFontSize);
      fontSizeInput.addEventListener("blur", applyFontSize);
      fontSizeInput.addEventListener("keydown", function (e) {
        if (e.key === "Enter") {
          e.preventDefault();
          applyFontSize();
        }
      });
    }
    var fontFamilySelect = byId("fontFamilySelect");
    if (fontFamilySelect) {
      fontFamilySelect.addEventListener("change", function () {
        var g = gridFn();
        if (g && g.formatSelectedCell) g.formatSelectedCell({ font_family: fontFamilySelect.value || "" });
      });
    }
    var fillBtn = byId("fillColorBtn");
    if (fillBtn) {
      var fillTrigger = (inst && inst.body ? inst.body : document).querySelector('[data-gb-color-for="fillColorBtn"]') || document.querySelector('[data-gb-color-for="fillColorBtn"]');
      if (window.GBColorPicker) window.GBColorPicker.enhance(fillBtn, { trigger: fillTrigger, title: "Fill color" });
      var applyFill = function () { var g = gridFn(); if (g && g.formatSelectedCell) g.formatSelectedCell({ background: fillBtn.value }); };
      fillBtn.addEventListener("gb-color-commit", applyFill);
      fillBtn.addEventListener("change", function () { if (!fillBtn._gbColorEnhanced) applyFill(); });
    }
    var textBtn = byId("textColorBtn");
    if (textBtn) {
      var textTrigger = (inst && inst.body ? inst.body : document).querySelector('[data-gb-color-for="textColorBtn"]') || document.querySelector('[data-gb-color-for="textColorBtn"]');
      if (window.GBColorPicker) window.GBColorPicker.enhance(textBtn, { trigger: textTrigger, title: "Text color" });
      var applyText = function () { var g = gridFn(); if (g && g.formatSelectedCell) g.formatSelectedCell({ color: textBtn.value }); };
      textBtn.addEventListener("gb-color-commit", applyText);
      textBtn.addEventListener("change", function () { if (!textBtn._gbColorEnhanced) applyText(); });
    }
    Array.prototype.forEach.call(
      all("#alignLeftBtn, #alignCenterBtn, #alignRightBtn"),
      function (btn) {
        btn.addEventListener("click", function (e) {
          e.preventDefault();
          var align = btn.getAttribute("data-align") || "left";
          Array.prototype.forEach.call(all("#alignLeftBtn, #alignCenterBtn, #alignRightBtn"), function (other) {
            other.setAttribute("aria-pressed", other === btn ? "true" : "false");
          });
          var g = gridFn(); if (g && g.formatSelectedCell) g.formatSelectedCell({ text_align: align });
        });
      },
    );
    var NUMBER_FORMATS = {
      currency: "R$ #,##0.00",
      percent: "0.00%",
      decimal: "0.00",
      general: "",
    };
    Array.prototype.forEach.call(
      all("#currencyBtn, #percentBtn, #decimalBtn, #clearFormatBtn"),
      function (btn) {
        btn.addEventListener("click", function (e) {
          e.preventDefault();
          var key = btn.getAttribute("data-number-format") || "general";
          formatSelectedRange({ format: NUMBER_FORMATS[key] || "" }, inst || gridFn());
        });
      },
    );
    var formulaInput = byId("formulaInput");
    if (formulaInput) {
      var commitFormula = function () {
        var g = gridFn();
        var val = formulaInput.value;
        if (!g || g.selectedRow == null || g.selectedCol == null) return;
        var ref = colName(g.selectedCol) + (g.selectedRow + 1);
        SheetAPI.updateCell(ref, val, inst || g).then(function (res) {
          if (res && res.success) {
            g.lastRenderedRange = null;
            if (g.requestRange) g.requestRange();
            if (typeof g.onCellValueChanged === "function") {
              g.onCellValueChanged(g.selectedRow, g.selectedCol, val);
            }
          }
        });
      };
      formulaInput.addEventListener("keydown", function (e) {
        if (e.key === "Enter") {
          e.preventDefault();
          commitFormula();
        }
      });
      formulaInput.addEventListener("change", commitFormula);
    }
  }
  window.initToolbar = initToolbar;

  window.addEventListener("resize", function () {
    if (VirtualGrid.root) {
      VirtualGrid.updateViewport();
      VirtualGrid.lastRenderedRange = null;
      VirtualGrid.requestRange();
    }
  });

  function initCollab(scopeBody) {
    if (!window.GBCollab) return;
    var inst = activeInstance(scopeBody);
    var gridFn = function () { return activeGrid(inst || scopeBody); };
    // Avoid reconnecting global collab transport per window in a way that breaks
    // existing single-connection behavior; still scope doc id and DOM targets.
    window.GBCollab.connect({
      app: "sheet",
      docId: D.currentSheetId(inst),
      collaboratorsEl: scopeEl("collaborators", inst || scopeBody),
      opLogUrl: function (docId, since) {
        return "/api/sheet/" + encodeURIComponent(docId) + "/ops?since=" + since;
      },
      opSeqOf: function (op) { return op.seq; },
      onConnect: function () {},
      onDisconnect: function () {
        if (window.SheetPresence) window.SheetPresence.clearAll();
      },
      onMessage: function (msg) {
        if (!msg || !window.SheetPresence) return;
        if (msg.msg_type === "cursor" || msg.msg_type === "cell_select") {
          window.SheetPresence.cursor(msg);
        }
      },
      onTyping: function (msg) {
        if (!msg || !window.SheetPresence) return;
        if (msg.msg_type === "typing_start") window.SheetPresence.typing(msg);
        else if (msg.msg_type === "typing_stop") window.SheetPresence.clearTyping(msg.user_id);
      },
      onPresence: function (users) {
        if (window.SheetPresence) window.SheetPresence.sync(users);
      },
      onEdit: function (msg) {
        var grid = gridFn();
        if (!msg || !grid) return;
        if (msg.payload !== undefined && msg.seq !== undefined) {
          const pl = msg.payload || {};
          msg.row = pl.row !== undefined ? pl.row : msg.row;
          msg.col = pl.col !== undefined ? pl.col : msg.col;
          msg.content = pl.value !== undefined ? pl.value : msg.content;
        }
        let row, col;
        if (msg.row !== undefined && msg.col !== undefined) {
          row = msg.row; col = msg.col;
        } else if (msg.position !== undefined) {
          row = Math.floor(msg.position / (grid.totalCols || 1));
          col = msg.position % (grid.totalCols || 1);
        } else {
          return;
        }
        const key = row + "," + col;
        const existing = grid.cells.get(key) || {};
        grid.cells.set(key, Object.assign({}, existing, { value: msg.content }));
        if (grid.lastRenderedRange) {
          const parts = String(grid.lastRenderedRange).split("|")[0].split(":");
          const start = parseInt(parts[0], 10), end = parseInt(parts[1], 10);
          if (row >= start && row < end) {
            const el = grid.bodyInner && grid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + col + '"]');
            if (el) el.textContent = msg.content || "";
          }
        }
      }
    });
  }

  function initAuth(scopeBody) {
    if (window.GBAuthGuard) window.GBAuthGuard.injectLoginButton(scopeEl("gb-auth-button", scopeBody));
  }

  function bootSheet() {
    var myBody = document.currentScript ? document.currentScript.closest('[id^="window-body-"]') : null;
    if (!myBody) {
      var els = document.querySelectorAll('#sheet-content');
      for (var i = 0; i < els.length; i++) {
        var p = els[i].closest('[id^="window-body-"]');
        if (p && !p.dataset.__sheetBooted) { myBody = p; break; }
      }
    }
    var winId = myBody ? (myBody.dataset.windowId || myBody.id.replace('window-body-', '')) : null;
    var inst = winId && D.ensureSheetInstance ? D.ensureSheetInstance(winId) : (D.getActiveSheetInstance ? D.getActiveSheetInstance() : null);
    if (inst) {
      inst.body = myBody || inst.body;
      if (D.setActiveSheetInstance) D.setActiveSheetInstance(inst);
    }
    var myData = (winId && window.__SHEET_DATA_MAP) ? window.__SHEET_DATA_MAP[winId] : null;

    initSidebar(myBody || inst);
    initAuth(myBody || inst);
    initCollab(myBody || inst);
    initToolbar(myBody || inst);

    // Prefer per-window boot promise/data to avoid cross-window hijack.
    var bootPromise = (inst && inst.bootPromise) || (myData && myData.boot) || window.__SHEET_BOOT || Promise.resolve();
    bootPromise.then(function () {
      var host = myBody ? myBody.querySelector('#sheet-content') : (inst && inst.body ? inst.body.querySelector('#sheet-content') : document.getElementById("sheet-content"));
      if (inst) inst.body = myBody || inst.body;
      var loadError = (inst && inst.loadError) || (myData && myData.loadError ? { message: myData.loadError, bucket: myData.requestedBucket || '', path: myData.requestedPath || '' } : null) || window.__SHEET_LOAD_ERROR;
      if (host && loadError) {
        gbSheetShowLoadError(host, loadError);
        if (myBody) myBody.dataset.__sheetBooted = '1';
        return;
      }
      if (host) {
        var grid = (inst && inst.grid) || activeGrid(inst);
        // Ensure this window has its own grid instance.
        if (!inst || !inst.grid || grid === window.SheetVirtualGrid && (window.__SHEET_ACTIVE_WINDOW_ID__ && winId && window.__SHEET_ACTIVE_WINDOW_ID__ !== winId)) {
          if (D.createVirtualGrid) {
            grid = D.createVirtualGrid();
            if (inst) { inst.grid = grid; grid.instance = inst; }
          }
        }
        if (inst) {
          inst.grid = grid || inst.grid;
          if (inst.grid) inst.grid.instance = inst;
        }
        if (!grid) return;
        if (inst && D.setActiveSheetInstance) D.setActiveSheetInstance(inst);
        grid.init(host);
        if (inst && D.setActiveSheetInstance) D.setActiveSheetInstance(inst);

        var loaded = (inst && inst.loadedSheet) || (myData && myData.loadedSheet) || window.__LOADED_SHEET;
        if (loaded && loaded.worksheets && loaded.worksheets.length > 0) {
          var savedIndex = 0;
          try {
            savedIndex = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10) || 0;
          } catch (_) {
            savedIndex = 0;
          }
          if (inst && typeof inst.worksheetIndex === "number") savedIndex = inst.worksheetIndex;
          if (savedIndex < 0 || savedIndex >= loaded.worksheets.length) savedIndex = 0;
          D.WORKSHEET_INDEX = savedIndex;
          if (inst) inst.worksheetIndex = savedIndex;
          try { sessionStorage.setItem("sheet_active_index", String(savedIndex)); } catch (_) {}
          var ws = loaded.worksheets[savedIndex];
          if (ws.data) {
            for (var cellRef in ws.data) {
              grid.cells.set(cellRef, ws.data[cellRef]);
            }
          }
          grid.requestSeq++;
          grid.totalRows = Math.max(grid.totalRows, 50);
          grid.render();
        }
        if (window.SheetAdvanced && window.SheetAdvanced.init) {
          var sid = (host.dataset.sheetId || (inst && inst.sheetId) || (loaded && loaded.id) || window.__SHEET_INITIAL_ID || "current");
          var adv = window.SheetAdvanced.init(host, { sheetId: sid });
          if (adv) {
            if (inst) inst.advanced = adv; else window.SheetAdvancedInstance = adv;
          }
        }
      }
      var bootCell = (inst && inst.bootCell) || (myData && myData.bootCell) || window.__SHEET_BOOT_CELL;
      if (bootCell && typeof gbSheetGotoCell === 'function') {
        if (inst && D.setActiveSheetInstance) D.setActiveSheetInstance(inst);
        gbSheetGotoCell(bootCell);
      }
      if (inst) inst.bootCell = null;
      if (!inst) window.__SHEET_BOOT_CELL = null;
      window.SheetAPI = SheetAPI;
      if (inst && inst.grid) window.SheetVirtualGrid = inst.grid;
      if (myBody) myBody.dataset.__sheetBooted = '1';
    });
  }

  if (document.readyState === "loading") {
    (function(){ var __cb = bootSheet; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    bootSheet();
  }
})();
