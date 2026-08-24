"use strict";

// GB Virtual Desktops (#1155): macOS-Spaces style desktops for the shell.
// Windows stay open globally; each desktop records which windows are visible
// so switching hides/shows the rest. A taskbar "Desktops" button opens a
// mini strip; Ctrl+Shift+Arrow switches; Ctrl+Shift+1..9 jumps.
//
// Persistence: localStorage["gb-desktops"] = { desktops: [{id,name}], current }.

window.GBVirtualDesktops = window.GBVirtualDesktops || {};

(function (mod) {
  var STORAGE_KEY = "gb-desktops";
  var state = { desktops: [], current: 0 };
  var strip = null;

  function wm() {
    return window.WindowManager || null;
  }

  function read() {
    try {
      var raw = JSON.parse(localStorage.getItem(STORAGE_KEY) || "null");
      if (raw && Array.isArray(raw.desktops) && raw.desktops.length) {
        state.desktops = raw.desktops;
        state.current = Math.min(raw.current || 0, state.desktops.length - 1);
        return;
      }
    } catch (e) { /* fall through */ }
    state.desktops = [{ id: "desk-1", name: "Desktop 1" }];
    state.current = 0;
  }

  function persist() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch (e) {}
    window.dispatchEvent(new CustomEvent("gb-desktops-changed", { detail: { desktops: state.desktops.slice(), current: state.current } }));
  }

  function desktopForWindow(id) {
    for (var i = 0; i < state.desktops.length; i++) {
      var list = state.desktops[i].windows || [];
      if (list.indexOf(id) !== -1) return i;
    }
    return -1;
  }

  function visibleOn(desktopIdx, id) {
    var d = state.desktops[desktopIdx];
    return !d || !d.windows || d.windows.indexOf(id) !== -1;
  }

  function applyVisibility() {
    var manager = wm();
    if (!manager) return;
    manager.openWindows.forEach(function (w) {
      var el = document.getElementById("window-" + w.id);
      if (!el) return;
      var show = visibleOn(state.current, w.id) || w.id === "chat";
      el.style.display = show && !w.isMinimized ? "flex" : "none";
      if (!show && !w.isMinimized) {
        // Keep the dock indicator accurate: hidden-by-desktop is not minimized.
        var dock = document.getElementById("dock-item-" + w.id);
        if (dock) dock.classList.remove("hidden-by-desktop");
      }
    });
    // Keep the chat window pinned across desktops for continuity.
    window.dispatchEvent(new CustomEvent("gb-desktop-switched", { detail: { current: state.current } }));
  }

  function switchTo(idx) {
    if (idx < 0 || idx >= state.desktops.length || idx === state.current) return;
    state.current = idx;
    persist();
    applyVisibility();
  }

  function addDesktop(name) {
    var n = state.desktops.length + 1;
    state.desktops.push({ id: "desk-" + Date.now(), name: name || ("Desktop " + n), windows: [] });
    state.current = state.desktops.length - 1;
    persist();
    applyVisibility();
    return state.desktops.length - 1;
  }

  function removeDesktop(idx) {
    if (state.desktops.length <= 1) return;
    // Move orphaned windows to the previous desktop.
    var removed = state.desktops[idx];
    var target = state.desktops[idx === 0 ? 1 : idx - 1];
    (removed.windows || []).forEach(function (id) {
      if (target.windows.indexOf(id) === -1) target.windows.push(id);
    });
    state.desktops.splice(idx, 1);
    if (state.current >= state.desktops.length) state.current = state.desktops.length - 1;
    persist();
    applyVisibility();
  }

  function renameDesktop(idx, name) {
    if (idx < 0 || idx >= state.desktops.length) return;
    state.desktops[idx].name = name || state.desktops[idx].name;
    persist();
  }

  // ── Taskbar strip ────────────────────────────────────────────

  function toggleStrip(anchor) {
    if (strip) { closeStrip(); return; }
    strip = document.createElement("div");
    strip.className = "gb-desktops-strip";
    strip.setAttribute("role", "menu");
    renderStrip();
    var rect = anchor.getBoundingClientRect();
    strip.style.left = Math.max(8, rect.left) + "px";
    strip.style.bottom = (window.innerHeight - rect.top) + 8 + "px";
    document.body.appendChild(strip);
    document.addEventListener("click", closeStrip, { once: true });
  }

  function renderStrip() {
    if (!strip) return;
    strip.innerHTML =
      '<div class="gb-desktops-strip-title">Virtual Desktops</div>' +
      '<div class="gb-desktops-strip-add" title="New desktop">+ New desktop</div>' +
      '<div class="gb-desktops-strip-list"></div>';
    var list = strip.querySelector(".gb-desktops-strip-list");
    state.desktops.forEach(function (d, i) {
      var item = document.createElement("div");
      item.className = "gb-desktop-chip" + (i === state.current ? " active" : "");
      item.innerHTML =
        '<span class="gb-desktop-chip-name"></span>' +
        '<span class="gb-desktop-chip-x" title="Remove">\u00d7</span>';
      item.querySelector(".gb-desktop-chip-name").textContent = d.name;
      item.querySelector(".gb-desktop-chip-name").addEventListener("dblclick", function () {
        var name = window.prompt("Desktop name:", d.name);
        if (name) renameDesktop(i, name);
        renderStrip();
      });
      item.addEventListener("click", function (e) {
        if (e.target.classList.contains("gb-desktop-chip-x")) {
          removeDesktop(i);
          renderStrip();
        } else {
          switchTo(i);
          closeStrip();
        }
      });
      list.appendChild(item);
    });
    strip.querySelector(".gb-desktops-strip-add").addEventListener("click", function () {
      addDesktop();
      renderStrip();
    });
  }

  function closeStrip() {
    if (strip) { strip.remove(); strip = null; }
  }

  // ── Keyboard ─────────────────────────────────────────────────

  function onKeyDown(e) {
    if (!e.ctrlKey || !e.shiftKey || e.altKey || e.metaKey) return;
    if (e.key === "ArrowRight") { e.preventDefault(); switchTo(state.current + 1); }
    else if (e.key === "ArrowLeft") { e.preventDefault(); switchTo(state.current - 1); }
    else {
      var n = parseInt(e.key, 10);
      if (n >= 1 && n <= 9) { e.preventDefault(); switchTo(n - 1); }
    }
  }

  // ── Window tracking ──────────────────────────────────────────

  function trackWindow(id) {
    var d = state.desktops[state.current];
    if (!d.windows) d.windows = [];
    // A window opened while on desktop N belongs to N unless already placed.
    if (desktopForWindow(id) === -1 && d.windows.indexOf(id) === -1) {
      d.windows.push(id);
      persist();
    }
  }

  // ── Init ─────────────────────────────────────────────────────

  mod.init = function () {
    if (mod.initialized) return;
    mod.initialized = true;
    read();
    persist();

    // Track window lifecycle via the WindowManager change event.
    window.addEventListener("gb-window-changed", function (e) {
      if (e.detail && e.detail.action === "open") trackWindow(e.detail.id);
    });

    // Expose the toggle (backed by the taskbar tray button) and the rest of
    // the public surface.
    mod.toggleStrip = toggleStrip;
    mod.switchTo = switchTo;
    mod.add = addDesktop;
    mod.list = function () { return state.desktops.slice(); };
    mod.current = function () { return state.current; };

    document.addEventListener("keydown", onKeyDown);
    window.addEventListener("gb-window-changed", applyVisibility);
    window.addEventListener("gb-window-closed", applyVisibility);
  };
})(window.GBVirtualDesktops);

// Friendly alias used by desktop.html, the tray button, and Mission Control:
//   VirtualDesktops.init(workspace) · toggleSwitcher() · list() · activeId() · switchTo(id)
window.VirtualDesktops = window.VirtualDesktops || (function () {
  function module() { return window.GBVirtualDesktops; }
  return {
    init: function () { module().init(); },
    toggleSwitcher: function () {
      module().toggleStrip(document.getElementById("trayVirtualDesktops"));
    },
    list: function () { return module().list(); },
    activeId: function () {
      var list = module().list();
      var cur = module().current();
      return (list[cur] ? list[cur].id : (list[0] ? list[0].id : "desk-1"));
    },
    switchTo: function (id) {
      var list = module().list();
      for (var i = 0; i < list.length; i++) {
        if (list[i].id === id) { module().switchTo(i); return; }
      }
    },
  };
})();
