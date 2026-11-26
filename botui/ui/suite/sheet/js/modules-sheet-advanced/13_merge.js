"use strict";
/* Sheet advanced module: 13_merge — merge/unmerge the real selection and render merged regions */

(function () {
  function showToast(msg, ok) {
    if (window.GBToasts && typeof window.GBToasts.show === "function") {
      window.GBToasts.show("Sheets", msg, ok === false ? "error" : "info");
      return;
    }
    if (typeof window.showToast === "function") {
      window.showToast(msg, ok);
      return;
    }
    console.info(msg);
  }
  let wrapped = false;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function currentSheetId() {
    if (window.SheetCore && window.SheetCore.currentSheetId) return window.SheetCore.currentSheetId();
    return window.__SHEET_INITIAL_ID || "current";
  }

  function isMergeableId(id) {
    return !!id && id !== "current" && id !== "loading" && id !== "new";
  }

  function wsIndex() {
    if (window.SheetCore && window.SheetCore.wsIndex) return window.SheetCore.wsIndex();
    try {
      const v = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      return isNaN(v) || v < 0 ? 0 : v;
    } catch (_) {
      return 0;
    }
  }

  function selection() {
    const adv = window.SheetAdvanced;
    return adv && adv.getSelection ? adv.getSelection() : null;
  }

  function mergedRanges() {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[wsIndex()]) return [];
    return sheet.worksheets[wsIndex()].merged_cells || [];
  }

  function authHeaders(extra) {
    const headers = extra ? Object.assign({}, extra) : {};
    let token = "";
    try {
      if (typeof window.getGBAccessToken === "function") {
        token = window.getGBAccessToken() || "";
      }
      if (!token) {
        token =
          localStorage.getItem("gb-access-token") ||
          sessionStorage.getItem("gb-access-token") ||
          "";
      }
    } catch (_) {
      token = "";
    }
    if (token) headers.Authorization = "Bearer " + token;
    return headers;
  }

  function rangePayload() {
    const sel = selection();
    const g = grid();
    if (!sel || !g) return null;
    return {
      sheet_id: currentSheetId(),
      worksheet_index: wsIndex(),
      start_row: sel.startRow,
      start_col: sel.startCol,
      end_row: sel.endRow,
      end_col: sel.endCol,
    };
  }

  function readErrorMessage(response) {
    return response
      .text()
      .then(function (text) {
        if (!text) return "Request failed";
        try {
          const data = JSON.parse(text);
          return data.message || data.error || data.detail || text;
        } catch (_) {
          return text;
        }
      })
      .catch(function () {
        return "Request failed";
      });
  }

  function mergeSelection() {
    const payload = rangePayload();
    if (!payload || (payload.start_row === payload.end_row && payload.start_col === payload.end_col)) {
      showToast("Select a range of more than one cell to merge");
      return Promise.resolve();
    }
    if (!isMergeableId(payload.sheet_id)) {
      showToast((typeof window.gbT === "function" ? window.gbT("sheet-merge-no-sheet", "Save or open a sheet before merging cells") : "Save or open a sheet before merging cells"), false);
      return Promise.resolve();
    }
    return fetch("/api/sheet/merge", {
      method: "POST",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify(payload),
    })
      .then(function (r) {
        if (!r.ok) {
          return readErrorMessage(r).then(function (msg) {
            throw new Error(msg || "Merge failed");
          });
        }
        return r.json().catch(function () { return {}; });
      })
      .then(function () {
        return reloadFromServer(true);
      })
      .catch(function (err) {
        showToast(err && err.message ? err.message : "Merge failed", false);
      });
  }

  function unmergeSelection() {
    const payload = rangePayload();
    if (!payload) return Promise.resolve();
    if (!isMergeableId(payload.sheet_id)) {
      showToast((typeof window.gbT === "function" ? window.gbT("sheet-unmerge-no-sheet", "Save or open a sheet before unmerging cells") : "Save or open a sheet before unmerging cells"), false);
      return Promise.resolve();
    }
    return fetch("/api/sheet/unmerge", {
      method: "POST",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify(payload),
    })
      .then(function (r) {
        if (!r.ok) {
          return readErrorMessage(r).then(function (msg) {
            throw new Error(msg || "Unmerge failed");
          });
        }
        return r.json().catch(function () { return {}; });
      })
      .then(function () {
        return reloadFromServer(true);
      })
      .catch(function (err) {
        showToast(err && err.message ? err.message : "Unmerge failed", false);
      });
  }

  function reloadFromServer(rerender) {
    const a = window.SheetAPI;
    if (!a || typeof a.load !== "function") return Promise.resolve();
    const id = currentSheetId();
    if (!isMergeableId(id)) return Promise.resolve();
    return a.load(id)
      .then(function (sheet) {
        if (sheet) {
          window.__LOADED_SHEET = sheet;
          window.__SHEET_INITIAL_ID = sheet.id || id;
        }
        if (rerender && window.SheetCore && window.SheetCore.rehydrateGrid) {
          window.SheetCore.rehydrateGrid();
        }
      })
      .catch(function (err) {
        console.warn("[sheet-merge] reload after merge/unmerge failed", err);
        var msg = err && err.message
          ? err.message
          : (typeof gbT === "function" ? gbT("sheet-merge-refresh-failed", "Could not refresh sheet after merge") : "Could not refresh sheet after merge");
        showToast(msg, false);
      });
  }

  function showToast(msg, isError) {
    const id = "ss-merge-toast";
    let toast = document.getElementById(id);
    if (!toast) {
      toast = document.createElement("div");
      toast.id = id;
      toast.style.cssText = "position:fixed;bottom:24px;left:50%;transform:translateX(-50%);background:var(--id-danger);color:var(--id-surface-solid);padding:10px 18px;border-radius:6px;font-size:13px;z-index:var(--z-system);box-shadow:0 4px 12px color-mix(in srgb, var(--id-ink) 30%, transparent);";
      document.body.appendChild(toast);
    }
    toast.textContent = msg;
    toast.style.display = "block";
    clearTimeout(toast.__timer);
    toast.__timer = setTimeout(function () { toast.style.display = "none"; }, 2600);
  }

  // Span of a merged region in pixels, using the real (imported) geometry of
  // every covered column/row — a fixed 96 px per column shrank merged headers
  // and indicator boxes on any workbook with custom widths (#2152).
  function spanWidth(g, first, last) {
    let width = 0;
    for (let c = first; c <= last; c++) {
      width += g.colWidthOf ? g.colWidthOf(c) : 96;
    }
    return width;
  }

  function spanHeight(g, first, last) {
    let height = 0;
    for (let r = first; r <= last; r++) {
      if (g.rowHeightOf) {
        height += g.rowHeightOf(r);
      } else if (window.SheetCore && window.SheetCore.rowHeight) {
        height += window.SheetCore.rowHeight(r);
      } else {
        height += 24;
      }
    }
    return height;
  }

  function renderMerges() {
    const g = grid();
    const merges = mergedRanges();
    if (!g || !merges.length) return;
    const visible = g.visibleRowRange();
    for (let i = 0; i < merges.length; i++) {
      const m = merges[i];
      if (m.end_row < visible.start || m.start_row >= visible.end) continue;
      for (let r = m.start_row; r <= m.end_row; r++) {
        for (let c = m.start_col; c <= m.end_col; c++) {
          const node = g.bodyInner.querySelector('[data-row="' + r + '"][data-col="' + c + '"]');
          if (!node) continue;
          if (r === m.start_row && c === m.start_col) {
            node.style.display = "block";
            node.style.width = spanWidth(g, m.start_col, m.end_col) + "px";
            node.style.height = spanHeight(g, m.start_row, m.end_row) + "px";
            node.style.zIndex = "4";
            node.style.overflow = "hidden";
            var mergedCell = g.cells.get(r + "," + c);
            node.textContent = mergedCell && mergedCell.value !== null && mergedCell.value !== undefined
              ? String(mergedCell.value) : "";
          } else {
            node.style.display = "none";
          }
        }
      }
    }
  }

  function wrapRender() {
    const g = grid();
    if (!g || wrapped) return;
    wrapped = true;
    const orig = g.render.bind(g);
    g.render = function () {
      orig();
      renderMerges();
    };
  }

  function wire() {
    const g = grid();
    if (!g || !g.render) {
      setTimeout(wire, 100);
      return;
    }
    wrapRender();
    const host = document.getElementById("sheet-app");
    if (host && !host.__mergeBound) {
      host.__mergeBound = true;
      const btn = document.getElementById("mergeCellsBtn");
      if (btn) {
        btn.addEventListener("click", function (e) {
          e.preventDefault();
          e.stopPropagation();
          mergeSelection();
        });
      }
    }
  }

  window.SheetMerge = {
    merge: mergeSelection,
    unmerge: unmergeSelection,
    render: renderMerges,
  };

  if (window.SheetCore) {
    window.SheetCore.merge = mergeSelection;
  }

  setTimeout(wire, 0);
})();