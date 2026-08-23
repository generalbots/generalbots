"use strict";

// GB Desktop Shortcuts (#1188): drag a file from the Drive app onto the
// desktop to create a <name>.shortcut icon. Each user gets an isolated
// desktop (store keyed by the signed-in identity). Double-click opens the
// target file's folder in Drive; right-click offers Open / Remove.
//
// Storage: localStorage["gb-desktop-icons:<userKey>"] as
//   [{ id, kind:"shortcut", name, path, bucket, x, y }, …]

window.GBDesktopShortcuts = window.GBDesktopShortcuts || {};

(function (mod) {
  var layer = null;
  var grid = { cellW: 92, cellH: 96, originX: 16, originY: 16 };

  function userKey() {
    try {
      var token = window.getGBAccessToken ? window.getGBAccessToken() : null;
      if (token) {
        var payload = JSON.parse(
          decodeURIComponent(
            escape(atob(token.split(".")[1].replace(/-/g, "+").replace(/_/g, "/")))
          )
        );
        if (payload && payload.sub) return String(payload.sub);
      }
    } catch (e) { /* fall through */ }
    return "local";
  }

  function storeKey() {
    return "gb-desktop-icons:" + userKey();
  }

  function readItems() {
    try {
      var raw = JSON.parse(localStorage.getItem(storeKey()) || "[]");
      return Array.isArray(raw) ? raw : [];
    } catch (e) {
      return [];
    }
  }

  function writeItems(items) {
    try {
      localStorage.setItem(storeKey(), JSON.stringify(items));
    } catch (e) {}
  }

  function uniqueName(items, base) {
    var name = base.indexOf(".") === -1 ? base + ".shortcut" : base;
    if (!name.endsWith(".shortcut")) name += ".shortcut";
    var taken = {};
    items.forEach(function (i) { taken[i.name] = true; });
    if (!taken[name]) return name;
    var dot = name.lastIndexOf(".shortcut");
    var stem = name.slice(0, dot);
    var n = 2;
    while (taken[stem + "-" + n + ".shortcut"]) n++;
    return stem + "-" + n + ".shortcut";
  }

  // ── Rendering ────────────────────────────────────────────────

  function fileGlyph(name) {
    var ext = (name.match(/\.([a-z0-9]+)\.shortcut$/i) || [])[1] || "";
    var color = "#84d669";
    if (/^(xlsx?|csv)$/.test(ext)) color = "#22c55e";
    else if (/^(pdf)$/.test(ext)) color = "#ef4444";
    else if (/^(docx?|txt|md)$/.test(ext)) color = "#3b82f6";
    else if (/^(png|jpe?g|gif|svg|webp)$/.test(ext)) color = "#ec4899";
    return (
      '<div class="gb-dicon-glyph" style="color:' + color + '">' +
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">' +
      '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>' +
      '<polyline points="14 2 14 8 20 8"/></svg>' +
      '<span class="gb-dicon-arrow">\u2197</span></div>'
    );
  }

  function renderIcon(item) {
    var el = document.createElement("div");
    el.className = "gb-desktop-icon";
    el.setAttribute("data-icon-id", item.id);
    el.style.left = item.x + "px";
    el.style.top = item.y + "px";
    el.innerHTML =
      fileGlyph(item.name) +
      '<span class="gb-dicon-label"></span>';
    el.querySelector(".gb-dicon-label").textContent = item.name;

    el.addEventListener("dblclick", function () { openShortcut(item); });
    el.addEventListener("click", function () {
      layer.querySelectorAll(".gb-desktop-icon.selected")
        .forEach(function (n) { n.classList.remove("selected"); });
      el.classList.add("selected");
    });
    el.addEventListener("contextmenu", function (e) {
      e.preventDefault();
      e.stopPropagation();
      openMenu(item, e.clientX, e.clientY);
    });

    makeDraggable(el, item);
    return el;
  }

  function renderAll() {
    if (!layer) return;
    layer.innerHTML = "";
    readItems().forEach(function (item) {
      layer.appendChild(renderIcon(item));
    });
  }

  function openShortcut(item) {
    if (window.openDeepLink) {
      window.openDeepLink("drive", { path: item.path || "" });
    }
  }

  function removeItem(id) {
    writeItems(readItems().filter(function (i) { return i.id !== id; }));
    renderAll();
  }

  // ── Context menu ─────────────────────────────────────────────

  function closeMenus() {
    document.querySelectorAll(".gb-dicon-menu").forEach(function (m) { m.remove(); });
  }

  function openMenu(item, x, y) {
    closeMenus();
    var menu = document.createElement("div");
    menu.className = "desktop-context-menu gb-dicon-menu";
    menu.style.left = Math.min(x, window.innerWidth - 190) + "px";
    menu.style.top = Math.min(y, window.innerHeight - 110) + "px";
    [
      { label: "Open", fn: function () { openShortcut(item); } },
      { label: "Open Drive", fn: function () { window.openDeepLink("drive", {}); } },
      { label: "Remove shortcut", fn: function () { removeItem(item.id); } },
    ].forEach(function (entry) {
      var it = document.createElement("div");
      it.className = "desktop-context-item";
      it.textContent = entry.label;
      it.addEventListener("click", function () { closeMenus(); entry.fn(); });
      menu.appendChild(it);
    });
    document.body.appendChild(menu);
    setTimeout(function () {
      document.addEventListener("click", closeMenus, { once: true });
    }, 0);
  }

  // ── Drag & drop ──────────────────────────────────────────────

  function makeDraggable(el, item) {
    el.draggable = true;
    el.addEventListener("dragstart", function (e) {
      e.dataTransfer.setData("application/x-gb-desktop-icon", item.id);
      e.dataTransfer.effectAllowed = "move";
    });
    el.addEventListener("dragend", function () {
      persistPositionFromEl(el);
    });
  }

  function persistPositionFromEl(el) {
    var id = el.getAttribute("data-icon-id");
    var items = readItems();
    items.forEach(function (i) {
      if (i.id === id) {
        i.x = Math.max(0, parseInt(el.style.left, 10) || 0);
        i.y = Math.max(0, parseInt(el.style.top, 10) || 0);
      }
    });
    writeItems(items);
  }

  function nextFreeCell(x, y) {
    var items = readItems();
    var col = Math.round((x - grid.originX) / grid.cellW);
    var row = Math.round((y - grid.originY) / grid.cellH);
    var taken = {};
    items.forEach(function (i) {
      var c = Math.round((i.x - grid.originX) / grid.cellW);
      var r = Math.round((i.y - grid.originY) / grid.cellH);
      (taken[c] = taken[c] || {})[r] = true;
    });
    // Also avoid cells covered by desktop widgets so shortcuts stay
    // clickable underneath nothing.
    if (layer && layer.parentElement) {
      layer.parentElement.querySelectorAll(".gb-widget").forEach(function (w) {
        var wr = w.getBoundingClientRect();
        var base = layer.getBoundingClientRect();
        var c0 = Math.round((wr.left - base.left - grid.originX) / grid.cellW);
        var c1 = Math.round((wr.right - base.left - grid.originX) / grid.cellW);
        var r0 = Math.round((wr.top - base.top - grid.originY) / grid.cellH);
        var r1 = Math.round((wr.bottom - base.top - grid.originY) / grid.cellH);
        for (var c = c0; c <= c1; c++) {
          for (var r = r0; r <= r1; r++) {
            (taken[c] = taken[c] || {})[r] = true;
          }
        }
      });
    }
    var rr = Math.max(0, row);
    for (; rr < 40; rr++) {
      if (!(taken[col] && taken[col][rr])) break;
    }
    // Column full: walk columns rightwards until a free cell exists.
    while (taken[col] && taken[col][rr]) {
      col++;
      rr = Math.max(0, row);
      for (; rr < 40; rr++) {
        if (!(taken[col] && taken[col][rr])) break;
      }
    }
    return {
      x: grid.originX + col * grid.cellW,
      y: grid.originY + rr * grid.cellH,
    };
  }

  function handleDrop(e) {
    var raw = e.dataTransfer.getData("application/x-gb-drive-file");
    if (!raw) return;
    e.preventDefault();
    var data = {};
    try { data = JSON.parse(raw); } catch (err) { return; }
    if (!data.path || data.type === "folder") return;

    var host = layer.parentElement;
    var rect = layer.getBoundingClientRect();
    var px = Math.max(0, e.clientX - rect.left - 30);
    var py = Math.max(0, e.clientY - rect.top - 20);
    var cell = nextFreeCell(px, py);

    var items = readItems();
    var item = {
      id: "sc-" + Date.now(),
      kind: "shortcut",
      name: uniqueName(items, data.name || "File"),
      path: data.path,
      bucket: data.bucket || "",
      x: Math.min(cell.x, (host ? host.clientWidth : 1200) - grid.cellW),
      y: Math.min(cell.y, (host ? host.clientHeight : 700) - grid.cellH),
    };
    items.push(item);
    writeItems(items);
    renderAll();
    var node = layer.querySelector('[data-icon-id="' + item.id + '"]');
    if (node) node.classList.add("selected");
  }

  // ── Init ─────────────────────────────────────────────────────

  mod.init = function (workspaceEl) {
    if (layer) return;
    var host = workspaceEl || document.getElementById("desktop-content");
    if (!host) return;

    layer = document.createElement("div");
    layer.id = "gb-desktop-icons";
    layer.className = "gb-desktop-icons";
    host.insertBefore(layer, host.firstChild);

    // Accept drops from the Drive app (and moves of our own icons).
    host.addEventListener("dragover", function (e) {
      if (e.dataTransfer.types.indexOf("application/x-gb-drive-file") !== -1 ||
          e.dataTransfer.types.indexOf("application/x-gb-desktop-icon") !== -1) {
        e.preventDefault();
        e.dataTransfer.dropEffect =
          e.dataTransfer.types.indexOf("application/x-gb-desktop-icon") !== -1
            ? "move" : "copy";
      }
    });
    host.addEventListener("drop", handleDrop);

    // Deselect when clicking empty desktop.
    host.addEventListener("click", function (e) {
      if (e.target === host || e.target === layer) {
        layer.querySelectorAll(".gb-desktop-icon.selected")
          .forEach(function (n) { n.classList.remove("selected"); });
      }
    });

    // Per-user isolation: reload the layer when auth identity changes.
    window.addEventListener("gb:auth:login", renderAll);
    window.addEventListener("gb:auth:logout", renderAll);

    renderAll();
  };
})(window.GBDesktopShortcuts);
