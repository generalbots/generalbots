"use strict";
/* sheet boot — toolbar, collaboration, auth and startup */

(function () {
  var D = window.SheetDefaults;
  var colName = D.colName;
  var initSidebar = D.initSidebar;
  var SheetAPI = D.SheetAPI;
  const VirtualGrid = D.virtualGrid;

  // Toolbar format button handlers
  /* Formats the selected cell with a payload that can carry either a `style`
     object or a `format` string, then mirrors it in the local cell cache so the
     grid repaints immediately (#2184). */
  function formatSelectedRange(payload) {
    var row = VirtualGrid.selectedRow;
    var col = VirtualGrid.selectedCol;
    if (row == null || col == null) return Promise.resolve(null);
    var body = {
      sheet_id: D.currentSheetId(),
      worksheet_index: D.WORKSHEET_INDEX,
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
        var existing = VirtualGrid.cells.get(key) || {};
        if (payload.format !== undefined) existing.format = payload.format;
        if (payload.style) {
          existing.style = Object.assign({}, existing.style || {}, payload.style);
        }
        VirtualGrid.cells.set(key, existing);
        VirtualGrid.lastRenderedRange = null;
        VirtualGrid.requestRange();
      })
      .catch(function () {});
  }

  function initToolbar() {
    var boldBtn = document.getElementById("boldBtn");
    if (boldBtn) {
      boldBtn.addEventListener("click", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ font_weight: "bold" });
      });
    }
    var italicBtn = document.getElementById("italicBtn");
    if (italicBtn) {
      italicBtn.addEventListener("click", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ font_style: "italic" });
      });
    }
    var underlineBtn = document.getElementById("underlineBtn");
    if (underlineBtn) {
      underlineBtn.addEventListener("click", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ text_decoration: "underline" });
      });
    }
    var strikeBtn = document.getElementById("strikeBtn");
    if (strikeBtn) {
      strikeBtn.addEventListener("click", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ text_decoration: "line-through" });
      });
    }
    var fontSizeInput = document.getElementById("fontSizeInput");
    if (fontSizeInput) {
      var parseFontSize = function (raw) {
        var text = String(raw == null ? "" : raw).trim().toLowerCase();
        var m = text.match(/(\d+(?:\.\d+)?)/);
        if (!m) return null;
        var n = Math.round(parseFloat(m[1]));
        if (!isFinite(n) || n < 1 || n > 200) return null;
        return n;
      };
      var applyFontSize = function () {
        var size = parseFontSize(fontSizeInput.value);
        if (size == null) return;
        fontSizeInput.value = String(size);
        VirtualGrid.formatSelectedCell({ font_size: size });
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
    var fillBtn = document.getElementById("fillColorBtn");
    if (fillBtn) {
      fillBtn.addEventListener("change", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ background: fillBtn.value });
      });
    }
    var textBtn = document.getElementById("textColorBtn");
    if (textBtn) {
      textBtn.addEventListener("change", function (e) {
        e.preventDefault();
        VirtualGrid.formatSelectedCell({ color: textBtn.value });
      });
    }
    // #2184 — alignment belongs to the formatting row of the two-row toolbar.
    Array.prototype.forEach.call(
      document.querySelectorAll("#alignLeftBtn, #alignCenterBtn, #alignRightBtn"),
      function (btn) {
        btn.addEventListener("click", function (e) {
          e.preventDefault();
          var align = btn.getAttribute("data-align") || "left";
          document.querySelectorAll("#alignLeftBtn, #alignCenterBtn, #alignRightBtn")
            .forEach(function (other) {
              other.setAttribute("aria-pressed", other === btn ? "true" : "false");
            });
          VirtualGrid.formatSelectedCell({ text_align: align });
        });
      },
    );
    // #2184 — one-click number formats (currency, percent, decimals, general).
    var NUMBER_FORMATS = {
      currency: "R$ #,##0.00",
      percent: "0.00%",
      decimal: "0.00",
      general: "",
    };
    Array.prototype.forEach.call(
      document.querySelectorAll("#currencyBtn, #percentBtn, #decimalBtn, #clearFormatBtn"),
      function (btn) {
        btn.addEventListener("click", function (e) {
          e.preventDefault();
          var key = btn.getAttribute("data-number-format") || "general";
          formatSelectedRange({ format: NUMBER_FORMATS[key] || "" });
        });
      },
    );
    var formulaInput = document.getElementById("formulaInput");
    if (formulaInput) {
      var commitFormula = function () {
        var val = formulaInput.value;
        if (VirtualGrid.selectedRow == null || VirtualGrid.selectedCol == null) return;
        var ref = colName(VirtualGrid.selectedCol) + (VirtualGrid.selectedRow + 1);
        SheetAPI.updateCell(ref, val).then(function (res) {
          if (res && res.success) {
            VirtualGrid.lastRenderedRange = null;
            VirtualGrid.requestRange();
          }
        });
      };
      formulaInput.addEventListener("keydown", function (e) {
        if (e.key === "Enter") {
          e.preventDefault();
          commitFormula();
        }
      });
      // Clicking away used to discard the typed value silently.
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

  function initCollab() {
    if (!window.GBCollab) return;
    window.GBCollab.connect({
      app: "sheet",
      docId: D.currentSheetId(),
      collaboratorsEl: document.getElementById("collaborators"),
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
        if (!msg) return;
        // Server-oplog replay (#789): ops from the catch-up endpoint carry the
        // payload under `payload` plus the assigned `seq`.
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
          row = Math.floor(msg.position / VirtualGrid.totalCols);
          col = msg.position % VirtualGrid.totalCols;
        } else {
          return;
        }
        const key = row + "," + col;
        const existing = VirtualGrid.cells.get(key) || {};
        VirtualGrid.cells.set(key, Object.assign({}, existing, { value: msg.content }));
        if (VirtualGrid.lastRenderedRange) {
          const parts = VirtualGrid.lastRenderedRange.split(":");
          const start = parseInt(parts[0], 10), end = parseInt(parts[1], 10);
          if (row >= start && row < end) {
            const el = VirtualGrid.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + col + '"]');
            if (el) el.textContent = msg.content || "";
          }
        }
      }
    });
  }

  function initAuth() {
    if (window.GBAuthGuard) window.GBAuthGuard.injectLoginButton(document.getElementById("gb-auth-button"));
  }

  function bootSheet() {
    // Discover per-window sheet data from __SHEET_DATA_MAP
    var myData = null;
    var myBody = document.currentScript ? document.currentScript.closest('[id^="window-body-"]') : null;
    if (!myBody) {
      // Fallback: find the visible sheet-content and climb
      var els = document.querySelectorAll('#sheet-content');
      for (var i = 0; i < els.length; i++) {
        var p = els[i].closest('[id^="window-body-"]');
        if (p && p.style.display !== 'none') { myBody = p; break; }
      }
    }
    if (myBody && myBody.dataset.windowId) {
      myData = (window.__SHEET_DATA_MAP || {})[myBody.dataset.windowId];
    }

    initSidebar();
    initAuth();
    initCollab();
    initToolbar();
    var bootPromise = window.__SHEET_BOOT || Promise.resolve();
    bootPromise.then(function () {
      var host = myBody ? myBody.querySelector('#sheet-content') : document.getElementById("sheet-content");
      // #2269 — a failed load shows its error instead of a blank, writable grid.
      if (host && window.__SHEET_LOAD_ERROR) {
        gbSheetShowLoadError(host, window.__SHEET_LOAD_ERROR);
        return;
      }
      if (host) {
        VirtualGrid.init(host);
        var loaded = myData && myData.loadedSheet ? myData.loadedSheet : window.__LOADED_SHEET;
        if (loaded && loaded.worksheets && loaded.worksheets.length > 0) {
          // Hydrate the worksheet the user was on; worksheet 0 was shown even
          // when sheet_active_index pointed at another tab, so edits landed on
          // a sheet the grid was not displaying.
          var savedIndex = 0;
          try {
            savedIndex = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10) || 0;
          } catch (_) {
            savedIndex = 0;
          }
          if (savedIndex < 0 || savedIndex >= loaded.worksheets.length) savedIndex = 0;
          D.WORKSHEET_INDEX = savedIndex;
          try { sessionStorage.setItem("sheet_active_index", String(savedIndex)); } catch (_) {}
          var ws = loaded.worksheets[savedIndex];
          if (ws.data) {
            for (var cellRef in ws.data) {
              VirtualGrid.cells.set(cellRef, ws.data[cellRef]);
            }
          }
          VirtualGrid.requestSeq++;
          VirtualGrid.totalRows = Math.max(VirtualGrid.totalRows, 50);
          VirtualGrid.render();
        }
        if (window.SheetAdvanced && window.SheetAdvanced.init) {
          // "current" here used to overwrite the id of the Drive/manifest
          // sheet that had just loaded, so every fetch-based control (format,
          // sort, filter, charts...) mutated a different, blank workbook.
          var adv = window.SheetAdvanced.init(host, {
            sheetId: (host.dataset.sheetId || window.__SHEET_INITIAL_ID ||
              (window.__LOADED_SHEET && window.__LOADED_SHEET.id) || "current")
          });
          if (adv) window.SheetAdvancedInstance = adv;
        }
      }
      // A deep link that names a cell (`app://sheet?file_id=…&cell=B7`) lands
      // on it once the workbook is rendered (#2271).
      if (window.__SHEET_BOOT_CELL && typeof gbSheetGotoCell === 'function') {
        gbSheetGotoCell(window.__SHEET_BOOT_CELL);
      }
      window.__SHEET_BOOT_CELL = null;
      window.SheetAPI = SheetAPI;
      window.SheetVirtualGrid = VirtualGrid;
    });
  }

  if (document.readyState === "loading") {
    (function(){ var __cb = bootSheet; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();
  } else {
    bootSheet();
  }
})();
