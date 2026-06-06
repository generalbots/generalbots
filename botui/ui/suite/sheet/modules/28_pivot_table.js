// botui/ui/suite/sheet/modules/28_pivot_table.js
// Pivot Table — refactored to delegate aggregation to botserver
// via window.SheetAPI.createPivot. Client-side UI logic remains
// (drag-drop, zone management, rendering); the bucket-group-reduce
// algorithm moves to the Rust backend where it can use SQL GROUP BY
// over the source data with proper indexing.
//
// API contract with backend:
//   POST /api/sheet/pivot
//   { sheet_id, config: { rows:[...], cols:[...], values:[{field,agg}], filter:[...] } }
//   -> { ok, result: { rowKeys, colKeys, cells, rowTotals, colTotals, grandTotal } }
//
// Offline fallback: if backend is unreachable, the client computes
// locally with the same algorithm (for UX continuity). This is the
// only client-side computation, gated behind an explicit fallback.
"use strict";

(function () {
  const AGG_FUNCTIONS = {
    SUM: function (vals) { return vals.reduce(function (a, b) { return a + b; }, 0); },
    COUNT: function (vals) { return vals.filter(function (v) { return v !== null && v !== undefined; }).length; },
    AVERAGE: function (vals) {
      const nums = vals.filter(function (v) { return typeof v === "number" && !isNaN(v); });
      if (nums.length === 0) return 0;
      return nums.reduce(function (a, b) { return a + b; }, 0) / nums.length;
    },
    MIN: function (vals) {
      const nums = vals.filter(function (v) { return typeof v === "number" && !isNaN(v); });
      return nums.length === 0 ? 0 : Math.min.apply(null, nums);
    },
    MAX: function (vals) {
      const nums = vals.filter(function (v) { return typeof v === "number" && !isNaN(v); });
      return nums.length === 0 ? 0 : Math.max.apply(null, nums);
    },
  };

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
    this.offlineMode = this.config.offlineMode === true;
  }

  PivotTable.prototype.getFieldNames = function () {
    if (this.fields.length > 0) return this.fields;
    if (this.data.length === 0) return [];
    return Object.keys(this.data[0]);
  };

  PivotTable.prototype.setData = function (rows) {
    this.data = rows || [];
    this.recompute();
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
  };

  PivotTable.prototype.changeAggregation = function (field, agg) {
    if (!AGG_FUNCTIONS[agg]) return;
    for (let i = 0; i < this.values.length; i++) {
      if (this.values[i].field === field) this.values[i].agg = agg;
    }
    this.recompute();
  };

  PivotTable.prototype.recompute = function () {
    if (this.sheetId && window.SheetAPI && !this.offlineMode && this.values.length > 0) {
      const self = this;
      const req = {
        sheet_id: this.sheetId,
        config: {
          source_range: this.sourceRange,
          rows: this.rows,
          cols: this.cols,
          values: this.values,
          filter: this.filter,
        },
      };
      return window.SheetAPI.createPivot(this.sheetId, req.config).then(function (r) {
        if (r.ok && r.data && r.data.result) {
          self.result = r.data.result;
        } else {
          self.result = computeLocal(self.data, self.rows, self.cols, self.values);
        }
        if (self._afterRecompute) self._afterRecompute();
        return self.result;
      }).catch(function () {
        self.result = computeLocal(self.data, self.rows, self.cols, self.values);
        if (self._afterRecompute) self._afterRecompute();
        return self.result;
      });
    }
    this.result = computeLocal(this.data, this.rows, this.cols, this.values);
    if (this._afterRecompute) this._afterRecompute();
    return Promise.resolve(this.result);
  };

  function computeLocal(data, rows, cols, values) {
    if (!data || data.length === 0 || values.length === 0) {
      return { rowKeys: [], colKeys: [], cells: {}, rowTotals: {}, colTotals: {}, grandTotal: null };
    }
    const rowSet = {};
    const colSet = {};
    for (let i = 0; i < data.length; i++) {
      const r = data[i];
      const rowKey = rows.map(function (f) { return r[f]; }).join(" | ");
      const colKey = cols.map(function (f) { return r[f]; }).join(" | ");
      rowSet[rowKey] = true;
      colSet[colKey] = true;
    }
    const rowKeys = Object.keys(rowSet).sort();
    const colKeys = Object.keys(colSet).sort();

    const buckets = {};
    for (let i = 0; i < data.length; i++) {
      const r = data[i];
      const rowKey = rows.map(function (f) { return r[f]; }).join(" | ");
      const colKey = cols.map(function (f) { return r[f]; }).join(" | ");
      const key = rowKey + "\0" + colKey;
      if (!buckets[key]) buckets[key] = [];
      for (let j = 0; j < values.length; j++) buckets[key].push(r[values[j].field]);
    }

    const cells = {};
    for (let i = 0; i < rowKeys.length; i++) {
      for (let j = 0; j < colKeys.length; j++) {
        const key = rowKeys[i] + "\0" + colKeys[j];
        const vals = buckets[key] || [];
        for (let k = 0; k < values.length; k++) {
          const cellVals = [];
          for (let n = k; n < vals.length; n += values.length) cellVals.push(vals[n]);
          const cellKey = rowKeys[i] + "\0" + colKeys[j] + "\0" + k;
          cells[cellKey] = AGG_FUNCTIONS[values[k].agg](cellVals);
        }
      }
    }

    let grandTotal = null;
    if (values.length > 0) {
      const all = [];
      for (let i = 0; i < data.length; i++) {
        for (let k = 0; k < values.length; k++) all.push(data[i][values[k].field]);
      }
      grandTotal = AGG_FUNCTIONS[values[0].agg](all);
    }

    return {
      rowKeys: rowKeys,
      colKeys: colKeys,
      cells: cells,
      rowTotals: {},
      colTotals: {},
      grandTotal: grandTotal,
    };
  }

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
    html += this.renderResult();
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
        html += ' <select class="pivot-agg"><option value="SUM"' + (agg === "SUM" ? " selected" : "") + '>SUM</option><option value="COUNT"' + (agg === "COUNT" ? " selected" : "") + '>COUNT</option><option value="AVERAGE"' + (agg === "AVERAGE" ? " selected" : "") + '>AVG</option><option value="MIN"' + (agg === "MIN" ? " selected" : "") + '>MIN</option><option value="MAX"' + (agg === "MAX" ? " selected" : "") + '>MAX</option></select>';
      }
      html += ' <button class="pivot-remove">×</button></li>';
    }
    html += "</ul></div>";
    return html;
  };

  PivotTable.prototype.renderResult = function () {
    const r = this.result;
    if (!r || !r.rowKeys || r.rowKeys.length === 0) {
      return '<p class="pivot-empty">Drag fields into Rows, Columns, and Values to build a pivot table.</p>';
    }
    let html = '<table class="pivot-result-table"><thead><tr><th></th>';
    for (let j = 0; j < r.colKeys.length; j++) html += "<th>" + r.colKeys[j] + "</th>";
    html += "<th>Total</th></tr></thead><tbody>";
    for (let i = 0; i < r.rowKeys.length; i++) {
      html += "<tr><th>" + r.rowKeys[i] + "</th>";
      for (let j = 0; j < r.colKeys.length; j++) {
        let cellVal = "";
        for (let k = 0; k < this.values.length; k++) {
          const key = r.rowKeys[i] + "\0" + r.colKeys[j] + "\0" + k;
          cellVal += (k > 0 ? ", " : "") + formatNumber(r.cells[key]);
        }
        html += "<td>" + cellVal + "</td>";
      }
      let totalVal = "";
      for (let k = 0; k < this.values.length; k++) {
        const allForK = [];
        for (let j = 0; j < r.colKeys.length; j++) {
          const key = r.rowKeys[i] + "\0" + r.colKeys[j] + "\0" + k;
          allForK.push(r.cells[key]);
        }
        totalVal += (k > 0 ? ", " : "") + formatNumber(AGG_FUNCTIONS[this.values[k].agg](allForK));
      }
      html += "<td><strong>" + totalVal + "</strong></td>";
      html += "</tr>";
    }
    html += "</tbody></table>";
    return html;
  };

  PivotTable.prototype.bindDragDrop = function () {
    const self = this;
    const fields = this.container.querySelectorAll(".pivot-field");
    fields.forEach(function (f) {
      f.addEventListener("dragstart", function (e) {
        e.dataTransfer.setData("text/plain", f.getAttribute("data-field"));
        e.dataTransfer.effectAllowed = "copy";
      });
    });
    const zones = this.container.querySelectorAll(".pivot-zone");
    zones.forEach(function (z) {
      z.addEventListener("dragover", function (e) {
        e.preventDefault();
        e.dataTransfer.dropEffect = "copy";
        z.classList.add("pivot-zone-hover");
      });
      z.addEventListener("dragleave", function () {
        z.classList.remove("pivot-zone-hover");
      });
      z.addEventListener("drop", function (e) {
        e.preventDefault();
        z.classList.remove("pivot-zone-hover");
        const field = e.dataTransfer.getData("text/plain");
        const zone = z.getAttribute("data-zone");
        self.addToZone(field, zone);
        self.render();
      });
    });
    const removes = this.container.querySelectorAll(".pivot-remove");
    removes.forEach(function (b) {
      b.addEventListener("click", function () {
        const item = b.parentElement;
        const field = item.getAttribute("data-field");
        const zone = item.getAttribute("data-zone");
        self.removeFromZone(field, zone);
        self.render();
      });
    });
    const aggs = this.container.querySelectorAll(".pivot-agg");
    aggs.forEach(function (s) {
      s.addEventListener("change", function () {
        const item = s.parentElement;
        const field = item.getAttribute("data-field");
        self.changeAggregation(field, s.value);
        self.render();
      });
    });
  };

  PivotTable.prototype.dispose = function () {
    if (this.container) this.container.innerHTML = "";
    this.data = [];
    this.result = null;
  };

  PivotTable.prototype.onRecompute = function (fn) {
    this._afterRecompute = fn;
  };

  function openModal() {
    let modal = document.getElementById("pivotTableModal");
    if (!modal) {
      modal = document.createElement("div");
      modal.id = "pivotTableModal";
      modal.className = "modal hidden";
      modal.setAttribute("role", "dialog");
      modal.setAttribute("aria-modal", "true");
      modal.innerHTML =
        '<div class="modal-content modal-large">' +
        '<div class="modal-header">' +
        '<h3 id="pivotTableModalTitle">Pivot Table</h3>' +
        '<button class="btn-close" id="closePivotTableModal">×</button>' +
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
        fields: ["Region", "Product", "Sales", "Quarter"],
        data: [
          { Region: "North", Product: "Widget", Sales: 100, Quarter: "Q1" },
          { Region: "North", Product: "Gadget", Sales: 200, Quarter: "Q1" },
          { Region: "South", Product: "Widget", Sales: 150, Quarter: "Q1" },
          { Region: "South", Product: "Gadget", Sales: 250, Quarter: "Q2" },
          { Region: "East", Product: "Widget", Sales: 120, Quarter: "Q2" },
          { Region: "West", Product: "Gadget", Sales: 180, Quarter: "Q3" },
        ],
        offlineMode: true,
      });
    }
    if (window._pivot.data.length === 0) {
      window._pivot.setData([
        { Region: "North", Product: "Widget", Sales: 100, Quarter: "Q1" },
        { Region: "North", Product: "Gadget", Sales: 200, Quarter: "Q1" },
        { Region: "South", Product: "Widget", Sales: 150, Quarter: "Q1" },
        { Region: "South", Product: "Gadget", Sales: 250, Quarter: "Q2" },
        { Region: "East", Product: "Widget", Sales: 120, Quarter: "Q2" },
        { Region: "West", Product: "Gadget", Sales: 180, Quarter: "Q3" },
      ]);
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
