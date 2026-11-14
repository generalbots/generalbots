"use strict";
/* Sheet advanced module: 12_export — export sheet as CSV/XLSX/Markdown download */

(function () {
  let menu = null;

  function currentSheetId() {
    const loaded = window.__LOADED_SHEET;
    if (loaded && loaded.id) return String(loaded.id);
    if (window.SheetCore && typeof window.SheetCore.currentSheetId === "function") {
      const id = window.SheetCore.currentSheetId();
      if (id) return String(id);
    }
    return window.__SHEET_INITIAL_ID ? String(window.__SHEET_INITIAL_ID) : "";
  }

  function isExportableId(id) {
    return !!id && id !== "current" && id !== "loading" && id !== "new";
  }

  function currentName() {
    const input = document.getElementById("sheetName");
    if (input && input.value) return input.value.trim();
    const sheet = window.__LOADED_SHEET;
    return (sheet && sheet.name) || "sheet";
  }

  function slug(s) {
    return String(s || "sheet")
      .replace(/[^\w\-]+/g, "_")
      .replace(/_+/g, "_")
      .replace(/^_|_$/g, "") || "sheet";
  }

  function download(blob, filename) {
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.style.display = "none";
    a.rel = "noopener";
    document.body.appendChild(a);
    a.click();
    // Keep the anchor attached briefly so the browser can process the click
    // before we remove it and revoke the blob URL.
    setTimeout(function () {
      if (a.parentNode) a.parentNode.removeChild(a);
      URL.revokeObjectURL(url);
    }, 1500);
  }

  function readErrorMessage(response) {
    const ct = (response.headers.get("content-type") || "").toLowerCase();
    if (ct.indexOf("application/json") !== -1) {
      return response.json().then(function (body) {
        if (body && typeof body.error === "string" && body.error) return body.error;
        return "Export failed";
      }).catch(function () { return "Export failed"; });
    }
    return response.text().then(function (text) {
      return (text && text.trim()) || "Export failed";
    }).catch(function () { return "Export failed"; });
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
    } catch (e) {
      token = "";
    }
    if (token) headers.Authorization = "Bearer " + token;
    return headers;
  }

  function exportFormat(format, ext) {
    const id = currentSheetId();
    if (!isExportableId(id)) {
      showToast("Save or open a sheet before exporting", false);
      return Promise.resolve();
    }
    return fetch("/api/sheet/export", {
      method: "POST",
      headers: authHeaders({
        "Content-Type": "application/json",
        "Accept": "application/octet-stream, */*"
      }),
      body: JSON.stringify({ id: id, format: format }),
    })
      .then(function (r) {
        if (!r.ok) {
          return readErrorMessage(r).then(function (msg) { throw new Error(msg); });
        }
        return r.blob();
      })
      .then(function (blob) {
        if (!blob || blob.size === 0) {
          throw new Error("Export returned an empty file");
        }
        const filename = slug(currentName()) + "." + ext;
        download(blob, filename);
        // The anchor is removed shortly after the click, so this toast is the
        // persistent confirmation the user (and the UI automation) can observe.
        showToast("Exported " + filename, true);
      })
      .catch(function (err) {
        const msg = (err && err.message) ? String(err.message) : "Export failed";
        showToast(msg, false);
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
    menu.appendChild(item("CSV", function () { exportFormat("csv", "csv"); }));
    menu.appendChild(item("XLSX", function () { exportFormat("xlsx", "xlsx"); }));
    menu.appendChild(item("Markdown", function () { exportFormat("markdown", "md"); }));
    // The API already streams real ODF and PDF bytes; the menu offered only
    // CSV/XLSX/Markdown, so those two formats were unreachable from the UI.
    menu.appendChild(item("ODS", function () { exportFormat("ods", "ods"); }));
    menu.appendChild(item("PDF", function () { exportFormat("pdf", "pdf"); }));
    menu.appendChild(item("HTML", function () { exportFormat("html", "html"); }));
    menu.appendChild(item("JSON", function () { exportFormat("json", "json"); }));
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
    btn.type = "button";
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
      if (menu && !menu.contains(e.target) && e.target.closest("#exportSheetBtn")) return;
      if (menu && !menu.contains(e.target)) closeMenu();
    }, true);
  }

  window.SheetExport = {
    csv: function () { return exportFormat("csv", "csv"); },
    xlsx: function () { return exportFormat("xlsx", "xlsx"); },
    markdown: function () { return exportFormat("markdown", "md"); },
    ods: function () { return exportFormat("ods", "ods"); },
    pdf: function () { return exportFormat("pdf", "pdf"); },
    html: function () { return exportFormat("html", "html"); },
    json: function () { return exportFormat("json", "json"); },
  };

  setTimeout(wire, 0);
})();