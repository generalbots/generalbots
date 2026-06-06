"use strict";

// botui/ui/suite/sheet/modules/28_pivot_table.js
// Pivot Table — SERVER-ONLY. All aggregation lives in botserver's
// /api/sheet/pivot handler. The client only manages UI (zone assignment,
// drag-drop, render). If the server is unreachable, recompute() rejects
// with an error; the caller decides what to display.
//
// API contract:
//   POST /api/sheet/pivot
//   { sheet_id, config: { rows:[...], cols:[...], values:[{field,agg}], filter:[...] } }
//   -> { ok, result: { rowKeys, colKeys, cells, rowTotals, colTotals, grandTotal } }
//
// Public API: window.SheetPivotTable = { create, PivotTable, openModal }
(function () {
  function PivotTable(config) {
    this.config = config || {};
    this.sourceRange = this.config.sourceRange || "A1:D100";
    this.container = this.config.container;
    this.fields = (this.config.fields || []).slice();
    this.rows = (this.config.rows || []).slice();
    this.cols = (this.config.cols || []).slice();
    this.values = (this.config.values || []).slice();
    this.filter = (this.config.filter || []).slice();
    this.data = (this.config.data || []).slice();
    this.result = null;
    this.sheetId = this.config.sheetId || null;
    this.lastError = null;
  }

  PivotTable.prototype.getFieldNames = function () {
    if (this.fields.length > 0) return this.fields;
    if (this.data.length === 0) return [];
    return Object.keys(this.data[0]);
  };

  PivotTable.prototype.setData = function (rows) {
    this.data = rows || [];
  };

  PivotTable.prototype.addToZone = function (field, zone) {
    if (!field) return;
    const zones = { row: "rows", col: "cols", value: "values" };
    const target = zones[zone];
    if (!target) return;
    this.rows = this.rows.filter(function (f) { return f !== field; });
    this.cols = this.cols.filter(function (f) { return f !== field; });
    this.values = this.values.filter(function (f) { return f.field !== field; });
    if (zone === "value") {
      this.values.push({ field: field, agg: "SUM" });
    } else {
      this[target].push(field);
    }
    this.recompute();
  };

  PivotTable.prototype.removeFromZone = function (field, zone) {
    if (zone === "value") {
      this.values = this.values.filter(function (f) { return f.field !== field; });
    } else {
      const arr = zone === "row" ? this.rows : this.cols;
      const filtered = arr.filter(function (f) { return f !== field; });
      if (zone === "row") this.rows = filtered; else this.cols = filtered;
    }
    this.recompute();
  }

  PivotTable.prototype.changeAggregation = function (field, agg) {
    const v = this.values.filter(function (vv) { return vv.field === field; })[0];
    if (v) v.agg = agg;
    this.recompute();
  };

  PivotTable.prototype.recompute = function () {
    const self = this;
    const API = window.SheetAPI;
    if (!API) {
      const err = new Error("SheetAPI not loaded; cannot compute pivot without server");
      self.lastError = err;
      if (self._afterRecompute) self._afterRecompute();
      return Promise.reject(err);
    }
    if (!self.sheetId) {
      const err = new Error("PivotTable.sheetId not set; cannot call server");
      self.lastError = err;
      if (self._afterRecompute) self._afterRecompute();
      return Promise.reject(err);
    }
    if (self.values.length === 0) {
      self.result = { rowKeys: [], colKeys: [], cells: {}, rowTotals: {}, colTotals: {}, grandTotal: null };
      self.lastError = null;
      if (self._afterRecompute) self._afterRecompute();
      return Promise.resolve(self.result);
    }
    const req = {
      source_range: self.sourceRange,
      rows: self.rows,
      cols: self.cols,
      values: self.values,
      filter: self.filter,
    };
    return API.createPivot(self.sheetId, req).then(function (r) {
      if (!r || !r.ok) {
        const err = new Error("Pivot server returned error: " + ((r && r.error && r.error.message) || "unknown"));
        self.lastError = err;
        if (self._afterRecompute) self._afterRecompute();
        return Promise.reject(err);
      }
      const data = r.data || {};
      self.result = data.result || { rowKeys: [], colKeys: [], cells: {}, rowTotals: {}, colTotals: {}, grandTotal: null };
      self.lastError = null;
      if (self._afterRecompute) self._afterRecompute();
      return self.result;
    }).catch(function (err) {
      self.lastError = err;
      if (self._afterRecompute) self._afterRecompute();
      return Promise.reject(err);
    });
  };

  function formatNumber(n) {
    if (typeof n !== "number" || isNaN(n)) return "";
    if (Number.isInteger(n)) return String(n);
    return n.toFixed(2);
  }

  PivotTable.prototype.render = function () {
    if (!this.container) return;
    const fields = this.getFieldNames();
    let html = '<div class="pivot-container">';
    html += '<div class="pivot-fields"><h4>Fields</h4><ul>';
    for (let i = 0; i < fields.length; i++) {
      html += '<li class="pivot-field" draggable="true" data-field="' + fields[i] + '">' + fields[i] + '</li>';
    }
    html += '</ul></div>';
    html += '<div class="pivot-zones">';
    html += this.renderZone("Rows", "row", this.rows);
    html += this.renderZone("Columns", "col", this.cols);
    html += this.renderZone("Values", "value", this.values.map(function (v) { return v.field; }));
    html += "</div>";
    html += '<div class="pivot-table">';
    if (this.lastError) {
      html += '<div class="pivot-error" role="alert">' + escapeHtml(this.lastError.message) + '</div>';
    } else {
      html += this.renderResult();
    }
    html += "</div>";
    html += "</div>";
    this.container.innerHTML = html;
    this.bindDragDrop();
  };

  PivotTable.prototype.renderZone = function (label, zone, items) {
    let html = '<div class="pivot-zone" data-zone="' + zone + '"><h4>' + label + '</h4><ul>';
    for (let i = 0; i < items.length; i++) {
      const f = items[i];
      html += '<li class="pivot-zone-item" data-field="' + f + '" data-zone="' + zone + '">' + f;
      if (zone === "value") {
        const v = this.values.filter(function (vv) { return vv.field === f; })[0];
        const agg = v ? v.agg : "SUM";
        html += ' <span class="pivot-agg">(' + agg + ')</span>';
      }
      html += ' <button class="pivot-remove" data-remove-field="' + f + '" data-remove-zone="' + zone + '">x</button>';
      html += '</li>';
    }
    html += '</ul></div>';
    return html;
  };

  function escapeHtml(s) {
    return String(s).replace(/[&<>"']/g, function (c) {
      return ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c];
    });
  }

  PivotTable.prototype.renderResult = function () {
    if (!this.result) return '<div class="pivot-empty">Configure rows/columns/values and the server will compute.</div>';
    const r = this.result;
    if (!r.rowKeys || r.rowKeys.length === 0) return '<div class="pivot-empty">No data.</div>';
    let html = '<table class="pivot-grid"><thead><tr><th></th>';
    for (let i = 0; i < r.colKeys.length; i++) html += '<th>' + escapeHtml(r.colKeys[i]) + '</th>';
    if (r.colKeys.length > 0) html += '<th>Total</th>';
    html += '</tr></thead><tbody>';
    for (let i = 0; i < r.rowKeys.length; i++) {
      const rk = r.rowKeys[i];
      html += '<tr><th>' + escapeHtml(rk) + '</th>';
      for (let j = 0; j < r.colKeys.length; j++) {
        const key = rk + "\0" + r.colKeys[j] + "\0" + 0;
        html += '<td>' + formatNumber(r.cells[key]) + '</td>';
      }
      if (r.colKeys.length > 0) html += '<td>' + formatNumber(r.rowTotals[rk]) + '</td>';
      html += '</tr>';
    }
    if (r.rowKeys.length > 0 && r.colKeys.length > 0) {
      html += '<tr><th>Total</th>';
      for (let j = 0; j < r.colKeys.length; j++) html += '<td>' + formatNumber(r.colTotals[r.colKeys[j]]) + '</td>';
      html += '<td>' + formatNumber(r.grandTotal) + '</td></tr>';
    }
    html += '</tbody></table>';
    return html;
  };

  PivotTable.prototype.bindDragDrop = function () {
    const c = this.container;
    if (!c) return;
    const fields = c.querySelectorAll(".pivot-field");
    const zones = c.querySelectorAll(".pivot-zone");
    const self = this;
    for (let i = 0; i < fields.length; i++) {
      fields[i].addEventListener("dragstart", function (e) {
        e.dataTransfer.setData("text/plain", fields[i].dataset.field);
      });
    }
    for (let i = 0; i < zones.length; i++) {
      zones[i].addEventListener("dragover", function (e) { e.preventDefault(); });
      zones[i].addEventListener("drop", function (e) {
        e.preventDefault();
        const field = e.dataTransfer.getData("text/plain");
        const zone = zones[i].dataset.zone;
        if (field && zone) self.addToZone(field, zone);
      });
    }
    const removeBtns = c.querySelectorAll(".pivot-remove");
    for (let i = 0; i < removeBtns.length; i++) {
      removeBtns[i].addEventListener("click", function () {
        self.removeFromZone(removeBtns[i].dataset.removeField, removeBtns[i].dataset.removeZone);
      });
    }
  };

  function openModal() {
    let modal = document.getElementById("pivotTableModal");
    if (!modal) {
      modal = document.createElement("div");
      modal.id = "pivotTableModal";
      modal.className = "modal hidden";
      modal.setAttribute("role", "dialog");
      modal.setAttribute("aria-modal", "true");
      modal.setAttribute("aria-labelledby", "pivotTitle");
      modal.innerHTML = '<div class="modal-content">' +
        '<div class="modal-header">' +
        '<h2 id="pivotTitle">Pivot Table</h2>' +
        '<button id="closePivotTableModal" class="close-button" aria-label="Close">x</button>' +
        '</div>' +
        '<div class="modal-body">' +
        '<div class="form-group"><label>Source Range</label>' +
        '<input type="text" id="pivotSourceRange" placeholder="A1:D100" /></div>' +
        '<div id="pivotTableContainer"></div>' +
        '</div></div>';
      document.body.appendChild(modal);
      document.getElementById("closePivotTableModal").addEventListener("click", function () {
        modal.classList.add("hidden");
      });
    }
    modal.classList.remove("hidden");
    const container = document.getElementById("pivotTableContainer");
    if (!window._pivot) {
      window._pivot = new PivotTable({
        container: container,
        sheetId: (document.getElementById("sheetName") || {}).value || null,
        fields: ["Region", "Product", "Sales", "Quarter"],
      });
    }
    window._pivot.container = container;
    window._pivot.render();
  }

  window.SheetPivotTable = {
    create: function (config) { return new PivotTable(config); },
    PivotTable: PivotTable,
    openModal: openModal,
  };
})();
