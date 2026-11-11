"use strict";
/* Sheet advanced module: 12_export — export sheet as CSV/XLSX/Markdown download */

(function () {
  let menu = null;

  function currentSheetId() {
    if (window.SheetCore && window.SheetCore.currentSheetId) return window.SheetCore.currentSheetId();
    return window.__SHEET_INITIAL_ID || "current";
  }

  function currentName() {
    const input = document.getElementById("sheetName");
    if (input && input.value) return input.value.trim();
    const sheet = window.__LOADED_SHEET;
    return (sheet && sheet.name) || "sheet";
  }

  function slug(s) {
    return s.replace(/[^\w\-]+/g, "_").replace(/_+/g, "_").replace(/^_|_$/g, "") || "sheet";
  }

  function download(blob, filename) {
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(function () { URL.revokeObjectURL(url); }, 4000);
  }

  function exportFormat(format, ext, mime) {
    return fetch("/api/sheet/export", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ id: currentSheetId(), format: format }),
    })
      .then(function (r) {
        if (!r.ok) throw new Error("export failed");
        return r.blob();
      })
      .then(function (blob) {
        var filename = slug(currentName()) + "." + ext;
        download(blob, filename);
        // The download anchor is removed immediately, so this toast is the only
        // confirmation the user (and the UI automation) gets that it worked.
        showToast("Exported " + filename, true);
      })
      .catch(function () {
        showToast("Export failed");
      });
  }

  function showToast(msg, ok) {
    const id = "ss-export-toast";
    let toast = document.getElementById(id);
    if (!toast) {
      toast = document.createElement("div");
      toast.id = id;
      toast.style.cssText = "position:fixed;bottom:24px;left:50%;transform:translateX(-50%);background:#16a34a;color:#fff;padding:10px 18px;border-radius:6px;font-size:13px;z-index:10000;box-shadow:0 4px 12px rgba(0,0,0,0.3);";
      document.body.appendChild(toast);
    }
    toast.textContent = msg;
    toast.style.background = ok ? "#16a34a" : "#dc2626";
    toast.style.display = "block";
    clearTimeout(toast.__timer);
    toast.__timer = setTimeout(function () { toast.style.display = "none"; }, 2600);
  }

  function openMenu(anchor) {
    closeMenu();
    menu = document.createElement("div");
    menu.className = "ss-export-menu";
    menu.style.cssText =
      "position:absolute;z-index:60;background:var(--sheet-bg-elevated, #1e293b);border:1px solid var(--sheet-border, #334155);border-radius:6px;" +
      "box-shadow:0 8px 24px rgba(0,0,0,0.4);min-width:180px;overflow:hidden;";
    const item = function (label, fn) {
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      // Screen readers and the UI orchestrator both need a label that says what
      // the item exports (the plain "CSV" is ambiguous once the menu closes).
      b.setAttribute("aria-label", "Export " + label);
      b.style.cssText =
        "display:block;width:100%;padding:8px 14px;background:none;border:none;color:var(--sheet-fg, #f8fafc);" +
        "text-align:left;font-size:13px;cursor:pointer;";
      b.addEventListener("mouseover", function () { b.style.background = "var(--sheet-border, #334155)"; });
      b.addEventListener("mouseout", function () { b.style.background = "none"; });
      b.addEventListener("click", function () { closeMenu(); fn(); });
      return b;
    };
    menu.appendChild(item("CSV", function () { exportFormat("csv", "csv", "text/csv"); }));
    menu.appendChild(item("XLSX", function () { exportFormat("xlsx", "xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"); }));
    menu.appendChild(item("Markdown", function () { exportFormat("markdown", "md", "text/markdown"); }));
    // The API already streams real ODF and PDF bytes; the menu offered only
    // CSV/XLSX/Markdown, so those two formats were unreachable from the UI.
    menu.appendChild(item("ODS", function () { exportFormat("ods", "ods", "application/vnd.oasis.opendocument.spreadsheet"); }));
    menu.appendChild(item("PDF", function () { exportFormat("pdf", "pdf", "application/pdf"); }));
    menu.appendChild(item("HTML", function () { exportFormat("html", "html", "text/html"); }));
    menu.appendChild(item("JSON", function () { exportFormat("json", "json", "application/json"); }));
    document.body.appendChild(menu);
    const rect = anchor.getBoundingClientRect();
    let left = rect.left;
    let top = rect.bottom + 4;
    if (left + 190 > window.innerWidth) left = window.innerWidth - 200;
    menu.style.left = left + "px";
    menu.style.top = top + "px";
  }

  function closeMenu() {
    if (menu) {
      menu.remove();
      menu = null;
    }
  }

  function wire() {
    const host = document.getElementById("sheet-app");
    if (!host) {
      setTimeout(wire, 100);
      return;
    }
    if (host.__exportBound) return;
    host.__exportBound = true;

    const btn = document.createElement("button");
    btn.className = "btn-icon";
    btn.id = "exportSheetBtn";
    btn.title = "Export";
    btn.setAttribute("aria-label", "Export");
    btn.style.cssText = "display:inline-flex;align-items:center;gap:4px;";
    // A trailing `+` used to concatenate this call's `undefined` return into
    // the markup, so the toolbar button rendered the word "undefined" and the
    // label lookup in the UI orchestrator could never match "Export".
    btn.innerHTML =
      '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>';

    btn.addEventListener("click", function (e) {
      e.preventDefault();
      openMenu(btn);
    });
    const right = host.querySelector(".toolbar-right");
    if (right) right.insertBefore(btn, right.firstChild);

    document.addEventListener("mousedown", function (e) {
      if (menu && !menu.contains(e.target) && e.target.id !== "exportSheetBtn") closeMenu();
    }, true);
  }

  window.SheetExport = {
    csv: function () { return exportFormat("csv", "csv", "text/csv"); },
    xlsx: function () { return exportFormat("xlsx", "xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"); },
    markdown: function () { return exportFormat("markdown", "md", "text/markdown"); },
    ods: function () { return exportFormat("ods", "ods", "application/vnd.oasis.opendocument.spreadsheet"); },
    pdf: function () { return exportFormat("pdf", "pdf", "application/pdf"); },
    html: function () { return exportFormat("html", "html", "text/html"); },
    json: function () { return exportFormat("json", "json", "application/json"); },
  };

  setTimeout(wire, 0);
})();