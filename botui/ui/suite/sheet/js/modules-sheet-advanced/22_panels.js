"use strict";
/* Sheet advanced module: 22_panels — protection, chart delete, import and pivot UI */

(function () {
  var CELL_LIMIT = 200;

  function accessToken() {
    try {
      if (typeof window.getGBAccessToken === "function") {
        var token = window.getGBAccessToken();
        if (token) return token;
      }
    } catch (_) {}
    return (
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") ||
      ""
    );
  }

  // Kept local: uses the sheet module's accessToken() helper.
  function authHeaders(extra) {
    var headers = extra || {};
    var token = accessToken();
    if (token) headers.Authorization = "Bearer " + token;
    return headers;
  }

  function liveSheetId() {
    return (
      (window.__LOADED_SHEET && window.__LOADED_SHEET.id) ||
      window.__SHEET_INITIAL_ID ||
      "current"
    );
  }

  function label(key, fallback) {
    if (typeof window._t === "function") {
      var text = window._t(key, fallback);
      if (text && text !== key) return text;
    }
    return fallback;
  }

  function notify(message, isError) {
    if (typeof window.sheetNotify === "function") {
      window.sheetNotify(message, isError);
      return;
    }
    console[isError ? "error" : "log"](message);
  }

  function parseJson(response) {
    return response
      .json()
      .catch(function () {
        return {};
      })
      .then(function (data) {
        if (!response.ok) {
          throw new Error((data && (data.error || data.message)) || "HTTP " + response.status);
        }
        return data;
      });
  }

  function postJson(url, body) {
    return fetch(url, {
      method: "POST",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify(body),
    }).then(parseJson);
  }

  function loadFreshSheet() {
    var id = liveSheetId();
    return fetch("/api/sheet/load?id=" + encodeURIComponent(id), {
      headers: authHeaders(),
    })
      .then(parseJson)
      .then(function (sheet) {
        window.__LOADED_SHEET = sheet;
        if (sheet && sheet.id) window.__SHEET_INITIAL_ID = sheet.id;
        return sheet;
      });
  }

  // Re-render a side panel from a freshly loaded Spreadsheet. Panels replaced
  // #sheet-content, so refreshing them never needs to touch the grid.
  function refreshPanel(name) {
    return loadFreshSheet()
      .then(function (sheet) {
        return fetch("/suite/sheet/fragments/" + name, {
          method: "POST",
          headers: authHeaders({ "Content-Type": "application/json" }),
          body: JSON.stringify(sheet),
        });
      })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.text();
      })
      .then(function (html) {
        var host = document.getElementById("sheet-content");
        if (host) host.innerHTML = html;
      });
  }

  function protectSheet(sheetId, worksheetIndex, protect) {
    var url = protect ? "/api/sheet/protect" : "/api/sheet/unprotect";
    var body = protect
      ? {
          sheet_id: sheetId,
          worksheet_index: worksheetIndex,
          protection: { protected: true },
        }
      : { sheet_id: sheetId, worksheet_index: worksheetIndex };
    return postJson(url, body)
      .then(function () {
        notify(label("protection.done", "Sheet protection updated"), false);
        return refreshPanel("protection");
      })
      .catch(function (error) {
        notify(label("protection.failed", "Protection change failed: ") + error.message, true);
      });
  }

  function deleteChart(worksheetIndex, chartId) {
    return postJson("/api/sheet/chart/delete", {
      sheet_id: liveSheetId(),
      worksheet_index: worksheetIndex,
      chart_id: chartId,
    })
      .then(function () {
        notify(label("charts.deleted", "Chart deleted"), false);
        return refreshPanel("charts");
      })
      .catch(function (error) {
        notify(label("charts.delete_failed", "Could not delete the chart: ") + error.message, true);
      });
  }

  function importFile(input) {
    var file = input && input.files && input.files[0];
    if (!file) return;
    var form = new FormData();
    form.append("file", file);
    fetch("/api/sheet/import", {
      method: "POST",
      headers: authHeaders(),
      body: form,
    })
      .then(parseJson)
      .then(function (sheet) {
        window.__LOADED_SHEET = sheet;
        if (sheet && sheet.id) window.__SHEET_INITIAL_ID = sheet.id;
        var nameInput = document.getElementById("sheetName");
        if (nameInput && sheet && sheet.name) nameInput.value = sheet.name;
        notify(label("import.done", "Workbook imported"), false);
        if (sheet && sheet.id && typeof window.loadSheet === "function") {
          window.loadSheet(sheet.id);
        }
      })
      .catch(function (error) {
        notify(label("import.failed", "Import failed: ") + error.message, true);
      })
      .then(function () {
        input.value = "";
      });
  }

  function escapeHtml(value) {
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function colName(index) {
    var n = index + 1;
    var name = "";
    while (n > 0) {
      var remainder = (n - 1) % 26;
      name = String.fromCharCode(65 + remainder) + name;
      n = Math.floor((n - 1) / 26);
    }
    return name;
  }

  function defaultRange() {
    var selection =
      window.SheetAdvanced && window.SheetAdvanced.getSelection
        ? window.SheetAdvanced.getSelection()
        : null;
    if (!selection) return "A1:C10";
    var single =
      selection.startRow === selection.endRow && selection.startCol === selection.endCol;
    if (single) return "A1:C10";
    return (
      colName(selection.startCol) +
      (selection.startRow + 1) +
      ":" +
      colName(selection.endCol) +
      (selection.endRow + 1)
    );
  }

  function fieldRow(key, fallback, name, placeholder, required) {
    return (
      '<label style="color:#f8fafc;font-size:13px;display:flex;flex-direction:column;gap:4px;">' +
      escapeHtml(label(key, fallback)) +
      '<input type="text" name="' +
      name +
      '" placeholder="' +
      escapeHtml(placeholder) +
      '"' +
      (required ? " required" : "") +
      ' style="padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;" />' +
      "</label>"
    );
  }

  function openPivot() {
    if (!window.__LOADED_SHEET) {
      notify(label("sidebar.load_first", "Load a spreadsheet first."), true);
      return;
    }
    var host = document.getElementById("sheet-modal-container") || document.body;
    var existing = document.getElementById("pivot-modal");
    if (existing) existing.remove();
    var modal = document.createElement("div");
    modal.className = "ss-modal";
    modal.id = "pivot-modal";
    modal.style.cssText =
      "position:fixed;inset:0;background:rgba(0,0,0,.7);z-index:1000;display:flex;align-items:center;justify-content:center;";
    modal.innerHTML =
      '<div class="ss-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:640px;width:92%;max-height:86vh;overflow:auto;">' +
      '<div class="ss-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">' +
      '<h3 style="margin:0;color:#f8fafc;font-size:16px;">' +
      escapeHtml(label("pivot.title", "Pivot Table")) +
      "</h3>" +
      '<button type="button" data-pivot-close="1" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>' +
      "</div>" +
      '<form id="pivot-form" style="padding:16px;display:flex;flex-direction:column;gap:10px;">' +
      fieldRow("pivot.source_range", "Source range", "source_range", "A1:C10", true) +
      fieldRow("pivot.rows", "Rows (comma-separated headers)", "rows", "Category", false) +
      fieldRow("pivot.cols", "Columns (optional)", "cols", "Region", false) +
      fieldRow("pivot.values", "Values (header)", "value_field", "Amount", true) +
      '<label style="color:#f8fafc;font-size:13px;display:flex;flex-direction:column;gap:4px;">' +
      escapeHtml(label("pivot.agg", "Aggregation")) +
      '<select name="agg" style="padding:8px;background:#0f172a;border:1px solid #334155;border-radius:4px;color:#f8fafc;">' +
      '<option value="SUM">SUM</option><option value="COUNT">COUNT</option>' +
      '<option value="AVG">AVG</option><option value="MIN">MIN</option><option value="MAX">MAX</option>' +
      "</select></label>" +
      '<button type="submit" style="background:#3b82f6;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">' +
      escapeHtml(label("pivot.run", "Run pivot")) +
      "</button>" +
      '<div id="pivot-result" style="min-height:48px;color:#cbd5e1;font-size:13px;"></div>' +
      "</form>" +
      "</div>";
    host.appendChild(modal);
    var form = modal.querySelector("#pivot-form");
    if (form && form.elements.source_range) form.elements.source_range.value = defaultRange();
    modal.querySelectorAll("[data-pivot-close]").forEach(function (button) {
      button.addEventListener("click", function () {
        modal.remove();
      });
    });
    if (form) {
      form.addEventListener("submit", function (event) {
        event.preventDefault();
        runPivot(modal);
      });
    }
  }

  function csvList(value) {
    return String(value || "")
      .split(",")
      .map(function (part) {
        return part.trim();
      })
      .filter(Boolean);
  }

  function formatValue(value) {
    if (typeof value === "number") return value.toLocaleString();
    if (value === null || value === undefined) return "";
    return escapeHtml(value);
  }

  function displayKey(value) {
    return escapeHtml(String(value || "").split("\u0000").join(" / ")) || "(blank)";
  }

  function renderPivot(container, result, valueLabel) {
    if (!result) {
      container.textContent = label("pivot.empty", "No result.");
      return;
    }
    var rowKeys = result.rowKeys || [];
    var colKeys = result.colKeys || [];
    var cells = result.cells || {};
    var rowTotals = result.rowTotals || {};
    var colTotals = result.colTotals || {};
    var shownRows = rowKeys.slice(0, CELL_LIMIT);
    var shownCols = colKeys.slice(0, CELL_LIMIT);
    var cellValue = function (rowKey, colKey) {
      return formatValue(cells[rowKey + "\u0000" + colKey + "\u0000" + 0]);
    };
    var th = 'padding:6px 8px;border-bottom:1px solid #334155;text-align:left;color:#f8fafc;';
    var td = "padding:6px 8px;border-bottom:1px solid #334155;color:#cbd5e1;";
    var html = '<div style="overflow:auto;max-height:50vh;"><table style="border-collapse:collapse;width:100%;font-size:12px;">';
    html += "<tr><th style=\"" + th + "\">" + escapeHtml(valueLabel) + "</th>";
    if (shownCols.length === 0) {
      html += '<th style="' + th + '">' + escapeHtml(valueLabel) + "</th>";
    }
    shownCols.forEach(function (colKey) {
      html += '<th style="' + th + '">' + displayKey(colKey) + "</th>";
    });
    html += '<th style="' + th + '">' + escapeHtml(label("pivot.total", "Total")) + "</th></tr>";
    shownRows.forEach(function (rowKey) {
      html += "<tr><th style=\"" + th + '">' + displayKey(rowKey) + "</th>";
      if (shownCols.length === 0) {
        html += '<td style="' + td + '">' + cellValue(rowKey, "") + "</td>";
      }
      shownCols.forEach(function (colKey) {
        html += '<td style="' + td + '">' + cellValue(rowKey, colKey) + "</td>";
      });
      html += '<td style="' + td + '">' + formatValue(rowTotals[rowKey]) + "</td></tr>";
    });
    html += '<tr><th style="' + th + '">' + escapeHtml(label("pivot.total", "Total")) + "</th>";
    if (shownCols.length === 0) {
      html += '<td style="' + td + '"></td>';
    }
    shownCols.forEach(function (colKey) {
      html += '<td style="' + td + '">' + formatValue(colTotals[colKey]) + "</td>";
    });
    html += '<td style="' + td + '">' + formatValue(result.grandTotal || 0) + "</td></tr>";
    html += "</table></div>";
    if (rowKeys.length > CELL_LIMIT || colKeys.length > CELL_LIMIT) {
      html +=
        '<div style="margin-top:8px;color:#f59e0b;font-size:12px;">' +
        escapeHtml(label("pivot.truncated", "Showing the first 200 rows/columns.")) +
        "</div>";
    }
    container.innerHTML = html;
  }

  function runPivot(modal) {
    var form = modal.querySelector("#pivot-form");
    var resultBox = modal.querySelector("#pivot-result");
    var submit = form.querySelector('button[type="submit"]');
    var body = {
      sheet_id: liveSheetId(),
      source_range: form.elements.source_range.value.trim(),
      rows: csvList(form.elements.rows.value),
      cols: csvList(form.elements.cols.value),
      values: [
        {
          field: form.elements.value_field.value.trim(),
          agg: form.elements.agg.value,
        },
      ],
    };
    if (!body.values[0].field) {
      resultBox.textContent = label("pivot.need_value", "Enter the header of the values column.");
      return;
    }
    submit.disabled = true;
    resultBox.textContent = label("pivot.running", "Calculating…");
    postJson("/api/sheet/pivot", body)
      .then(function (data) {
        renderPivot(resultBox, data && data.result, body.values[0].field);
      })
      .catch(function (error) {
        resultBox.textContent = label("pivot.failed", "Pivot failed: ") + error.message;
      })
      .then(function () {
        submit.disabled = false;
      });
  }

  window.GBSheetRefreshPanel = refreshPanel;
  window.GBSheetRefreshProtection = function () {
    return refreshPanel("protection");
  };
  window.GBSheetRefreshCharts = function () {
    return refreshPanel("charts");
  };
  window.GBSheetProtectSheet = protectSheet;
  window.GBSheetDeleteChart = deleteChart;
  window.GBSheetImportFile = importFile;
  window.GBSheetPivotOpen = openPivot;
})();
