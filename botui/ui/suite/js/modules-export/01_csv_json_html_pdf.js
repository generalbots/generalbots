"use strict";
/* ExportImport module 01: CSV, JSON, HTML, PDF (print) */
(function (window) {
  function escapeCsv(v) {
    if (v == null) return "";
    const s = String(v);
    if (/[",\n]/.test(s)) return '"' + s.replace(/"/g, '""') + '"';
    return s;
  }
  function download(filename, mime, data) {
    const blob = (data instanceof Blob) ? data : new Blob([data], { type: mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    setTimeout(() => { document.body.removeChild(a); URL.revokeObjectURL(url); }, 100);
  }
  function readFile(file) {
    return new Promise((resolve, reject) => {
      const r = new FileReader();
      r.onload = () => resolve(r.result);
      r.onerror = () => reject(r.error);
      r.readAsText(file);
    });
  }

  function exportCSV(grid, opts) {
    const range = (opts && opts.range) || grid.dataset.range || null;
    const data = readGridAsArray(grid, range);
    const csv = data.map(row => row.map(escapeCsv).join(",")).join("\n");
    download((opts && opts.filename) || "export.csv", "text/csv;charset=utf-8;", csv);
    return csv;
  }

  function importCSV(grid, text, opts) {
    const lines = text.split(/\r?\n/);
    const rows = lines.map(line => {
      const out = [];
      let cur = "", inQ = false;
      for (let i = 0; i < line.length; i++) {
        const c = line[i];
        if (inQ) {
          if (c === '"' && line[i + 1] === '"') { cur += '"'; i++; }
          else if (c === '"') inQ = false;
          else cur += c;
        } else {
          if (c === '"') inQ = true;
          else if (c === ",") { out.push(cur); cur = ""; }
          else cur += c;
        }
      }
      out.push(cur);
      return out;
    }).filter(r => r.length > 1 || (r[0] && r[0].length));
    populateGridFromArray(grid, rows, (opts && opts.startCell) || "A1");
    return rows.length;
  }

  function exportJSON(grid, opts) {
    const data = readGridAsArray(grid, (opts && opts.range) || null);
    const obj = {
      version: "1.0",
      exportedAt: new Date().toISOString(),
      range: (opts && opts.range) || "all",
      data: data
    };
    const json = JSON.stringify(obj, null, 2);
    download((opts && opts.filename) || "export.json", "application/json", json);
    return json;
  }

  function importJSON(grid, text, opts) {
    const obj = JSON.parse(text);
    const data = obj.data || obj;
    populateGridFromArray(grid, data, (opts && opts.startCell) || "A1");
    return data.length;
  }

  function exportHTML(elem, opts) {
    const clone = elem.cloneNode(true);
    const css = (opts && opts.css) || "body{font-family:sans-serif;padding:24px;}table{border-collapse:collapse;width:100%;}td,th{border:1px solid #ccc;padding:4px;}";
    const html = "<!DOCTYPE html><html><head><meta charset='utf-8'><style>" + css + "</style></head><body>" + clone.outerHTML + "</body></html>";
    download((opts && opts.filename) || "export.html", "text/html;charset=utf-8;", html);
    return html;
  }

  function exportPDF(elem, opts) {
    const win = window.open("", "_blank");
    const css = (opts && opts.css) || "body{font-family:sans-serif;padding:24px;color:#000;background:#fff;}table{border-collapse:collapse;width:100%;}td,th{border:1px solid #ccc;padding:4px;}";
    const html = "<!DOCTYPE html><html><head><meta charset='utf-8'><title>Print</title><style>" + css + "</style></head><body>" + elem.outerHTML + "<script>window.onload=()=>{setTimeout(()=>{window.print();},200);};</script></body></html>";
    win.document.write(html);
    win.document.close();
  }

  function readGridAsArray(grid, range) {
    if (range) {
      const m = range.match(/^([A-Z]+)(\d+):([A-Z]+)(\d+)$/);
      if (m) {
        const c1 = colToIdx(m[1]), r1 = parseInt(m[2], 10) - 1;
        const c2 = colToIdx(m[3]), r2 = parseInt(m[4], 10) - 1;
        const out = [];
        for (let r = r1; r <= r2; r++) {
          const row = [];
          for (let c = c1; c <= c2; c++) {
            const cell = grid.querySelector("[data-row='" + r + "'][data-col='" + c + "']");
            row.push(cell ? cell.textContent : "");
          }
          out.push(row);
        }
        return out;
      }
    }
    const cells = grid.querySelectorAll("[data-row][data-col]");
    const maxR = Math.max.apply(null, Array.from(cells).map(c => parseInt(c.dataset.row, 10))) || 0;
    const maxC = Math.max.apply(null, Array.from(cells).map(c => parseInt(c.dataset.col, 10))) || 0;
    const out = [];
    for (let r = 0; r <= maxR; r++) {
      const row = [];
      for (let c = 0; c <= maxC; c++) {
        const cell = grid.querySelector("[data-row='" + r + "'][data-col='" + c + "']");
        row.push(cell ? cell.textContent : "");
      }
      out.push(row);
    }
    return out;
  }

  function populateGridFromArray(grid, data, startCell) {
    const m = startCell.match(/^([A-Z]+)(\d+)$/);
    if (!m) return;
    const startC = colToIdx(m[1]);
    const startR = parseInt(m[2], 10) - 1;
    data.forEach((row, ri) => {
      row.forEach((val, ci) => {
        const cell = grid.querySelector("[data-row='" + (startR + ri) + "'][data-col='" + (startC + ci) + "']");
        if (cell) cell.textContent = val;
      });
    });
  }

  function colToIdx(letters) {
    let n = 0;
    for (let i = 0; i < letters.length; i++) n = n * 26 + (letters.charCodeAt(i) - 64);
    return n - 1;
  }

  window.ExportImportCSV = {
    exportCSV: exportCSV,
    importCSV: importCSV,
    exportJSON: exportJSON,
    importJSON: importJSON,
    exportHTML: exportHTML,
    exportPDF: exportPDF,
    readFile: readFile,
    readGridAsArray: readGridAsArray,
    populateGridFromArray: populateGridFromArray,
    colToIdx: colToIdx,
    download: download
  };
})(window);
