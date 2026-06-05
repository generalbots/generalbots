"use strict";

/**
 * Module 20a: Tables in slides for Slides.
 * Real <table> rendering with rows/cols, cell padding, headers,
 * and an edit-in-place dialog. Tables are inserted as elements
 * on the active slide, sized as percentage of the canvas.
 *
 * Public API: window.SlidesTables = { openTableModal, insertTable,
 *   renderTable }.
 */

(function () {
  function getState() { return window.state || null; }

  function ensureTableModal() {
    let m = document.getElementById("slidesTableModal");
    if (m) return m;
    m = document.createElement("div");
    m.id = "slidesTableModal";
    m.style.cssText = "position:fixed;inset:0;background:rgba(0,0,0,0.5);z-index:9999;display:none;align-items:center;justify-content:center;";
    m.innerHTML = `
      <div style="background:#fff;border-radius:8px;padding:24px;min-width:480px;max-width:90%;">
        <h3 style="margin:0 0 16px 0;">Insert Table</h3>
        <div style="margin-bottom:12px;display:flex;gap:12px;align-items:center;">
          <label>Rows: <input type="number" id="stRows" value="3" min="1" style="width:80px;padding:4px;" /></label>
          <label>Cols: <input type="number" id="stCols" value="3" min="1" style="width:80px;padding:4px;" /></label>
          <label><input type="checkbox" id="stHeaders" checked /> First row as headers</label>
        </div>
        <div id="stPreview" style="margin-bottom:12px;max-height:240px;overflow:auto;"></div>
        <div style="display:flex;gap:8px;justify-content:flex-end;">
          <button id="stCancel" style="padding:6px 16px;">Cancel</button>
          <button id="stInsert" style="padding:6px 16px;background:#1a73e8;color:#fff;border:0;border-radius:4px;">Insert</button>
        </div>
      </div>
    `;
    document.body.appendChild(m);
    function refresh() {
      const r = parseInt(m.querySelector("#stRows").value) || 3;
      const c = parseInt(m.querySelector("#stCols").value) || 3;
      const hasHeaders = m.querySelector("#stHeaders").checked;
      const preview = m.querySelector("#stPreview");
      preview.innerHTML = "";
      preview.appendChild(renderTable(r, c, hasHeaders));
    }
    m.querySelector("#stRows").addEventListener("input", refresh);
    m.querySelector("#stCols").addEventListener("input", refresh);
    m.querySelector("#stHeaders").addEventListener("change", refresh);
    m.querySelector("#stCancel").addEventListener("click", function () { m.style.display = "none"; });
    m.querySelector("#stInsert").addEventListener("click", function () {
      const r = parseInt(m.querySelector("#stRows").value) || 3;
      const c = parseInt(m.querySelector("#stCols").value) || 3;
      const hasHeaders = m.querySelector("#stHeaders").checked;
      insertTable(r, c, hasHeaders);
      m.style.display = "none";
    });
    setTimeout(refresh, 50);
    return m;
  }

  function openTableModal() {
    const m = ensureTableModal();
    m.style.display = "flex";
  }

  function renderTable(rows, cols, hasHeaders) {
    const t = document.createElement("table");
    t.style.cssText = "border-collapse:collapse;width:100%;font-size:13px;";
    for (let r = 0; r < rows; r++) {
      const tr = document.createElement("tr");
      for (let c = 0; c < cols; c++) {
        const cell = document.createElement(hasHeaders && r === 0 ? "th" : "td");
        cell.contentEditable = "true";
        cell.style.cssText = "border:1px solid #888;padding:6px;min-width:60px;" + (hasHeaders && r === 0 ? "background:#f4f4f4;font-weight:bold;" : "");
        cell.textContent = hasHeaders && r === 0 ? "Header " + (c + 1) : "";
        tr.appendChild(cell);
      }
      t.appendChild(tr);
    }
    return t;
  }

  function insertTable(rows, cols, hasHeaders) {
    const s = getState();
    if (!s) return null;
    const wrapper = document.createElement("div");
    wrapper.className = "slide-element slide-table";
    wrapper.style.cssText = "position:absolute;left:15%;top:15%;width:60%;height:auto;";
    wrapper.appendChild(renderTable(rows, cols, hasHeaders));
    const canvas = document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas");
    if (canvas) canvas.appendChild(wrapper);
    const slide = (s.slides || [])[s.currentSlide || 0];
    if (slide) {
      if (!slide.elements) slide.elements = [];
      slide.elements.push({ type: "table", rows, cols, hasHeaders, x: 15, y: 15, width: 60, height: 30 });
    }
    return wrapper;
  }

  function attach() {
    const tableBtn = document.getElementById("insertTableBtn");
    if (tableBtn) tableBtn.addEventListener("click", function (e) { e.preventDefault(); e.stopPropagation(); openTableModal(); });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SlidesTables = { openTableModal, renderTable, insertTable };
})();
