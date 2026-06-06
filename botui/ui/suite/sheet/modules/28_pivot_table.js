// botui/ui/suite/sheet/modules/28_pivot_table.js
// Pivot Table skeleton: drag fields into Rows/Columns/Values/Filter
// zones and aggregate. No UI framework — pure DOM rendering with
// drag-and-drop. Computes SUM, COUNT, AVERAGE, MIN, MAX.
//
// API:
//   window.SheetPivotTable.create({ sourceRange, container })
//     -> PivotTable instance
//   window.SheetPivotTable.openModal()
//   pivot.dispose()
//
// Storage of source data is delegated to the formula engine — the
// pivot reads from a range using the same A1 notation parser as
// 24b. For now, the sourceRange is a static array of objects
// passed in; integration with the live sheet can be added by
// calling pivot.setData(rows).
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
  }

  PivotTable.prototype.setData = function (rows) {
    this.data = rows || [];
    this.recompute();
  };

  PivotTable.prototype.getFieldNames = function () {
    if (this.fields.length > 0) return this.fields;
    if (this.data.length === 0) return [];
    const first = this.data[0];
    return Object.keys(first);
  };

  PivotTable.prototype.addToZone = function (field, zone) {
    if (!field) return;
    const zones = { row: "rows", col: "cols", value: "values" };
    const target = zones[zone];
    if (!target) return;
    // Remove from all other zones
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
      this[zone === "row" ? "rows" : "cols"] = (zone === "row" ? this.rows : this.cols).filter(function (f) { return f !== field; });
    }
    this.recompute();
  };

  PivotTable.prototype.changeAggregation = function (field, agg) {
    if (!AGG_FUNCTIONS[agg]) return;
    for (let i = 0; i < this.values.length; i++) {
      if (this.values[i].field === field) {
        this.values[i].agg = agg;
      }
    }
    this.recompute();
  };

  PivotTable.prototype.recompute = function () {
    if (this.data.length === 0) {
      this.result = { rowKeys: [], colKeys: [], cells: {}, rowTotals: {}, colTotals: {}, grandTotal: null };
      return this.result;
    }
    const rowSet = {};
    const colSet = {};
    for (let i = 0; i < this.data.length; i++) {
      const r = this.data[i];
      const rowKey = this.rows.map(function (f) { return r[f]; }).join(" | ");
      const colKey = this.cols.map(function (f) { return r[f]; }).join(" | ");
      rowSet[rowKey] = true;
      colSet[colKey] = true;
    }
    const rowKeys = Object.keys(rowSet).sort();
    const colKeys = Object.keys(colSet).sort();

    const buckets = {};
    for (let i = 0; i < this.data.length; i++) {
      const r = this.data[i];
      const rowKey = this.rows.map(function (f) { return r[f]; }).join(" | ");
      const colKey = this.cols.map(function (f) { return r[f]; }).join(" | ");
      const key = rowKey + "\0" + colKey;
      if (!buckets[key]) buckets[key] = [];
      for (let j = 0; j < this.values.length; j++) {
        buckets[key].push(r[this.values[j].field]);
      }
    }

    const cells = {};
    const rowTotals = {};
    const colTotals = {};
    let grandTotalAll = [];

    for (let i = 0; i < rowKeys.length; i++) {
      for (let j = 0; j < colKeys.length; j++) {
        const key = rowKeys[i] + "\0" + colKeys[j];
        const vals = buckets[key] || [];
        for (let k = 0; k < this.values.length; k++) {
          const cellVals = [];
          for (let n = k; n < vals.length; n += this.values.length) {
            cellVals.push(vals[n]);
          }
          const cellKey = rowKeys[i] + "\0" + colKeys[j] + "\0" + k;
          cells[cellKey] = AGG_FUNCTIONS[this.values[k].agg](cellVals);
        }
        if (!rowTotals[rowKeys[i]]) rowTotals[rowKeys[i]] = [];
        rowTotals[rowKeys[i]].push.apply(rowTotals[rowKeys[i]], vals);
      }
      if (!colTotals[rowKeys[i]]) colTotals[rowKeys[i]] = 0;
    }

    for (let j = 0; j < colKeys.length; j++) {
      colTotals[colKeys[j]] = [];
      for (let i = 0; i < rowKeys.length; i++) {
        const key = rowKeys[i] + "\0" + colKeys[j];
        const vals = buckets[key] || [];
        colTotals[colKeys[j]].push.apply(colTotals[colKeys[j]], vals);
      }
    }

    for (let k = 0; k < this.values.length; k++) {
      const allForK = [];
      for (let j = 0; j < colKeys.length; j++) {
        for (let i = 0; i < rowKeys.length; i++) {
          const key = rowKeys[i] + "\0" + colKeys[j];
          const vals = buckets[key] || [];
          for (let n = k; n < vals.length; n += this.values.length) {
            allForK.push(vals[n]);
          }
        }
      }
      if (k === 0) grandTotalAll = allForK;
    }

    this.result = {
      rowKeys: rowKeys,
      colKeys: colKeys,
      cells: cells,
      rowTotals: rowTotals,
      colTotals: colTotals,
      grandTotal: this.values.length > 0 ? AGG_FUNCTIONS[this.values[0].agg](grandTotalAll) : null,
    };
    return this.result;
  };

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
        html += ' <select class="pivot-agg"><option value="SUM"' + (v.agg === "SUM" ? " selected" : "") + '>SUM</option><option value="COUNT"' + (v.agg === "COUNT" ? " selected" : "") + '>COUNT</option><option value="AVERAGE"' + (v.agg === "AVERAGE" ? " selected" : "") + '>AVG</option><option value="MIN"' + (v.agg === "MIN" ? " selected" : "") + '>MIN</option><option value="MAX"' + (v.agg === "MAX" ? " selected" : "") + '>MAX</option></select>';
      }
      html += ' <button class="pivot-remove">×</button></li>';
    }
    html += "</ul></div>";
    return html;
  };

  PivotTable.prototype.renderResult = function () {
    const r = this.result;
    if (!r || r.rowKeys.length === 0) {
      return '<p class="pivot-empty">Drag fields into Rows, Columns, and Values to build a pivot table.</p>';
    }
    let html = '<table class="pivot-result-table"><thead><tr><th></th>';
    for (let j = 0; j < r.colKeys.length; j++) {
      html += "<th>" + r.colKeys[j] + "</th>";
    }
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

  function formatNumber(n) {
    if (typeof n !== "number" || isNaN(n)) return "";
    if (Number.isInteger(n)) return String(n);
    return n.toFixed(2);
  }

  PivotTable.prototype.dispose = function () {
    if (this.container) this.container.innerHTML = "";
    this.data = [];
    this.result = null;
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
      window._pivot = new PivotTable({ container: container, fields: ["Region", "Product", "Sales", "Quarter"] });
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
